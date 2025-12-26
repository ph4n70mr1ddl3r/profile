//! Composer component for message composition
//!
//! This module provides the message composer UI component that handles
//! message input, signing, and sending.

use crate::state::session::SharedKeyState;
use crate::state::composer::SharedComposerState;
use crate::state::lobby::SharedLobbyState;
use crate::ui::lobby_state::LobbyUser;
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
/// to the server via WebSocket.
pub struct MessageComposer {
    /// Shared key state for signing
    key_state: SharedKeyState,
    /// Shared composer state for draft preservation
    composer_state: SharedComposerState,
    /// Shared lobby state for recipient lookup
    lobby_state: SharedLobbyState,
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
    ) -> Self {
        Self {
            key_state,
            composer_state,
            lobby_state,
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
        state.selected_user().and_then(|key| state.get_user(key).cloned())
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
) -> Arc<Mutex<MessageComposer>> {
    Arc::new(Mutex::new(MessageComposer::new(
        key_state,
        composer_state,
        lobby_state,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::session::create_shared_key_state;
    use crate::state::composer::create_shared_composer_state;
    use crate::state::lobby::create_shared_lobby_state;
    use crate::ui::lobby_state::LobbyUser;
    use profile_shared::generate_private_key;
    use profile_shared::derive_public_key;

    #[tokio::test]
    async fn test_composer_creation() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();

        let composer = MessageComposer::new(
            key_state,
            composer_state,
            lobby_state,
        );

        assert!(composer.send_callback.is_none());
        assert!(composer.status_callback.is_none());
    }

    #[tokio::test]
    async fn test_composer_draft_operations() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();

        let composer = create_message_composer(key_state, composer_state.clone(), lobby_state);

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

        let composer = create_message_composer(key_state, composer_state, lobby_state);

        let result = composer.lock().await.send_message("").await;
        assert!(matches!(result, SendMessageResult::EmptyMessage));
    }

    #[tokio::test]
    async fn test_send_no_recipient() {
        let key_state = create_shared_key_state();
        let composer_state = create_shared_composer_state();
        let lobby_state = create_shared_lobby_state();

        // Create composer - no recipient in lobby
        let composer = create_message_composer(key_state, composer_state, lobby_state);

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

        // Add a recipient to lobby
        {
            let mut state = lobby_state.lock().await;
            state.add_user(LobbyUser::new("test_recipient_1234567890abcdef1234567890abcdef12345678".to_string(), true));
            state.select("test_recipient_1234567890abcdef1234567890abcdef12345678");
        }

        // Create composer - no send callback set
        let composer = create_message_composer(key_state, composer_state, lobby_state);

        // No send callback - should return Disconnected
        let result = composer.lock().await.send_message("Hello").await;
        assert!(matches!(result, SendMessageResult::Disconnected));
    }
}
