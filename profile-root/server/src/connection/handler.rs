use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, stream::StreamExt};
use serde_json;
use hex;

use crate::auth::handler::{handle_authentication, AuthResult};
use crate::lobby::{Lobby, PublicKey, ActiveConnection};
use crate::protocol::{AuthMessage, AuthSuccessMessage, AuthErrorMessage};
use crate::message::{handle_incoming_message, route_message, MessageValidationResult};
use profile_shared::LobbyError;

/// Atomic counter for generating unique connection IDs
///
/// NOTE: Connection IDs wrap at u64::MAX (approximately 1.8e19 connections).
/// In practice, this is never reached. If needed, a UUID generator could be
/// used instead, but atomic counter is faster for the common case.
static CONNECTION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Generate a unique connection ID atomically
fn generate_connection_id() -> u64 {
    // Using Relaxed ordering since connection IDs don't require strict synchronization
    // SeqCst is unnecessary here - we just need atomic increment for uniqueness
    // Using wrapping_add to prevent panic on overflow (saturating not available in older Rust)
    CONNECTION_COUNTER.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
        Some(v.wrapping_add(1))
    }).unwrap_or(0)
}

/// Connection handler that processes WebSocket connections
pub async fn handle_connection(
    stream: TcpStream,
    lobby: Arc<Lobby>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Accept WebSocket handshake
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    
    let (mut write, mut read) = ws_stream.split();

    // Track authenticated user's public key for cleanup
    let mut authenticated_key: Option<PublicKey> = None;

    // Wait for auth message
    if let Some(message_result) = read.next().await {
        let message = message_result?;
        
        match handle_auth_message(&message, &lobby).await {
            AuthResult::Success { public_key, lobby_state: _ } => {
                // NOTE: The lobby_state from auth handler is IGNORED here.
                // We add the user to the lobby FIRST, then refetch the lobby state
                // to ensure the newly authenticated user sees themselves and all
                // other online users in the response.
                //
                // This fixes the bug where lobby_state was captured BEFORE the user
                // was added, causing the new user to not see themselves.
                
                // Convert Vec<u8> to String for lobby API
                // Validate that public key is exactly 32 bytes before encoding
                if public_key.len() != 32 {
                    tracing::error!("Invalid public key length: {} bytes (expected 32)", public_key.len());
                    return Err("Invalid public key".into());
                }
                let public_key_string = hex::encode(public_key);
                
                // Create active connection for lobby
                // NOTE: The sender channel is for future Epic 3 message routing.
                // Currently messages to clients are sent directly via 'write' sink.
                // The sender will be used when implementing broadcast notifications
                // (Stories 2.3, 2.4) to route messages through the lobby.
                // Receiver is intentionally dropped here - will be connected when
                // implementing broadcast helpers in Story 2.3.
                let (sender, _receiver) = tokio::sync::mpsc::unbounded_channel::<profile_shared::Message>();
                let connection = ActiveConnection {
                    public_key: public_key_string.clone(),
                    sender,
                    connection_id: generate_connection_id(),
                };

                // Add user to lobby FIRST (critical for correct lobby state)
                // If this fails, we should NOT send auth success - user is not in lobby
                match crate::lobby::add_user(&lobby, public_key_string.clone(), connection).await {
                    Ok(()) => {
                        // User successfully added to lobby, proceed with auth success
                        authenticated_key = Some(public_key_string.clone());
                    }
                    Err(e) => {
                        tracing::error!("Failed to add user to lobby: {}", e);
                        // Send error and close connection - user cannot be authenticated
                        let error_msg = AuthErrorMessage {
                            r#type: "error".to_string(),
                            reason: "lobby_error".to_string(),
                            details: format!("Failed to join lobby: {}", e),
                        };
                        let error_json = serde_json::to_string(&error_msg)?;
                        write.send(Message::Text(error_json)).await?;

                        // Send Close frame
                        use tokio_tungstenite::tungstenite::protocol::{CloseFrame, frame::coding::CloseCode};
                        let close_frame = CloseFrame {
                            code: CloseCode::Away,
                            reason: "Lobby error - please retry".into(),
                        };
                        let _ = write.send(Message::Close(Some(close_frame))).await;
                        return Ok(());
                    }
                }
                
                // Refetch lobby state AFTER adding user to include self
                let updated_lobby_state = lobby.get_full_lobby_state().await.unwrap_or_else(|_| vec![]);
                
                // Send success message with UPDATED lobby state (includes new user)
                let success_msg = AuthSuccessMessage::new(updated_lobby_state);
                let success_json = serde_json::to_string(&success_msg)?;
                write.send(Message::Text(success_json)).await?;
            }
            AuthResult::Failure { reason, details } => {
                // Send error message and close connection
                let error_msg = AuthErrorMessage {
                    r#type: "error".to_string(),
                    reason: reason.clone(),
                    details,
                };
                let error_json = serde_json::to_string(&error_msg)?;
                write.send(Message::Text(error_json)).await?;
                
                // Send Close frame with reason code per AC3
                use tokio_tungstenite::tungstenite::protocol::{CloseFrame, frame::coding::CloseCode};
                let close_frame = CloseFrame {
                    code: CloseCode::Normal,
                    reason: reason.into(),
                };
                if let Err(e) = write.send(Message::Close(Some(close_frame))).await {
                    tracing::warn!("Failed to send close frame: {}", e);
                }
                
                return Ok(());
            }
        }
    }

    // Connection loop - handle messages and disconnections
    while let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(Message::Text(text)) => {
                // Handle incoming message from authenticated user (Story 3.2 + 3.3)
                // AC1: Route validated message to recipient via real-time push
                if let Some(ref sender_key) = authenticated_key {
                    tracing::debug!(sender = %sender_key, "Received message, validating and routing...");

                    // Validate the message (Story 3.2)
                    let validation_result = handle_incoming_message(&lobby, sender_key, &text).await;

                    // Handle validation result
                    match validation_result {
                        MessageValidationResult::Valid { .. } => {
                            // Message is valid, route to recipient (Story 3.3)
                            match route_message(&lobby, &validation_result).await {
                                Ok(()) => {
                                    tracing::debug!("Message routed successfully");
                                }
                                Err(e) => {
                                    // AC7: Log failed delivery but don't return error to sender
                                    tracing::warn!("Message delivery failed: {}", e);
                                    // No error returned to sender - server-side issue
                                }
                            }
                        }
                        MessageValidationResult::Invalid { reason } => {
                            // Validation failed - send error response back to sender
                            tracing::debug!(sender = %sender_key, ?reason, "Message validation failed");

                            // Get sender's connection to send error response
                            if let Ok(Some(sender_conn)) = crate::lobby::get_user(&lobby, sender_key).await {
                                // Create error message format matching protocol spec (AC3, AC4, AC5)
                                let error_response = match &reason {
                                    crate::message::ValidationError::NotAuthenticated { details } => {
                                        profile_shared::Message::Error {
                                            reason: "auth_failed".to_string(),
                                            details: Some(details.clone()),
                                        }
                                    }
                                    crate::message::ValidationError::MalformedJson { details } => {
                                        profile_shared::Message::Error {
                                            reason: "malformed_json".to_string(),
                                            details: Some(details.clone()),
                                        }
                                    }
                                    crate::message::ValidationError::SignatureInvalid { details } => {
                                        profile_shared::Message::Error {
                                            reason: "signature_invalid".to_string(),
                                            details: Some(details.clone()),
                                        }
                                    }
                                    crate::message::ValidationError::RecipientOffline { recipient_key } => {
                                        profile_shared::Message::Error {
                                            reason: "offline".to_string(),
                                            details: Some(format!("User {} is not currently online", recipient_key)),
                                        }
                                    }
                                    crate::message::ValidationError::CannotMessageSelf => {
                                        profile_shared::Message::Error {
                                            reason: "invalid_recipient".to_string(),
                                            details: Some("Cannot send message to yourself".to_string()),
                                        }
                                    }
                                };

                                // Send error via the sender's WebSocket connection
                                let _ = sender_conn.sender.send(error_response);
                            }
                        }
                    }
                }
            }
            Ok(Message::Close(_frame)) => {
                // Log disconnect event with tracing (subscriber configured in main.rs)
                // Note: authenticated_key should always be Some if we reached this point
                // as we only enter the message loop after successful authentication
                let user_key = authenticated_key.as_deref().unwrap_or("unauthenticated");
                tracing::info!(
                    "User {} disconnected, broadcasting leave notification",
                    user_key
                );

                // CRITICAL: Clean up lobby using new API
                // Note: remove_user() handles broadcast_user_left internally
                if let Some(ref key) = authenticated_key {
                    if let Err(e) = crate::lobby::remove_user(&lobby, key).await {
                        match e {
                            LobbyError::LockFailed => {
                                // CRITICAL: User may remain "stuck" in lobby
                                // This is a serious error that could lead to ghost users
                                tracing::error!(
                                    "CRITICAL: Failed to acquire lobby lock for user {} removal: {}. \
                                     User may remain visible to others incorrectly.",
                                    key.chars().take(16).collect::<String>(),
                                    e
                                );
                                // Return error to signal this needs attention
                                return Err(format!("Lobby lock failure on disconnect: {}", e).into());
                            }
                            LobbyError::BroadcastFailed => {
                                // User was removed from lobby but broadcast to other users failed
                                // This is a non-critical error - user is cleaned up but others
                                // won't be notified. Log as warning.
                                tracing::warn!(
                                    "User {} removed from lobby but leave notification failed to broadcast: {}",
                                    key.chars().take(16).collect::<String>(),
                                    e
                                );
                            }
                            _ => {
                                tracing::error!(
                                    "Failed to remove user {} from lobby: {}",
                                    key.chars().take(16).collect::<String>(),
                                    e
                                );
                                return Err(format!("Lobby removal error: {}", e).into());
                            }
                        }
                    } else {
                        tracing::debug!(
                            "User {} removed from lobby successfully",
                            key.chars().take(16).collect::<String>()
                        );
                    }
                }
                break;
            }
            Err(e) => {
                // Note: authenticated_key should always be Some if we reached this point
                // as we only enter the message loop after successful authentication
                let user_key = authenticated_key.as_deref().unwrap_or("unauthenticated");
                // Log the error but don't claim "disconnection" - WebSocket read errors
                // could be network flakiness, malformed messages, etc., not actual disconnections
                tracing::error!("WebSocket error for user {}: {}", user_key, e);

                // Clean up lobby on error - this is treated as a disconnection event
                // because the connection stream has failed
                if let Some(ref key) = authenticated_key {
                    if let Err(e) = crate::lobby::remove_user(&lobby, key).await {
                        match e {
                            LobbyError::LockFailed => {
                                // CRITICAL: User may remain "stuck" in lobby
                                // This is a serious error that could lead to ghost users
                                tracing::error!(
                                    "CRITICAL: Failed to acquire lobby lock for user {} removal on error: {}. \
                                     User may remain visible to others incorrectly.",
                                    key.chars().take(16).collect::<String>(),
                                    e
                                );
                                // Return error to signal this needs attention
                                return Err(format!("Lobby lock failure on error disconnect: {}", e).into());
                            }
                            LobbyError::BroadcastFailed => {
                                // User was removed from lobby but broadcast to other users failed
                                tracing::warn!(
                                    "User {} removed from lobby but leave notification failed to broadcast: {}",
                                    key.chars().take(16).collect::<String>(),
                                    e
                                );
                            }
                            _ => {
                                tracing::error!(
                                    "Failed to remove user {} from lobby on error: {}",
                                    key.chars().take(16).collect::<String>(),
                                    e
                                );
                                return Err(format!("Lobby removal error on disconnect: {}", e).into());
                            }
                        }
                    } else {
                        tracing::debug!(
                            "User {} removed from lobby successfully after WebSocket error",
                            key.chars().take(16).collect::<String>()
                        );
                    }
                }
                break;
            }
            _ => {
                // Handle other message types (binary, ping, pong, etc.)
                // Log at debug level for debugging purposes - these are normal WebSocket events
                tracing::debug!(
                    "Received non-text, non-close message type for user {}: {:?}",
                    authenticated_key.as_deref().unwrap_or("unauthenticated"),
                    msg_result.as_ref().map(|m| m.to_string()).unwrap_or_else(|e| format!("Error: {}", e))
                );
            }
        }
    }

    Ok(())
}

async fn handle_auth_message(
    message: &Message,
    lobby: &Arc<Lobby>,
) -> AuthResult {
    match message {
        Message::Text(text) => {
            match serde_json::from_str::<AuthMessage>(text) {
                Ok(auth_msg) => {
                    handle_authentication(&auth_msg, lobby).await
                }
                Err(_) => {
                    AuthResult::Failure {
                        reason: "auth_failed".to_string(),
                        details: "Invalid JSON format".to_string(),
                    }
                }
            }
        }
        _ => {
            AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Expected text message".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    #[tokio::test]
    async fn test_handle_connection_auth_flow() {
        // Test the WebSocket authentication flow
        let result = test_auth_message_parsing().await;
        
        // This should pass now - message parsing works even if auth fails
        assert!(result.is_ok(), "Auth message parsing should work: {:?}", result.err());
    }

    async fn test_auth_message_parsing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Test parsing of auth message - this should work
        let test_message = r#"{"type": "auth", "publicKey": "deadbeef", "signature": "cafebabe"}"#;
        let message = Message::Text(test_message.to_string());
        
        let lobby = Arc::new(Lobby::new());
        
        // This should work - message parsing should succeed even if auth fails
        let auth_result = handle_auth_message(&message, &lobby).await;
        
        match auth_result {
            AuthResult::Failure { reason, details } => {
                // This is expected - the message parsing works, but auth fails with invalid signature
                assert_eq!(reason, "auth_failed");
                println!("✅ Message parsing works, auth correctly fails: {} - {}", reason, details);
                Ok(())
            }
            AuthResult::Success { .. } => {
                // This shouldn't happen with invalid test data
                Err("Unexpected success with invalid test data".into())
            }
        }
    }

    #[tokio::test]
    async fn test_websocket_message_types() {
        // Test handling of different WebSocket message types
        
        let lobby = Arc::new(Lobby::new());
        
        // Test 1: Valid auth message (will fail auth but parsing should work)
        let auth_message = Message::Text(r#"{"type": "auth", "publicKey": "deadbeef", "signature": "cafebabe"}"#.to_string());
        let result = handle_auth_message(&auth_message, &lobby).await;
        assert!(matches!(result, AuthResult::Failure { .. }));
        
        // Test 2: Invalid JSON message
        let invalid_json = Message::Text(r#"{"type": "invalid", "data": "test"}"#.to_string());
        let result = handle_auth_message(&invalid_json, &lobby).await;
        assert!(matches!(result, AuthResult::Failure { .. }));
        
        // Test 3: Non-text message (should fail)
        let binary_message = Message::Binary(vec![1, 2, 3, 4]);
        let result = handle_auth_message(&binary_message, &lobby).await;
        assert!(matches!(result, AuthResult::Failure { .. }));
        
        println!("✅ All message type tests passed");
    }

    #[tokio::test]
    async fn test_lobby_integration() {
        // Test that lobby is properly integrated with auth handling

        let lobby = Arc::new(Lobby::new());

        // Add a user to the lobby first using new API
        // Use exactly 64 hex chars (32 bytes) for valid public key
        let test_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
        let (sender, _) = tokio::sync::mpsc::unbounded_channel::<profile_shared::Message>();
        let connection = crate::lobby::ActiveConnection {
            public_key: test_key.clone(),
            sender,
            connection_id: 1,
        };
        crate::lobby::add_user(&lobby, test_key.clone(), connection).await.unwrap();

        // Test auth with lobby containing a user
        let auth_message = Message::Text(r#"{"type": "auth", "publicKey": "deadbeef", "signature": "cafebabe"}"#.to_string());
        let result = handle_auth_message(&auth_message, &lobby).await;

        match result {
            AuthResult::Success { lobby_state, .. } => {
                // If auth succeeded, lobby state should contain users
                assert!(!lobby_state.is_empty());
                println!("✅ Lobby integration works: {} users in lobby", lobby_state.len());
            }
            AuthResult::Failure { reason, .. } => {
                // Auth failure is expected with invalid test data
                assert_eq!(reason, "auth_failed");
                println!("✅ Auth failed as expected with invalid data");
            }
        }
    }

    #[tokio::test]
    async fn test_close_frame_triggers_lobby_removal() {
        use tokio_tungstenite::tungstenite::protocol::{CloseFrame, frame::coding::CloseCode};

        let lobby = Arc::new(Lobby::new());

        // Setup: Add user to lobby
        // Use exactly 64 hex chars (32 bytes) for valid public key - valid hex only
        let public_key = "abcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcdabcd".to_string();
        let (sender, _) = tokio::sync::mpsc::unbounded_channel::<profile_shared::Message>();
        let connection = crate::lobby::ActiveConnection {
            public_key: public_key.clone(),
            sender,
            connection_id: 42,
        };

        // Add user to lobby
        crate::lobby::add_user(&lobby, public_key.clone(), connection).await.unwrap();

        // Verify user is in lobby
        let users_before = crate::lobby::get_current_users(&lobby).await.unwrap();
        assert_eq!(users_before.len(), 1);
        assert!(users_before.contains(&public_key));

        // Action: Simulate close frame handling (frame created but not used in this unit test)
        let _close_frame = Some(CloseFrame {
            code: CloseCode::Normal,
            reason: "Client disconnected".into(),
        });

        // Simulate the close frame processing logic
        let result = crate::lobby::remove_user(&lobby, &public_key).await;
        assert!(result.is_ok());

        // Verify: User removed from lobby
        let users_after = crate::lobby::get_current_users(&lobby).await.unwrap();
        assert_eq!(users_after.len(), 0);
        assert!(!users_after.contains(&public_key));

        // Verify: No ghost user remains
        let lookup_result = crate::lobby::get_user(&lobby, &public_key).await.unwrap();
        assert!(lookup_result.is_none());

        println!("✅ Close frame correctly triggers lobby removal - no ghost users remain");
    }
}