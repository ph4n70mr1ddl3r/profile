//! WebSocket protocol message types
//!
//! This module defines the message formats for client-server communication
//! required by Story 1.5 (Authentication) and subsequent stories.

use serde::{Deserialize, Serialize};

/// Authentication message sent by client during WebSocket handshake
///
/// Follows Architecture Decision 4: Uses `publicKey` and `signature` field names in JSON
/// but snake_case in Rust to avoid compiler warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthMessage {
    pub r#type: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

/// Successful authentication response with full lobby state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSuccessMessage {
    pub r#type: String,
    pub users: Vec<String>, // List of online users (hex-encoded public keys)
}

/// Authentication error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthErrorMessage {
    pub r#type: String,
    pub reason: String,
    pub details: String,
}

/// General error message for other protocol errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub r#type: String,
    pub reason: String,
    pub details: Option<String>,
}

/// Client message request for sending a message to another user
///
/// Sent by client to server after Story 3.1 (composer implementation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub r#type: String,
    #[serde(rename = "recipientPublicKey")]
    pub recipient_public_key: String,
    pub message: String,
    #[serde(rename = "senderPublicKey")]
    pub sender_public_key: String,
    pub signature: String,
    pub timestamp: String,
}

/// Close frame reason codes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloseReason {
    AuthFailed,
    ServerShutdown,
    Timeout,
    ClientDisconnect,
}

impl CloseReason {
    pub fn as_str(&self) -> &str {
        match self {
            CloseReason::AuthFailed => "auth_failed",
            CloseReason::ServerShutdown => "server_shutdown",
            CloseReason::Timeout => "timeout",
            CloseReason::ClientDisconnect => "client_disconnect",
        }
    }

    /// Parse a close reason from string
    /// Returns None if the string doesn't match a known close reason
    #[allow(clippy::should_implement_trait)]
    pub fn parse_close_reason(s: &str) -> Option<Self> {
        match s {
            "auth_failed" => Some(CloseReason::AuthFailed),
            "server_shutdown" => Some(CloseReason::ServerShutdown),
            "timeout" => Some(CloseReason::Timeout),
            "client_disconnect" => Some(CloseReason::ClientDisconnect),
            _ => None,
        }
    }
}

impl AuthMessage {
    /// Create a new authentication message
    pub fn new(public_key: String, signature: String) -> Self {
        Self {
            r#type: "auth".to_string(),
            public_key,
            signature,
        }
    }
}

impl AuthSuccessMessage {
    /// Create a new authentication success message
    pub fn new(users: Vec<String>) -> Self {
        Self {
            r#type: "auth_success".to_string(),
            users,
        }
    }
}

impl AuthErrorMessage {
    /// Create a new authentication error message
    pub fn new(reason: String, details: String) -> Self {
        Self {
            r#type: "error".to_string(),
            reason,
            details,
        }
    }
}

impl ErrorMessage {
    /// Create a new general error message
    pub fn new(reason: String) -> Self {
        Self {
            r#type: "error".to_string(),
            reason,
            details: None,
        }
    }

    /// Create a new error message with details
    pub fn with_details(reason: String, details: String) -> Self {
        Self {
            r#type: "error".to_string(),
            reason,
            details: Some(details),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_message_creation() {
        let msg = AuthMessage::new("abc123".to_string(), "def456".to_string());
        assert_eq!(msg.r#type, "auth");
        assert_eq!(msg.public_key, "abc123");
        assert_eq!(msg.signature, "def456");
    }

    #[test]
    fn test_auth_success_message_creation() {
        let users = vec!["user1".to_string(), "user2".to_string()];
        let msg = AuthSuccessMessage::new(users.clone());
        assert_eq!(msg.r#type, "auth_success");
        assert_eq!(msg.users, users);
    }

    #[test]
    fn test_auth_error_message_creation() {
        let msg = AuthErrorMessage::new("auth_failed".to_string(), "Invalid signature".to_string());
        assert_eq!(msg.r#type, "error");
        assert_eq!(msg.reason, "auth_failed");
        assert_eq!(msg.details, "Invalid signature");
    }

    #[test]
    fn test_error_message_creation() {
        let msg = ErrorMessage::new("connection_lost".to_string());
        assert_eq!(msg.r#type, "error");
        assert_eq!(msg.reason, "connection_lost");
    }

    #[test]
    fn test_serde_serialization() {
        // Test that messages can be serialized and deserialized
        let original = AuthMessage::new("abc123".to_string(), "def456".to_string());
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: AuthMessage = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original.r#type, deserialized.r#type);
        assert_eq!(original.public_key, deserialized.public_key);
        assert_eq!(original.signature, deserialized.signature);
    }

    #[test]
    fn test_close_reason_conversions() {
        assert_eq!(CloseReason::AuthFailed.as_str(), "auth_failed");
        assert_eq!(CloseReason::ServerShutdown.as_str(), "server_shutdown");
        assert_eq!(CloseReason::Timeout.as_str(), "timeout");
        assert_eq!(CloseReason::ClientDisconnect.as_str(), "client_disconnect");

        assert_eq!(
            CloseReason::parse_close_reason("auth_failed"),
            Some(CloseReason::AuthFailed)
        );
        assert_eq!(
            CloseReason::parse_close_reason("server_shutdown"),
            Some(CloseReason::ServerShutdown)
        );
        assert_eq!(
            CloseReason::parse_close_reason("timeout"),
            Some(CloseReason::Timeout)
        );
        assert_eq!(
            CloseReason::parse_close_reason("client_disconnect"),
            Some(CloseReason::ClientDisconnect)
        );
        assert_eq!(CloseReason::parse_close_reason("unknown"), None);
    }
}
