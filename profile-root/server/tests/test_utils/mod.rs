//! Shared test utilities for lobby tests
//!
//! This module consolidates common test helper functions used across
//! lobby tests to avoid code duplication.
//!
use tokio::sync::mpsc;
use profile_shared::Message;
use profile_server::lobby::ActiveConnection;

/// Create a test ActiveConnection with a given public key and connection ID
///
/// This is a simple version used by integration and isolated tests.
/// For tests that need auto-generated 64-char hex keys, use
/// `profile_server::lobby::manager::tests::create_test_connection` instead.
pub fn create_test_connection(key: &str, connection_id: u64) -> ActiveConnection {
    let (sender, _) = mpsc::unbounded_channel::<Message>();
    ActiveConnection {
        public_key: key.to_string(),
        sender,
        connection_id,
    }
}

/// Create a test ActiveConnection with a custom sender channel
///
/// This is used by broadcast tests that need to capture messages
/// sent to specific users (to verify broadcast functionality).
///
/// # Arguments
///
/// * `key` - The public key for the connection
/// * `connection_id` - Unique connection identifier
/// * `sender` - Custom mpsc sender channel for receiving messages
///
/// # Returns
///
/// ActiveConnection with the provided sender channel
pub fn create_test_connection_with_sender(
    key: &str,
    connection_id: u64,
    sender: mpsc::UnboundedSender<Message>,
) -> ActiveConnection {
    ActiveConnection {
        public_key: key.to_string(),
        sender,
        connection_id,
    }
}
