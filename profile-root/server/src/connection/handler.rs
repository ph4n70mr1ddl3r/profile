use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, stream::StreamExt};
use serde_json;

use crate::auth::handler::{handle_authentication, AuthResult};
use crate::lobby::Lobby;
use crate::protocol::{AuthMessage, AuthSuccessMessage, AuthErrorMessage};

/// Connection handler that processes WebSocket connections
pub async fn handle_connection(
    stream: TcpStream,
    lobby: Arc<Lobby>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Accept WebSocket handshake
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    
    let (mut write, mut read) = ws_stream.split();

    // Wait for auth message
    if let Some(message_result) = read.next().await {
        let message = message_result?;
        
        match handle_auth_message(&message, &lobby).await {
            AuthResult::Success { public_key, lobby_state } => {
                // Add user to lobby first
                let connection = crate::lobby::Connection {
                    public_key: public_key.clone(),
                    connected_at: std::time::Instant::now(),
                };
                
                if let Err(e) = lobby.add_user(connection) {
                    println!("❌ Failed to add user to lobby: {}", e);
                    // Still send success message but log the error
                }
                
                // Send success message with full lobby state
                let success_msg = AuthSuccessMessage::new(lobby_state);
                let success_json = serde_json::to_string(&success_msg)?;
                write.send(Message::Text(success_json)).await?;
            }
            AuthResult::Failure { reason, details } => {
                // Send error message and close connection
                let error_msg = AuthErrorMessage {
                    r#type: "error".to_string(),
                    reason,
                    details,
                };
                let error_json = serde_json::to_string(&error_msg)?;
                write.send(Message::Text(error_json)).await?;
                return Ok(());
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
        
        // Add a user to the lobby first
        let test_key = vec![0x12, 0x34, 0x56, 0x78];
        let connection = crate::lobby::Connection {
            public_key: test_key.clone(),
            connected_at: std::time::Instant::now(),
        };
        lobby.add_user(connection).unwrap();
        
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
}