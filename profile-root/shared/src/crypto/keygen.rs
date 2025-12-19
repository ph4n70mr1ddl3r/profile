//! Key generation and derivation using ed25519-dalek

use crate::crypto::error::CryptoError;
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;
use zeroize::Zeroizing;

/// Generate a new 256-bit (32-byte) ed25519 private key
/// 
/// Uses a cryptographically secure random number generator (OsRng)
/// Returns the key wrapped in zeroize::Zeroizing for secure memory handling
pub fn generate_private_key() -> Result<Zeroizing<Vec<u8>>, CryptoError> {
    let mut key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut key_bytes);
    
    // Verify the key is valid by creating a SigningKey from it
    let _signing_key = SigningKey::from_bytes(&key_bytes);

    let key_vec = key_bytes.to_vec();
    key_bytes.zeroize();

    Ok(Zeroizing::new(key_vec))
}

/// Derive the public key from a private key
/// 
/// Takes a private key (as zeroized bytes) and returns the corresponding
/// 32-byte ed25519 public key
pub fn derive_public_key(private_key: &Zeroizing<Vec<u8>>) -> Result<Vec<u8>, CryptoError> {
    if private_key.len() != 32 {
        return Err(CryptoError::InvalidKeyFormat(
            format!("Expected 32-byte private key, got {}", private_key.len()),
        ));
    }

    // Convert bytes to SigningKey
    let mut key_bytes = <[u8; 32]>::try_from(private_key.as_ref())
        .map_err(|_| CryptoError::InvalidKeyFormat("Cannot convert to [u8; 32]".into()))?;
    
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    let public_key = verifying_key.to_bytes().to_vec();

    key_bytes.zeroize();

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
        assert_ne!(key1[..], key2[..], "Generated keys should be different");
    }

    #[test]
    fn test_derive_public_key_determinism() {
        // Same private key should always produce the same public key
        let private_key = Zeroizing::new(vec![42u8; 32]);
        let public_key1 = derive_public_key(&private_key).unwrap();
        let public_key2 = derive_public_key(&private_key).unwrap();
        
        assert_eq!(public_key1, public_key2, "Derivation should be deterministic");
    }

    #[test]
    fn test_derive_public_key_length() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();
        assert_eq!(public_key.len(), 32, "Public key must be 32 bytes for ed25519");
    }

    #[test]
    fn test_derive_public_key_invalid_length() {
        let invalid_key = Zeroizing::new(vec![42u8; 16]); // Wrong length
        let result = derive_public_key(&invalid_key);
        assert!(
            result.is_err(),
            "Should reject keys with wrong length"
        );
    }

    #[test]
    fn test_derived_key_never_equals_private_key() {
        // Public and private keys should not be identical
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();
        assert_ne!(
            private_key[..],
            public_key[..],
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
                    &keys[i][..],
                    &keys[j][..],
                    "All generated keys should be unique"
                );
            }
        }
    }
}
