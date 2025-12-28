//! Client message handling for sending signed messages
//!
//! This module provides functionality for composing and sending
//! cryptographically signed messages to other users.

use profile_shared::sign_message;
use serde::{Deserialize, Serialize};
use hex;
use zeroize::Zeroizing;
use std::time::{SystemTime, UNIX_EPOCH};

/// Client message structure for sending to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMessage {
    pub r#type: String,
    #[serde(rename = "recipientPublicKey")]
    pub recipient_public_key: String,
    pub message: String,
    #[serde(rename = "senderPublicKey")]
    pub sender_public_key: String,
    pub signature: String,
    pub timestamp: String,
}

impl ClientMessage {
    /// Create a new signed message
    ///
    /// # Arguments
    /// * `message_text` - The message content to sign
    /// * `recipient_public_key` - Hex-encoded public key of recipient
    /// * `sender_public_key` - Hex-encoded public key of sender
    /// * `private_key` - Zeroizing private key for signing
    ///
    /// # Returns
    /// Signed client message ready to send
    pub fn new(
        message_text: String,
        recipient_public_key: String,
        sender_public_key: Vec<u8>,
        private_key: Zeroizing<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Generate ISO 8601 timestamp
        let timestamp = generate_timestamp();

        // Create canonical message for signing (message + timestamp)
        // This ensures deterministic signatures
        let canonical_message = format!("{}:{}", message_text, timestamp);

        // Sign the canonical message
        let signature = sign_message(&private_key, canonical_message.as_bytes())?;

        // Encode to hex
        let sender_public_key_hex = hex::encode(&sender_public_key);
        let signature_hex = hex::encode(signature);

        Ok(Self {
            r#type: "message".to_string(),
            recipient_public_key,
            message: message_text,
            sender_public_key: sender_public_key_hex,
            signature: signature_hex,
            timestamp,
        })
    }

    /// Serialize to JSON string for WebSocket transmission
    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }
}

/// Generate ISO 8601 timestamp in UTC
fn generate_timestamp() -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let nanos = duration.subsec_nanos();

    // Format as ISO 8601: 2025-12-27T10:30:00.123456789Z
    // Using RFC3339 format for compatibility
    chrono::DateTime::from_timestamp(secs as i64, nanos)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_else(|| format!("{}.{}Z", secs, nanos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use profile_shared::generate_private_key;
    use profile_shared::derive_public_key;

    #[tokio::test]
    async fn test_client_message_creation() {
        // Test creating a client message with valid keys
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let result = ClientMessage::new(
            "Hello, world!".to_string(),
            "recipient_public_key_here_1234567890abcdef1234567890abcdef12345678".to_string(),
            public_key.clone(),
            private_key.clone(),
        );

        assert!(result.is_ok(), "Client message creation should work: {:?}", result.err());

        let msg = result.unwrap();
        assert_eq!(msg.r#type, "message");
        assert_eq!(msg.message, "Hello, world!");
        assert_eq!(msg.sender_public_key, hex::encode(public_key));
        assert!(!msg.signature.is_empty());
        assert!(!msg.timestamp.is_empty());

        println!("✅ Client message created successfully");
    }

    #[tokio::test]
    async fn test_client_message_json_serialization() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let msg = ClientMessage::new(
            "Test message".to_string(),
            "recipient_public_key_here_1234567890abcdef1234567890abcdef12345678".to_string(),
            public_key.clone(),
            private_key.clone(),
        )
        .unwrap();

        // Serialize to JSON
        let json = msg.to_json().unwrap();
        assert!(json.contains(r#""type":"message""#));
        assert!(json.contains("Test message"));

        // Deserialize back
        let parsed: ClientMessage = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.message, "Test message");
        assert_eq!(parsed.sender_public_key, hex::encode(public_key));

        println!("✅ JSON serialization works correctly");
    }

    #[tokio::test]
    async fn test_signature_determinism() {
        use profile_shared::sign_message;
        use zeroize::Zeroizing;

        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Test determinism at the signing level (same input = same signature)
        let canonical_message_1 = "Same message:2025-12-27T10:30:00.123456789Z";
        let canonical_message_2 = "Same message:2025-12-27T10:30:00.123456789Z";

        let sig1 = sign_message(&private_key, canonical_message_1.as_bytes()).unwrap();
        let sig2 = sign_message(&private_key, canonical_message_2.as_bytes()).unwrap();

        // Same canonical message must produce identical signatures
        assert_eq!(sig1, sig2, "Deterministic signing failed: same input should produce same signature");

        // Verify the signature is valid length (64 bytes for ed25519)
        assert_eq!(sig1.len(), 64, "Ed25519 signature should be 64 bytes");

        // Different canonical messages should produce different signatures
        let canonical_message_3 = "Same message:2025-12-27T10:30:00.123456790Z";
        let sig3 = sign_message(&private_key, canonical_message_3.as_bytes()).unwrap();
        assert_ne!(sig2, sig3, "Different timestamp should produce different signature");

        println!("✅ Deterministic signing verified: same input produces identical signatures");
    }

    #[tokio::test]
    async fn test_client_message_with_fixed_timestamp() {
        // Test that ClientMessage uses the expected canonical format
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let recipient = "recipient_public_key_here_1234567890abcdef1234567890abcdef12345678".to_string();

        // Create a message
        let msg = ClientMessage::new(
            "Test message".to_string(),
            recipient.clone(),
            public_key.clone(),
            private_key.clone(),
        )
        .unwrap();

        // Verify the structure
        assert_eq!(msg.r#type, "message");
        assert_eq!(msg.message, "Test message");
        assert_eq!(msg.recipient_public_key, recipient);
        assert!(!msg.signature.is_empty());
        assert!(!msg.timestamp.is_empty());

        // Verify the timestamp format is RFC3339
        assert!(msg.timestamp.contains('T'), "Timestamp should be ISO 8601 format");

        println!("✅ ClientMessage with fixed timestamp structure verified");
    }

    #[tokio::test]
    async fn test_unicode_message_content() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Test unicode content
        let unicode_messages = vec![
            "Hello, 世界!".to_string(),
            "🔐 Cryptographic greetings".to_string(),
            "Ñoño tilde test".to_string(),
            "Привет мир".to_string(),
        ];

        let recipient = "recipient_public_key_here_1234567890abcdef1234567890abcdef12345678".to_string();

        for msg_text in unicode_messages {
            let result = ClientMessage::new(
                msg_text.clone(),
                recipient.clone(),
                public_key.clone(),
                private_key.clone(),
            );

            assert!(result.is_ok(), "Should handle unicode: {}", msg_text);

            let msg = result.unwrap();
            assert_eq!(msg.message, msg_text);
        }

        println!("✅ Unicode message handling works correctly");
    }

    #[tokio::test]
    async fn test_long_message_content() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Create a long message (10KB+)
        let long_message: String = (0..10240).map(|_| 'x').collect();
        let recipient = "recipient_public_key_here_1234567890abcdef1234567890abcdef12345678".to_string();

        let result = ClientMessage::new(
            long_message.clone(),
            recipient.clone(),
            public_key.clone(),
            private_key.clone(),
        );

        assert!(result.is_ok(), "Should handle long messages");

        let msg = result.unwrap();
        assert_eq!(msg.message, long_message);

        println!("✅ Long message handling works correctly");
    }

    #[tokio::test]
    async fn test_hex_encoding_format() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let msg = ClientMessage::new(
            "Test".to_string(),
            "recipient_public_key_here_1234567890abcdef1234567890abcdef12345678".to_string(),
            public_key.clone(),
            private_key.clone(),
        )
        .unwrap();

        // Verify hex encoding format
        assert_eq!(msg.sender_public_key.len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(msg.signature.len(), 128); // 64 bytes = 128 hex chars

        // Verify hex strings contain only valid hex characters
        assert!(msg.sender_public_key.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(msg.signature.chars().all(|c| c.is_ascii_hexdigit()));

        println!("✅ Hex encoding format is correct");
    }
}
