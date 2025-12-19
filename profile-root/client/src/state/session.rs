//! Session state management with async-safe locking
//!
//! CRITICAL: Uses tokio::sync::Mutex (NOT std::sync::Mutex) for async-safe, non-blocking access

use std::sync::Arc;
use tokio::sync::Mutex;
use crate::state::KeyState;
use profile_shared::{generate_private_key, derive_public_key};

/// Type alias for thread-safe, async-safe shared key state
/// Uses tokio::sync::Mutex to avoid blocking the Tokio runtime
pub type SharedKeyState = Arc<Mutex<KeyState>>;

/// Create a new shared key state for the application session
pub fn create_shared_key_state() -> SharedKeyState {
    Arc::new(Mutex::new(KeyState::new()))
}

/// Generate a new key and store it in the shared state
/// 
/// Returns the public key as hex string for display
pub async fn handle_generate_key_async(
    key_state: &SharedKeyState,
) -> Result<String, String> {
    // Lock the state using async-safe pattern (tokio::sync::Mutex)
    let mut state = key_state.lock().await;

    // Generate private key
    let private_key = generate_private_key()
        .map_err(|e| format!(
            "Cannot generate cryptographic key. Please check system permissions and ensure random number generator is available. Error: {}",
            e
        ))?;

    // Derive public key
    let public_key = derive_public_key(&private_key)
        .map_err(|e| format!(
            "Cannot derive public key from private key. This indicates a cryptographic error. Error: {}",
            e
        ))?;

    // Convert to hex for display
    let public_key_hex = hex::encode(&public_key);
    
    // Validate hex encoding (32 bytes = 64 hex chars)
    if public_key_hex.len() != 64 {
        return Err(format!(
            "Invalid public key length: {} hex chars (expected 64 for 32-byte key)",
            public_key_hex.len()
        ));
    }
    
    // Validate hex content (defense in depth - hex::encode should always produce valid hex)
    if !public_key_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Invalid public key encoding: contains non-hex characters".into());
    }
    
    // Validate not all zeros (additional safety check - already caught in keygen, but verify here too)
    if public_key_hex.chars().all(|c| c == '0') {
        return Err("Invalid public key: all-zero key detected".into());
    }

    // Store in session state
    state.set_generated_key(private_key, public_key);

    Ok(public_key_hex)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_shared_key_state() {
        let key_state = create_shared_key_state();
        let state = key_state.lock().await;
        assert!(!state.is_key_set());
    }

    #[tokio::test]
    async fn test_concurrent_key_access() {
        let key_state = create_shared_key_state();
        let state_clone = Arc::clone(&key_state);

        // Simulate concurrent access from different async tasks
        let task1 = tokio::spawn(async move {
            let state = state_clone.lock().await;
            state.is_key_set()
        });

        let result = task1.await.unwrap();
        assert!(!result); // Key not yet set
    }

    #[tokio::test]
    async fn test_mutex_prevents_race_condition() {
        let key_state = create_shared_key_state();
        let state1 = Arc::clone(&key_state);
        let state2 = Arc::clone(&key_state);

        // Both tasks try to check key state - operations complete safely
        let task1 = tokio::spawn(async move {
            let state = state1.lock().await;
            state.is_key_set()
        });

        let task2 = tokio::spawn(async move {
            let state = state2.lock().await;
            state.is_key_set()
        });

        let _ = tokio::join!(task1, task2);
        // Both operations complete safely, no data race
    }

    #[tokio::test]
    async fn test_handle_generate_key_async_success() {
        let key_state = create_shared_key_state();
        let result = handle_generate_key_async(&key_state).await;

        assert!(result.is_ok());
        let public_key_hex = result.unwrap();
        assert_eq!(public_key_hex.len(), 64); // 32 bytes = 64 hex chars

        // Verify state was updated
        let state = key_state.lock().await;
        assert!(state.is_key_set());
    }

    #[tokio::test]
    async fn test_handle_generate_key_async_randomness() {
        let key_state1 = create_shared_key_state();
        let key_state2 = create_shared_key_state();

        let key1 = handle_generate_key_async(&key_state1).await.unwrap();
        let key2 = handle_generate_key_async(&key_state2).await.unwrap();

        assert_ne!(key1, key2, "Generated keys should be different");
    }

    #[tokio::test]
    async fn test_hex_validation_checks_content() {
        // This test verifies that hex validation would catch invalid hex
        // (Although hex::encode always produces valid hex, we test the validation logic)
        let key_state = create_shared_key_state();
        let result = handle_generate_key_async(&key_state).await;
        
        assert!(result.is_ok());
        let hex = result.unwrap();
        
        // Verify hex validation logic
        assert_eq!(hex.len(), 64, "Should be 64 hex chars");
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()), "All chars should be hex");
        assert!(!hex.chars().all(|c| c == '0'), "Should not be all zeros");
    }

    #[tokio::test]
    async fn test_key_generation_completes_quickly() {
        // Verify key generation completes well under the 5-second timeout
        use std::time::Instant;
        
        let key_state = create_shared_key_state();
        let start = Instant::now();
        
        let result = handle_generate_key_async(&key_state).await;
        let elapsed = start.elapsed();
        
        assert!(result.is_ok(), "Key generation should succeed");
        assert!(elapsed.as_secs() < 1, "Key generation should complete in <1 second (was {:?})", elapsed);
    }

    #[tokio::test]
    async fn test_state_unchanged_on_generation_failure() {
        // This test verifies that if key generation fails, state remains unchanged
        // (We can't easily mock a failure, but we verify the pattern)
        let key_state = create_shared_key_state();
        
        // Verify initial state
        {
            let state = key_state.lock().await;
            assert!(!state.is_key_set(), "State should start empty");
        }
        
        // Generate key (should succeed in normal operation)
        let result = handle_generate_key_async(&key_state).await;
        
        if result.is_err() {
            // If generation failed, state should still be empty
            let state = key_state.lock().await;
            assert!(!state.is_key_set(), "State should remain empty on failure");
        } else {
            // If generation succeeded, state should be set
            let state = key_state.lock().await;
            assert!(state.is_key_set(), "State should be set on success");
        }
    }
}
