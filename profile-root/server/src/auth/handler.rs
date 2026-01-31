//! Authentication handler for WebSocket connections
//!
//! This module handles user authentication using cryptographic signatures
//! as specified in Story 1.5 requirements.

use crate::lobby::Lobby;
use crate::protocol::{AuthErrorMessage, AuthMessage, AuthSuccessMessage};
use hex;
use profile_shared::errors::CryptoError;
use profile_shared::{verify_signature, PublicKey};

/// Authentication result indicating success or failure
#[derive(Debug, Clone)]
pub enum AuthResult {
    Success {
        public_key: PublicKey,
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
/// 1. Validates input lengths and formats to prevent DoS attacks
/// 2. Uses `hex` crate to decode `publicKey` (JSON field) and `signature` from JSON
/// 3. Calls `shared::verify_signature` for literal string "auth"
/// 4. Returns appropriate success/failure result
pub async fn handle_authentication(auth_message: &AuthMessage, lobby: &Lobby) -> AuthResult {
    // Validate input lengths to prevent DoS attacks
    if auth_message.public_key.len() > 1024 {
        return AuthResult::Failure {
            reason: "auth_failed".to_string(),
            details: "Public key too long (max 1024 characters)".to_string(),
        };
    }

    if auth_message.signature.len() > 2048 {
        return AuthResult::Failure {
            reason: "auth_failed".to_string(),
            details: "Signature too long (max 2048 characters)".to_string(),
        };
    }

    // Validate hex format of public key
    if !auth_message
        .public_key
        .chars()
        .all(|c| c.is_ascii_hexdigit())
    {
        return AuthResult::Failure {
            reason: "auth_failed".to_string(),
            details: "Public key must be hexadecimal (0-9, a-f)".to_string(),
        };
    }

    // Validate minimum length for ed25519 public keys (64 hex chars = 32 bytes)
    if auth_message.public_key.len() < 64 {
        return AuthResult::Failure {
            reason: "auth_failed".to_string(),
            details: "Public key too short (must be 64 hexadecimal characters)".to_string(),
        };
    }

    // Decode hex-encoded public key
    let public_key = match hex::decode(&auth_message.public_key) {
        Ok(key) => {
            // Validate public key length (ed25519 keys are 32 bytes)
            if key.len() != 32 {
                return AuthResult::Failure {
                    reason: "auth_failed".to_string(),
                    details: "Invalid public key length (must be 32 bytes)".to_string(),
                };
            }
            key
        }
        Err(_) => {
            return AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Invalid hex encoding in publicKey".to_string(),
            };
        }
    };

    // Decode hex-encoded signature
    let signature = match hex::decode(&auth_message.signature) {
        Ok(sig) => {
            // Validate signature length (ed25519 signatures are 64 bytes)
            if sig.len() != 64 {
                return AuthResult::Failure {
                    reason: "auth_failed".to_string(),
                    details: "Invalid signature length (must be 64 bytes)".to_string(),
                };
            }
            sig
        }
        Err(_) => {
            return AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Invalid hex encoding in signature".to_string(),
            };
        }
    };

    // Convert Vec<u8> to PublicKey for verification
    let public_key_wrapper = match PublicKey::new(public_key) {
        Ok(key) => key,
        Err(_) => {
            return AuthResult::Failure {
                reason: "auth_failed".to_string(),
                details: "Invalid public key format".to_string(),
            };
        }
    };

    // Verify signature for literal string "auth" using shared crypto module
    let verification_result = verify_signature(&public_key_wrapper, b"auth", &signature);

    match verification_result {
        Ok(_) => {
            // Signature is valid - user authenticated successfully
            match lobby.get_full_lobby_state().await {
                Ok(lobby_state) => AuthResult::Success {
                    public_key: public_key_wrapper,
                    lobby_state,
                },
                Err(_) => AuthResult::Failure {
                    reason: "auth_failed".to_string(),
                    details: "Failed to get lobby state".to_string(),
                },
            }
        }
        Err(CryptoError::InvalidSignature(_)) => AuthResult::Failure {
            reason: "auth_failed".to_string(),
            details: "Signature did not verify".to_string(),
        },
        Err(CryptoError::InvalidKey(_)) => AuthResult::Failure {
            reason: "auth_failed".to_string(),
            details: "Invalid public key".to_string(),
        },
        Err(CryptoError::VerificationFailed(_)) => AuthResult::Failure {
            reason: "auth_failed".to_string(),
            details: "Signature verification failed".to_string(),
        },
        Err(e) => AuthResult::Failure {
            reason: "auth_failed".to_string(),
            details: format!("Authentication error: {}", e),
        },
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
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_authentication(&auth_message, &lobby));

        match result {
            AuthResult::Failure { reason, details } => {
                assert_eq!(reason, "auth_failed");
                assert!(details.contains("hexadecimal"));
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
        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(handle_authentication(&auth_message, &lobby));

        match result {
            AuthResult::Failure { reason, details } => {
                assert_eq!(reason, "auth_failed");
                // The error could be "Signature did not verify" or similar
                assert!(
                    details.contains("verify")
                        || details.contains("Signature")
                        || details.contains("auth")
                );
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

        let error_msg =
            create_error_message("auth_failed".to_string(), "Invalid signature".to_string());
        assert_eq!(error_msg.r#type, "error");
        assert_eq!(error_msg.reason, "auth_failed");
        assert_eq!(error_msg.details, "Invalid signature");
    }
}
