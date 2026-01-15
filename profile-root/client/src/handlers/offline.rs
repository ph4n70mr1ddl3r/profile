//! Offline notification handling for message delivery failures
//!
//! This module provides support for handling scenarios where messages
//! cannot be delivered because the recipient is offline.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Notification type from server
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflineNotification {
    pub r#type: String,
    pub event: String,
    pub recipient: String,
    pub message: Option<String>, // Original message that failed
    pub timestamp: Option<String>,
}

/// Parse an offline notification from JSON
pub fn parse_offline_notification(json: &str) -> Result<OfflineNotification, String> {
    serde_json::from_str(json).map_err(|e| format!("Failed to parse notification: {}", e))
}

/// Create an offline notification for sending to user
pub fn create_offline_notification(
    recipient_key: &str,
    failed_message: Option<&str>,
) -> OfflineNotification {
    OfflineNotification {
        r#type: "notification".to_string(),
        event: "recipient_offline".to_string(),
        recipient: recipient_key.to_string(),
        message: failed_message.map(|s| s.to_string()),
        timestamp: Some(chrono::Utc::now().to_rfc3339()),
    }
}

/// Undelivered message state
///
/// Represents a message that failed to deliver due to recipient being offline.
#[derive(Debug, Clone)]
pub struct UndeliveredMessage {
    /// The original message content
    pub content: String,
    /// Recipient's public key
    pub recipient_key: String,
    /// Original timestamp
    pub timestamp: String,
    /// Whether notification has been dismissed
    pub notification_dismissed: bool,
    /// Retry count
    pub retry_count: u32,
}

impl UndeliveredMessage {
    /// Create a new undelivered message
    pub fn new(content: String, recipient_key: String, timestamp: String) -> Self {
        Self {
            content,
            recipient_key,
            timestamp,
            notification_dismissed: false,
            retry_count: 0,
        }
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Dismiss the notification
    pub fn dismiss_notification(&mut self) {
        self.notification_dismissed = true;
    }

    /// Check if notification should be shown
    pub fn should_show_notification(&self) -> bool {
        !self.notification_dismissed
    }
}

/// Shared undelivered messages store
pub type SharedUndeliveredMessages = Arc<Mutex<Vec<UndeliveredMessage>>>;

/// Create a new shared undelivered messages store
pub fn create_shared_undelivered_messages() -> SharedUndeliveredMessages {
    Arc::new(Mutex::new(Vec::new()))
}

/// Add an undelivered message
pub async fn add_undelivered_message(
    store: &SharedUndeliveredMessages,
    content: &str,
    recipient_key: &str,
    timestamp: &str,
) {
    let mut messages = store.lock().await;
    messages.push(UndeliveredMessage::new(
        content.to_string(),
        recipient_key.to_string(),
        timestamp.to_string(),
    ));
}

/// Get undelivered messages for a recipient
pub async fn get_undelivered_for_recipient(
    store: &SharedUndeliveredMessages,
    recipient_key: &str,
) -> Vec<UndeliveredMessage> {
    let messages = store.lock().await;
    messages
        .iter()
        .filter(|m| m.recipient_key == recipient_key)
        .cloned()
        .collect()
}

/// Clear undelivered messages for a recipient (when they come online)
pub async fn clear_undelivered_for_recipient(
    store: &SharedUndeliveredMessages,
    recipient_key: &str,
) {
    let mut messages = store.lock().await;
    messages.retain(|m| m.recipient_key != recipient_key);
}

/// Dismiss notification for a specific message
pub async fn dismiss_notification(store: &SharedUndeliveredMessages, recipient_key: &str) {
    let mut messages = store.lock().await;
    for msg in messages.iter_mut() {
        if msg.recipient_key == recipient_key {
            msg.dismiss_notification();
        }
    }
}

/// Format notification message for display
pub fn format_notification_message(notification: &OfflineNotification) -> String {
    let key_short = format_public_key_short(&notification.recipient);
    format!("User {} is offline. Message not delivered.", key_short)
}

/// Format key for display (first 8 chars + "...")
fn format_public_key_short(key: &str) -> String {
    if key.len() > 16 {
        format!("{}...{}", &key[..8], &key[key.len() - 8..])
    } else {
        key.to_string()
    }
}

/// Create a display message for an undelivered message with warning badge
pub fn create_undelivered_display_message(
    msg: &UndeliveredMessage,
    is_self: bool,
) -> crate::ui::chat::DisplayMessage {
    crate::ui::chat::DisplayMessage {
        id: format!("undelivered-{}", msg.timestamp),
        sender_key: msg.recipient_key.clone(),
        sender_key_short: format_public_key_short(&msg.recipient_key),
        content: msg.content.clone(),
        timestamp: crate::ui::chat::format_timestamp(&msg.timestamp),
        signature: "".to_string(), // No signature for undelivered messages
        is_verified: false,        // Undelivered = not verified
        is_self,
        original_timestamp: msg.timestamp.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::messages::create_shared_message_history;

    #[test]
    fn test_parse_offline_notification() {
        let json = r#"{"type":"notification","event":"recipient_offline","recipient":"abc123","message":"Hello","timestamp":"2025-12-27T10:30:00Z"}"#;
        let result = parse_offline_notification(json);
        assert!(result.is_ok());
        let notification = result.unwrap();
        assert_eq!(notification.event, "recipient_offline");
        assert_eq!(notification.recipient, "abc123");
        assert_eq!(notification.message, Some("Hello".to_string()));
    }

    #[test]
    fn test_create_offline_notification() {
        let notification = create_offline_notification("recipient_key", Some("Hello"));
        assert_eq!(notification.r#type, "notification");
        assert_eq!(notification.event, "recipient_offline");
        assert_eq!(notification.recipient, "recipient_key");
        assert_eq!(notification.message, Some("Hello".to_string()));
    }

    #[test]
    fn test_undelivered_message_creation() {
        let msg = UndeliveredMessage::new(
            "Hello".to_string(),
            "recipient".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );
        assert_eq!(msg.content, "Hello");
        assert_eq!(msg.recipient_key, "recipient");
        assert_eq!(msg.retry_count, 0);
        assert!(!msg.notification_dismissed);
    }

    #[test]
    fn test_undelivered_retry() {
        let mut msg = UndeliveredMessage::new(
            "Hello".to_string(),
            "recipient".to_string(),
            "t".to_string(),
        );
        assert_eq!(msg.retry_count, 0);
        msg.increment_retry();
        assert_eq!(msg.retry_count, 1);
        msg.increment_retry();
        assert_eq!(msg.retry_count, 2);
    }

    #[test]
    fn test_undelivered_dismiss() {
        let mut msg = UndeliveredMessage::new(
            "Hello".to_string(),
            "recipient".to_string(),
            "t".to_string(),
        );
        assert!(msg.should_show_notification());
        msg.dismiss_notification();
        assert!(!msg.should_show_notification());
    }

    #[test]
    fn test_format_notification_message() {
        let notification = create_offline_notification(
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab",
            None,
        );
        let msg = format_notification_message(&notification);
        assert!(msg.contains("User"));
        assert!(msg.contains("is offline"));
        assert!(msg.contains("Message not delivered"));
    }

    #[tokio::test]
    async fn test_add_undelivered_message() {
        let store = create_shared_undelivered_messages();
        add_undelivered_message(&store, "Hello", "recipient", "2025-12-27T10:30:00Z").await;

        let messages = store.lock().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello");
    }

    #[tokio::test]
    async fn test_get_undelivered_for_recipient() {
        let store = create_shared_undelivered_messages();
        add_undelivered_message(&store, "Hello", "recipient1", "2025-12-27T10:30:00Z").await;
        add_undelivered_message(&store, "World", "recipient2", "2025-12-27T10:31:00Z").await;
        add_undelivered_message(&store, "Test", "recipient1", "2025-12-27T10:32:00Z").await;

        let for_recipient1 = get_undelivered_for_recipient(&store, "recipient1").await;
        assert_eq!(for_recipient1.len(), 2);

        let for_recipient2 = get_undelivered_for_recipient(&store, "recipient2").await;
        assert_eq!(for_recipient2.len(), 1);
    }

    #[tokio::test]
    async fn test_clear_undelivered_for_recipient() {
        let store = create_shared_undelivered_messages();
        add_undelivered_message(&store, "Hello", "recipient1", "2025-12-27T10:30:00Z").await;
        add_undelivered_message(&store, "World", "recipient2", "2025-12-27T10:31:00Z").await;

        clear_undelivered_for_recipient(&store, "recipient1").await;

        let messages = store.lock().await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].recipient_key, "recipient2");
    }

    #[tokio::test]
    async fn test_dismiss_notification() {
        let store = create_shared_undelivered_messages();
        add_undelivered_message(&store, "Hello", "recipient1", "2025-12-27T10:30:00Z").await;
        add_undelivered_message(&store, "World", "recipient2", "2025-12-27T10:31:00Z").await;

        dismiss_notification(&store, "recipient1").await;

        let messages = store.lock().await;
        assert!(!messages[0].should_show_notification());
        assert!(messages[1].should_show_notification());
    }

    #[test]
    fn test_create_undelivered_display_message() {
        let msg = UndeliveredMessage::new(
            "Hello".to_string(),
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        let display = create_undelivered_display_message(&msg, true);
        assert_eq!(display.content, "Hello");
        assert!(!display.is_verified); // Undelivered = not verified
        assert!(display.is_self);
    }
}
