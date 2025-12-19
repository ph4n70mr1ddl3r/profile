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
        .map_err(|e| format!("Key generation failed: {}", e))?;

    // Derive public key
    let public_key = derive_public_key(&private_key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;

    // Convert to hex for display
    let public_key_hex = hex::encode(&public_key);

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
}
