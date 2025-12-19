//! Key generation handler - bridges UI and crypto operations

use crate::state::SharedKeyState;

/// Handle the "Generate New Key" button press
/// 
/// Generates a new private key, derives its public key, stores both in session,
/// and returns the public key as hex for UI display
pub async fn handle_generate_new_key(key_state: &SharedKeyState) -> Result<String, String> {
    crate::state::handle_generate_key_async(key_state).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::create_shared_key_state;

    #[tokio::test]
    async fn test_handle_generate_new_key_success() {
        let key_state = create_shared_key_state();
        let result = handle_generate_new_key(&key_state).await;

        assert!(result.is_ok());
        let public_key = result.unwrap();
        assert_eq!(public_key.len(), 64); // 32 bytes = 64 hex chars
    }

    #[tokio::test]
    async fn test_handle_generate_new_key_stores_in_state() {
        let key_state = create_shared_key_state();
        let _ = handle_generate_new_key(&key_state).await.unwrap();

        // Verify state was updated
        let state = key_state.lock().await;
        assert!(state.is_key_set());
        assert!(state.public_key().is_some());
    }

    #[tokio::test]
    async fn test_concurrent_generation_requests_are_safe() {
        // Test that multiple concurrent calls to handle_generate_new_key don't cause issues
        // (The UI layer has a re-entry guard, but the handler itself should be safe)
        use std::sync::Arc;
        
        let key_state = create_shared_key_state();
        let key_state_clone = Arc::clone(&key_state);
        
        // Spawn two concurrent generation requests
        let task1 = tokio::spawn(async move {
            handle_generate_new_key(&key_state).await
        });
        
        let task2 = tokio::spawn(async move {
            handle_generate_new_key(&key_state_clone).await
        });
        
        // Both should complete successfully (though they'll overwrite each other's keys)
        let result1 = task1.await.unwrap();
        let result2 = task2.await.unwrap();
        
        assert!(result1.is_ok(), "First generation should succeed");
        assert!(result2.is_ok(), "Second generation should succeed");
        
        // Keys should be different (randomness)
        assert_ne!(result1.unwrap(), result2.unwrap(), "Concurrent generations should produce different keys");
    }

    #[tokio::test]
    async fn test_generation_is_idempotent_when_called_sequentially() {
        // Test that calling handle_generate_new_key multiple times works correctly
        // Each call should generate a NEW key (not reuse the previous one)
        let key_state = create_shared_key_state();
        
        let key1 = handle_generate_new_key(&key_state).await.unwrap();
        let key2 = handle_generate_new_key(&key_state).await.unwrap();
        let key3 = handle_generate_new_key(&key_state).await.unwrap();
        
        // All keys should be different
        assert_ne!(key1, key2, "Sequential generations should produce different keys");
        assert_ne!(key2, key3, "Sequential generations should produce different keys");
        assert_ne!(key1, key3, "Sequential generations should produce different keys");
        
        // State should contain the LAST generated key
        let state = key_state.lock().await;
        assert!(state.is_key_set());
        assert_eq!(hex::encode(state.public_key().unwrap()), key3, "State should contain last generated key");
    }
}
