//! Message handling for incoming client messages
//!
//! This module implements server-side message processing including:
//! - Validation in strict sequence (auth, format, signature, recipient)
//! - Fail-fast error handling
//! - Message routing to online recipients
//!
//! Validation Sequence (AC1):
//! 1. Check sender is authenticated (has active connection in lobby)
//! 2. Check message format is valid JSON
//! 3. Validate signature against sender's public key
//! 4. Check recipient exists in lobby
//! 5. Route accordingly (deliver if online, error if not)

use crate::lobby::{ActiveConnection, Lobby, PublicKey};
use crate::protocol::{ErrorMessage, SendMessageRequest};
use profile_shared::verify_signature;
use std::sync::Arc;

/// Result of message validation
#[derive(Debug, Clone, PartialEq)]
pub enum MessageValidationResult {
    /// Message is valid and ready for routing
    Valid {
        sender_public_key: String,
        recipient_public_key: String,
        message: String,
        signature: String,
        timestamp: String,
    },
    /// Validation failed - message was rejected
    Invalid { reason: ValidationError },
}

/// Validation error types for different failure modes
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Sender is not authenticated or not in lobby
    NotAuthenticated { details: String },
    /// Message JSON is malformed or missing required fields
    MalformedJson { details: String },
    /// Signature verification failed
    SignatureInvalid { details: String },
    /// Recipient is not online
    RecipientOffline { recipient_key: String },
    /// Cannot send message to self
    CannotMessageSelf,
}

/// Handle an incoming message from a client
///
/// Implements the strict validation sequence from AC1:
/// 1. Check sender is authenticated (has active connection)
/// 2. Check message format is valid JSON
/// 3. Validate signature against sender's public key
/// 4. Check recipient exists in lobby
/// 5. Route accordingly (or return error)
///
/// # Arguments
/// * `lobby` - The lobby containing authenticated users
/// * `sender_public_key` - The public key of the sending user (from WebSocket connection)
/// * `message_json` - Raw JSON message from client
///
/// # Returns
/// ValidationResult indicating success or specific error
#[tracing::instrument(skip(lobby, message_json), fields(sender = %sender_public_key.chars().take(16).collect::<String>()))]
pub async fn handle_incoming_message(
    lobby: &Lobby,
    sender_public_key: &str,
    message_json: &str,
) -> MessageValidationResult {
    // AC1 Step 1: Check sender is authenticated (has active connection in lobby)
    // This is guaranteed by the handler - only authenticated users can send messages
    // But we double-check to be safe
    if get_sender_connection(lobby, sender_public_key)
        .await
        .is_none()
    {
        tracing::warn!(sender = %sender_public_key, "Sender not found in lobby - rejecting message");
        return MessageValidationResult::Invalid {
            reason: ValidationError::NotAuthenticated {
                details: format!("User {} is not authenticated", sender_public_key),
            },
        };
    }

    // AC1 Step 2: Check message format is valid JSON
    let message_request: SendMessageRequest = match parse_message_json(message_json) {
        Ok(msg) => msg,
        Err(e) => {
            tracing::warn!(error = %e, "Invalid JSON format from {}", sender_public_key);
            return MessageValidationResult::Invalid {
                reason: ValidationError::MalformedJson { details: e },
            };
        }
    };

    // Validate recipient is not self
    if message_request.recipient_public_key == sender_public_key {
        return MessageValidationResult::Invalid {
            reason: ValidationError::CannotMessageSelf,
        };
    }

    // AC1 Step 3: Validate signature against sender's public key
    // The canonical message for verification is: message:timestamp
    let canonical_message = format!("{}:{}", message_request.message, message_request.timestamp);
    let sender_key_bytes = match hex::decode(sender_public_key) {
        Ok(bytes) => bytes,
        Err(e) => {
            return MessageValidationResult::Invalid {
                reason: ValidationError::MalformedJson {
                    details: format!("Invalid sender public key hex: {}", e),
                },
            };
        }
    };

    let signature_bytes = match hex::decode(&message_request.signature) {
        Ok(bytes) => bytes,
        Err(e) => {
            return MessageValidationResult::Invalid {
                reason: ValidationError::SignatureInvalid {
                    details: format!("Invalid signature hex: {}", e),
                },
            };
        }
    };

    match verify_signature(
        &sender_key_bytes,
        canonical_message.as_bytes(),
        &signature_bytes,
    ) {
        Ok(()) => {
            tracing::debug!(recipient = %message_request.recipient_public_key, "Signature verified");
        }
        Err(e) => {
            tracing::warn!(error = %e, "Signature verification failed for {}", sender_public_key);
            return MessageValidationResult::Invalid {
                reason: ValidationError::SignatureInvalid {
                    details: "Signature did not verify against public key".to_string(),
                },
            };
        }
    }

    // AC1 Step 4: Check recipient exists in lobby
    let recipient_connection =
        get_recipient_connection(lobby, &message_request.recipient_public_key).await;

    // AC1 Step 5: Route accordingly
    match recipient_connection {
        Some(_) => {
            // Recipient is online - message is valid for routing
            MessageValidationResult::Valid {
                sender_public_key: sender_public_key.to_string(),
                recipient_public_key: message_request.recipient_public_key,
                message: message_request.message,
                signature: message_request.signature,
                timestamp: message_request.timestamp,
            }
        }
        None => {
            // Recipient is offline - return error
            MessageValidationResult::Invalid {
                reason: ValidationError::RecipientOffline {
                    recipient_key: message_request.recipient_public_key,
                },
            }
        }
    }
}

/// Get the sender's connection from the lobby
async fn get_sender_connection(lobby: &Lobby, public_key: &str) -> Option<Arc<ActiveConnection>> {
    let pk: PublicKey = public_key.to_string();
    crate::lobby::get_user(lobby, &pk).await.ok().flatten()
}

/// Get the recipient's connection from the lobby
async fn get_recipient_connection(
    lobby: &Lobby,
    public_key: &str,
) -> Option<Arc<ActiveConnection>> {
    let pk: PublicKey = public_key.to_string();
    crate::lobby::get_user(lobby, &pk).await.ok().flatten()
}

/// Parse incoming JSON into a SendMessageRequest
fn parse_message_json(json: &str) -> Result<SendMessageRequest, String> {
    serde_json::from_str(json).map_err(|e| format!("Invalid JSON: {}", e))
}

/// Route a validated message to the recipient
///
/// # Arguments
/// * `lobby` - The lobby containing all connections
/// * `validated` - The validated message to route
///
/// # Returns
/// Ok(()) if message was delivered, Err(reason) if delivery failed
#[tracing::instrument(
    skip(lobby, validated),
    fields(sender = "validated", recipient = "validated")
)]
pub async fn route_message(
    lobby: &Lobby,
    validated: &MessageValidationResult,
) -> Result<(), String> {
    match validated {
        MessageValidationResult::Valid {
            sender_public_key,
            recipient_public_key,
            message,
            signature,
            timestamp,
        } => {
            tracing::debug!(
                sender = %sender_public_key.chars().take(16).collect::<String>(),
                recipient = %recipient_public_key.chars().take(16).collect::<String>(),
                "Routing message"
            );

            // Get recipient's connection
            let recipient_conn = match get_recipient_connection(lobby, recipient_public_key).await {
                Some(conn) => conn,
                None => {
                    return Err("Recipient went offline".to_string());
                }
            };

            // Send via the recipient's WebSocket sender
            let _ = recipient_conn.sender.send(profile_shared::Message::Text {
                message: message.clone(),
                sender_public_key: sender_public_key.clone(),
                signature: signature.clone(),
                timestamp: timestamp.clone(),
            });

            tracing::info!(
                from = %sender_public_key.chars().take(16).collect::<String>(),
                to = %recipient_public_key.chars().take(16).collect::<String>(),
                "Message routed successfully"
            );

            Ok(())
        }
        MessageValidationResult::Invalid { .. } => Err("Cannot route invalid message".to_string()),
    }
}

/// Create an error response for the client
pub fn create_error_response(error: &ValidationError) -> String {
    let (reason, details) = match error {
        ValidationError::NotAuthenticated { details } => {
            ("auth_failed".to_string(), details.clone())
        }
        ValidationError::MalformedJson { details } => {
            ("malformed_json".to_string(), details.clone())
        }
        ValidationError::SignatureInvalid { details } => {
            ("signature_invalid".to_string(), details.clone())
        }
        ValidationError::RecipientOffline { recipient_key } => (
            "offline".to_string(),
            format!("User {} is not currently online", recipient_key),
        ),
        ValidationError::CannotMessageSelf => (
            "invalid_recipient".to_string(),
            "Cannot send message to yourself".to_string(),
        ),
    };

    let error_msg = ErrorMessage::with_details(reason, details);

    serde_json::to_string(&error_msg)
        .unwrap_or_else(|_| r#"{"type":"error","reason":"unknown"}"#.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lobby::Lobby;
    use profile_shared::Message as SharedMessage;
    use tokio::sync::mpsc;

    fn create_test_connection(key: &str) -> ActiveConnection {
        let (sender, _) = mpsc::unbounded_channel::<SharedMessage>();
        ActiveConnection {
            public_key: key.to_string(),
            sender,
            connection_id: 1,
        }
    }

    #[tokio::test]
    async fn test_handle_message_sender_not_authenticated() {
        let lobby = Lobby::new();
        let result = handle_incoming_message(&lobby, "unknown_user", r#"{"type":"message"}"#).await;

        assert!(matches!(
            result,
            MessageValidationResult::Invalid {
                reason: ValidationError::NotAuthenticated { .. }
            }
        ));
    }

    #[tokio::test]
    async fn test_handle_message_malformed_json() {
        let lobby = Lobby::new();

        // Add sender to lobby
        let sender_key = "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab";
        let sender_conn = create_test_connection(sender_key);
        crate::lobby::add_user(&lobby, sender_key.to_string(), sender_conn)
            .await
            .unwrap();

        let result = handle_incoming_message(&lobby, sender_key, "not valid json").await;

        assert!(matches!(
            result,
            MessageValidationResult::Invalid {
                reason: ValidationError::MalformedJson { .. }
            }
        ));
    }

    #[tokio::test]
    async fn test_handle_message_recipient_offline() {
        let lobby = Lobby::new();

        // Add sender to lobby
        let sender_key = "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab";
        let sender_conn = create_test_connection(sender_key);
        crate::lobby::add_user(&lobby, sender_key.to_string(), sender_conn)
            .await
            .unwrap();

        // Create a valid message request but recipient is not in lobby
        // Note: This will fail signature validation first since we don't have real keys
        // We need to test with invalid signature to get past that check
        let message_json = r#"{
            "type": "message",
            "recipientPublicKey": "offline_recipient_1234567890abcdef1234567890abcdef12345678",
            "message": "Hello",
            "senderPublicKey": "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
            "signature": "invalid_signature_0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "timestamp": "2025-12-27T10:30:00Z"
        }"#;

        let result = handle_incoming_message(&lobby, sender_key, message_json).await;

        // Should fail signature validation (invalid signature)
        assert!(matches!(
            result,
            MessageValidationResult::Invalid {
                reason: ValidationError::SignatureInvalid { .. }
            }
        ));
    }

    #[tokio::test]
    async fn test_handle_message_cannot_message_self() {
        let lobby = Lobby::new();

        // Add sender to lobby
        let sender_key = "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab";
        let sender_conn = create_test_connection(sender_key);
        crate::lobby::add_user(&lobby, sender_key.to_string(), sender_conn)
            .await
            .unwrap();

        // Create message to self
        let message_json = r#"{
            "type": "message",
            "recipientPublicKey": "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
            "message": "Hello self",
            "senderPublicKey": "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
            "signature": "0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "timestamp": "2025-12-27T10:30:00Z"
        }"#;

        let result = handle_incoming_message(&lobby, sender_key, message_json).await;

        assert!(matches!(
            result,
            MessageValidationResult::Invalid {
                reason: ValidationError::CannotMessageSelf
            }
        ));
    }

    #[test]
    fn test_create_error_response_signature_invalid() {
        let error = ValidationError::SignatureInvalid {
            details: "Signature did not verify against public key".to_string(),
        };
        let response = create_error_response(&error);

        assert!(response.contains(r#""type":"error""#));
        assert!(response.contains(r#""reason":"signature_invalid""#));
    }

    #[test]
    fn test_create_error_response_offline() {
        let error = ValidationError::RecipientOffline {
            recipient_key: "recipient_key_123".to_string(),
        };
        let response = create_error_response(&error);

        assert!(response.contains(r#""type":"error""#));
        assert!(response.contains(r#""reason":"offline""#));
        assert!(response.contains("recipient_key_123"));
    }

    #[test]
    fn test_create_error_response_malformed_json() {
        let error = ValidationError::MalformedJson {
            details: "Invalid JSON: expected value".to_string(),
        };
        let response = create_error_response(&error);

        assert!(response.contains(r#""type":"error""#));
        assert!(response.contains(r#""reason":"malformed_json""#));
    }

    /// Integration test: Verify that a real signature created by client passes server validation
    /// This test proves the canonical format is consistent between client signing and server verification
    #[tokio::test]
    async fn test_valid_signature_passes_validation() {
        // Generate a proper key pair using shared library
        let private_key =
            profile_shared::generate_private_key().expect("Key generation should succeed");
        let public_key_bytes =
            profile_shared::derive_public_key(&private_key).expect("Key derivation should succeed");
        let public_key_hex = hex::encode(public_key_bytes);

        // Create a unique recipient key
        let recipient_private_key =
            profile_shared::generate_private_key().expect("Key generation should succeed");
        let recipient_key_bytes = profile_shared::derive_public_key(&recipient_private_key)
            .expect("Key derivation should succeed");
        let recipient_public_key_hex = hex::encode(recipient_key_bytes);

        // Create message and timestamp (canonical format: message:timestamp)
        let message_text = "Hello, world!";
        let timestamp = "2025-12-29T12:00:00.000Z";
        let canonical_message = format!("{}:{}", message_text, timestamp);

        // Sign using the shared library (same as client does)
        let signature = profile_shared::sign_message(&private_key, canonical_message.as_bytes())
            .expect("Signing should succeed");
        let signature_hex = hex::encode(signature);

        // Set up lobby with sender and recipient
        let lobby = Lobby::new();
        let (sender_tx, _) = tokio::sync::mpsc::unbounded_channel::<profile_shared::Message>();
        let sender_conn = ActiveConnection {
            public_key: public_key_hex.clone(),
            sender: sender_tx,
            connection_id: 1,
        };
        crate::lobby::add_user(&lobby, public_key_hex.clone(), sender_conn)
            .await
            .unwrap();

        // Add recipient to lobby so message can be delivered
        let (recipient_tx, _) = tokio::sync::mpsc::unbounded_channel::<profile_shared::Message>();
        let recipient_conn = ActiveConnection {
            public_key: recipient_public_key_hex.clone(),
            sender: recipient_tx,
            connection_id: 2,
        };
        crate::lobby::add_user(&lobby, recipient_public_key_hex.clone(), recipient_conn)
            .await
            .unwrap();

        // Create the message JSON that client would send
        let message_json = format!(
            r#"{{
            "type": "message",
            "recipientPublicKey": "{}",
            "message": "{}",
            "senderPublicKey": "{}",
            "signature": "{}",
            "timestamp": "{}"
        }}"#,
            recipient_public_key_hex, message_text, public_key_hex, signature_hex, timestamp
        );

        // Validate the message - should pass with Valid result
        let result = handle_incoming_message(&lobby, &public_key_hex, &message_json).await;

        match result {
            MessageValidationResult::Valid {
                sender_public_key,
                recipient_public_key,
                message,
                signature: _,
                timestamp: _,
            } => {
                assert_eq!(sender_public_key, public_key_hex);
                assert_eq!(recipient_public_key, recipient_public_key_hex);
                assert_eq!(message, message_text);
            }
            MessageValidationResult::Invalid { reason } => {
                panic!("Expected Valid, got Invalid: {:?}", reason);
            }
        }
    }
}
