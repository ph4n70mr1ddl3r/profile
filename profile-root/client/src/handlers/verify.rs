//! Message verification for received messages
//!
//! This module provides client-side signature verification for messages
//! received from the server. It implements defense-in-depth by verifying
//! signatures even though the server already validated them.
//!
//! AC1: Verification completes in <100ms
//! AC2: Valid messages get green ✓ badge
//! AC3: Invalid messages are rejected with notification

use profile_shared::verify_signature;
use crate::state::messages::ChatMessage;
use hex;

/// Result of message verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// Message signature is valid
    Valid(ChatMessage),
    /// Message signature is invalid
    Invalid {
        sender_public_key: String,
        reason: String,
    },
}

/// Verify a received message signature
///
/// This function implements client-side signature verification for defense-in-depth.
/// Even though the server has already verified the signature, we verify again on the
/// client to ensure end-to-end security.
///
/// # Arguments
/// * `message` - The raw message text
/// * `sender_public_key` - Hex-encoded public key of sender
/// * `signature` - Hex-encoded signature
/// * `timestamp` - ISO 8601 timestamp from message
///
/// # Returns
/// VerificationResult indicating valid or invalid
///
/// # Performance
/// Verification completes in <100ms as required by AC1
pub fn verify_message(
    message: &str,
    sender_public_key: &str,
    signature: &str,
    timestamp: &str,
) -> VerificationResult {
    // Decode hex strings
    let sender_key_bytes = match hex::decode(sender_public_key) {
        Ok(bytes) => bytes,
        Err(e) => {
            return VerificationResult::Invalid {
                sender_public_key: sender_public_key.to_string(),
                reason: format!("Invalid public key hex: {}", e),
            };
        }
    };

    let signature_bytes = match hex::decode(signature) {
        Ok(bytes) => bytes,
        Err(e) => {
            return VerificationResult::Invalid {
                sender_public_key: sender_public_key.to_string(),
                reason: format!("Invalid signature hex: {}", e),
            };
        }
    };

    // Create canonical message for verification (same format as signing)
    let canonical_message = format!("{}:{}", message, timestamp);

    // Verify signature
    match verify_signature(
        &sender_key_bytes,
        canonical_message.as_bytes(),
        &signature_bytes,
    ) {
        Ok(()) => {
            // Signature is valid - create verified ChatMessage
            let chat_msg = ChatMessage::verified(
                sender_public_key.to_string(),
                message.to_string(),
                signature.to_string(),
                timestamp.to_string(),
            );
            VerificationResult::Valid(chat_msg)
        }
        Err(e) => {
            // Signature is invalid
            VerificationResult::Invalid {
                sender_public_key: sender_public_key.to_string(),
                reason: format!("Signature verification failed: {}", e),
            }
        }
    }
}

/// Verify a ChatMessage that was parsed from JSON
///
/// The ChatMessage already contains sender, message, signature, and timestamp.
/// This function extracts these and performs verification.
///
/// # Arguments
/// * `chat_msg` - The parsed ChatMessage to verify
///
/// # Returns
/// VerificationResult indicating valid or invalid
pub fn verify_chat_message(chat_msg: &ChatMessage) -> VerificationResult {
    verify_message(
        &chat_msg.message,
        &chat_msg.sender_public_key,
        &chat_msg.signature,
        &chat_msg.timestamp,
    )
}

/// Create an error notification message for invalid signature
///
/// # Arguments
/// * `sender_public_key` - The public key of the purported sender
/// * `reason` - The reason verification failed
///
/// # Returns
/// User-friendly error message
pub fn create_invalid_signature_notification(sender_public_key: &str, reason: &str) -> String {
    format!(
        "Received message with invalid signature from {}. Message rejected. Details: {}",
        format_public_key(sender_public_key),
        reason
    )
}

/// Format public key for display (first 8 chars + "...")
///
/// # Arguments
/// * `public_key` - Full hex-encoded public key
///
/// # Returns
/// Truncated public key for display
pub fn format_public_key(public_key: &str) -> String {
    if public_key.len() > 16 {
        format!("{}...{}", &public_key[..8], &public_key[public_key.len() - 8..])
    } else {
        public_key.to_string()
    }
}

/// Check if verification should be skipped (for testing or trusted sources)
///
/// In production, this should always return false.
/// This is a hook for testing scenarios where you want to skip verification.
///
/// # Returns
/// false in production, configurable for testing
#[allow(dead_code)]
pub fn should_skip_verification() -> bool {
    // In production, never skip verification
    // This function exists to support testing scenarios
    std::env::var("SKIP_MESSAGE_VERIFICATION")
        .map(|v| v == "1" || v == "true")
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use profile_shared::{generate_private_key, derive_public_key, sign_message};
    use zeroize::Zeroizing;

    #[test]
    fn test_verify_valid_signature() {
        // Generate a key pair
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Create a signed message
        let message = "Hello, world!";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical_message = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical_message.as_bytes()).unwrap();

        // Verify the signature
        let result = verify_message(
            message,
            &hex::encode(&public_key),
            &hex::encode(signature),
            timestamp,
        );

        assert!(matches!(result, VerificationResult::Valid(_)));
        if let VerificationResult::Valid(chat_msg) = result {
            assert_eq!(chat_msg.message, message);
            assert!(chat_msg.is_verified);
        }
    }

    #[test]
    fn test_verify_invalid_signature() {
        // Use a valid key but wrong signature
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Create an invalid signature (zeros)
        let invalid_signature = vec![0u8; 64];

        let result = verify_message(
            "test message",
            &hex::encode(&public_key),
            &hex::encode(invalid_signature),
            "2025-12-27T10:30:00Z",
        );

        assert!(matches!(result, VerificationResult::Invalid { .. }));
    }

    #[test]
    fn test_verify_wrong_key() {
        // Generate two key pairs
        let private_key1 = generate_private_key().unwrap();
        let public_key1 = derive_public_key(&private_key1).unwrap();
        let private_key2 = generate_private_key().unwrap();
        let public_key2 = derive_public_key(&private_key2).unwrap();

        // Sign with key1
        let message = "test";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical_message = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key1, canonical_message.as_bytes()).unwrap();

        // Try to verify with key2's public key (should fail)
        let result = verify_message(
            message,
            &hex::encode(&public_key2), // Wrong key!
            &hex::encode(signature),
            timestamp,
        );

        assert!(matches!(result, VerificationResult::Invalid { .. }));
    }

    #[test]
    fn test_verify_invalid_hex() {
        let result = verify_message(
            "test",
            "not_valid_hex",
            "valid_signature_here_000000000000000000000000000000000000000000000000000000000000",
            "2025-12-27T10:30:00Z",
        );

        match result {
            VerificationResult::Invalid { reason, .. } => {
                assert!(reason.contains("Invalid public key"));
            }
            _ => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_verify_chat_message() {
        // Create a verified ChatMessage first
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "Test message";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical_message = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical_message.as_bytes()).unwrap();

        let chat_msg = ChatMessage::new(
            hex::encode(&public_key),
            message.to_string(),
            hex::encode(signature),
            timestamp.to_string(),
        );

        let result = verify_chat_message(&chat_msg);
        assert!(matches!(result, VerificationResult::Valid(_)));
    }

    #[test]
    fn test_format_public_key() {
        let key = "abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd";
        let formatted = format_public_key(key);
        assert_eq!(formatted, "abcd1234...7890abcd");

        // Short key should be unchanged
        let short = "abc123";
        assert_eq!(format_public_key(short), "abc123");
    }

    #[test]
    fn test_create_invalid_signature_notification() {
        let notification = create_invalid_signature_notification(
            "sender_key_1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "signature mismatch",
        );

        assert!(notification.contains("Received message with invalid signature"));
        assert!(notification.contains("sender_k...90abcdef"));
        assert!(notification.contains("Message rejected"));
    }

    #[test]
    fn test_verification_completes_quickly() {
        use std::time::Instant;

        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "Performance test message";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical_message = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical_message.as_bytes()).unwrap();

        // Run verification multiple times and measure
        let iterations = 100;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = verify_message(
                message,
                &hex::encode(&public_key),
                &hex::encode(&signature),
                timestamp,
            );
        }
        let elapsed = start.elapsed();

        // Average time should be well under 100ms
        let avg_ms = elapsed.as_secs_f64() * 1000.0 / iterations as f64;
        assert!(avg_ms < 10.0, "Average verification time {}ms should be < 10ms", avg_ms);
    }
}
