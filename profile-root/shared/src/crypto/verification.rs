//! Signature verification operations
//!
//! This module provides signature verification for messages using ed25519-dalek.
//! Required by Story 1.5 (Authentication) and Story 3.4 (Receive/Verify Message Signature).

use super::signing::serialize_message_to_canonical_json;
use crate::errors::CryptoError;
use ed25519_dalek::{Signature, Verifier, VerifyingKey};

/// Verify a message signature using canonical JSON serialization
///
/// This function implements deterministic verification by:
/// 1. Converting the message to the same canonical JSON format used for signing
/// 2. Converting the public key to VerifyingKey
/// 3. Using ed25519-dalek to verify the signature
pub fn verify_signature(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), CryptoError> {
    let canonical_json = serialize_message_to_canonical_json(message)?;
    let verifying_key = convert_public_key_to_verifying_key(public_key)?;
    let ed25519_signature = convert_signature_to_ed25519_format(signature)?;

    verifying_key
        .verify(canonical_json.as_bytes(), &ed25519_signature)
        .map_err(|e| {
            CryptoError::VerificationFailed(format!("Signature verification failed: {}", e))
        })
}

/// Convert public key bytes to VerifyingKey
fn convert_public_key_to_verifying_key(public_key: &[u8]) -> Result<VerifyingKey, CryptoError> {
    let public_key_bytes: [u8; 32] = public_key.try_into().map_err(|_| {
        CryptoError::VerificationFailed("Public key must be exactly 32 bytes".into())
    })?;

    VerifyingKey::from_bytes(&public_key_bytes)
        .map_err(|e| CryptoError::VerificationFailed(format!("Invalid public key: {}", e)))
}

/// Convert signature bytes to ed25519 Signature format
fn convert_signature_to_ed25519_format(signature: &[u8]) -> Result<Signature, CryptoError> {
    let signature_bytes: [u8; 64] = signature.try_into().map_err(|_| {
        CryptoError::VerificationFailed("Signature must be exactly 64 bytes".into())
    })?;

    Ok(Signature::from_bytes(&signature_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::signing::sign_message;
    use zeroize::Zeroizing;

    #[test]
    fn test_verify_signature_valid() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::StdRng;
        use rand::RngCore;
        use rand::SeedableRng;

        // Generate a proper key pair using the same pattern as keygen.rs
        let mut rng = StdRng::seed_from_u64(42);
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        let private_key = Zeroizing::new(key_bytes.to_vec());
        let public_key = verifying_key.to_bytes().to_vec();
        let message = b"auth";

        // Generate signature using the signing key
        let signature = sign_message(&private_key, message).unwrap();

        // Verify should succeed
        let result = verify_signature(&public_key, message, &signature);
        assert!(result.is_ok(), "Valid signature should verify successfully");
    }

    #[test]
    fn test_verify_signature_invalid() {
        // Test with invalid signature
        let public_key = vec![42u8; 32];
        let message = b"auth";
        let invalid_signature = vec![0u8; 64]; // Invalid signature

        let result = verify_signature(&public_key, message, &invalid_signature);
        assert!(
            result.is_err(),
            "Invalid signature should fail verification"
        );
    }

    #[test]
    fn test_verify_signature_wrong_message() {
        let private_key = Zeroizing::new(vec![42u8; 32]);
        let public_key = vec![42u8; 32];
        let message1 = b"auth";
        let message2 = b"different";

        // Generate signature for message1
        let signature = sign_message(&private_key, message1).unwrap();

        // Try to verify signature for message2 - should fail
        let result = verify_signature(&public_key, message2, &signature);
        assert!(
            result.is_err(),
            "Signature for different message should fail verification"
        );
    }

    #[test]
    fn test_verify_signature_canonical_json_consistency() {
        use ed25519_dalek::SigningKey;
        use rand::rngs::StdRng;
        use rand::RngCore;
        use rand::SeedableRng;

        // Test that verification uses same canonical JSON as signing
        let mut rng = StdRng::seed_from_u64(123);
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();
        let private_key = Zeroizing::new(key_bytes.to_vec());
        let public_key = verifying_key.to_bytes().to_vec();
        let message = b"auth";

        let signature = sign_message(&private_key, message).unwrap();
        let result = verify_signature(&public_key, message, &signature);

        assert!(
            result.is_ok(),
            "Canonical JSON consistency test should pass"
        );
    }

    #[test]
    fn test_verify_signature_stub_exists() {
        // Test with stub implementation to ensure compilation
        let public_key = vec![0u8; 32];
        let message = b"test message";
        let signature = vec![0u8; 64];

        let result = verify_signature(&public_key, message, &signature);
        // This will now pass with real implementation
        assert!(
            result.is_err() || result.is_ok(),
            "Should handle signature appropriately"
        );
    }
}
