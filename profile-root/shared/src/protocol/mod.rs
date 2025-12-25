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
        joined: Vec<LobbyUserCompact>,
        left: Vec<String>,
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

/// Represents a user in the lobby (full user with status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyUser {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

/// Represents a compact user in lobby updates (no status field - always "online" when joining)
///
/// TODO: Consolidate LobbyUser, LobbyUserWithStatus, and LobbyUserCompact
/// into a single type with optional status field to reduce bug risk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyUserCompact {
    #[serde(rename = "publicKey")]
    pub public_key: String,
}

/// Lobby message from server - sent on successful authentication
/// Contains the initial state of all online users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyMessage {
    #[serde(default)]
    pub r#type: String,
    pub users: Vec<LobbyUserWithStatus>,
}

/// Individual user in lobby message with online status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyUserWithStatus {
    #[serde(rename = "publicKey")]
    pub public_key: String,
    #[serde(default)]
    pub status: String, // "online" or "offline"
}

/// Lobby update message - delta updates for join/leave events
///
/// NOTE: This struct is kept for JSON deserialization from external messages.
/// The primary type used for sending messages is [`Message::LobbyUpdate`].
/// Having a separate deserialization type prevents tight coupling between
/// the protocol enum and incoming message formats.
///
/// DESIGN RATIONALE (Per-Departure Notifications):
/// When multiple users disconnect simultaneously, each disconnection triggers
/// a separate broadcast rather than batching into a single message. This design:
///
/// - **Simplicity**: Each leave event is atomic and independent
/// - **Timeliness**: Clients receive immediate feedback when any user leaves
/// - **Consistency**: No edge cases around batch ordering or partial failures
/// - **AC Compliance**: Matches AC#1 format `{left: [{publicKey: "..."}]}` (single user per message)
///
/// The alternative (batched notifications) would require:
/// - Waiting for multiple disconnects before sending (delay)
/// - Complex ordering guarantees
/// - More complex client-side handling
///
/// See: Story 2.4 Review Follow-up [MEDIUM] - Document per-departure notification design
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyUpdateMessage {
    #[serde(default)]
    pub r#type: String,
    pub joined: Vec<LobbyUserCompact>, // No status field - always "online"
    pub left: Vec<String>,             // Public keys of departed users
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
    pub fn new_lobby_joined(joined_users: Vec<LobbyUserCompact>) -> Self {
        Self::LobbyUpdate {
            joined: joined_users,
            left: vec![],
        }
    }

    /// Create a lobby update with left users
    pub fn new_lobby_left(left_users: Vec<String>) -> Self {
        Self::LobbyUpdate {
            joined: vec![],
            left: left_users,
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

    #[test]
    fn test_lobby_message_deserialization() {
        let json = r#"{"type":"lobby","users":[{"publicKey":"key1","status":"online"},{"publicKey":"key2","status":"online"}]}"#;
        let msg: LobbyMessage = serde_json::from_str(json).unwrap();

        assert_eq!(msg.users.len(), 2);
        assert_eq!(msg.users[0].public_key, "key1");
        assert_eq!(msg.users[0].status, "online");
        assert_eq!(msg.users[1].public_key, "key2");
        assert_eq!(msg.users[1].status, "online");
    }

    #[test]
    fn test_lobby_message_empty_users() {
        let json = r#"{"type":"lobby","users":[]}"#;
        let msg: LobbyMessage = serde_json::from_str(json).unwrap();

        assert!(msg.users.is_empty());
    }

    #[test]
    fn test_lobby_update_message_deserialization() {
        let json = r#"{"type":"lobby_update","joined":[{"publicKey":"new_user"}],"left":["old_user"]}"#;
        let msg: LobbyUpdateMessage = serde_json::from_str(json).unwrap();

        assert_eq!(msg.joined.len(), 1);
        assert_eq!(msg.joined[0].public_key, "new_user");
        assert_eq!(msg.left.len(), 1);
        assert_eq!(msg.left[0], "old_user");
    }

    #[test]
    fn test_lobby_user_compact() {
        let user = LobbyUserCompact {
            public_key: "compact_key".to_string(),
        };
        assert_eq!(user.public_key, "compact_key");

        // Verify serialization roundtrip
        let json = serde_json::to_string(&user).unwrap();
        let deserialized: LobbyUserCompact = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.public_key, "compact_key");
    }

    #[test]
    fn test_lobby_user_with_status() {
        let user = LobbyUserWithStatus {
            public_key: "status_key".to_string(),
            status: "online".to_string(),
        };
        assert_eq!(user.public_key, "status_key");
        assert_eq!(user.status, "online");

        // Test offline status
        let offline_user = LobbyUserWithStatus {
            public_key: "offline_key".to_string(),
            status: "offline".to_string(),
        };
        assert_eq!(offline_user.status, "offline");
    }

    #[test]
    fn test_lobby_update_just_joined() {
        let json = r#"{"type":"lobby_update","joined":[{"publicKey":"user1"},{"publicKey":"user2"}],"left":[]}"#;
        let msg: LobbyUpdateMessage = serde_json::from_str(json).unwrap();

        assert_eq!(msg.joined.len(), 2);
        assert!(msg.left.is_empty());
    }

    #[test]
    fn test_lobby_update_just_left() {
        let json = r#"{"type":"lobby_update","joined":[],"left":["user1","user2","user3"]}"#;
        let msg: LobbyUpdateMessage = serde_json::from_str(json).unwrap();

        assert!(msg.joined.is_empty());
        assert_eq!(msg.left.len(), 3);
    }
}