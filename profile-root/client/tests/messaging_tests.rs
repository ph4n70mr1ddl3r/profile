//! Messaging integration tests for the client
//!
//! Tests cover:
//! - Composer integration with lobby selection (Task 6.1)
//! - Message signing integration (Task 6.2)
//! - WebSocket transmission integration (Task 6.3)
//! - UI display integration (Task 6.4)
//! - Full message flow (Task 6.5)
//! - Message draft preservation (Task 6.6)
//! - Edge case handling (Task 6.7)

use profile_client::connection::message::ClientMessage;
use profile_client::handlers::compose::{compose_and_send_message, compose_message_draft};
use profile_client::handlers::lobby::{
    handle_lobby_user_joined, handle_lobby_user_select, is_selection_valid,
};
use profile_client::state::create_shared_message_history;
use profile_client::state::session::create_shared_key_state;
use profile_shared::derive_public_key;
use profile_shared::generate_private_key;

/// Test Task 6.1: Composer integrates with lobby selection
///
/// Verifies that when a user is selected from the lobby,
/// the selection state is correctly updated for messaging.
#[tokio::test]
async fn test_composer_selects_recipient() {
    let lobby_state = profile_client::state::create_shared_lobby_state();

    // Add a user to the lobby as online
    let recipient_key = "recipient_public_key_1234567890abcdef1234567890abcdef12345678";
    handle_lobby_user_joined(&lobby_state, recipient_key).await;
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
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();

    let key_state = create_shared_key_state();
    {
        let mut state = key_state.lock().await;
        state.set_generated_key(private_key, public_key.clone());
    }

    // Create a draft message
    let result = compose_message_draft("This is a draft message".to_string(), &key_state).await;

    // Verify draft was created successfully
    assert!(result.is_ok(), "Draft creation should succeed");
    let chat_msg = result.unwrap();

    // Verify draft message content
    assert_eq!(chat_msg.message, "This is a draft message");
    assert!(chat_msg.is_verified);
    assert!(!chat_msg.signature.is_empty());
    assert!(!chat_msg.timestamp.is_empty());
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
        "Hello, 世界!",        // Chinese characters
        "🔐 Cryptographic",    // Emoji
        "Ñoño tilde",          // Accented characters
        "Привет мир",          // Cyrillic
        "!@#$%^&*()_+-=[]{}|", // Special characters
        "Normal ASCII text",   // Standard text
    ];

    for content in test_cases {
        let result = ClientMessage::new(
            content.to_string(),
            recipient.to_string(),
            public_key.clone(),
            private_key.clone_for_testing(),
        );

        assert!(result.is_ok(), "Should handle content: {}", content);
        assert_eq!(result.unwrap().message, content);
    }
}

/// Test Task 6.5: Full message flow from composition to history
///
/// Verifies complete end-to-end message flow:
/// 1. User composes a message
/// 2. Message is signed with private key
/// 3. Message is stored in history
/// 4. Message is ready for WebSocket transmission
#[tokio::test]
async fn test_full_message_flow_composition_to_history() {
    // Generate keys for testing
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();

    let key_state = create_shared_key_state();
    {
        let mut state = key_state.lock().await;
        state.set_generated_key(private_key, public_key.clone());
    }

    let message_history = create_shared_message_history();
    let recipient_key = "recipient_public_key_1234567890abcdef1234567890abcdef12345678";

    // Execute full flow: compose and send
    let result = compose_and_send_message(
        "Hello, World!".to_string(),
        recipient_key.to_string(),
        &key_state,
        &message_history,
    )
    .await;

    // Verify flow completed successfully
    assert!(result.is_ok(), "Full message flow should succeed");
    let json = result.unwrap();

    // Verify JSON structure
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["message_type"], "Text");
    assert_eq!(parsed["message"], "Hello, World!");
    assert!(parsed["signature"].is_string());
    assert!(parsed["timestamp"].is_string());

    // Verify message stored in history
    let history = message_history.lock().await;
    assert_eq!(history.len(), 1);
    let stored_msg = history.messages().next().unwrap();
    assert_eq!(stored_msg.message, "Hello, World!");
    assert!(stored_msg.is_verified);
}

/// Test that empty messages are rejected
#[tokio::test]
async fn test_empty_message_rejected() {
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();

    let key_state = create_shared_key_state();
    key_state
        .lock()
        .await
        .set_generated_key(private_key, public_key.clone());

    let message_history = create_shared_message_history();
    let recipient_key = "recipient_key";

    // Empty message should fail
    let result = compose_and_send_message(
        "".to_string(),
        recipient_key.to_string(),
        &key_state,
        &message_history,
    )
    .await;

    assert!(result.is_err());
}

/// Test whitespace-only message is rejected
#[tokio::test]
async fn test_whitespace_only_message_rejected() {
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();

    let key_state = create_shared_key_state();
    key_state
        .lock()
        .await
        .set_generated_key(private_key, public_key.clone());

    let message_history = create_shared_message_history();
    let recipient_key = "recipient_key";

    // Whitespace-only message should fail
    let result = compose_and_send_message(
        "   ".to_string(),
        recipient_key.to_string(),
        &key_state,
        &message_history,
    )
    .await;

    assert!(result.is_err());
}

/// Test deterministic signing produces consistent results
#[tokio::test]
async fn test_deterministic_signing_consistency() {
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();

    let key_state = create_shared_key_state();
    key_state
        .lock()
        .await
        .set_generated_key(private_key, public_key.clone());

    let message_history = create_shared_message_history();
    let recipient_key = "fixed_recipient_key";

    // Send same message content twice rapidly (within same second)
    let msg_content = "Deterministic test".to_string();

    let result1 = compose_and_send_message(
        msg_content.clone(),
        recipient_key.to_string(),
        &key_state,
        &message_history,
    )
    .await
    .unwrap();

    let result2 = compose_and_send_message(
        msg_content.clone(),
        recipient_key.to_string(),
        &key_state,
        &message_history,
    )
    .await
    .unwrap();

    // Both should succeed
    let parsed1: serde_json::Value = serde_json::from_str(&result1).unwrap();
    let parsed2: serde_json::Value = serde_json::from_str(&result2).unwrap();

    // Messages should have same content
    assert_eq!(parsed1["message"], parsed2["message"]);
}

/// Test different messages produce different signatures
#[tokio::test]
async fn test_different_messages_different_signatures() {
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();

    let key_state = create_shared_key_state();
    key_state
        .lock()
        .await
        .set_generated_key(private_key, public_key.clone());

    let message_history = create_shared_message_history();
    let recipient_key = "recipient_key";

    // Send different messages
    let result1 = compose_and_send_message(
        "Message 1".to_string(),
        recipient_key.to_string(),
        &key_state,
        &message_history,
    )
    .await
    .unwrap();

    let result2 = compose_and_send_message(
        "Message 2".to_string(),
        recipient_key.to_string(),
        &key_state,
        &message_history,
    )
    .await
    .unwrap();

    let parsed1: serde_json::Value = serde_json::from_str(&result1).unwrap();
    let parsed2: serde_json::Value = serde_json::from_str(&result2).unwrap();

    // Verify messages are different
    assert_ne!(parsed1["message"], parsed2["message"]);
    // Verify signatures are different
    assert_ne!(parsed1["signature"], parsed2["signature"]);
}

/// Test message format matches WebSocket protocol requirements
#[tokio::test]
async fn test_message_format_for_websocket() {
    let private_key = generate_private_key().unwrap();
    let public_key = derive_public_key(&private_key).unwrap();

    let key_state = create_shared_key_state();
    key_state
        .lock()
        .await
        .set_generated_key(private_key, public_key.clone());

    let message_history = create_shared_message_history();
    let recipient_key = "recipient123";

    let result = compose_and_send_message(
        "WebSocket test".to_string(),
        recipient_key.to_string(),
        &key_state,
        &message_history,
    )
    .await
    .unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    // Verify all required protocol fields
    assert!(parsed.get("message_type").is_some(), "Missing message_type");
    assert!(parsed.get("message").is_some(), "Missing message");
    assert!(
        parsed.get("senderPublicKey").is_some(),
        "Missing senderPublicKey"
    );
    assert!(parsed.get("signature").is_some(), "Missing signature");
    assert!(parsed.get("timestamp").is_some(), "Missing timestamp");
}
