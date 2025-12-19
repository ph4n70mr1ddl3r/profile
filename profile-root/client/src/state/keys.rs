//! Key state management for user session

use profile_shared::{PrivateKey, PublicKey};

/// Manages the user's cryptographic keys during a session
#[derive(Debug)]
pub struct KeyState {
    private_key: Option<PrivateKey>,
    public_key: Option<PublicKey>,
}

impl KeyState {
    /// Create a new, empty KeyState
    pub fn new() -> Self {
        Self {
            private_key: None,
            public_key: None,
        }
    }

    /// Store a newly generated or imported key pair
    pub fn set_generated_key(&mut self, private_key: PrivateKey, public_key: PublicKey) {
        self.private_key = Some(private_key);
        self.public_key = Some(public_key);
    }

    /// Get a reference to the stored private key (if any)
    /// 
    /// # Usage
    /// - Story 1.5: Used for authentication signature generation
    /// - Story 3.x: Used for message signing
    /// 
    /// Currently only used in tests. The `#[allow(dead_code)]` will be removed
    /// when Story 1.5 is implemented.
    #[allow(dead_code)]
    pub fn private_key(&self) -> Option<&PrivateKey> {
        self.private_key.as_ref()
    }

    /// Get a reference to the stored public key (if any)
    /// 
    /// # Usage
    /// - Throughout the application for key display
    /// - Story 1.3: Public key display and copying
    /// - Story 2.x: Lobby user identification
    /// 
    /// Currently only used in tests. The `#[allow(dead_code)]` will be removed
    /// when Story 1.3 is implemented.
    #[allow(dead_code)]
    pub fn public_key(&self) -> Option<&PublicKey> {
        self.public_key.as_ref()
    }

    /// Check if both private and public keys are set
    /// 
    /// # Usage
    /// - Determines if user has completed key setup flow
    /// - Guards operations that require authenticated state
    /// 
    /// Currently only used in tests. The `#[allow(dead_code)]` will be removed
    /// when key-dependent features are implemented.
    #[allow(dead_code)]
    pub fn is_key_set(&self) -> bool {
        self.private_key.is_some() && self.public_key.is_some()
    }
}

impl Default for KeyState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_state_initialization() {
        let state = KeyState::new();
        assert!(!state.is_key_set());
        assert_eq!(state.private_key(), None);
        assert_eq!(state.public_key(), None);
    }

    #[test]
    fn test_key_state_stores_keys() {
        let mut state = KeyState::new();
        let private = profile_shared::PrivateKey::new(vec![0u8; 32]);
        let public = vec![1u8; 32];

        state.set_generated_key(private, public.clone());
        assert!(state.is_key_set());
        assert_eq!(state.public_key().unwrap(), &public);
    }

    #[test]
    fn test_default_trait() {
        let state = KeyState::default();
        assert!(!state.is_key_set());
    }
}
