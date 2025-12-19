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
}