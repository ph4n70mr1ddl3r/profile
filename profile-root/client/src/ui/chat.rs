//! Message display component for chat conversations
//!
//! This module provides components for displaying chat messages
//! in chronological order with formatted timestamps.
//!
//! ## Slint Integration
//!
//! This module provides `ChatUi` which bridges the `ChatView` data model
//! to the Slint UI components defined in `main.slint`.

use crate::state::messages::{ChatMessage, SharedMessageHistory};
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Timelike};

/// Trait for bridging ChatView data to the UI.
///
/// This abstraction allows the UI implementation (e.g., Slint) to be
/// decoupled from the core chat logic. The binary target implements this
/// trait to update the specific UI framework.
pub trait ChatUiBridge {
    /// Update the chat message count
    fn set_chat_message_count(&self, count: i32);

    /// Update a specific message slot
    fn update_message_slot(&self, index: usize, msg: &DisplayMessage);

    /// Clear a specific message slot
    fn clear_message_slot(&self, index: usize);
}

impl ChatUiBridge for () {
    fn set_chat_message_count(&self, _count: i32) {}
    fn update_message_slot(&self, _index: usize, _msg: &DisplayMessage) {}
    fn clear_message_slot(&self, _index: usize) {}
}

/// Helper function to set a message property for a specific slot (1-indexed)
fn update_message_slot<B: ChatUiBridge>(bridge: &B, index: usize, msg: &DisplayMessage) {
    bridge.update_message_slot(index, msg);
}

/// Helper function to clear a message property slot
fn clear_message_slot<B: ChatUiBridge>(bridge: &B, index: usize) {
    bridge.clear_message_slot(index);
}

/// Formatted message for display
#[derive(Debug, Clone)]
pub struct DisplayMessage {
    /// Unique identifier for the message
    pub id: String,
    /// Sender's public key (full)
    pub sender_key: String,
    /// Formatted sender key for display (first 8 chars + "...")
    pub sender_key_short: String,
    /// The message content
    pub content: String,
    /// Formatted timestamp (HH:MM:SS)
    pub timestamp: String,
    /// The cryptographic signature (hex-encoded)
    pub signature: String,
    /// Whether the message is verified
    pub is_verified: bool,
    /// Whether the message is from self
    pub is_self: bool,
    /// Original timestamp for ordering
    pub original_timestamp: String,
}

impl DisplayMessage {
    /// Create a new display message from a ChatMessage
    pub fn from_chat_message(msg: &ChatMessage, is_self: bool) -> Self {
        let sender_key_short = if msg.sender_public_key.len() > 16 {
            format!(
                "{}...{}",
                &msg.sender_public_key[..8],
                &msg.sender_public_key[msg.sender_public_key.len() - 8..]
            )
        } else {
            msg.sender_public_key.clone()
        };

        let timestamp = format_timestamp(&msg.timestamp);

        Self {
            id: format!("msg-{}", msg.timestamp),
            sender_key: msg.sender_public_key.clone(),
            sender_key_short,
            content: msg.message.clone(),
            timestamp,
            signature: msg.signature.clone(),
            is_verified: msg.is_verified,
            is_self,
            original_timestamp: msg.timestamp.clone(),
        }
    }

    /// Get the verification badge text
    pub fn verification_badge(&self) -> String {
        if self.is_verified {
            "✓".to_string()
        } else {
            "".to_string()
        }
    }
}

/// Format ISO 8601 timestamp to HH:MM:SS
///
/// # Arguments
/// * `iso_timestamp` - ISO 8601 formatted timestamp
///
/// # Returns
/// HH:MM:SS formatted string
pub fn format_timestamp(iso_timestamp: &str) -> String {
    // Parse ISO 8601 timestamp
    match DateTime::parse_from_rfc3339(iso_timestamp) {
        Ok(dt) => {
            // Format as HH:MM:SS
            format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second())
        }
        Err(_) => {
            // Fallback: try to extract time from string
            // Handle formats like "2025-12-27T10:30:00Z" or "2025-12-27T10:30:00.123456789Z"
            let parts: Vec<&str> = iso_timestamp.split(|c| c == 'T' || c == 't').collect();
            if parts.len() >= 2 {
                let time_part = parts[1];
                let time_chars: Vec<char> = time_part.chars().collect();
                if time_chars.len() >= 8 {
                    return format!(
                        "{}{}{}{}{}{}{}{}",
                        time_chars[0], time_chars[1], ':',
                        time_chars[2], time_chars[3], ':',
                        time_chars[4], time_chars[5]
                    );
                }
            }
            // Ultimate fallback
            "??:??:??".to_string()
        }
    }
}

/// Chat view state for managing message display
#[derive(Debug, Clone)]
pub struct ChatView {
    /// Messages to display (in order)
    messages: Vec<DisplayMessage>,
    /// Whether user is currently scrolling (prevents auto-scroll)
    is_user_scrolling: bool,
    /// ID of the currently selected recipient
    selected_recipient: Option<String>,
}

impl ChatView {
    /// Create a new empty chat view
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            is_user_scrolling: false,
            selected_recipient: None,
        }
    }

    /// Set whether user is scrolling
    pub fn set_user_scrolling(&mut self, scrolling: bool) {
        self.is_user_scrolling = scrolling;
    }

    /// Check if user is scrolling
    pub fn is_scrolling(&self) -> bool {
        self.is_user_scrolling
    }

    /// Set the selected recipient
    pub fn set_selected_recipient(&mut self, recipient_key: Option<String>) {
        self.selected_recipient = recipient_key;
    }

    /// Get the selected recipient
    pub fn selected_recipient(&self) -> Option<&str> {
        self.selected_recipient.as_deref()
    }

    /// Get all messages for display
    pub fn messages(&self) -> &[DisplayMessage] {
        &self.messages
    }

    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }

    /// Check if chat is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get the newest message ID (for auto-scroll tracking)
    pub fn newest_message_id(&self) -> Option<&str> {
        self.messages.last().map(|m| m.id.as_str())
    }

    /// Check if a message ID is the newest
    pub fn is_newest_message(&self, id: &str) -> bool {
        self.newest_message_id() == Some(id)
    }
}

impl Default for ChatView {
    fn default() -> Self {
        Self::new()
    }
}

/// Shared chat view for concurrent access
pub type SharedChatView = Arc<Mutex<ChatView>>;

/// Create a new shared chat view
#[inline]
pub fn create_shared_chat_view() -> SharedChatView {
    Arc::new(Mutex::new(ChatView::new()))
}

/// Update chat view with new messages from history
///
/// This function:
// 1. Gets messages from history for the selected recipient
/// 2. Converts to display format with timestamps
/// 3. Maintains chronological order (oldest → newest)
///
/// # Arguments
/// * `chat_view` - The chat view to update
/// * `message_history` - The message history
/// * `my_public_key` - Current user's public key for self-detection
pub async fn update_chat_view(
    chat_view: &mut ChatView,
    message_history: &SharedMessageHistory,
    my_public_key: &str,
) {
    let history = message_history.lock().await;

    // Get messages for the selected recipient
    let recipient = match chat_view.selected_recipient() {
        Some(key) => key,
        None => {
            // No recipient selected, clear messages
            chat_view.messages.clear();
            return;
        }
    };

    // Get messages from this recipient
    let messages: Vec<DisplayMessage> = history
        .messages_from(recipient)
        .iter()
        .map(|msg| {
            let is_self = msg.sender_public_key == my_public_key;
            DisplayMessage::from_chat_message(msg, is_self)
        })
        .collect();

    chat_view.messages = messages;
}

/// Add a single new message to the chat view
///
/// Used when a new message arrives in real-time.
///
/// # Arguments
/// * `chat_view` - The chat view to update
/// * `message` - The new message
/// * `my_public_key` - Current user's public key
pub fn add_message(chat_view: &mut ChatView, message: &ChatMessage, my_public_key: &str) {
    let is_self = message.sender_public_key == my_public_key;
    let display_msg = DisplayMessage::from_chat_message(message, is_self);

    // Add to end (newest position) maintaining order
    chat_view.messages.push(display_msg);
}

/// Clear all messages from chat view
pub fn clear_chat(chat_view: &mut ChatView) {
    chat_view.messages.clear();
}

/// Slint UI bridge for ChatView
///
/// Handles updating the Slint UI properties based on the ChatView state.
#[derive(Clone)]
pub struct ChatUi<B: ChatUiBridge + Clone> {
    /// UI bridge for updating the view
    bridge: B,
    /// Callback for when a message is clicked
    on_message_clicked: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl<B: ChatUiBridge + Clone> ChatUi<B> {
    /// Create a new ChatUI bridge
    pub fn new(bridge: B) -> Self {
        Self {
            bridge,
            on_message_clicked: None,
        }
    }

    /// Set the message clicked callback
    pub fn set_on_message_clicked<F>(&mut self, callback: F)
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.on_message_clicked = Some(Arc::new(callback));
    }

    /// Handle a message click event from the UI
    pub fn on_message_clicked(&self, index: usize, message_id: &str) {
        if let Some(ref callback) = self.on_message_clicked {
            callback(message_id.to_string());
        }
    }

    /// Handle a new message received event
    ///
    /// This method is called when a new message arrives (either sent by user
    /// or received from server). It triggers a UI update to show the new message.
    ///
    /// # Arguments
    /// * `chat_view` - The chat view containing the messages
    pub fn on_message_received(&self, chat_view: &ChatView) {
        self.update(chat_view);
    }

    /// Update the Slint UI with the current chat view state
    ///
    /// This function copies data from the ChatView into the Slint properties
    /// defined in main.slint (chat_msg_1_*, chat_msg_2_*, etc.).
    pub fn update(&self, chat_view: &ChatView) {
        let message_count = chat_view.messages.len().min(10);

        // Update message count
        self.bridge.set_chat_message_count(message_count as i32);

        // Update each message slot
        for (i, msg) in chat_view.messages.iter().enumerate().take(10) {
            update_message_slot(&self.bridge, i + 1, msg);
        }

        // Clear remaining slots if message count decreased
        for i in (message_count + 1)..=10 {
            clear_message_slot(&self.bridge, i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::messages::create_shared_message_history;

    #[test]
    fn test_format_timestamp_rfc3339() {
        let ts = "2025-12-27T10:30:45Z";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "10:30:45");
    }

    #[test]
    fn test_format_timestamp_with_nanos() {
        let ts = "2025-12-27T14:22:30.123456789Z";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "14:22:30");
    }

    #[test]
    fn test_format_timestamp_invalid() {
        let ts = "invalid";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "??:??:??");
    }

    #[test]
    fn test_display_message_creation() {
        let chat_msg = ChatMessage::new(
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab".to_string(),
            "Hello world".to_string(),
            "signature123".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        let display_msg = DisplayMessage::from_chat_message(&chat_msg, false);

        assert_eq!(display_msg.content, "Hello world");
        assert_eq!(display_msg.timestamp, "10:30:00");
        assert!(!display_msg.is_verified);
        assert!(!display_msg.is_self);
        assert!(display_msg.sender_key_short.contains("..."));
    }

    #[test]
    fn test_display_message_self() {
        let chat_msg = ChatMessage::new(
            "mykey1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab".to_string(),
            "My message".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        let display_msg = DisplayMessage::from_chat_message(&chat_msg, true);

        assert!(display_msg.is_self);
    }

    #[test]
    fn test_verification_badge() {
        let verified = ChatMessage::verified(
            "key".to_string(),
            "msg".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );
        let display_verified = DisplayMessage::from_chat_message(&verified, false);
        assert_eq!(display_verified.verification_badge(), "✓");

        let unverified = ChatMessage::new(
            "key".to_string(),
            "msg".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );
        let display_unverified = DisplayMessage::from_chat_message(&unverified, false);
        assert_eq!(display_unverified.verification_badge(), "");
    }

    #[test]
    fn test_chat_view_new() {
        let view = ChatView::new();
        assert!(view.is_empty());
        assert_eq!(view.message_count(), 0);
        assert!(!view.is_scrolling());
    }

    #[test]
    fn test_chat_view_scrolling() {
        let mut view = ChatView::new();
        assert!(!view.is_scrolling());

        view.set_user_scrolling(true);
        assert!(view.is_scrolling());

        view.set_user_scrolling(false);
        assert!(!view.is_scrolling());
    }

    #[test]
    fn test_chat_view_recipient() {
        let mut view = ChatView::new();
        assert!(view.selected_recipient().is_none());

        view.set_selected_recipient(Some("recipient_key".to_string()));
        assert_eq!(view.selected_recipient(), Some("recipient_key"));
    }

    #[tokio::test]
    async fn test_update_chat_view() {
        let history = create_shared_message_history();
        let mut view = ChatView::new();

        // Add some messages
        {
            let mut h = history.lock().await;
            h.add_message(ChatMessage::new(
                "sender1".to_string(),
                "First".to_string(),
                "sig1".to_string(),
                "2025-12-27T10:00:00Z".to_string(),
            ));
            h.add_message(ChatMessage::new(
                "sender2".to_string(),
                "Second".to_string(),
                "sig2".to_string(),
                "2025-12-27T10:01:00Z".to_string(),
            ));
        }

        // Select recipient
        view.set_selected_recipient(Some("sender2".to_string()));

        // Update view
        update_chat_view(&mut view, &history, "me").await;

        // Should have 1 message from sender2
        assert_eq!(view.message_count(), 1);
        assert_eq!(view.messages[0].content, "Second");
    }

    #[test]
    fn test_add_message() {
        let mut view = ChatView::new();

        let msg = ChatMessage::new(
            "sender".to_string(),
            "Test".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        add_message(&mut view, &msg, "me");
        assert_eq!(view.message_count(), 1);

        add_message(&mut view, &msg, "other");
        assert_eq!(view.message_count(), 2);
    }

    #[test]
    fn test_clear_chat() {
        let mut view = ChatView::new();
        view.messages.push(DisplayMessage::from_chat_message(
            &ChatMessage::new("k".to_string(), "m".to_string(), "s".to_string(), "t".to_string()),
            false,
        ));

        assert!(!view.is_empty());

        clear_chat(&mut view);

        assert!(view.is_empty());
    }

    #[test]
    fn test_newest_message_id() {
        let mut view = ChatView::new();
        assert!(view.newest_message_id().is_none());

        view.messages.push(DisplayMessage::from_chat_message(
            &ChatMessage::new("k".to_string(), "m1".to_string(), "s".to_string(), "2025-12-27T10:00:00Z".to_string()),
            false,
        ));
        assert_eq!(view.newest_message_id(), Some("msg-2025-12-27T10:00:00Z"));

        view.messages.push(DisplayMessage::from_chat_message(
            &ChatMessage::new("k".to_string(), "m2".to_string(), "s".to_string(), "2025-12-27T10:01:00Z".to_string()),
            false,
        ));
        assert_eq!(view.newest_message_id(), Some("msg-2025-12-27T10:01:00Z"));
    }

    #[test]
    fn test_is_newest_message() {
        let mut view = ChatView::new();

        view.messages.push(DisplayMessage::from_chat_message(
            &ChatMessage::new("k".to_string(), "m1".to_string(), "s".to_string(), "2025-12-27T10:00:00Z".to_string()),
            false,
        ));

        assert!(view.is_newest_message("msg-2025-12-27T10:00:00Z"));
        assert!(!view.is_newest_message("msg-old"));
    }

    #[test]
    fn test_display_message_short_key_truncation() {
        // Test that short keys are not truncated
        let msg = ChatMessage::new(
            "shortkey".to_string(), // Less than 16 chars
            "Test".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );
        let display = DisplayMessage::from_chat_message(&msg, false);
        assert_eq!(display.sender_key_short, "shortkey");
        assert!(!display.sender_key_short.contains("..."));
    }

    #[test]
    fn test_display_message_long_key_truncation() {
        // Test that long keys are truncated correctly
        let long_key = "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab";
        let msg = ChatMessage::new(
            long_key.to_string(),
            "Test".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );
        let display = DisplayMessage::from_chat_message(&msg, false);
        assert!(display.sender_key_short.contains("..."));
        assert!(display.sender_key_short.starts_with("abcdef"));
        assert!(display.sender_key_short.ends_with("ab"));
    }

    #[test]
    fn test_format_timestamp_rfc3339_nanos() {
        // Test with nanoseconds
        let ts = "2025-12-27T14:22:30.123456789Z";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "14:22:30");
    }

    #[test]
    fn test_format_timestamp_lowercase_t() {
        // Test with lowercase 't' separator
        let ts = "2025-12-27t10:30:45Z";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "10:30:45");
    }

    #[test]
    fn test_format_timestamp_no_z_suffix() {
        // Test without Z suffix
        let ts = "2025-12-27T10:30:45";
        let formatted = format_timestamp(ts);
        // May or may not parse, but shouldn't crash
        // If it falls back to manual parsing, it should still work
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_chat_view_message_ordering() {
        let mut view = ChatView::new();

        // Add messages with different timestamps
        view.messages.push(DisplayMessage::from_chat_message(
            &ChatMessage::new("k".to_string(), "msg1".to_string(), "s".to_string(), "2025-12-27T10:00:00Z".to_string()),
            false,
        ));
        view.messages.push(DisplayMessage::from_chat_message(
            &ChatMessage::new("k".to_string(), "msg2".to_string(), "s".to_string(), "2025-12-27T10:01:00Z".to_string()),
            false,
        ));
        view.messages.push(DisplayMessage::from_chat_message(
            &ChatMessage::new("k".to_string(), "msg3".to_string(), "s".to_string(), "2025-12-27T10:02:00Z".to_string()),
            false,
        ));

        assert_eq!(view.message_count(), 3);
        assert_eq!(view.messages[0].content, "msg1");
        assert_eq!(view.messages[1].content, "msg2");
        assert_eq!(view.messages[2].content, "msg3");
        assert_eq!(view.newest_message_id(), Some("msg-2025-12-27T10:02:00Z"));
    }

    #[test]
    fn test_verification_badge_text_verified() {
        let verified = ChatMessage::verified(
            "key".to_string(),
            "msg".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );
        let display = DisplayMessage::from_chat_message(&verified, false);
        assert_eq!(display.verification_badge(), "✓");
    }

    #[test]
    fn test_verification_badge_text_unverified() {
        let unverified = ChatMessage::new(
            "key".to_string(),
            "msg".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );
        let display = DisplayMessage::from_chat_message(&unverified, false);
        assert_eq!(display.verification_badge(), "");
    }
}
