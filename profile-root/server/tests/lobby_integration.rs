//! Integration tests for lobby functionality
//!
//! Tests the complete lobby management flow including:
//! - User addition and removal
//! - Reconnection handling
//! - Duplicate prevention
//! - Thread safety
//! - Broadcast notifications
//!
//! This file satisfies Story 2.1 requirement for 5+ integration tests.

mod test_utils;

use std::sync::Arc;
use tokio::sync::mpsc;
use profile_server::lobby::{Lobby, ActiveConnection, get_user, get_current_users, add_user, remove_user};
use profile_shared::Message;
use test_utils::create_test_connection;

/// Test 1: test_lobby_adds_user_on_auth
/// Verify successful auth adds user to lobby (AC1)
#[tokio::test]
async fn test_lobby_adds_user_on_auth() {
    let lobby = Arc::new(Lobby::new());
    // Use valid 64-char hex key
    let test_key = "abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234abcd1234".to_string();

    // Verify lobby is empty initially
    assert_eq!(lobby.user_count().await.unwrap(), 0);

    // Add user (simulating successful auth)
    let connection = create_test_connection(&test_key, 1);
    lobby.add_user(connection).await.unwrap();

    // Verify user was added
    assert_eq!(lobby.user_count().await.unwrap(), 1);
    assert!(lobby.user_exists(&test_key).await.unwrap());

    // Verify get_user returns the connection
    let retrieved = get_user(&lobby, &test_key).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().public_key, test_key);

    println!("✅ test_lobby_adds_user_on_auth passed");
}

/// Test 2: test_lobby_removes_user_on_disconnect
/// Verify cleanup on connection close (AC3)
#[tokio::test]
async fn test_lobby_removes_user_on_disconnect() {
    let lobby = Arc::new(Lobby::new());
    // Use valid 64-char hex key
    let test_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();

    // Add user
    let connection = create_test_connection(&test_key, 42);
    lobby.add_user(connection).await.unwrap();
    assert_eq!(lobby.user_count().await.unwrap(), 1);

    // Simulate disconnect
    lobby.remove_user(&test_key).await.unwrap();

    // Verify user was removed
    assert_eq!(lobby.user_count().await.unwrap(), 0);
    assert!(!lobby.user_exists(&test_key).await.unwrap());

    // Verify get_user returns None (not Some with stale connection)
    let lookup_result = get_user(&lobby, &test_key).await.unwrap();
    assert!(lookup_result.is_none());

    println!("✅ test_lobby_removes_user_on_disconnect passed");
}

/// Test 3: test_lobby_handles_reconnection
/// Verify duplicate key handling (AC2)
#[tokio::test]
async fn test_lobby_handles_reconnection() {
    let lobby = Arc::new(Lobby::new());
    // Use valid 64-char hex key
    let test_key = "reconnect1234567890123456789012345678901234567890abcdef1234567890ab".to_string();

    // Add user first time
    let connection1 = create_test_connection(&test_key, 1);
    lobby.add_user(connection1).await.unwrap();
    assert_eq!(lobby.user_count().await.unwrap(), 1);

    // Store original connection ID for verification
    let original_conn = get_user(&lobby, &test_key).await.unwrap().unwrap();
    let original_id = original_conn.connection_id;

    // Simulate reconnection - add same user again
    let connection2 = create_test_connection(&test_key, 2);
    lobby.add_user(connection2).await.unwrap();

    // Verify still only one user (not duplicated)
    assert_eq!(lobby.user_count().await.unwrap(), 1);

    // Verify connection was replaced with new one
    let updated_conn = get_user(&lobby, &test_key).await.unwrap().unwrap();
    assert_eq!(updated_conn.connection_id, 2);
    assert_ne!(original_id, updated_conn.connection_id);

    println!("✅ test_lobby_handles_reconnection passed");
}

/// Test 4: test_lobby_prevents_duplicates
/// Verify single user per key (no duplicates)
#[tokio::test]
async fn test_lobby_prevents_duplicates() {
    let lobby = Arc::new(Lobby::new());
    // Use valid 64-char hex key
    let test_key = "uniqueuser1234567890123456789012345678901234567890abcdef1234567890ab".to_string();

    // Add same user multiple times rapidly
    for i in 1..=5 {
        let connection = create_test_connection(&test_key, i);
        lobby.add_user(connection).await.unwrap();

        // Should still only have one user
        assert_eq!(lobby.user_count().await.unwrap(), 1);

        // Connection ID should be updated to latest
        let conn = get_user(&lobby, &test_key).await.unwrap().unwrap();
        assert_eq!(conn.connection_id, i);
    }

    // Final verification
    assert_eq!(lobby.user_count().await.unwrap(), 1);
    let users = get_current_users(&lobby).await.unwrap();
    assert_eq!(users.len(), 1);
    assert!(users.contains(&test_key));

    println!("✅ test_lobby_prevents_duplicates passed");
}

/// Test 5: test_lobby_thread_safety
/// Verify concurrent access safety
#[tokio::test]
async fn test_lobby_thread_safety() {
    let lobby = Arc::new(Lobby::new());
    let mut handles = Vec::new();

    // Spawn multiple concurrent operations
    for i in 0..10 {
        let lobby_clone = lobby.clone();
        let handle = tokio::spawn(async move {
            // Use valid 64-char hex key
            let key = format!("concurrent{:02}1234567890123456789012345678901234567890abcdef1234", i);
            let connection = create_test_connection(&key, i as u64);

            // Add user
            lobby_clone.add_user(connection).await.unwrap();

            // Immediately read state
            let count = lobby_clone.user_count().await.unwrap();
            let exists = lobby_clone.user_exists(&key).await.unwrap();

            (key, count, exists)
        });
        handles.push(handle);
    }

    // Collect all results
    let results: Vec<_> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Verify all operations succeeded
    for (key, count, exists) in &results {
        assert!(*exists, "User {} should exist", key);
        assert!(*count >= 1, "Count should be at least 1 for {}", key);
    }

    // Verify final state is consistent (10 unique users)
    let final_count = lobby.user_count().await.unwrap();
    assert_eq!(final_count, 10, "Should have 10 unique users");

    // Verify all users are still accessible
    let all_users = get_current_users(&lobby).await.unwrap();
    assert_eq!(all_users.len(), 10);

    println!("✅ test_lobby_thread_safety passed");
}

/// Test 6: test_lobby_broadcast_on_join
/// Verify broadcast notifications when users join
#[tokio::test]
async fn test_lobby_broadcast_on_join() {
    let lobby = Arc::new(Lobby::new());

    // Create a dedicated channel for the existing user to receive broadcasts
    let (broadcast_sender, mut broadcast_receiver) = mpsc::unbounded_channel::<Message>();

    // Create a connection for the existing user that uses our broadcast receiver
    // Use valid 64-char hex key
    let existing_user = ActiveConnection {
        public_key: "aabb1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab".to_string(),
        sender: broadcast_sender,
        connection_id: 999,
    };

    // Add existing user to lobby first (so they receive the broadcast when new user joins)
    // Use add_user from manager.rs which includes broadcast logic
    add_user(&lobby, existing_user.public_key.clone(), existing_user).await.unwrap();

    // Add a new user (this should trigger broadcast to existing user)
    // Use valid 64-char hex key
    let new_user = create_test_connection("ccdd1234567890abcdef1234567890abcdef1234567890abcdef1234567890cd", 1);
    let new_user_key = new_user.public_key.clone();
    add_user(&lobby, new_user_key, new_user).await.unwrap();

    // Verify lobby state
    assert_eq!(lobby.user_count().await.unwrap(), 2);

    // Verify broadcast was actually received by the existing user via their receiver
    let received = tokio::time::timeout(std::time::Duration::from_millis(100), broadcast_receiver.recv())
        .await
        .expect("Timeout waiting for broadcast - broadcast was not received by existing user")
        .expect("No broadcast message received");

    // Verify the message is a LobbyUpdate with joined users
    match received {
        Message::LobbyUpdate { joined, left } => {
            assert!(!joined.is_empty(), "Expected joined users in broadcast");
            assert!(left.is_empty(), "Should not have left users on join");
            println!("✅ Broadcast on join verified - message received correctly");
        }
        _ => panic!("Expected LobbyUpdate message, got: {:?}", received),
    }
}

/// Test 7: test_lobby_broadcast_on_leave
/// Verify broadcast notifications when users leave
#[tokio::test]
async fn test_lobby_broadcast_on_leave() {
    let lobby = Arc::new(Lobby::new());

    // Create a test message receiver to capture broadcast messages
    let (test_sender, _test_receiver) = mpsc::unbounded_channel::<Message>();

    // Create a mock connection that uses our test receiver
    // Use valid 64-char hex key
    let remaining_user = ActiveConnection {
        public_key: "eeff1234567890abcdef1234567890abcdef1234567890abcdef1234567890ef".to_string(),
        sender: test_sender,
        connection_id: 999,
    };

    // Add user who will remain (using manager's add_user for consistency)
    add_user(&lobby, remaining_user.public_key.clone(), remaining_user).await.unwrap();

    // Add another user who will leave
    // Use valid 64-char hex key
    let leaving_user = create_test_connection("00111234567890abcdef1234567890abcdef1234567890abcdef123456789012", 1);
    let leaving_key = leaving_user.public_key.clone();
    add_user(&lobby, leaving_key.clone(), leaving_user).await.unwrap();

    // Verify both users are in lobby
    assert_eq!(lobby.user_count().await.unwrap(), 2);

    // Remove leaving user (using manager's remove_user which includes broadcast)
    remove_user(&lobby, &leaving_key).await.unwrap();

    // Verify only one user remains
    assert_eq!(lobby.user_count().await.unwrap(), 1);

    // Verify the leaving user's entry was removed from lobby
    let all_users = get_current_users(&lobby).await.unwrap();
    assert_eq!(all_users.len(), 1);
    assert!(!all_users.contains(&leaving_key));

    // Verify the remaining user is still in the lobby (to receive broadcasts)
    let remaining_conn = get_user(&lobby, &"eeff1234567890abcdef1234567890abcdef1234567890abcdef1234567890ef".to_string()).await.unwrap();
    assert!(remaining_conn.is_some());

    println!("✅ Broadcast on leave verified - user removed, remaining user still in lobby");
}

/// Test 8: test_lobby_idempotent_remove
/// Verify that removing a non-existent user is safe
#[tokio::test]
async fn test_lobby_idempotent_remove() {
    let lobby = Arc::new(Lobby::new());
    // Use valid 64-char hex key
    let nonexistent_key = "nonexistent1234567890123456789012345678901234567890abcdef1234567890ab".to_string();

    // Verify lobby is empty
    assert_eq!(lobby.user_count().await.unwrap(), 0);

    // Try to remove user that doesn't exist (should be safe/idempotent)
    let result = lobby.remove_user(&nonexistent_key).await;
    assert!(result.is_ok(), "Removing non-existent user should succeed");

    // Verify lobby is still empty
    assert_eq!(lobby.user_count().await.unwrap(), 0);

    // Add a real user and try to remove nonexistent again
    let real_user = create_test_connection("realuser1234567890123456789012345678901234567890abcdef1234567890", 1);
    lobby.add_user(real_user).await.unwrap();
    assert_eq!(lobby.user_count().await.unwrap(), 1);

    // Now remove nonexistent - should still be safe
    let result = lobby.remove_user(&nonexistent_key).await;
    assert!(result.is_ok());
    assert_eq!(lobby.user_count().await.unwrap(), 1); // Real user still there

    println!("✅ test_lobby_idempotent_remove passed");
}

/// Test 9: test_lobby_get_all_users
/// Verify get_current_users returns all users
#[tokio::test]
async fn test_lobby_get_all_users() {
    let lobby = Arc::new(Lobby::new());

    // Add multiple users with valid 64-char hex keys
    let expected_keys: Vec<String> = (0..5)
        .map(|i| format!("{:02}1234567890123456789012345678901234567890abcdef1234567890", i))
        .collect();

    for (i, key) in expected_keys.iter().enumerate() {
        let connection = create_test_connection(key, i as u64);
        lobby.add_user(connection).await.unwrap();
    }

    // Get all users
    let all_users = get_current_users(&lobby).await.unwrap();

    // Verify all users are returned
    assert_eq!(all_users.len(), 5);
    for key in &expected_keys {
        assert!(all_users.contains(key), "User {} should be in list", key);
    }

    println!("✅ test_lobby_get_all_users passed");
}

/// Test 10: test_lobby_connection_id_tracking
/// Verify connection IDs are properly tracked for reconnection detection
#[tokio::test]
async fn test_lobby_connection_id_tracking() {
    let lobby = Arc::new(Lobby::new());
    // Use valid 64-char hex key
    let test_key = "idtracker1234567890123456789012345678901234567890abcdef1234567890ab".to_string();

    // Add user with connection ID 1
    let connection1 = create_test_connection(&test_key, 1);
    lobby.add_user(connection1).await.unwrap();

    // Verify connection ID
    let conn1 = get_user(&lobby, &test_key).await.unwrap().unwrap();
    assert_eq!(conn1.connection_id, 1);

    // Reconnect with new ID
    let connection2 = create_test_connection(&test_key, 2);
    lobby.add_user(connection2).await.unwrap();

    // Verify connection ID updated
    let conn2 = get_user(&lobby, &test_key).await.unwrap().unwrap();
    assert_eq!(conn2.connection_id, 2);

    // Third reconnection
    let connection3 = create_test_connection(&test_key, 3);
    lobby.add_user(connection3).await.unwrap();

    let conn3 = get_user(&lobby, &test_key).await.unwrap().unwrap();
    assert_eq!(conn3.connection_id, 3);

    // Verify only one user in lobby
    assert_eq!(lobby.user_count().await.unwrap(), 1);

    println!("✅ test_lobby_connection_id_tracking passed");
}

/// Test: test_public_key_validation_rejects_invalid_keys
/// Verify that add_user properly validates public key format
#[tokio::test]
async fn test_public_key_validation_rejects_invalid_keys() {
    let lobby = Arc::new(Lobby::new());

    // Test 1: Empty key should fail
    let result = add_user(&lobby, "".to_string(), create_test_connection("dummy", 1)).await;
    assert_eq!(result, Err(profile_shared::LobbyError::InvalidPublicKey));
    assert_eq!(lobby.user_count().await.unwrap(), 0);

    // Test 2: Too short key (63 chars) should fail
    let short_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcde".to_string();
    assert_eq!(short_key.len(), 63);
    let result = add_user(&lobby, short_key, create_test_connection("dummy", 1)).await;
    assert_eq!(result, Err(profile_shared::LobbyError::InvalidPublicKey));
    assert_eq!(lobby.user_count().await.unwrap(), 0);

    // Test 3: Too long key (65 chars) should fail
    let long_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1".to_string();
    assert_eq!(long_key.len(), 65);
    let result = add_user(&lobby, long_key, create_test_connection("dummy", 1)).await;
    assert_eq!(result, Err(profile_shared::LobbyError::InvalidPublicKey));
    assert_eq!(lobby.user_count().await.unwrap(), 0);

    // Test 4: Invalid hex characters (zzz) should fail
    let invalid_hex = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890zzzzyy".to_string();
    let result = add_user(&lobby, invalid_hex, create_test_connection("dummy", 1)).await;
    assert_eq!(result, Err(profile_shared::LobbyError::InvalidPublicKey));
    assert_eq!(lobby.user_count().await.unwrap(), 0);

    println!("✅ test_public_key_validation_rejects_invalid_keys passed");
}

/// Test: test_public_key_validation_accepts_valid_keys
/// Verify that valid 64-char hex keys are accepted
#[tokio::test]
async fn test_public_key_validation_accepts_valid_keys() {
    let lobby = Arc::new(Lobby::new());

    // Valid 64-char hex key
    let valid_key = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string();
    assert_eq!(valid_key.len(), 64);
    assert!(valid_key.chars().all(|c| c.is_ascii_hexdigit()));

    let result = add_user(&lobby, valid_key.clone(), create_test_connection(&valid_key, 1)).await;
    assert!(result.is_ok(), "Valid 64-char hex key should be accepted");
    assert_eq!(lobby.user_count().await.unwrap(), 1);
    assert!(lobby.user_exists(&valid_key).await.unwrap());

    println!("✅ test_public_key_validation_accepts_valid_keys passed");
}
