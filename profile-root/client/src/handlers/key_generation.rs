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
}
