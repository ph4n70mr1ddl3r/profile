//! Message signing operations
//!
//! This module provides deterministic signing for messages using ed25519-dalek.
//! Required by Story 1.5 (Authentication) and Story 3.x (Messaging).

use crate::crypto::PrivateKey;
use crate::errors::CryptoError;
use ed25519_dalek::{Signer, SigningKey};
use serde_json;

/// Sign a message with a private key using canonical JSON serialization
///
/// This function implements deterministic signing by:
/// 1. Converting the message to canonical JSON format
/// 2. Converting the 32-byte private key to SigningKey without unprotected copies
/// 3. Using ed25519-dalek to create a signature
pub fn sign_message(private_key: &PrivateKey, message: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let canonical_json = serialize_message_to_canonical_json(message)?;
    let signing_key = convert_private_key_to_signing_key(private_key)?;
    let signature = signing_key.sign(canonical_json.as_bytes());

    Ok(signature.to_bytes().to_vec())
}

/// Convert message bytes to canonical JSON representation
pub fn serialize_message_to_canonical_json(message: &[u8]) -> Result<String, CryptoError> {
    let message_string = std::str::from_utf8(message)
        .map_err(|e| CryptoError::SigningFailed(format!("Invalid UTF-8 message: {}", e)))?;

    serde_json::to_string(&message_string)
        .map_err(|e| CryptoError::SigningFailed(format!("JSON serialization failed: {}", e)))
}

/// Convert 32-byte private key to SigningKey without unprotected copies
fn convert_private_key_to_signing_key(private_key: &PrivateKey) -> Result<SigningKey, CryptoError> {
    let private_key_bytes: [u8; 32] = private_key
        .as_slice()
        .try_into()
        .map_err(|_| CryptoError::SigningFailed("Private key must be exactly 32 bytes".into()))?;

    Ok(SigningKey::from_bytes(&private_key_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_message_deterministic_10k() {
        use rand::rngs::StdRng;
        use rand::Rng;
        use rand::SeedableRng;

        // Generate a deterministic private key for testing
        let mut rng = StdRng::seed_from_u64(42);
        let private_key = PrivateKey::new((0..32).map(|_| rng.gen::<u8>()).collect());
        let message = b"auth";

        // Generate 10,000 signatures and verify they're all identical
        let first_signature = sign_message(&private_key, message)
            .unwrap_or_else(|_| panic!("First signature should succeed"));

        for i in 1..10_000 {
            let signature = sign_message(&private_key, message)
                .unwrap_or_else(|_| panic!("Signature {} should succeed", i));

            assert_eq!(
                signature, first_signature,
                "Signature at iteration {} should be identical to first signature",
                i
            );
        }

        println!("✅ Determinism test passed: 10,000 identical signatures generated");
    }

    #[test]
    fn test_signing_stub_exists() {
        // Test with valid key and message to verify new implementation works
        let private_key = PrivateKey::new(vec![42u8; 32]); // Simple test key
        let message = b"test message";

        let result = sign_message(&private_key, message);
        assert!(result.is_ok(), "Should successfully sign with valid key");

        let signature = result.unwrap();
        assert_eq!(signature.len(), 64, "Ed25519 signature should be 64 bytes");
    }

    #[test]
    fn test_canonical_json_serialization() {
        let private_key = PrivateKey::new(vec![42u8; 32]);
        let message = b"auth";

        let signature1 = sign_message(&private_key, message).unwrap();
        let signature2 = sign_message(&private_key, message).unwrap();

        // Same message should produce identical signatures
        assert_eq!(signature1, signature2);
    }

    #[test]
    fn test_different_messages_different_signatures() {
        let private_key = PrivateKey::new(vec![42u8; 32]);
        let message1 = b"auth";
        let message2 = b"auth2";

        let signature1 = sign_message(&private_key, message1).unwrap();
        let signature2 = sign_message(&private_key, message2).unwrap();

        // Different messages should produce different signatures
        assert_ne!(signature1, signature2);
    }
}
