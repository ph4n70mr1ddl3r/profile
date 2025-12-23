//! Authentication handler for WebSocket connections
//!
//! This module handles user authentication using cryptographic signatures
//! as specified in Story 1.5 requirements.

use profile_shared::verify_signature;
use profile_shared::errors::CryptoError;
use crate::protocol::{AuthMessage, AuthSuccessMessage, AuthErrorMessage};
use crate::lobby::Lobby;
use hex;

/// Authentication result indicating success or failure
#[derive(Debug, Clone)]
pub enum AuthResult {
    Success {
        public_key: Vec<u8>,
        lobby_state: Vec<String>,
    },
    Failure {
        reason: String,
        details: String,
    },
}

/// Handle authentication request from client
///
/// This function:
/// 1. Uses `hex` crate to decode `publicKey` (JSON field) and `signature` from JSON
/// 2. Calls `shared::verify_signature` for the literal string "auth"
/// 3. Returns appropriate success/failure result
pub async fn handle_authentication(
    auth_message: &AuthMessage,
    lobby: &Lobby,
) -> AuthResult {
    // Decode hex-encoded public key
    let public_key = match hex::decode(&auth_message.public_key) {
        Ok(key) => key,
        Err(_) => {
            return AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Invalid hex encoding in publicKey".to_string(),
            };
        }
    };

    // Decode hex-encoded signature
    let signature = match hex::decode(&auth_message.signature) {
        Ok(sig) => sig,
        Err(_) => {
            return AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Invalid hex encoding in signature".to_string(),
            };
        }
    };

    // Verify signature for literal string "auth" using shared crypto module
    let verification_result = verify_signature(&public_key, b"auth", &signature);

    match verification_result {
        Ok(_) => {
            // Signature is valid - user authenticated successfully
            match lobby.get_full_lobby_state().await {
                Ok(lobby_state) => {
                    AuthResult::Success {
                        public_key,
                        lobby_state,
                    }
                }
                Err(_) => {
                    AuthResult::Failure {
                        reason: "auth_failed".to_string(),
                        details: "Failed to get lobby state".to_string(),
                    }
                }
            }
        }
        Err(CryptoError::InvalidSignature(_)) => {
            AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Signature did not verify".to_string(),
            }
        }
        Err(CryptoError::InvalidKey(_)) => {
            AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Invalid public key".to_string(),
            }
        }
        Err(CryptoError::VerificationFailed(_)) => {
            AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Signature verification failed".to_string(),
            }
        }
        Err(_) => {
            AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Authentication error".to_string(),
            }
        }
    }
}

/// Create success response message
pub fn create_success_message(lobby_state: Vec<String>) -> AuthSuccessMessage {
    AuthSuccessMessage::new(lobby_state)
}

/// Create error response message
pub fn create_error_message(reason: String, details: String) -> AuthErrorMessage {
    AuthErrorMessage::new(reason, details)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_authentication_invalid_hex() {
        let auth_message = AuthMessage {
            r#type: "auth".to_string(),
            public_key: "invalid_hex!".to_string(),
            signature: "abc123".to_string(),
        };
        
        let lobby = Lobby::new();
        let result = tokio::runtime::Runtime::new().unwrap().block_on(handle_authentication(&auth_message, &lobby));
        
        match result {
            AuthResult::Failure { reason, details } => {
                assert_eq!(reason, "auth_failed");
                assert!(details.contains("Invalid hex encoding"));
            }
            AuthResult::Success { .. } => {
                panic!("Authentication should have failed with invalid hex");
            }
        }
    }

    #[test]
    fn test_handle_authentication_wrong_signature() {
        let public_key = vec![42u8; 32];
        let wrong_signature = vec![99u8; 64]; // Wrong signature
        
        let auth_message = AuthMessage {
            r#type: "auth".to_string(),
            public_key: hex::encode(&public_key),
            signature: hex::encode(&wrong_signature),
        };
        
        let lobby = Lobby::new();
        let result = tokio::runtime::Runtime::new().unwrap().block_on(handle_authentication(&auth_message, &lobby));
        
        match result {
            AuthResult::Failure { reason, details } => {
                assert_eq!(reason, "auth_failed");
                // The error could be "Signature did not verify" or similar
                assert!(details.contains("verify") || details.contains("Signature") || details.contains("auth"));
            }
            AuthResult::Success { .. } => {
                panic!("Authentication should have failed with wrong signature");
            }
        }
    }

    #[test]
    fn test_message_creation() {
        let lobby_state = vec!["user1".to_string(), "user2".to_string()];
        let success_msg = create_success_message(lobby_state.clone());
        assert_eq!(success_msg.r#type, "auth_success");
        assert_eq!(success_msg.users, lobby_state);
        
        let error_msg = create_error_message("auth_failed".to_string(), "Invalid signature".to_string());
        assert_eq!(error_msg.r#type, "error");
        assert_eq!(error_msg.reason, "auth_failed");
        assert_eq!(error_msg.details, "Invalid signature");
    }
}