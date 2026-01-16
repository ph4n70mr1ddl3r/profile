//! Shared cryptographic operations for Profile
//!
//! This module provides the foundation for all cryptographic operations:
//! - Key generation and derivation
//! - Message signing (Story 1.5+)
//! - Signature verification (Story 3.x+)
//!
//! All operations use ed25519-dalek 2.1+ for deterministic, industry-standard signing.

pub mod keygen;
pub mod signing;
pub mod verification;

pub use keygen::{derive_public_key, generate_private_key};
pub use signing::sign_message;
pub use verification::verify_signature;

/// Private key type - always zeroize-protected
///
/// # Security Notes
/// ⚠️ **CRITICAL**: Never clone this type - it defeats zeroize protection
/// ⚠️ **CRITICAL**: Never unwrap to `Vec<u8>` and re-wrap - creates unprotected copy
/// ⚠️ **CRITICAL**: Debug formatting WILL expose bytes - never use `{:?}` with raw
///     PrivateKey. Always wrap in a struct with custom Debug impl.
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
pub type PrivateKey = zeroize::Zeroizing<Vec<u8>>;

/// Public key type - raw 32 bytes (ed25519)
pub type PublicKey = Vec<u8>;
