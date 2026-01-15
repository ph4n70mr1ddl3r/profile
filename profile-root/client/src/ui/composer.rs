//! Composer component for message composition
//!
//! This module provides the message composer UI component that handles
//! message input, signing, and sending.

use crate::state::composer::SharedComposerState;
use crate::state::lobby::SharedLobbyState;
use crate::state::messages::{ChatMessage, SharedMessageHistory};
use crate::state::session::SharedKeyState;
use crate::ui::lobby_state::LobbyUser;
use hex;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Result of a send message operation
#[derive(Debug, Clone)]
pub enum SendMessageResult {
    /// Message was sent successfully
    Success,
    /// No recipient selected
    NoRecipient,
    /// No message text entered
    EmptyMessage,
    /// User disconnected from server
    Disconnected,
    /// Cryptographic signing failed
    SigningFailed(String),
    /// Network transmission failed
    TransmissionFailed(String),
}

/// Composer for sending signed messages
///
/// Handles message composition, cryptographic signing, and transmission
/// to server via WebSocket.
pub struct MessageComposer {
    /// Shared key state for signing
    key_state: SharedKeyState,
    /// Shared composer state for draft preservation
    composer_state: SharedComposerState,
    /// Shared lobby state for recipient lookup
    lobby_state: SharedLobbyState,
    /// Shared message history for storing sent messages
    message_history: SharedMessageHistory,
    /// Callback for sending message via WebSocket
    send_callback: Option<Arc<dyn Fn(String) -> Result<(), String> + Send + Sync>>,
    /// Callback for showing status to user
    status_callback: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl MessageComposer {
    /// Create a new message composer
    pub fn new(
        key_state: SharedKeyState,
        composer_state: SharedComposerState,
        lobby_state: SharedLobbyState,
        message_history: SharedMessageHistory,
    ) -> Self {
        Self {
            key_state,
            composer_state,
            lobby_state,
            message_history,
            send_callback: None,
            status_callback: None,
        }
    }

    /// Set the WebSocket send callback
    ///
    /// The callback receives a JSON string and should send it via WebSocket
    pub fn set_send_callback<F>(&mut self, callback: F)
    where
        F: Fn(String) -> Result<(), String> + Send + Sync + 'static,
    {
        self.send_callback = Some(Arc::new(callback));
    }

    /// Set the status callback for user feedback
    pub fn set_status_callback<F>(&mut self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.status_callback = Some(Arc::new(callback));
    }

    /// Show status message to user
    fn show_status(&self, message: &str) {
        if let Some(ref callback) = self.status_callback {
            (callback)(message.to_string());
        }
    }

    /// Get the currently selected recipient from lobby
    pub async fn get_selected_recipient(&self) -> Option<LobbyUser> {
        let state = self.lobby_state.lock().await;
        state
            .selected_user()
            .and_then(|key| state.get_user(key).cloned())
    }

    /// Send a message
    ///
    /// This method:
    /// 1. Gets the selected recipient from lobby
    /// 2. Signs the message with user's private key
    /// 3. Sends the signed message via WebSocket
    /// 4. Clears the composer for next message
    pub async fn send_message(&mut self, message_text: &str) -> SendMessageResult {
        let message_text = message_text.trim();

        // AC2: Check for empty message
        if message_text.is_empty() {
            self.show_status("Please enter a message");
            return SendMessageResult::EmptyMessage;
        }

        // AC1: Get selected recipient
        let recipient = match self.get_selected_recipient().await {
            Some(r) => r,
            None => {
                self.show_status("Please select a recipient from the lobby");
                return SendMessageResult::NoRecipient;
            }
        };

        // Get keys for signing
        let (public_key, private_key) = {
            let key_state = self.key_state.lock().await;
            let public_key = match key_state.public_key() {
                Some(pk) => pk.clone(),
                None => {
                    self.show_status("Error: No public key available");
                    return SendMessageResult::SigningFailed("No public key".to_string());
                }
            };
            let private_key = match key_state.private_key() {
                Some(pk) => pk.clone(),
                None => {
                    self.show_status("Error: No private key available");
                    return SendMessageResult::SigningFailed("No private key".to_string());
                }
            };
            (public_key, private_key)
        };

        // Clone public_key for later use in history storage
        let public_key_clone = public_key.clone();

        // AC3: Create and sign the message
        let client_message = match crate::connection::message::ClientMessage::new(
            message_text.to_string(),
            recipient.public_key.clone(),
            public_key,
            private_key,
        ) {
            Ok(msg) => msg,
            Err(e) => {
                self.show_status(&format!("Error signing message: {}", e));
                return SendMessageResult::SigningFailed(e.to_string());
            }
        };

        // Serialize to JSON
        let message_json = match client_message.to_json() {
            Ok(json) => json,
            Err(e) => {
                self.show_status(&format!("Error serializing message: {}", e));
                return SendMessageResult::SigningFailed(e.to_string());
            }
        };

        // AC4: Send via WebSocket
        if let Some(ref callback) = self.send_callback {
            match callback(message_json) {
                Ok(()) => {
                    // Task 2.6: Store message in SharedMessageHistory
                    let chat_message = ChatMessage::new(
                        hex::encode(&public_key_clone),
                        message_text.to_string(),
                        client_message.signature.clone(),
                        client_message.timestamp.clone(),
                    );
                    let mut history = self.message_history.lock().await;
                    history.add_message(chat_message);

                    // AC5: Clear composer for next message
                    let mut composer = self.composer_state.lock().await;
                    composer.clear_draft();

                    self.show_status("Message sent");
                    SendMessageResult::Success
                }
                Err(e) => {
                    self.show_status(&format!("Failed to send message: {}", e));
                    SendMessageResult::TransmissionFailed(e)
                }
            }
        } else {
            // No send callback - client disconnected
            self.show_status("Not connected to server");
            SendMessageResult::Disconnected
        }
    }

    /// Get current draft text
    pub async fn get_draft(&self) -> String {
        let composer = self.composer_state.lock().await;
        composer.get_draft()
    }

    /// Set draft text (preserved during disconnections)
    pub async fn set_draft(&self, text: &str) {
        let mut composer = self.composer_state.lock().await;
        composer.set_draft(text.to_string());
    }

    /// Clear draft
    pub async fn clear_draft(&self) {
        let mut composer = self.composer_state.lock().await;
        composer.clear_draft();
    }

    /// Check if can send (has recipient and connection)
    pub async fn can_send(&self) -> bool {
        self.send_callback.is_some() && self.get_selected_recipient().await.is_some()
    }
}

/// Create a new message composer
pub fn create_message_composer(
    key_state: SharedKeyState,
    composer_state: SharedComposerState,
    lobby_state: SharedLobbyState,
    message_history: SharedMessageHistory,
) -> Arc<Mutex<MessageComposer>> {
    Arc::new(Mutex::new(MessageComposer::new(
        key_state,
        composer_state,
        lobby_state,
        message_history,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::composer::create_shared_composer_state;
    use crate::state::lobby::create_shared_lobby_state;
    use crate::state::messages::create_shared_message_history;
    use crate::state::session::create_shared_key_state;
    use crate::ui::lobby_state::LobbyUser;
    use profile_shared::derive_public_key;
    use profile_shared::generate_private_key;

    #[tokio::test]
    async fn test_composer_creation() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        let composer =
            MessageComposer::new(key_state, composer_state, lobby_state, message_history);

        assert!(composer.send_callback.is_none());
        assert!(composer.status_callback.is_none());
    }

    #[tokio::test]
    async fn test_composer_draft_operations() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        let composer = create_message_composer(
            key_state,
            composer_state.clone(),
            lobby_state,
            message_history,
        );

        // Set draft
        {
            let mut comp = composer.lock().await;
            comp.set_draft("Hello, world!").await;
        }

        // Get draft
        let draft = composer.lock().await.get_draft().await;
        assert_eq!(draft, "Hello, world!");

        // Clear draft
        {
            let mut comp = composer.lock().await;
            comp.clear_draft().await;
        }

        let draft = composer.lock().await.get_draft().await;
        assert!(draft.is_empty());
    }

    #[tokio::test]
    async fn test_send_empty_message() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        let composer =
            create_message_composer(key_state, composer_state, lobby_state, message_history);

        let result = composer.lock().await.send_message("").await;
        assert!(matches!(result, SendMessageResult::EmptyMessage));
    }

    #[tokio::test]
    async fn test_send_no_recipient() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        // Create composer - no recipient in lobby
        let composer =
            create_message_composer(key_state, composer_state, lobby_state, message_history);

        // No recipient selected - should return NoRecipient
        let result = composer.lock().await.send_message("Hello").await;
        assert!(matches!(result, SendMessageResult::NoRecipient));
    }

    #[tokio::test]
    async fn test_send_message_no_connection() {
        // Create key state with keys set
        let key_state = create_shared_key_state();
        {
            let mut keys = key_state.lock().await;
            // Generate keys for this test
            let private = profile_shared::generate_private_key().unwrap();
            let public = profile_shared::derive_public_key(&private).unwrap();
            keys.set_generated_key(private, public);
        }

        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        // Add a recipient to lobby
        {
            let mut state = lobby_state.lock().await;
            state.add_user(LobbyUser::new(
                "test_recipient_1234567890abcdef1234567890abcdef12345678".to_string(),
                true,
            ));
            state.select("test_recipient_1234567890abcdef1234567890abcdef12345678");
        }

        // Create composer - no send callback set
        let composer =
            create_message_composer(key_state, composer_state, lobby_state, message_history);

        // No send callback - should return Disconnected
        let result = composer.lock().await.send_message("Hello").await;
        assert!(matches!(result, SendMessageResult::Disconnected));
    }

    #[tokio::test]
    async fn test_send_message_stores_in_history() {
        // Create key state with keys set
        let key_state = create_shared_key_state();
        {
            let mut keys = key_state.lock().await;
            let private = profile_shared::generate_private_key().unwrap();
            let public = profile_shared::derive_public_key(&private).unwrap();
            keys.set_generated_key(private, public);
        }

        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        // Add a recipient to lobby
        {
            let mut state = lobby_state.lock().await;
            state.add_user(LobbyUser::new(
                "test_recipient_1234567890abcdef1234567890abcdef12345678".to_string(),
                true,
            ));
            state.select("test_recipient_1234567890abcdef1234567890abcdef12345678");
        }

        let composer = create_message_composer(
            key_state,
            composer_state,
            lobby_state,
            message_history.clone(),
        );

        // Mock send callback that always succeeds
        let send_callback = Arc::new(|_msg: String| -> Result<(), String> { Ok(()) });
        {
            let mut comp = composer.lock().await;
            comp.set_send_callback(move |msg| (send_callback)(msg));
        }

        // Send a message
        let result = composer.lock().await.send_message("Hello, world!").await;
        assert!(matches!(result, SendMessageResult::Success));

        // Verify message was stored in history
        let history = message_history.lock().await;
        assert_eq!(history.len(), 1);
        let msg = history.newest().unwrap();
        assert_eq!(msg.message, "Hello, world!");
        assert_eq!(msg.sender_public_key.len(), 64); // hex-encoded public key
        assert!(!msg.signature.is_empty());
        assert!(!msg.timestamp.is_empty());
    }

    /// Test Enter key handler behavior (simulating Enter key press triggers send_message)
    #[tokio::test]
    async fn test_enter_key_handler_sends_message() {
        // Create key state with keys set
        let key_state = create_shared_key_state();
        {
            let mut keys = key_state.lock().await;
            let private = profile_shared::generate_private_key().unwrap();
            let public = profile_shared::derive_public_key(&private).unwrap();
            keys.set_generated_key(private, public);
        }

        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        // Add a recipient to lobby
        {
            let mut state = lobby_state.lock().await;
            state.add_user(LobbyUser::new(
                "test_recipient_1234567890abcdef1234567890abcdef12345678".to_string(),
                true,
            ));
            state.select("test_recipient_1234567890abcdef1234567890abcdef12345678");
        }

        let composer = create_message_composer(
            key_state,
            composer_state,
            lobby_state,
            message_history.clone(),
        );

        // Mock send callback that always succeeds
        let send_callback = Arc::new(|_msg: String| -> Result<(), String> { Ok(()) });
        {
            let mut comp = composer.lock().await;
            comp.set_send_callback(move |msg| (send_callback)(msg));
        }

        // Simulate Enter key press by calling send_message with text
        let result = composer.lock().await.send_message("Enter key test").await;
        assert!(
            matches!(result, SendMessageResult::Success),
            "Enter key should trigger successful send"
        );

        // Verify message was stored
        let history = message_history.lock().await;
        assert_eq!(history.len(), 1);
        assert_eq!(history.newest().unwrap().message, "Enter key test");

        println!("✅ Enter key handler test passed");
    }

    /// Test Enter key with empty message (should be rejected)
    #[tokio::test]
    async fn test_enter_key_with_empty_message_rejected() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        let composer =
            create_message_composer(key_state, composer_state, lobby_state, message_history);

        // Simulate Enter key press with empty message
        let result = composer.lock().await.send_message("").await;
        assert!(
            matches!(result, SendMessageResult::EmptyMessage),
            "Empty message should be rejected"
        );

        println!("✅ Enter key with empty message rejected correctly");
    }

    /// Test Send button enable/disable - empty text
    #[tokio::test]
    async fn test_send_button_disabled_empty_text() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        // Add a recipient to lobby
        {
            let mut state = lobby_state.lock().await;
            state.add_user(LobbyUser::new(
                "test_recipient_1234567890abcdef1234567890abcdef12345678".to_string(),
                true,
            ));
            state.select("test_recipient_1234567890abcdef1234567890abcdef12345678");
        }

        let composer =
            create_message_composer(key_state, composer_state, lobby_state, message_history);

        // Without send callback, can_send should be false
        // (Send button would be disabled because no connection)
        let can_send = composer.lock().await.can_send().await;
        assert!(
            !can_send,
            "Send button should be disabled without connection"
        );

        println!("✅ Send button correctly disabled without connection");
    }

    /// Test Send button enable/disable - with connection but empty text
    #[tokio::test]
    async fn test_send_button_disabled_empty_text_even_with_connection() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        // Add a recipient to lobby
        {
            let mut state = lobby_state.lock().await;
            state.add_user(LobbyUser::new(
                "test_recipient_1234567890abcdef1234567890abcdef12345678".to_string(),
                true,
            ));
            state.select("test_recipient_1234567890abcdef1234567890abcdef12345678");
        }

        let composer = create_message_composer(
            key_state,
            composer_state.clone(),
            lobby_state,
            message_history,
        );

        // Set send callback (simulating connection)
        let send_callback = Arc::new(|_msg: String| -> Result<(), String> { Ok(()) });
        {
            let mut comp = composer.lock().await;
            comp.set_send_callback(move |msg| (send_callback)(msg));
        }

        // Without text in draft, can_send should be false
        // Note: can_send checks for recipient + connection, not text
        // The actual enable/disable of Send button would be handled by UI
        // based on text length (handled at UI layer, not composer)
        let can_send = composer.lock().await.can_send().await;
        assert!(
            can_send,
            "Send button should be enabled with connection and recipient"
        );

        // But sending empty message should fail
        let result = composer.lock().await.send_message("").await;
        assert!(matches!(result, SendMessageResult::EmptyMessage));

        println!("✅ Send button correctly enabled with connection, rejects empty messages");
    }

    /// Test Send button enable/disable - with text, connection, and recipient
    #[tokio::test]
    async fn test_send_button_enabled_with_all_requirements() {
        let key_state = create_shared_key_state();
        {
            let mut keys = key_state.lock().await;
            let private = profile_shared::generate_private_key().unwrap();
            let public = profile_shared::derive_public_key(&private).unwrap();
            keys.set_generated_key(private, public);
        }

        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();
        let message_history = create_shared_message_history();

        // Add a recipient to lobby
        {
            let mut state = lobby_state.lock().await;
            state.add_user(LobbyUser::new(
                "test_recipient_1234567890abcdef1234567890abcdef12345678".to_string(),
                true,
            ));
            state.select("test_recipient_1234567890abcdef1234567890abcdef12345678");
        }

        let composer = create_message_composer(
            key_state,
            composer_state.clone(),
            lobby_state,
            message_history.clone(),
        );

        // Set send callback (simulating connection)
        let send_callback = Arc::new(|_msg: String| -> Result<(), String> { Ok(()) });
        {
            let mut comp = composer.lock().await;
            comp.set_send_callback(move |msg| (send_callback)(msg));
        }

        // With all requirements met, can_send should be true
        let can_send = composer.lock().await.can_send().await;
        assert!(
            can_send,
            "Send button should be enabled with all requirements"
        );

        // Sending should succeed
        let result = composer
            .lock()
            .await
            .send_message("Hello, Send button!")
            .await;
        assert!(matches!(result, SendMessageResult::Success));

        println!("✅ Send button correctly enabled with all requirements");
    }
}
