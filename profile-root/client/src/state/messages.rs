//! Message history management for chat conversations
//!
//! This module provides thread-safe message history storage
//! that maintains messages in chronological order by timestamp.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Represents a chat message in the message history
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The sender's public key (hex-encoded)
    pub sender_public_key: String,
    /// The message content
    pub message: String,
    /// The cryptographic signature (hex-encoded)
    pub signature: String,
    /// ISO 8601 timestamp when message was sent
    pub timestamp: String,
    /// Whether this message was verified (signature valid)
    pub is_verified: bool,
}

impl ChatMessage {
    /// Create a new unverified message
    pub fn new(
        sender_public_key: String,
        message: String,
        signature: String,
        timestamp: String,
    ) -> Self {
        Self {
            sender_public_key,
            message,
            signature,
            timestamp,
            is_verified: false,
        }
    }

    /// Create a verified message
    pub fn verified(
        sender_public_key: String,
        message: String,
        signature: String,
        timestamp: String,
    ) -> Self {
        Self {
            sender_public_key,
            message,
            signature,
            timestamp,
            is_verified: true,
        }
    }
}

/// Serializable message for state persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageSerializable {
    #[serde(rename = "senderPublicKey")]
    pub sender_public_key: String,
    pub message: String,
    pub signature: String,
    pub timestamp: String,
    #[serde(rename = "isVerified")]
    pub is_verified: bool,
}

impl From<ChatMessage> for ChatMessageSerializable {
    fn from(msg: ChatMessage) -> Self {
        Self {
            sender_public_key: msg.sender_public_key,
            message: msg.message,
            signature: msg.signature,
            timestamp: msg.timestamp,
            is_verified: msg.is_verified,
        }
    }
}

impl From<ChatMessageSerializable> for ChatMessage {
    fn from(msg: ChatMessageSerializable) -> Self {
        Self {
            sender_public_key: msg.sender_public_key,
            message: msg.message,
            signature: msg.signature,
            timestamp: msg.timestamp,
            is_verified: msg.is_verified,
        }
    }
}

/// Serializable message history for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHistorySerializable {
    #[serde(rename = "messages")]
    pub messages: Vec<ChatMessageSerializable>,
}

/// Thread-safe message history with chronological ordering
///
/// Messages are stored in a VecDeque to support:
/// - O(1) append for new messages (newest at end)
/// - O(1) prepend for historical messages (oldest at start)
/// - Efficient iteration for display
///
/// Messages are ordered by timestamp (oldest at front, newest at back)
#[derive(Debug, Clone)]
pub struct MessageHistory {
    /// Messages in chronological order (oldest → newest)
    messages: VecDeque<ChatMessage>,
    /// Maximum number of messages to keep in history
    max_capacity: usize,
}

impl MessageHistory {
    /// Create a new empty message history
    ///
    /// # Arguments
    /// * `max_capacity` - Maximum messages to retain (FIFO eviction)
    #[inline]
    pub fn new(max_capacity: usize) -> Self {
        Self {
            messages: VecDeque::with_capacity(max_capacity),
            max_capacity,
        }
    }

    /// Create with default capacity (1000 messages)
    #[inline]
    pub fn with_default_capacity() -> Self {
        Self::new(1000)
    }

    /// Add a message to the history (maintains chronological order)
    ///
    /// Messages are inserted based on their timestamp.
    /// If timestamp equals an existing message, appends after.
    /// If capacity is exceeded, oldest messages are evicted.
    ///
    /// # Arguments
    /// * `message` - The message to add
    pub fn add_message(&mut self, message: ChatMessage) {
        // Find the correct position based on timestamp
        let insert_pos = self
            .messages
            .iter()
            .position(|msg| msg.timestamp > message.timestamp)
            .unwrap_or(self.messages.len());

        // Insert at correct position to maintain order
        self.messages.insert(insert_pos, message);

        // Evict oldest messages if over capacity
        while self.messages.len() > self.max_capacity {
            self.messages.pop_front();
        }
    }

    /// Add multiple messages (more efficient than individual adds)
    ///
    /// # Arguments
    /// * `messages` - Iterator of messages to add
    pub fn add_messages<I>(&mut self, messages: I)
    where
        I: IntoIterator<Item = ChatMessage>,
    {
        for message in messages {
            self.add_message(message);
        }
    }

    /// Get all messages in chronological order
    ///
    /// # Returns
    /// Iterator of references to all messages (oldest → newest)
    #[inline]
    pub fn messages(&self) -> impl Iterator<Item = &ChatMessage> {
        self.messages.iter()
    }

    /// Get all messages as owned values
    ///
    /// # Returns
    /// Clone of the messages vector
    #[inline]
    pub fn messages_cloned(&self) -> Vec<ChatMessage> {
        self.messages.clone().into_iter().collect()
    }

    /// Get the number of messages
    ///
    /// # Returns
    /// Current message count
    #[inline]
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if history is empty
    ///
    /// # Returns
    /// true if no messages
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Get the newest message
    ///
    /// # Returns
    /// Some(message) if history not empty, None otherwise
    #[inline]
    pub fn newest(&self) -> Option<&ChatMessage> {
        self.messages.back()
    }

    /// Get the oldest message
    ///
    /// # Returns
    /// Some(message) if history not empty, None otherwise
    #[inline]
    pub fn oldest(&self) -> Option<&ChatMessage> {
        self.messages.front()
    }

    /// Clear all messages
    #[inline]
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get messages for a specific sender
    ///
    /// # Arguments
    /// * `public_key` - The sender's public key
    ///
    /// # Returns
    /// All messages from this sender
    pub fn messages_from(&self, public_key: &str) -> Vec<&ChatMessage> {
        self.messages
            .iter()
            .filter(|msg| msg.sender_public_key == public_key)
            .collect()
    }

    /// Check if any messages exist from a specific sender
    ///
    /// # Arguments
    /// * `public_key` - The sender's public key
    ///
    /// # Returns
    /// true if at least one message exists
    pub fn has_messages_from(&self, public_key: &str) -> bool {
        self.messages
            .iter()
            .any(|msg| msg.sender_public_key == public_key)
    }

    /// Get messages within a time range
    ///
    /// # Arguments
    /// * `start` - Start timestamp (inclusive)
    /// * `end` - End timestamp (inclusive)
    ///
    /// # Returns
    /// All messages within the range
    pub fn messages_in_range(&self, start: &str, end: &str) -> Vec<&ChatMessage> {
        self.messages
            .iter()
            .filter(|msg| msg.timestamp.as_str() >= start && msg.timestamp.as_str() <= end)
            .collect()
    }

    /// Serialize to JSON for persistence
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        let serializable: MessageHistorySerializable = self.into();
        serde_json::to_string(&serializable)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let serializable: MessageHistorySerializable = serde_json::from_str(json)?;
        Ok(serializable.into())
    }
}

impl Default for MessageHistory {
    fn default() -> Self {
        Self::with_default_capacity()
    }
}

impl From<MessageHistorySerializable> for MessageHistory {
    fn from(serializable: MessageHistorySerializable) -> Self {
        let messages: VecDeque<ChatMessage> = serializable
            .messages
            .into_iter()
            .map(|msg| msg.into())
            .collect();
        Self {
            messages,
            max_capacity: 1000,
        }
    }
}

impl From<&MessageHistory> for MessageHistorySerializable {
    fn from(history: &MessageHistory) -> Self {
        Self {
            messages: history
                .messages
                .iter()
                .map(|msg| msg.clone().into())
                .collect(),
        }
    }
}

impl From<MessageHistory> for MessageHistorySerializable {
    fn from(history: MessageHistory) -> Self {
        Self {
            messages: history.messages.into_iter().map(|msg| msg.into()).collect(),
        }
    }
}

/// Shared reference to message history for concurrent access
pub type SharedMessageHistory = Arc<Mutex<MessageHistory>>;

/// Create a new shared message history
#[inline]
pub fn create_shared_message_history() -> SharedMessageHistory {
    Arc::new(Mutex::new(MessageHistory::with_default_capacity()))
}

/// Create with custom capacity
#[inline]
pub fn create_shared_message_history_with_capacity(capacity: usize) -> SharedMessageHistory {
    Arc::new(Mutex::new(MessageHistory::new(capacity)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_history() {
        let history = MessageHistory::with_default_capacity();
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_add_single_message() {
        let mut history = MessageHistory::with_default_capacity();
        let msg = ChatMessage::new(
            "sender_key".to_string(),
            "Hello".to_string(),
            "sig".to_string(),
            "2025-12-27T10:00:00Z".to_string(),
        );

        history.add_message(msg);

        assert!(!history.is_empty());
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_messages_chronological_order() {
        let mut history = MessageHistory::with_default_capacity();

        // Add messages out of order
        history.add_message(ChatMessage::new(
            "sender".to_string(),
            "middle".to_string(),
            "sig".to_string(),
            "2025-12-27T10:02:00Z".to_string(),
        ));

        history.add_message(ChatMessage::new(
            "sender".to_string(),
            "first".to_string(),
            "sig".to_string(),
            "2025-12-27T10:00:00Z".to_string(),
        ));

        history.add_message(ChatMessage::new(
            "sender".to_string(),
            "last".to_string(),
            "sig".to_string(),
            "2025-12-27T10:05:00Z".to_string(),
        ));

        // Should be in chronological order
        let messages: Vec<&str> = history.messages().map(|m| m.message.as_str()).collect();

        assert_eq!(messages, vec!["first", "middle", "last"]);
    }

    #[test]
    fn test_add_messages() {
        let mut history = MessageHistory::with_default_capacity();

        let messages = vec![
            ChatMessage::new(
                "s1".to_string(),
                "msg1".to_string(),
                "sig1".to_string(),
                "t1".to_string(),
            ),
            ChatMessage::new(
                "s2".to_string(),
                "msg2".to_string(),
                "sig2".to_string(),
                "t2".to_string(),
            ),
        ];

        history.add_messages(messages);

        assert_eq!(history.len(), 2);
    }

    #[test]
    fn test_clear() {
        let mut history = MessageHistory::with_default_capacity();
        history.add_message(ChatMessage::new(
            "sender".to_string(),
            "Hello".to_string(),
            "sig".to_string(),
            "2025-12-27T10:00:00Z".to_string(),
        ));

        history.clear();

        assert!(history.is_empty());
    }

    #[test]
    fn test_capacity_limit() {
        let mut history = MessageHistory::new(3);

        for i in 0..5 {
            history.add_message(ChatMessage::new(
                "sender".to_string(),
                format!("message {}", i),
                "sig".to_string(),
                format!("2025-12-27T10:0{}00Z", i),
            ));
        }

        // Should only have 3 messages (oldest evicted)
        assert_eq!(history.len(), 3);
        assert_eq!(history.oldest().unwrap().message, "message 2");
        assert_eq!(history.newest().unwrap().message, "message 4");
    }

    #[test]
    fn test_messages_from_sender() {
        let mut history = MessageHistory::with_default_capacity();

        history.add_message(ChatMessage::new(
            "sender_a".to_string(),
            "from A".to_string(),
            "sig".to_string(),
            "2025-12-27T10:00:00Z".to_string(),
        ));

        history.add_message(ChatMessage::new(
            "sender_b".to_string(),
            "from B".to_string(),
            "sig".to_string(),
            "2025-12-27T10:01:00Z".to_string(),
        ));

        history.add_message(ChatMessage::new(
            "sender_a".to_string(),
            "from A again".to_string(),
            "sig".to_string(),
            "2025-12-27T10:02:00Z".to_string(),
        ));

        let from_a: Vec<&str> = history
            .messages_from("sender_a")
            .iter()
            .map(|m| m.message.as_str())
            .collect();

        assert_eq!(from_a, vec!["from A", "from A again"]);
    }

    #[test]
    fn test_has_messages_from() {
        let mut history = MessageHistory::with_default_capacity();

        history.add_message(ChatMessage::new(
            "sender_a".to_string(),
            "Hello".to_string(),
            "sig".to_string(),
            "2025-12-27T10:00:00Z".to_string(),
        ));

        assert!(history.has_messages_from("sender_a"));
        assert!(!history.has_messages_from("sender_b"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut history = MessageHistory::with_default_capacity();
        history.add_message(ChatMessage::verified(
            "sender".to_string(),
            "Hello".to_string(),
            "sig".to_string(),
            "2025-12-27T10:00:00Z".to_string(),
        ));

        let json = history.to_json().unwrap();
        let restored = MessageHistory::from_json(&json).unwrap();

        assert_eq!(restored.len(), 1);
        let msg = restored.messages().next().unwrap();
        assert_eq!(msg.message, "Hello");
        assert!(msg.is_verified);
    }

    #[test]
    fn test_newest_and_oldest() {
        let mut history = MessageHistory::with_default_capacity();

        assert!(history.newest().is_none());
        assert!(history.oldest().is_none());

        history.add_message(ChatMessage::new(
            "sender".to_string(),
            "first".to_string(),
            "sig".to_string(),
            "2025-12-27T10:00:00Z".to_string(),
        ));

        history.add_message(ChatMessage::new(
            "sender".to_string(),
            "last".to_string(),
            "sig".to_string(),
            "2025-12-27T10:05:00Z".to_string(),
        ));

        assert_eq!(history.oldest().unwrap().message, "first");
        assert_eq!(history.newest().unwrap().message, "last");
    }
}
