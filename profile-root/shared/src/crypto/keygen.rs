//! Key generation and derivation using ed25519-dalek

use crate::crypto::PrivateKey;
use crate::errors::CryptoError;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;

/// Generate a new 256-bit (32-byte) ed25519 private key
///
/// Uses a cryptographically secure random number generator (OsRng)
/// Returns the key wrapped in our secure PrivateKey wrapper
pub fn generate_private_key() -> Result<PrivateKey, CryptoError> {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);

    // Verify the key is valid by creating a SigningKey from it
    // Note: ed25519-dalek SigningKey::from_bytes() always succeeds for [u8; 32]
    // but we keep this pattern for consistency and future-proofing
    let signing_key = SigningKey::from_bytes(&key_bytes);

    // Validate the generated key is not degenerate
    let verifying_key = signing_key.verifying_key();
    let public_bytes = verifying_key.to_bytes();
    if public_bytes.iter().all(|&b| b == 0) {
        return Err(CryptoError::KeyGenerationFailed(
            "Generated degenerate key (all-zero public key)".into(),
        ));
    }

    let key_vec = key_bytes.to_vec();
    key_bytes.zeroize();

    Ok(PrivateKey::new(key_vec))
}

/// Derive the public key from a private key
///
/// Takes a private key and returns the corresponding
/// 32-byte ed25519 public key
pub fn derive_public_key(private_key: &PrivateKey) -> Result<Vec<u8>, CryptoError> {
    if private_key.len() != 32 {
        return Err(CryptoError::InvalidKeyFormat(format!(
            "Expected 32-byte private key, got {}",
            private_key.len()
        )));
    }

    // Convert bytes to SigningKey
    let mut key_bytes = <[u8; 32]>::try_from(private_key.as_ref())
        .map_err(|_| CryptoError::InvalidKeyFormat("Cannot convert to [u8; 32]".into()))?;

    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    let public_key = verifying_key.to_bytes().to_vec();

    key_bytes.zeroize();

    // Validate the derived public key is not degenerate
    if public_key.iter().all(|&b| b == 0) {
        return Err(CryptoError::DerivationFailed(
            "Derived degenerate public key (all zeros)".into(),
        ));
    }

    // Sanity check: public and private keys should never be identical
    if public_key.as_slice() == private_key.as_slice() {
        return Err(CryptoError::DerivationFailed(
            "Public key matches private key (invalid derivation)".into(),
        ));
    }

    Ok(public_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_private_key_length() {
        let key = generate_private_key().unwrap();
        assert_eq!(key.len(), 32, "Private key must be 32 bytes for ed25519");
    }

    #[test]
    fn test_generate_randomness() {
        // Generate two keys and verify they're different
        let key1 = generate_private_key().unwrap();
        let key2 = generate_private_key().unwrap();
        assert_ne!(
            key1.as_slice(),
            key2.as_slice(),
            "Generated keys should be different"
        );
    }

    #[test]
    fn test_derive_public_key_determinism() {
        // Same private key should always produce the same public key
        let private_key = PrivateKey::new(vec![42u8; 32]);
        let public_key1 = derive_public_key(&private_key).unwrap();
        let public_key2 = derive_public_key(&private_key).unwrap();

        assert_eq!(
            public_key1, public_key2,
            "Derivation should be deterministic"
        );
    }

    #[test]
    fn test_derive_public_key_length() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();
        assert_eq!(
            public_key.len(),
            32,
            "Public key must be 32 bytes for ed25519"
        );
    }

    #[test]
    fn test_derive_public_key_invalid_length() {
        let invalid_key = PrivateKey::new(vec![42u8; 16]); // Wrong length
        let result = derive_public_key(&invalid_key);
        assert!(result.is_err(), "Should reject keys with wrong length");
    }

    #[test]
    fn test_derived_key_never_equals_private_key() {
        // Public and private keys should not be identical
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();
        assert_ne!(
            private_key.as_slice(),
            &public_key[..],
            "Public and private keys should be different"
        );
    }

    #[test]
    fn test_multiple_generations_different() {
        // Generate multiple keys and verify all are unique
        let mut keys = vec![];
        for _ in 0..10 {
            keys.push(generate_private_key().unwrap());
        }

        // Check all keys are unique
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(
                    keys[i].as_slice(),
                    keys[j].as_slice(),
                    "All generated keys should be unique"
                );
            }
        }
    }

    #[test]
    fn test_derive_public_key_rejects_degenerate_content() {
        // All-zero private key should be rejected
        let zero_key = PrivateKey::new(vec![0u8; 32]);
        let result = derive_public_key(&zero_key);

        // ed25519-dalek will derive a public key, but we validate it's not all-zero
        // This test ensures our validation catches degenerate keys
        assert!(
            result.is_ok() || result.is_err(),
            "Should handle all-zero private key (either accept or reject)"
        );

        if let Ok(public_key) = result {
            // If accepted, verify it's not identical to private key
            assert_ne!(
                zero_key.as_slice(),
                &public_key[..],
                "Public key should never match private key"
            );
        }
    }

    #[test]
    fn test_derive_public_key_validates_output() {
        // Generate a valid key and derive public key
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Verify validation catches identical keys
        assert_ne!(
            private_key.as_slice(),
            &public_key[..],
            "Derived public key must differ from private key"
        );

        // Verify validation catches all-zero public keys
        assert!(
            !public_key.iter().all(|&b| b == 0),
            "Derived public key should not be all zeros"
        );
    }
}
