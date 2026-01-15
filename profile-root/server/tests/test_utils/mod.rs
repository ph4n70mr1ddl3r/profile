//! Shared test utilities for lobby tests
//!
//! This module consolidates common test helper functions used across
//! lobby tests to avoid code duplication.
//!
use profile_server::lobby::ActiveConnection;
use profile_shared::Message;
use tokio::sync::mpsc;

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
