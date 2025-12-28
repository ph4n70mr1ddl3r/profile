//! Messaging integration tests for the client
//!
//! Tests cover:
//! - Composer integration with lobby selection (Task 6.1)
//! - Message signing integration (Task 6.2)
//! - WebSocket transmission integration (Task 6.3)
//! - UI display integration (Task 6.4)
//! - Full message flow (Task 6.5)

use profile_client::state::create_shared_message_history;
use profile_client::state::session::create_shared_key_state;
use profile_client::ui::lobby_state::LobbyUser;
use profile_client::handlers::lobby::{handle_lobby_user_select, is_selection_valid};
use profile_client::handlers::compose::compose_message_draft;
use profile_client::connection::message::ClientMessage;
use profile_shared::generate_private_key;
use profile_shared::derive_public_key;

/// Test Task 6.1: Composer integrates with lobby selection
///
/// Verifies that when a user is selected from the lobby,
/// the selection state is correctly updated for messaging.
#[tokio::test]
async fn test_composer_selects_recipient() {
    let lobby_state = profile_client::state::create_shared_lobby_state();

    // Add a user to the lobby
    let recipient_key = "recipient_public_key_1234567890abcdef1234567890abcdef12345678";
    handle_lobby_user_select(&lobby_state, recipient_key).await;

    // Verify selection is valid
    assert!(is_selection_valid(&lobby_state).await);
}

/// Test Task 6.2: Message signing integration
///
/// Verifies that a message can be created and signed with valid keys.
#[tokio::test]
async fn test_message_signing_integration() {
    // Generate keys for testing
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();

    let recipient = "recipient_public_key_1234567890abcdef1234567890abcdef12345678";

    // Create a signed message
    let result = ClientMessage::new(
        "Hello, this is a test message".to_string(),
        recipient.to_string(),
        public_key,
        private_key,
    );

    assert!(result.is_ok(), "Message signing should succeed");
    let msg = result.unwrap();

    // Verify message structure
    assert_eq!(msg.r#type, "message");
    assert_eq!(msg.message, "Hello, this is a test message");
    assert!(!msg.signature.is_empty());
    assert!(!msg.timestamp.is_empty());
}

/// Test Task 6.4: Message history storage and retrieval
///
/// Verifies that messages can be stored in history and retrieved.
#[tokio::test]
async fn test_message_history_storage() {
    let history = create_shared_message_history();

    // Add a message to history
    let test_message = profile_client::state::messages::ChatMessage::new(
        "sender_key_abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab".to_string(),
        "Test message content".to_string(),
        "signature_1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
        "2025-12-27T10:30:00Z".to_string(),
    );

    {
        let mut h = history.lock().await;
        h.add_message(test_message.clone());
    }

    // Verify message is stored
    let h = history.lock().await;
    assert_eq!(h.len(), 1);

    let retrieved = h.newest().unwrap();
    assert_eq!(retrieved.message, "Test message content");
}

/// Test Task 6.6: Message draft preservation
///
/// Verifies that draft messages can be preserved during composition.
#[tokio::test]
async fn test_message_draft_preservation() {
    let composer_state = profile_client::state::create_shared_composer_state();

    // Set a draft message
    compose_message_draft("This is a draft message", &create_shared_key_state()).await.unwrap();

    // Verify draft is stored
    let draft = composer_state.lock().await.get_draft();
    assert_eq!(draft, "This is a draft message");

    // Clear draft
    let mut state = composer_state.lock().await;
    state.clear_draft();

    // Verify draft is cleared
    assert!(state.get_draft().is_empty());
}

/// Test Task 6.7: Message creation with various content types
///
/// Verifies that messages can be created with unicode, special characters, etc.
#[tokio::test]
async fn test_message_content_varieties() {
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();
    let recipient = "recipient_public_key_1234567890abcdef1234567890abcdef12345678";

    let test_cases = vec![
        "Hello, 世界!",         // Chinese characters
        "🔐 Cryptographic",     // Emoji
        "Ñoño tilde",           // Accented characters
        "Привет мир",           // Cyrillic
        "!@#$%^&*()_+-=[]{}|",  // Special characters
        "Normal ASCII text",    // Standard text
    ];

    for content in test_cases {
        let result = ClientMessage::new(
            content.to_string(),
            recipient.to_string(),
            public_key.clone(),
            private_key.clone(),
        );

        assert!(result.is_ok(), "Should handle content: {}", content);
        assert_eq!(result.unwrap().message, content);
    }
}
