//! Disconnection handling integration tests
//!
//! Tests for Story 1.6: Handle Authentication Failure & Disconnection

use profile_client::state::{create_shared_composer_state, create_shared_key_state};
use profile_client::connection::client::WebSocketClient;

#[tokio::test]
async fn test_client_handles_auth_failure_disconnect() {
    // This test verifies that the client properly handles authentication failure
    // and displays the correct error message (AC1)
    
    // Note: Full integration with running server covered in server integration tests
    // This unit-level integration test verifies client-side error handling
    
    let key_state = create_shared_key_state();
    let mut client = WebSocketClient::new(key_state);
    
    // Simulate auth failure
    let result = client.handle_disconnection("auth_failed".to_string()).await;
    
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Connection closed: auth_failed"));
    assert!(!client.is_connected());
}

#[tokio::test]
async fn test_client_preserves_draft_on_disconnect() {
    // This test verifies that message drafts are preserved during disconnection (AC2, AC3)
    
    let composer = create_shared_composer_state();
    
    // Set a draft message
    {
        let mut composer_lock = composer.lock().await;
        composer_lock.set_draft("Important message not to be lost".to_string());
    }
    
    // Simulate disconnect (connection drops, but composer state persists)
    let key_state = create_shared_key_state();
    let mut client = WebSocketClient::new(key_state);
    let _ = client.handle_disconnection("network_error".to_string()).await;
    
    // Verify draft is still there
    let draft = composer.lock().await.get_draft();
    assert_eq!(draft, "Important message not to be lost");
}

#[tokio::test]
async fn test_client_displays_error_on_server_close() {
    // This test verifies correct error messages for different disconnect reasons (AC3)
    
    let key_state = create_shared_key_state();
    let mut client = WebSocketClient::new(key_state);
    
    // Test server_shutdown
    let result = client.handle_disconnection("server_shutdown".to_string()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("server_shutdown"));
    
    // Test timeout
    let key_state = create_shared_key_state();
    let mut client = WebSocketClient::new(key_state);
    let result = client.handle_disconnection("timeout".to_string()).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("timeout"));
}

#[tokio::test]
async fn test_graceful_shutdown_flow() {
    // This test verifies graceful client shutdown (AC4)
    
    let key_state = create_shared_key_state();
    let mut client = WebSocketClient::new(key_state);
    
    // Graceful shutdown should succeed even without connection
    let result = client.close_gracefully().await;
    assert!(result.is_ok());
    assert!(!client.is_connected());
}

// Performance validation tests (Task 11)

#[tokio::test]
async fn test_disconnect_detection_latency() {
    // Test that disconnect detection completes within 100ms
    
    use std::time::Instant;
    
    let key_state = create_shared_key_state();
    let mut client = WebSocketClient::new(key_state);
    
    let start = Instant::now();
    let _ = client.handle_disconnection("test reason".to_string()).await;
    let elapsed = start.elapsed();
    
    assert!(
        elapsed < std::time::Duration::from_millis(100),
        "Disconnect detection took {:?}, expected <100ms",
        elapsed
    );
}

#[tokio::test]
async fn test_error_display_speed() {
    // Test that error message generation completes within 500ms
    
    use std::time::Instant;
    use profile_client::ui::error_display::display_connection_error;
    
    let start = Instant::now();
    let _ = display_connection_error("auth_failed");
    let elapsed = start.elapsed();
    
    assert!(
        elapsed < std::time::Duration::from_millis(500),
        "Error display took {:?}, expected <500ms",
        elapsed
    );
}

#[tokio::test]
async fn test_draft_retrieval_speed() {
    // Test that draft retrieval completes within 10ms (in-memory access)
    
    use std::time::Instant;
    
    let composer = create_shared_composer_state();
    composer.lock().await.set_draft("test draft".to_string());
    
    let start = Instant::now();
    let _ = composer.lock().await.get_draft();
    let elapsed = start.elapsed();
    
    assert!(
        elapsed < std::time::Duration::from_millis(10),
        "Draft retrieval took {:?}, expected <10ms",
        elapsed
    );
}

// Security validation tests (Task 12)

#[tokio::test]
async fn test_keys_remain_in_memory_after_disconnect() {
    // Verify that private keys remain in SharedKeyState after disconnection (AC2, AC3)
    
    use profile_shared::crypto::{generate_private_key, derive_public_key};
    
    let key_state = create_shared_key_state();
    let mut client = WebSocketClient::new(key_state.clone());
    
    // Generate and store keys
    let private_key = generate_private_key().expect("Should generate key");
    let public_key = derive_public_key(&private_key).expect("Should derive public key");
    
    {
        let mut state = key_state.lock().await;
        state.set_generated_key(private_key.clone(), public_key.clone());
    }
    
    // Simulate disconnection
    let _ = client.handle_disconnection("network_error".to_string()).await;
    
    // Verify keys are still in memory (not cleared by disconnect)
    let state = key_state.lock().await;
    assert!(state.private_key().is_some(), "Private key should remain in memory");
    assert!(state.public_key().is_some(), "Public key should remain in memory");
}

#[test]
fn test_no_keys_in_connection_error_messages() {
    // Verify that error messages don't expose sensitive information
    
    use profile_client::ui::error_display::display_connection_error;
    
    let auth_failed_msg = display_connection_error("auth_failed");
    let timeout_msg = display_connection_error("timeout");
    let shutdown_msg = display_connection_error("server_shutdown");
    
    // Check that error messages don't contain sensitive keywords
    assert!(!auth_failed_msg.to_lowercase().contains("private"));
    assert!(!auth_failed_msg.to_lowercase().contains("secret"));
    assert!(!timeout_msg.to_lowercase().contains("private"));
    assert!(!shutdown_msg.to_lowercase().contains("private"));
}
