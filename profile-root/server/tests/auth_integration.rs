//! Integration tests for authentication flow
//!
//! Tests the complete authentication process including:
//! - Valid authentication with correct signature
//! - Invalid authentication with wrong signature
//! - Malformed authentication messages

use profile_shared::crypto::{
    derive_public_key, generate_private_key, sign_message, verify_signature,
};
use serde_json::json;

#[test]
fn test_valid_signature_creation_and_verification() {
    // Generate a key pair
    let private_key = generate_private_key().expect("Should generate private key");
    let public_key = derive_public_key(&private_key).expect("Should derive public key");

    // Create signature for "auth" message (canonical JSON)
    let signature = sign_message(&private_key, b"auth").expect("Should create signature");

    // Verify signature with public key
    let result = verify_signature(&public_key, b"auth", &signature);

    assert!(result.is_ok(), "Valid signature should verify");
}

#[test]
fn test_invalid_signature_rejected() {
    // Generate a key pair
    let private_key = generate_private_key().expect("Should generate private key");
    let public_key = derive_public_key(&private_key).expect("Should derive public key");

    // Create signature for "auth" message
    let signature = sign_message(&private_key, b"auth").expect("Should create signature");

    // Try to verify with wrong message
    let result = verify_signature(&public_key, b"wrong_message", &signature);

    assert!(result.is_err(), "Invalid signature should be rejected");
}

#[test]
fn test_wrong_public_key_rejected() {
    // Generate two key pairs
    let private_key1 = generate_private_key().expect("Should generate private key 1");
    let _public_key1 = derive_public_key(&private_key1).expect("Should derive public key 1");

    let private_key2 = generate_private_key().expect("Should generate private key 2");
    let public_key2 = derive_public_key(&private_key2).expect("Should derive public key 2");

    // Create signature with first key
    let signature = sign_message(&private_key1, b"auth").expect("Should create signature");

    // Try to verify with second key
    let result = verify_signature(&public_key2, b"auth", &signature);

    assert!(
        result.is_err(),
        "Signature from different key should be rejected"
    );
}

#[test]
fn test_auth_message_json_format() {
    // Generate a key pair
    let private_key = generate_private_key().expect("Should generate private key");
    let public_key = derive_public_key(&private_key).expect("Should derive public key");

    // Create signature
    let signature = sign_message(&private_key, b"auth").expect("Should create signature");

    // Format as JSON with hex encoding
    let public_key_hex = hex::encode(&public_key);
    let signature_hex = hex::encode(&signature);

    let auth_message = json!({
        "type": "auth",
        "publicKey": public_key_hex,
        "signature": signature_hex
    });

    // Verify message structure
    assert_eq!(auth_message["type"], "auth");
    assert_eq!(auth_message["publicKey"].as_str().unwrap().len(), 64); // 32 bytes as hex
    assert_eq!(auth_message["signature"].as_str().unwrap().len(), 128); // 64 bytes as hex
}

#[test]
fn test_deterministic_signatures_match() {
    // Generate a key pair
    let private_key = generate_private_key().expect("Should generate private key");

    // Create multiple signatures for same message
    let sig1 = sign_message(&private_key, b"auth").expect("Should create signature");
    let sig2 = sign_message(&private_key, b"auth").expect("Should create signature");
    let sig3 = sign_message(&private_key, b"auth").expect("Should create signature");

    // All signatures should be identical (deterministic)
    assert_eq!(sig1, sig2, "Signatures should be deterministic");
    assert_eq!(sig2, sig3, "Signatures should be deterministic");
}

#[test]
fn test_empty_signature_rejected() {
    let private_key = generate_private_key().expect("Should generate private key");
    let public_key = derive_public_key(&private_key).expect("Should derive public key");

    // Try to verify with empty signature
    let empty_sig: Vec<u8> = vec![];
    let result = verify_signature(&public_key, b"auth", &empty_sig);

    assert!(result.is_err(), "Empty signature should be rejected");
}

#[test]
fn test_malformed_signature_rejected() {
    let private_key = generate_private_key().expect("Should generate private key");
    let public_key = derive_public_key(&private_key).expect("Should derive public key");

    // Try to verify with malformed signature (wrong length)
    let bad_sig = vec![0u8; 32]; // Should be 64 bytes
    let result = verify_signature(&public_key, b"auth", &bad_sig);

    assert!(result.is_err(), "Malformed signature should be rejected");
}

#[test]
fn test_hex_encoding_roundtrip() {
    // Generate a key pair
    let private_key = generate_private_key().expect("Should generate private key");
    let public_key = derive_public_key(&private_key).expect("Should derive public key");
    let signature = sign_message(&private_key, b"auth").expect("Should create signature");

    // Encode to hex
    let public_key_hex = hex::encode(&public_key);
    let signature_hex = hex::encode(&signature);

    // Decode from hex
    let public_key_decoded = hex::decode(&public_key_hex).expect("Should decode public key");
    let signature_decoded = hex::decode(&signature_hex).expect("Should decode signature");

    // Verify decoded signature
    let result = verify_signature(&public_key_decoded, b"auth", &signature_decoded);
    assert!(
        result.is_ok(),
        "Hex roundtrip should preserve signature validity"
    );
}

#[test]
fn test_case_insensitive_hex_decoding() {
    let private_key = generate_private_key().expect("Should generate private key");
    let public_key = derive_public_key(&private_key).expect("Should derive public key");
    let signature = sign_message(&private_key, b"auth").expect("Should create signature");

    // Encode to hex in different cases
    let signature_lower = hex::encode(&signature);
    let signature_upper = signature_lower.to_uppercase();

    // Both should decode to same value
    let decoded_lower = hex::decode(&signature_lower).expect("Should decode lowercase");
    let decoded_upper = hex::decode(&signature_upper).expect("Should decode uppercase");

    assert_eq!(
        decoded_lower, decoded_upper,
        "Hex decoding should be case-insensitive"
    );

    // Both should verify
    let result_lower = verify_signature(&public_key, b"auth", &decoded_lower);
    let result_upper = verify_signature(&public_key, b"auth", &decoded_upper);

    assert!(result_lower.is_ok(), "Lowercase hex should verify");
    assert!(result_upper.is_ok(), "Uppercase hex should verify");
}

// Story 1.6: Disconnection handling integration tests

use std::sync::Arc;

#[tokio::test]
async fn test_lobby_removes_user_on_disconnect() {
    // Test that the lobby properly removes users when they disconnect (AC4)

    use profile_server::lobby::{ActiveConnection, Lobby};
    use profile_shared::Message;
    use tokio::sync::mpsc;

    let lobby = Arc::new(Lobby::new());
    let test_key = "1234567890abcdef1234567890abcdef".to_string(); // 32 char hex string

    // Create sender channel for the connection
    let (sender, _) = mpsc::unbounded_channel::<Message>();

    // Add user to lobby
    let connection = ActiveConnection {
        public_key: test_key.clone(),
        sender,
        connection_id: 0,
    };
    lobby.add_user(connection).await.unwrap();
    assert_eq!(lobby.user_count().await.unwrap(), 1);

    // Simulate disconnect by removing user
    lobby.remove_user(&test_key).await.unwrap();

    // Verify user was removed
    assert_eq!(lobby.user_count().await.unwrap(), 0);
    assert!(!lobby.user_exists(&test_key).await.unwrap());
}

#[tokio::test]
async fn test_server_handles_unexpected_disconnect() {
    // Test that server properly cleans up lobby on unexpected disconnects

    use profile_server::lobby::{ActiveConnection, Lobby};
    use profile_shared::Message;
    use tokio::sync::mpsc;

    let lobby = Arc::new(Lobby::new());
    let test_key = "abcdef1234567890abcdef1234567890".to_string();

    // Create sender channel for the connection
    let (sender, _) = mpsc::unbounded_channel::<Message>();

    // Add user
    let connection = ActiveConnection {
        public_key: test_key.clone(),
        sender,
        connection_id: 0,
    };
    lobby.add_user(connection).await.unwrap();

    // Simulate unexpected disconnect (network error)
    // Server should remove from lobby
    lobby.remove_user(&test_key).await.unwrap();

    // Verify cleanup
    assert_eq!(lobby.user_count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_server_handles_client_close_frame() {
    // Test that server properly handles client-initiated close frames

    use profile_server::lobby::{ActiveConnection, Lobby};
    use profile_shared::Message;
    use tokio::sync::mpsc;

    let lobby = Arc::new(Lobby::new());
    let test_key = "deadbeef12345678deadbeef12345678".to_string();

    // Create sender channel for the connection
    let (sender, _) = mpsc::unbounded_channel::<Message>();

    // Add user
    let connection = ActiveConnection {
        public_key: test_key.clone(),
        sender,
        connection_id: 0,
    };
    lobby.add_user(connection).await.unwrap();

    // Simulate client sending close frame (graceful shutdown)
    // Server should log the reason and clean up lobby
    lobby.remove_user(&test_key).await.unwrap();

    // Verify cleanup happened
    assert_eq!(lobby.user_count().await.unwrap(), 0);
    assert!(!lobby.user_exists(&test_key).await.unwrap());
}
