//! Composer event handlers
//!
//! This module provides handlers for composer UI events including
//! message sending, draft management, and status updates.

use crate::ui::composer::{MessageComposer, SendMessageResult, create_message_composer};
use crate::state::session::SharedKeyState;
use crate::state::composer::SharedComposerState;
use crate::state::lobby::SharedLobbyState;
use crate::state::messages::SharedMessageHistory;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Handle send message action
///
/// This is the main entry point for sending messages from the UI.
///
/// # Arguments
/// * `composer` - The message composer
/// * `message_text` - The text to send
///
/// # Returns
/// Result indicating success or failure type
pub async fn handle_send_message(
    composer: &Arc<Mutex<MessageComposer>>,
    message_text: &str,
) -> SendMessageResult {
    let mut comp = composer.lock().await;
    comp.send_message(message_text).await
}

/// Handle text change in composer
///
/// Updates the draft and checks if send button should be enabled.
///
/// # Arguments
/// * `composer` - The message composer
/// * `text` - The new text content
pub async fn handle_composer_text_change(
    composer: &Arc<Mutex<MessageComposer>>,
    text: &str,
) {
    let comp = composer.lock().await;
    comp.set_draft(text).await;
}

/// Clear composer after successful send
///
/// # Arguments
/// * `composer` - The message composer
pub async fn handle_composer_clear(composer: &Arc<Mutex<MessageComposer>>) {
    let comp = composer.lock().await;
    comp.clear_draft().await;
}

/// Check if composer can send
///
/// # Arguments
/// * `composer` - The message composer
///
/// # Returns
/// True if message can be sent
pub async fn handle_composer_can_send(composer: &Arc<Mutex<MessageComposer>>) -> bool {
    let comp = composer.lock().await;
    comp.can_send().await
}

/// Get current draft from composer
///
/// # Arguments
/// * `composer` - The message composer
///
/// # Returns
/// Current draft text
pub async fn handle_composer_get_draft(composer: &Arc<Mutex<MessageComposer>>) -> String {
    let comp = composer.lock().await;
    comp.get_draft().await
}

/// Set status callback for composer feedback
///
/// # Arguments
/// * `composer` - The message composer
/// * `callback` - Callback function for status updates
pub async fn handle_composer_set_status_callback<F>(
    composer: &Arc<Mutex<MessageComposer>>,
    callback: F,
)
where
    F: Fn(String) + Send + Sync + 'static,
{
    let mut comp = composer.lock().await;
    comp.set_status_callback(callback);
}

/// Set WebSocket send callback for composer
///
/// # Arguments
/// * `composer` - The message composer
/// * `callback` - Callback function for sending messages
pub async fn handle_composer_set_send_callback<F>(
    composer: &Arc<Mutex<MessageComposer>>,
    callback: F,
)
where
    F: Fn(String) -> Result<(), String> + Send + Sync + 'static,
{
    let mut comp = composer.lock().await;
    comp.set_send_callback(callback);
}

/// Create a new composer with shared state
pub fn create_composer_with_state(
    key_state: SharedKeyState,
    composer_state: SharedComposerState,
    lobby_state: SharedLobbyState,
    message_history: SharedMessageHistory,
) -> Arc<Mutex<MessageComposer>> {
    create_message_composer(key_state, composer_state, lobby_state, message_history)
}

/// Map send result to user-friendly message
pub fn get_send_result_message(result: &SendMessageResult) -> String {
    match result {
        SendMessageResult::Success => "Message sent successfully".to_string(),
        SendMessageResult::NoRecipient => "Please select a recipient from the lobby".to_string(),
        SendMessageResult::EmptyMessage => "Please enter a message".to_string(),
        SendMessageResult::Disconnected => "Not connected to server".to_string(),
        SendMessageResult::SigningFailed(e) => format!("Signing failed: {}", e),
        SendMessageResult::TransmissionFailed(e) => format!("Failed to send: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::session::create_shared_key_state;
    use crate::state::composer::create_shared_composer_state;
    use crate::state::lobby::create_shared_lobby_state;
    use crate::state::messages::create_shared_message_history;
    use crate::ui::lobby_state::LobbyUser;

    #[tokio::test]
    async fn test_handle_send_empty_message() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        let composer = create_composer_with_state(key_state, composer_state, lobby_state, message_history);

        let result = handle_send_message(&composer, "").await;
        assert!(matches!(result, SendMessageResult::EmptyMessage));
    }

    #[tokio::test]
    async fn test_handle_composer_text_change() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        let composer = create_composer_with_state(key_state, composer_state.clone(), lobby_state, message_history);

        handle_composer_text_change(&composer, "Hello").await;

        let draft = handle_composer_get_draft(&composer).await;
        assert_eq!(draft, "Hello");
    }

    #[tokio::test]
    async fn test_handle_composer_clear() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        let composer = create_composer_with_state(key_state, composer_state.clone(), lobby_state, message_history);

        // Set some text
        handle_composer_text_change(&composer, "Test").await;
        assert_eq!(handle_composer_get_draft(&composer).await, "Test");

        // Clear it
        handle_composer_clear(&composer).await;

        let draft = handle_composer_get_draft(&composer).await;
        assert!(draft.is_empty());
    }

    #[tokio::test]
    async fn test_handle_composer_can_send_no_connection() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        let composer = create_composer_with_state(key_state, composer_state, lobby_state, message_history);

        // No connection, no recipient - should return false
        let can_send = handle_composer_can_send(&composer).await;
        assert!(!can_send);
    }

    #[tokio::test]
    async fn test_handle_composer_can_send_with_recipient() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        // Add a recipient
        {
            let mut state = lobby_state.lock().await;
            state.add_user(LobbyUser::new("test_recipient_1234567890abcdef1234567890abcdef12345678".to_string(), true));
            state.select("test_recipient_1234567890abcdef1234567890abcdef12345678");
        }

        let composer = create_composer_with_state(key_state, composer_state, lobby_state, message_history);

        // Has recipient but no connection - should still be false
        let can_send = handle_composer_can_send(&composer).await;
        assert!(!can_send);
    }

    #[tokio::test]
    async fn test_get_send_result_message() {
        assert_eq!(
            get_send_result_message(&SendMessageResult::Success),
            "Message sent successfully"
        );
        assert_eq!(
            get_send_result_message(&SendMessageResult::NoRecipient),
            "Please select a recipient from the lobby"
        );
        assert_eq!(
            get_send_result_message(&SendMessageResult::EmptyMessage),
            "Please enter a message"
        );
        assert_eq!(
            get_send_result_message(&SendMessageResult::Disconnected),
            "Not connected to server"
        );
        assert!(get_send_result_message(&SendMessageResult::SigningFailed("test".to_string()))
            .contains("Signing failed"));
        assert!(get_send_result_message(&SendMessageResult::TransmissionFailed("test".to_string()))
            .contains("Failed to send"));
    }
}
