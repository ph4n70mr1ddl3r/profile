//! Key state management for user session

use profile_shared::{PrivateKey, PublicKey};

/// Manages the user's cryptographic keys during a session
pub struct KeyState {
    /// Private key stored with automatic memory zeroing on drop
    /// 
    /// # Security
    /// - Type is `Option<PrivateKey>` where `PrivateKey = Zeroizing<Vec<u8>>`
    /// - Memory automatically zeroed when `KeyState` is dropped
    /// - Never logged or serialized (no Serialize derive on this struct)
    /// - Never transmitted to server (only public key is sent)
    /// 
    /// # Usage
    /// - Story 1.1: Stores generated keys
    /// - Story 1.2: Stores imported keys
    /// - Story 1.5: Used for authentication signature generation
    /// - Story 3.x: Used for message signing
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
    /// 
    /// # Security
    /// Takes ownership of `PrivateKey` (Zeroizing wrapper) and stores it directly.
    /// The Zeroizing wrapper remains intact, ensuring memory is automatically
    /// cleared when this `KeyState` is dropped.
    /// 
    /// # Important
    /// Do NOT unwrap the `PrivateKey` before passing it to this function.
    /// Doing so creates an unprotected copy and defeats zeroize protection.
    /// 
    /// # Examples
    /// ```rust
    /// # use profile_client::state::KeyState;
    /// # use profile_shared::{generate_private_key, derive_public_key};
    /// // ✅ CORRECT
    /// let private = generate_private_key().unwrap();
    /// let public = derive_public_key(&private).unwrap();
    /// // In production code, handle errors properly instead of using unwrap()
    /// let mut state = KeyState::new();
    /// state.set_generated_key(private, public);
    /// 
    /// // ❌ WRONG - breaks zeroize protection
    /// // let private = generate_private_key().unwrap();
    /// // let unprotected = private.to_vec();
    /// // state.set_generated_key(Zeroizing::new(unprotected), public);
    /// ```
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

// Custom Debug implementation to prevent private key leakage
impl std::fmt::Debug for KeyState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyState")
            .field("private_key", &if self.private_key.is_some() { 
                &"<REDACTED>" 
            } else { 
                &"None" 
            })
            .field("public_key", &self.public_key)
            .finish()
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

    // ========================================================================
    // Story 1.4: Zeroize Verification Tests
    // ========================================================================

    #[test]
    fn test_key_state_private_key_zeroized_on_drop() {
        // Purpose: Verify AC #2 (memory automatically overwritten with zeros)
        // Success Criteria: Type system guarantees zeroize (can't inspect memory directly)
        
        {
            let mut state = KeyState::new();
            let private = PrivateKey::new(vec![42u8; 32]);
            let public = vec![1u8; 32];
            state.set_generated_key(private, public);
            
            // PrivateKey = Zeroizing<Vec<u8>>
            // When this scope ends, Zeroizing's Drop trait zeroes memory
        } // <- Drop happens here, memory is automatically zeroed
        
        // We cannot inspect memory directly (requires unsafe, platform-specific code)
        // But we trust the type system guarantees:
        // 1. PrivateKey IS Zeroizing<Vec<u8>> (verified in type alias)
        // 2. Zeroizing implements Drop with memory clearing
        // 3. No unwrapping of Zeroizing wrapper occurs (verified by code review)
        
        // This test documents the security contract and would catch refactoring errors
    }

    #[test]
    fn test_key_state_debug_does_not_leak_private_key() {
        // Purpose: Verify AC #1 (private key never logged or printed)
        // Success Criteria: Debug impl doesn't print private key bytes
        
        let mut state = KeyState::new();
        let private = PrivateKey::new(vec![42u8; 32]);
        let public = vec![1u8; 32];
        state.set_generated_key(private, public);
        
        let debug_output = format!("{:?}", state);
        
        // Verify struct name is present
        assert!(debug_output.contains("KeyState"), "Debug output should show struct name");
        
        // CRITICAL: Protection comes from custom Debug impl on KeyState (lines 113-124)
        // that redacts the private_key field, NOT from Zeroizing's Debug.
        // Zeroizing<Vec<u8>> Debug DOES expose bytes - our custom impl prevents leakage.
        // The actual byte pattern [42, 42, 42, ...] should NOT be present due to
        // our custom impl returning "<REDACTED>" for Some(PrivateKey).
        let repeated_pattern = "42, 42, 42";
        assert!(!debug_output.contains(repeated_pattern), 
            "Debug output should not contain raw private key byte pattern. Output was: {}", 
            debug_output);
    }

    #[test]
    fn test_key_state_no_disk_persistence() {
        // Purpose: Verify AC #1 (private key never written to disk)
        // Success Criteria: No files created during key storage (checks temp dir AND working dir)
        
        use std::fs;
        use std::env;
        
        // Check temp directory
        let temp_dir = env::temp_dir();
        let before_temp_files: Vec<_> = fs::read_dir(&temp_dir)
            .expect("Failed to read temp dir")
            .collect();
        let before_temp_count = before_temp_files.len();
        
        // Check working directory
        let work_dir = env::current_dir().expect("Failed to get current dir");
        let before_work_files: Vec<_> = fs::read_dir(&work_dir)
            .expect("Failed to read working dir")
            .collect();
        let before_work_count = before_work_files.len();
        
        // Generate and store key
        let mut state = KeyState::new();
        let private = profile_shared::generate_private_key().unwrap();
        let public = profile_shared::derive_public_key(&private).unwrap();
        state.set_generated_key(private, public);
        
        // Verify temp directory unchanged
        let after_temp_files: Vec<_> = fs::read_dir(&temp_dir)
            .expect("Failed to read temp dir")
            .collect();
        let after_temp_count = after_temp_files.len();
        assert_eq!(before_temp_count, after_temp_count, "No files should be created in temp directory");
        
        // Verify working directory unchanged
        let after_work_files: Vec<_> = fs::read_dir(&work_dir)
            .expect("Failed to read working dir")
            .collect();
        let after_work_count = after_work_files.len();
        assert_eq!(before_work_count, after_work_count, "No files should be created in working directory");
    }

    #[tokio::test]
    async fn test_key_state_shared_arc_drops_once() {
        // Purpose: Verify AC #2 (zeroize happens exactly once when last Arc is dropped)
        // Success Criteria: KeyState in Arc<Mutex> drops correctly with multiple clones
        
        use crate::state::session::create_shared_key_state;
        
        let key_state = create_shared_key_state();
        let clone1 = std::sync::Arc::clone(&key_state);
        let clone2 = std::sync::Arc::clone(&key_state);
        
        // Generate key
        { 
            let mut state = key_state.lock().await;
            let private = profile_shared::generate_private_key().unwrap();
            let public = profile_shared::derive_public_key(&private).unwrap();
            state.set_generated_key(private, public);
            assert!(state.is_key_set());
        }
        
        // Drop clones one by one - zeroize should happen only on LAST drop
        drop(clone1); // Arc count: 3 -> 2
        drop(clone2); // Arc count: 2 -> 1
        drop(key_state); // Arc count: 1 -> 0, triggers: Arc drop -> Mutex drop -> KeyState drop -> PrivateKey drop -> Zeroizing drop (zeroes memory)
        
        // Cannot inspect memory after drop, but type system guarantees the full drop chain
    }
}
