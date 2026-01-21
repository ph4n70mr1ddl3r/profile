//! Shared cryptographic operations for Profile
//!
//! This module provides the foundation for all cryptographic operations:
//! - Key generation and derivation
//! - Message signing (Story 1.5+)
//! - Signature verification (Story 3.x+)
//!
//! All operations use ed25519-dalek 2.1+ for deterministic, industry-standard signing.

use sha2::Digest;

pub mod keygen;
pub mod signing;
pub mod verification;

pub use keygen::{derive_public_key, generate_private_key};
pub use signing::sign_message;
pub use verification::verify_signature;

/// Secure private key wrapper with safe debug implementation
///
/// This wrapper prevents accidental exposure of private key material through
/// debug formatting while maintaining zeroize protection.
///
/// # Security Notes
/// ⚠️ **CRITICAL**: Never clone this type - it defeats zeroize protection
/// ⚠️ **CRITICAL**: Never unwrap to `Vec<u8>` and re-wrap - creates unprotected copy
/// ⚠️ **CORRECT**: Pass `PrivateKey` directly to functions that need it  
///
/// # Memory Safety
/// When `PrivateKey` goes out of scope, the `Zeroizing` wrapper's `Drop` trait
/// automatically overwrites memory with zeros before deallocation. This provides
/// protection against casual memory inspection and data leaks.
///
/// # Limitations
/// - Protection is best-effort using compiler barriers
/// - NOT protected against sophisticated hardware attacks (cold boot, DMA)
/// - Industry-standard approach used by cryptographic libraries
///
/// # Examples
/// ```rust
/// use profile_shared::{generate_private_key, derive_public_key, PrivateKey};
///
/// // ✅ CORRECT - Keep Zeroizing wrapper intact
/// let private: PrivateKey = generate_private_key().unwrap();
/// let public = derive_public_key(&private).unwrap();
/// // Note: In production code, handle errors properly instead of using unwrap()
///
/// // ❌ WRONG - Unwrapping breaks protection
/// // let private: PrivateKey = generate_private_key().unwrap();
/// // let unprotected: Vec<u8> = private.to_vec(); // Creates unprotected copy!
/// ```
pub struct PrivateKey(zeroize::Zeroizing<Vec<u8>>);

impl PrivateKey {
    /// Create a new PrivateKey from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(zeroize::Zeroizing::new(bytes))
    }

    /// Get a reference to the inner bytes
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Get the length of the key
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the key is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl AsRef<[u8]> for PrivateKey {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl zeroize::Zeroize for PrivateKey {
    fn zeroize(&mut self) {
        self.0.zeroize();
    }
}

impl std::fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PrivateKey")
            .field("length", &self.len())
            .field(
                "sha256_hash",
                &format!("{:x}", sha2::Sha256::digest(self.as_slice())),
            )
            .finish()
    }
}

/// Public key type - raw 32 bytes (ed25519)
pub type PublicKey = Vec<u8>;
