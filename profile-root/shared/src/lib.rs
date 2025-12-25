//! Profile shared cryptographic library.

pub mod crypto;
pub mod errors;
pub mod protocol;

pub use crypto::{
    derive_public_key, generate_private_key, sign_message, verify_signature, PrivateKey, PublicKey,
};
pub use errors::{CryptoError, LobbyError};
pub use protocol::{Message, LobbyUser, LobbyUserCompact};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_api_completeness() {
        // This test ensures all required functions are exported for downstream stories
        // If you remove any of the pub use statements above, this test will fail
        // and warn you that future stories will break
        
        // Verify that all key functions are accessible at crate level
        let _ = generate_private_key;
        let _ = derive_public_key;
        let _ = sign_message;
        let _ = verify_signature;
    }
}
