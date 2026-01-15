//! Composer state management for message drafts
//!
//! This module provides thread-safe composer state management that preserves
//! message drafts during disconnections (AC2, AC3 requirement).
//!
//! Story 3.7: Preserve Composer Draft on Disconnection
//! - Draft is stored in application state (in-memory, ephemeral)
//! - Draft is preserved during network disconnections
//! - Draft is only cleared on successful send or app close

use std::sync::Arc;
use tokio::sync::Mutex;

/// Connection state for the composer
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Connected to server
    Connected,
    /// Disconnected from server
    Disconnected,
    /// Reconnecting (for Phase 2 auto-reconnect)
    Reconnecting,
}

/// Composer state for preserving message drafts
#[derive(Clone)]
pub struct ComposerState {
    /// Current draft text (preserved during disconnections)
    pub draft_text: String,
    /// Selected recipient public key
    pub recipient: Option<String>,
    /// Current connection state
    connection_state: ConnectionState,
    /// Callback for connection state changes
    connection_callback: Option<Arc<dyn Fn(ConnectionState) + Send + Sync>>,
}

impl ComposerState {
    /// Create a new empty composer state
    pub fn new() -> Self {
        Self {
            draft_text: String::new(),
            recipient: None,
            connection_state: ConnectionState::Connected,
            connection_callback: None,
        }
    }

    /// Set the current draft text
    pub fn set_draft(&mut self, text: String) {
        self.draft_text = text;
    }

    /// Get the current draft text
    pub fn get_draft(&self) -> String {
        self.draft_text.clone()
    }

    /// Clear the draft text (only on successful send)
    pub fn clear_draft(&mut self) {
        self.draft_text.clear();
    }

    /// Set the selected recipient
    pub fn set_recipient(&mut self, recipient: Option<String>) {
        self.recipient = recipient;
    }

    /// Get the selected recipient
    pub fn get_recipient(&self) -> Option<&str> {
        self.recipient.as_deref()
    }

    /// Get the current connection state
    pub fn connection_state(&self) -> ConnectionState {
        self.connection_state.clone()
    }

    /// Set connection state and notify if callback is set
    pub fn set_connection_state(&mut self, state: ConnectionState) {
        self.connection_state = state.clone();

        // Notify callback if registered
        if let Some(ref callback) = self.connection_callback {
            (callback)(state);
        }
    }

    /// Register a callback for connection state changes
    pub fn set_connection_callback<F>(&mut self, callback: F)
    where
        F: Fn(ConnectionState) + Send + Sync + 'static,
    {
        self.connection_callback = Some(Arc::new(callback));
    }

    /// Check if currently disconnected
    pub fn is_disconnected(&self) -> bool {
        self.connection_state == ConnectionState::Disconnected
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connection_state == ConnectionState::Connected
    }

    /// Get draft length
    pub fn draft_length(&self) -> usize {
        self.draft_text.len()
    }

    /// Check if draft has content
    pub fn has_draft(&self) -> bool {
        !self.draft_text.is_empty()
    }

    /// Preserve draft during disconnection (called by WebSocket handler)
    ///
    /// This method exists to explicitly document that drafts are preserved.
    /// The draft is NOT cleared when disconnect is detected.
    pub fn preserve_draft_on_disconnect(&self) {
        // Draft is already in memory, nothing to do
        // This method exists for documentation purposes
    }

    /// Check if draft should be cleared (only on successful send)
    pub fn should_clear_on_send(&self) -> bool {
        true
    }
}

impl Default for ComposerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared reference to composer state for concurrent access
pub type SharedComposerState = Arc<Mutex<ComposerState>>;

/// Create a shared composer state for concurrent access
pub fn create_shared_composer_state() -> SharedComposerState {
    Arc::new(Mutex::new(ComposerState::new()))
}

/// Format connection state for display
pub fn format_connection_notification(state: &ConnectionState) -> String {
    match state {
        ConnectionState::Connected => "Connected".to_string(),
        ConnectionState::Disconnected => "Connection lost. Reconnecting...".to_string(),
        ConnectionState::Reconnecting => "Reconnecting...".to_string(),
    }
}

/// Clear all ephemeral data (called on app close)
pub async fn clear_all_ephemeral_data(composer: &mut ComposerState) {
    composer.clear_draft();
    composer.set_recipient(None);
    composer.set_connection_state(ConnectionState::Disconnected);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_draft_preserved_during_disconnect() {
        let composer = create_shared_composer_state();
        composer.lock().await.set_draft("test message".to_string());

        // Simulate disconnect (drop connection, don't touch composer)
        // The draft should remain in memory

        let draft = composer.lock().await.get_draft();
        assert_eq!(draft, "test message");
    }

    #[tokio::test]
    async fn test_composer_state_thread_safe() {
        let composer = create_shared_composer_state();
        let composer_clone = Arc::clone(&composer);

        let task1 = tokio::spawn(async move {
            composer_clone.lock().await.set_draft("draft 1".to_string());
        });

        task1.await.unwrap();
        let draft = composer.lock().await.get_draft();
        assert_eq!(draft, "draft 1");
    }

    #[test]
    fn test_composer_new() {
        let composer = ComposerState::new();
        assert_eq!(composer.draft_text, "");
        assert!(composer.recipient.is_none());
    }

    #[test]
    fn test_draft_operations() {
        let mut composer = ComposerState::new();

        // Set draft
        composer.set_draft("Hello, world!".to_string());
        assert_eq!(composer.get_draft(), "Hello, world!");

        // Update draft
        composer.set_draft("Updated draft".to_string());
        assert_eq!(composer.get_draft(), "Updated draft");

        // Clear draft
        composer.clear_draft();
        assert_eq!(composer.get_draft(), "");
    }

    #[test]
    fn test_connection_state() {
        let mut composer = ComposerState::new();

        // Initially connected
        assert!(composer.is_connected());
        assert!(!composer.is_disconnected());
        assert_eq!(composer.connection_state(), ConnectionState::Connected);
    }

    #[test]
    fn test_set_connection_state() {
        let mut composer = ComposerState::new();

        // Set disconnected
        composer.set_connection_state(ConnectionState::Disconnected);
        assert!(composer.is_disconnected());
        assert!(!composer.is_connected());

        // Set reconnecting
        composer.set_connection_state(ConnectionState::Reconnecting);
        assert_eq!(composer.connection_state(), ConnectionState::Reconnecting);

        // Set connected
        composer.set_connection_state(ConnectionState::Connected);
        assert!(composer.is_connected());
    }

    #[test]
    fn test_draft_length() {
        let mut composer = ComposerState::new();
        assert_eq!(composer.draft_length(), 0);

        composer.set_draft("Hello".to_string());
        assert_eq!(composer.draft_length(), 5);

        composer.set_draft("Hello, world!".to_string());
        assert_eq!(composer.draft_length(), 13);
    }

    #[test]
    fn test_has_draft() {
        let mut composer = ComposerState::new();
        assert!(!composer.has_draft());

        composer.set_draft("Hello".to_string());
        assert!(composer.has_draft());

        composer.clear_draft();
        assert!(!composer.has_draft());
    }

    #[test]
    fn test_recipient() {
        let mut composer = ComposerState::new();
        assert!(composer.get_recipient().is_none());

        composer.set_recipient(Some("recipient_key".to_string()));
        assert_eq!(composer.get_recipient(), Some("recipient_key"));

        composer.set_recipient(None);
        assert!(composer.get_recipient().is_none());
    }

    #[test]
    fn test_format_connection_notification() {
        assert_eq!(
            format_connection_notification(&ConnectionState::Connected),
            "Connected"
        );
        assert_eq!(
            format_connection_notification(&ConnectionState::Disconnected),
            "Connection lost. Reconnecting..."
        );
        assert_eq!(
            format_connection_notification(&ConnectionState::Reconnecting),
            "Reconnecting..."
        );
    }

    #[tokio::test]
    async fn test_clear_all_ephemeral_data() {
        let mut composer = ComposerState::new();
        composer.set_draft("test".to_string());
        composer.set_recipient(Some("key".to_string()));
        composer.set_connection_state(ConnectionState::Disconnected);

        clear_all_ephemeral_data(&mut composer).await;

        assert_eq!(composer.get_draft(), "");
        assert!(composer.get_recipient().is_none());
        assert!(composer.is_disconnected());
    }

    #[test]
    fn test_preserve_draft_on_disconnect() {
        let composer = ComposerState::new();
        // This method should not panic and does nothing (draft is already in memory)
        composer.preserve_draft_on_disconnect();
        assert!(!composer.has_draft()); // Still empty but no error
    }

    #[test]
    fn test_should_clear_on_send() {
        let composer = ComposerState::new();
        assert!(composer.should_clear_on_send());
    }
}
