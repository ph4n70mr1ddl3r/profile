//! Protocol message types for client-server communication
//!
//! This module defines all message types used in the WebSocket protocol
//! for authentication, messaging, and lobby updates.

use serde::{Deserialize, Serialize};

/// General message type for WebSocket communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum Message {
    /// Text message from one user to another
    Text {
        message: String,
        #[serde(rename = "senderPublicKey")]
        sender_public_key: String,
        signature: String,
        timestamp: String,
    },
    /// Lobby update with user join/leave events
    LobbyUpdate {
        joined: Option<Vec<LobbyUser>>,
        left: Option<Vec<LobbyUser>>,
    },
    /// Error message
    Error {
        reason: String,
        details: Option<String>,
    },
    /// Authentication message
    Auth {
        #[serde(rename = "publicKey")]
        public_key: String,
        signature: String,
    },
    /// Close frame
    Close,
}

/// Represents a user in the lobby
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyUser {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

impl Message {
    /// Create a new text message
    pub fn new_text(
        message: String,
        sender_public_key: String,
        signature: String,
        timestamp: String,
    ) -> Self {
        Self::Text {
            message,
            sender_public_key,
            signature,
            timestamp,
        }
    }

    /// Create a lobby update with joined users
    pub fn new_lobby_joined(joined_users: Vec<LobbyUser>) -> Self {
        Self::LobbyUpdate {
            joined: Some(joined_users),
            left: None,
        }
    }

    /// Create a lobby update with left users
    pub fn new_lobby_left(left_users: Vec<LobbyUser>) -> Self {
        Self::LobbyUpdate {
            joined: None,
            left: Some(left_users),
        }
    }

    /// Create an error message
    pub fn new_error(reason: String, details: Option<String>) -> Self {
        Self::Error { reason, details }
    }

    /// Create an authentication message
    pub fn new_auth(public_key: String, signature: String) -> Self {
        Self::Auth {
            public_key,
            signature,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_text_creation() {
        let msg = Message::new_text(
            "Hello".to_string(),
            "sender_key".to_string(),
            "signature".to_string(),
            "2025-12-20T10:00:00Z".to_string(),
        );

        match msg {
            Message::Text {
                message,
                sender_public_key,
                signature,
                timestamp,
            } => {
                assert_eq!(message, "Hello");
                assert_eq!(sender_public_key, "sender_key");
                assert_eq!(signature, "signature");
                assert_eq!(timestamp, "2025-12-20T10:00:00Z");
            }
            _ => panic!("Expected Text message"),
        }
    }

    #[test]
    fn test_lobby_user_creation() {
        let user = LobbyUser {
            public_key: "test_key".to_string(),
        };
        assert_eq!(user.public_key, "test_key");
    }

    #[test]
    fn test_serialization() {
        let msg = Message::new_text(
            "Test message".to_string(),
            "test_key".to_string(),
            "test_sig".to_string(),
            "2025-12-20T10:00:00Z".to_string(),
        );

        let serialized = serde_json::to_string(&msg).unwrap();
        let deserialized: Message = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            Message::Text {
                message,
                sender_public_key,
                signature,
                timestamp,
            } => {
                assert_eq!(message, "Test message");
                assert_eq!(sender_public_key, "test_key");
                assert_eq!(signature, "test_sig");
                assert_eq!(timestamp, "2025-12-20T10:00:00Z");
            }
            _ => panic!("Expected Text message after deserialization"),
        }
    }
}