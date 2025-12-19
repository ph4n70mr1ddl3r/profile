//! Shared cryptographic operations for Profile
//!
//! This module provides the foundation for all cryptographic operations:
//! - Key generation and derivation
//! - Message signing (Story 1.5+)
//! - Signature verification (Story 3.x+)
//!
//! All operations use ed25519-dalek 2.1+ for deterministic, industry-standard signing.

pub mod error;
pub mod keygen;
pub mod signing;
pub mod verification;

// Core public API - CRITICAL for downstream stories
pub use error::CryptoError;
pub use keygen::{derive_public_key, generate_private_key};
pub use signing::sign_message;
pub use verification::verify_signature;

/// Private key type - always zeroize-protected
/// Never clone this type - it defeats the purpose of zeroize protection
pub type PrivateKey = zeroize::Zeroizing<Vec<u8>>;

/// Public key type - raw 32 bytes (ed25519)
pub type PublicKey = Vec<u8>;
