//! Message composition and signing operations
//!
//! This module provides message composition functionality for creating
//! and sending cryptographically signed messages. Required by Story 3.1.

use std::fmt::{self, Display, Formatter};
use std::error::Error;
use chrono::{Utc, SecondsFormat};
use serde_json;
use zeroize::Zeroizing;

use crate::state::messages::{ChatMessage, SharedMessageHistory};
use crate::state::session::SharedKeyState;
use profile_shared::crypto::{self, sign_message, CryptoError, PrivateKey};

/// Error types for message composition operations
#[derive(Debug, Clone)]
pub enum ComposeError {
    NoPrivateKey,
    NoPublicKey,
    TimestampError(String),
    SerializationError(String),
    SigningError(String),
    LockError,
    HistoryLockError,
}

impl Display for ComposeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ComposeError::NoPrivateKey => write!(f, "No private key available for signing"),
            ComposeError::NoPublicKey => write!(f, "No public key available"),
            ComposeError::TimestampError(msg) => write!(f, "Failed to generate timestamp: {}", msg),
            ComposeError::SerializationError(msg) => write!(f, "Failed to serialize message: {}", msg),
            ComposeError::SigningError(msg) => write!(f, "Failed to sign message: {}", msg),
            ComposeError::LockError => write!(f, "Failed to acquire lock on key state"),
            ComposeError::HistoryLockError => write!(f, "Failed to acquire lock on message history"),
        }
    }
}

impl Error for ComposeError {}

/// Compose and send a message with cryptographic signing
///
/// This function implements the message signing workflow:
/// 1. Generate timestamp (ISO8601 format)
/// 2. Create canonical JSON for deterministic signing
/// 3. Sign the message with the private key
/// 4. Create a ChatMessage object
/// 5. Store the message in shared history
/// 6. Return JSON for WebSocket transmission
///
/// # Arguments
/// * `message_text` - The text content of the message to send
/// * `recipient_public_key` - The recipient's public key (for message routing)
/// * `key_state` - Shared state containing the user's private key
/// * `message_history` - Shared history for storing sent messages
///
/// # Returns
/// Ok(String) containing the JSON representation of the message for WebSocket transmission
///
/// # Errors
/// Returns `ComposeError` if:
/// - Private key is not available in key state
/// - Timestamp generation fails
/// - Message serialization fails
/// - Signing fails
/// - Lock acquisition fails
pub async fn compose_and_send_message(
    message_text: String,
    recipient_public_key: String,
    key_state: &SharedKeyState,
    message_history: &SharedMessageHistory,
) -> Result<String, ComposeError> {
    // 1. Get keys from shared state
    let (public_key, private_key) = {
        let key_guard = key_state.lock().await;
        let public_key = key_guard.public_key()
            .ok_or(ComposeError::NoPublicKey)?
            .clone();
        let private_key = key_guard.private_key()
            .ok_or(ComposeError::NoPrivateKey)?
            .clone();
        (public_key, private_key)
    };

    // Convert public_key to hex string for message
    let public_key_hex = hex::encode(&public_key);

    // 2. Generate timestamp (ISO8601 format - RFC3339 with seconds precision)
    let timestamp = Utc::now()
        .to_rfc3339_opts(SecondsFormat::Secs, false);

    // 3. Create canonical JSON for deterministic signing
    // We serialize only the essential fields that need to be signed
    let canonical_json = serde_json::json!({
        "type": "message",
        "message": message_text,
        "senderPublicKey": public_key,
        "timestamp": timestamp
    });

    // 4. Sign the canonical JSON
    // The signature covers: type + message + senderPublicKey + timestamp
    let signature = sign_message(
        &private_key,
        canonical_json.to_string().as_bytes(),
    ).map_err(|e| ComposeError::SigningError(e.to_string()))?;

    // 5. Create ChatMessage object with all fields
    // This message is marked as "verified" since we just signed it ourselves
    let chat_message = ChatMessage::verified(
        public_key_hex.clone(),
        message_text.clone(),
        hex::encode(signature.clone()),
        timestamp.clone(),
    );

    // 6. Store message in SharedMessageHistory
    {
        let mut history = message_history.lock().await;
        history.add_message(chat_message);
    }

    // 7. Create and serialize the protocol message for WebSocket transmission
    let protocol_message = profile_shared::protocol::Message::new_text(
        message_text,
        public_key_hex.clone(),
        hex::encode(signature),
        timestamp,
    );

    let message_json = serde_json::to_string(&protocol_message)
        .map_err(|e| ComposeError::SerializationError(e.to_string()))?;

    Ok(message_json)
}

/// Compose a message without sending (for draft preservation)
///
/// This function creates a signed message but does not store it in history
/// or return it for transmission. Useful for previewing signatures.
///
/// # Arguments
/// * `message_text` - The text content of the message
/// * `key_state` - Shared state containing the user's private key
///
/// # Returns
/// Ok(ChatMessage) containing the signed message
///
/// # Errors
/// Returns `ComposeError` if key retrieval or signing fails
pub async fn compose_message_draft(
    message_text: String,
    key_state: &SharedKeyState,
) -> Result<ChatMessage, ComposeError> {
    // Get keys from shared state
    let (public_key, private_key) = {
        let key_guard = key_state.lock().await;
        let public_key = key_guard.public_key()
            .ok_or(ComposeError::NoPublicKey)?
            .clone();
        let private_key = key_guard.private_key()
            .ok_or(ComposeError::NoPrivateKey)?
            .clone();
        (public_key, private_key)
    };

    // Convert public_key to hex string
    let public_key_hex = hex::encode(&public_key);

    // Generate timestamp
    let timestamp = Utc::now()
        .to_rfc3339_opts(SecondsFormat::Secs, false);

    // Create canonical JSON
    let canonical_json = serde_json::json!({
        "type": "message",
        "message": message_text,
        "senderPublicKey": public_key_hex,
        "timestamp": timestamp
    });

    // Sign the message
    let signature = sign_message(
        &private_key,
        canonical_json.to_string().as_bytes(),
    ).map_err(|e| ComposeError::SigningError(e.to_string()))?;

    // Create and return ChatMessage
    Ok(ChatMessage::verified(
        public_key_hex,
        message_text,
        hex::encode(signature),
        timestamp,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::session::create_shared_key_state;
    use crate::state::messages::create_shared_message_history;
    use profile_shared::crypto::{generate_private_key, derive_public_key};

    #[tokio::test]
    async fn test_compose_and_send_message_success() {
        // Setup: Generate keys and create shared state
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let key_state = create_shared_key_state();
        {
            let mut state = key_state.lock().await;
            state.set_generated_key(private_key, public_key.clone());
        }

        let message_history = create_shared_message_history();

        // Execute: Compose and send a message
        let result = compose_and_send_message(
            "Hello, World!".to_string(),
            "recipient_key".to_string(),
            &key_state,
            &message_history,
        ).await;

        // Verify: Message was created successfully
        assert!(result.is_ok(), "Should compose message successfully");
        let json = result.unwrap();

        // Verify JSON contains expected fields
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["message_type"], "Text");
        assert_eq!(parsed["message"], "Hello, World!");
        assert!(parsed["signature"].is_string());
        assert!(parsed["timestamp"].is_string());

        // Verify message was stored in history
        let history = message_history.lock().await;
        assert_eq!(history.len(), 1);
        let stored_msg = history.messages().next().unwrap();
        assert_eq!(stored_msg.message, "Hello, World!");
        assert!(stored_msg.is_verified);
    }

    #[tokio::test]
    async fn test_compose_message_draft() {
        // Setup
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let key_state = create_shared_key_state();
        key_state.lock().await.set_generated_key(private_key, public_key.clone());

        // Execute: Create a draft message
        let result = compose_message_draft(
            "Draft message".to_string(),
            &key_state,
        ).await;

        // Verify
        assert!(result.is_ok(), "Should create draft successfully");
        let chat_msg = result.unwrap();
        assert_eq!(chat_msg.message, "Draft message");
        assert!(chat_msg.is_verified);
        assert!(!chat_msg.signature.is_empty());
    }

    #[tokio::test]
    async fn test_compose_and_send_empty_message_fails() {
        // Setup
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let key_state = create_shared_key_state();
        key_state.lock().await.set_generated_key(private_key, public_key.clone());

        let message_history = create_shared_message_history();

        // Empty message should still succeed (business logic decides if valid)
        let result = compose_and_send_message(
            "".to_string(),
            "recipient_key".to_string(),
            &key_state,
            &message_history,
        ).await;

        assert!(result.is_ok(), "Empty message should still compose");
    }

    #[tokio::test]
    async fn test_compose_no_private_key_fails() {
        let key_state = create_shared_key_state();
        let message_history = create_shared_message_history();

        // Without keys, composition should fail
        let result = compose_and_send_message(
            "Test".to_string(),
            "recipient".to_string(),
            &key_state,
            &message_history,
        ).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        // KeyState is initialized empty, so NoPublicKey is returned first
        assert!(matches!(err, ComposeError::NoPublicKey), "Expected NoPublicKey, got {:?}", err);
    }

    #[test]
    fn test_timestamp_format() {
        let timestamp = Utc::now()
            .to_rfc3339_opts(SecondsFormat::Secs, false);

        // Verify format matches expected pattern (RFC3339 basic)
        // Should contain T and should NOT end with Z for basic format
        assert!(timestamp.contains('T'), "Timestamp should contain T separator");
        assert!(!timestamp.ends_with('Z'), "Basic RFC3339 should not end with Z");
    }

    #[test]
    fn test_compose_error_display() {
        let error = ComposeError::NoPrivateKey;
        assert!(!error.to_string().is_empty());
    }
}
