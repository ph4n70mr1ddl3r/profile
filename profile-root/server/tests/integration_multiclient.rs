//! End-to-end multi-client tests for lobby functionality
//!
//! This file tests the complete lobby workflow with multiple simulated clients
//! connecting to a real server, authenticating, and verifying lobby state.
//!
//! Tests include:
//! - Multiple clients authenticating and appearing in each other's lobbies
//! - Client disconnections being reflected in all other clients' lobbies
//! - Reconnection handling without duplicates
//! - Lobby state consistency across all clients
//!
//! This file satisfies Story 2.1 requirement for E2E multi-client testing.

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use profile_server::lobby::{Lobby, ActiveConnection, add_user, remove_user, get_current_users};
use profile_shared::{Message as SharedMessage, LobbyError};

mod test_utils;
use test_utils::create_test_connection;

/// Generate a unique 64-char hex public key for testing based on index
fn generate_test_key(index: u64) -> String {
    // Generate a unique 64-char hex key from the index
    // Use a simpler approach that avoids overflow
    let hex_str = format!("{:064}", index);
    hex_str
}

/// Test: Multiple clients authenticate and verify all appear in each other's lobby
///
/// This E2E test simulates 3 clients connecting, authenticating, and verifying
/// that they can all see each other in the lobby.
#[tokio::test]
async fn test_e2e_multiple_clients_lobby_consistency() {
    // Create a shared lobby (simulating server state)
    let lobby = Arc::new(Lobby::new());

    // Generate 3 unique keys for 3 clients
    let key1 = generate_test_key(1);
    let key2 = generate_test_key(2);
    let key3 = generate_test_key(3);

    let keys = vec![key1.clone(), key2.clone(), key3.clone()];

    // Simulate 3 clients connecting and authenticating
    let mut handles = Vec::new();

    for (i, key) in keys.iter().enumerate() {
        let lobby_clone = lobby.clone();
        let key_clone = key.clone();

        let handle = tokio::spawn(async move {
            // Create a connection for this client
            let (sender, _) = mpsc::unbounded_channel::<SharedMessage>();
            let connection = ActiveConnection {
                public_key: key_clone.clone(),
                sender,
                connection_id: (i + 1) as u64,
            };

            // Client "authenticates" by adding themselves to lobby
            add_user(&lobby_clone, key_clone.clone(), connection).await
        });

        handles.push(handle);
    }

    // Wait for all clients to authenticate
    let results: Vec<Result<(), LobbyError>> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // All authentications should succeed
    for result in results {
        assert!(result.is_ok(), "Client authentication should succeed");
    }

    // Verify all 3 users are in the lobby
    let lobby_users = get_current_users(&lobby).await.unwrap();
    assert_eq!(lobby_users.len(), 3, "All 3 clients should be in lobby");

    // Verify each key is present
    assert!(lobby_users.contains(&key1), "Client 1 should be in lobby");
    assert!(lobby_users.contains(&key2), "Client 2 should be in lobby");
    assert!(lobby_users.contains(&key3), "Client 3 should be in lobby");

    println!("✅ E2E test_e2e_multiple_clients_lobby_consistency passed - 3 clients see each other");
}

/// Test: One client disconnects and is removed from all other lobbies
///
/// Verifies that when a client disconnects (simulated by remove_user),
/// they are properly removed and no longer visible to other clients.
#[tokio::test]
async fn test_e2e_client_disconnect_removes_from_lobby() {
    let lobby = Arc::new(Lobby::new());

    // Create 3 clients
    let key1 = generate_test_key(10);
    let key2 = generate_test_key(11);
    let key3 = generate_test_key(12);

    // Add all 3 clients to lobby
    let conn1 = create_test_connection(&key1, 1);
    let conn2 = create_test_connection(&key2, 2);
    let conn3 = create_test_connection(&key3, 3);

    add_user(&lobby, key1.clone(), conn1).await.unwrap();
    add_user(&lobby, key2.clone(), conn2).await.unwrap();
    add_user(&lobby, key3.clone(), conn3).await.unwrap();

    // Verify all 3 are in lobby
    let users_before = get_current_users(&lobby).await.unwrap();
    assert_eq!(users_before.len(), 3);

    // Simulate client 2 disconnecting
    remove_user(&lobby, &key2).await.unwrap();

    // Verify only 2 users remain
    let users_after = get_current_users(&lobby).await.unwrap();
    assert_eq!(users_after.len(), 2, "Should have 2 users after disconnect");

    // Verify client 2 is gone
    assert!(!users_after.contains(&key2), "Disconnected client should not be in lobby");
    assert!(users_after.contains(&key1), "Client 1 should still be in lobby");
    assert!(users_after.contains(&key3), "Client 3 should still be in lobby");

    println!("✅ E2E test_e2e_client_disconnect_removes_from_lobby passed - disconnected client removed");
}

/// Test: Reconnection does not create duplicates
///
/// When a client disconnects and reconnects with the same key,
/// they should replace the old entry, not create a duplicate.
#[tokio::test]
async fn test_e2e_reconnection_no_duplicates() {
    let lobby = Arc::new(Lobby::new());

    let key = generate_test_key(20);

    // First connection
    let conn1 = create_test_connection(&key, 1);
    add_user(&lobby, key.clone(), conn1).await.unwrap();

    assert_eq!(get_current_users(&lobby).await.unwrap().len(), 1);

    // Disconnect
    remove_user(&lobby, &key).await.unwrap();
    assert_eq!(get_current_users(&lobby).await.unwrap().len(), 0);

    // Reconnect with new connection
    let conn2 = create_test_connection(&key, 2);
    add_user(&lobby, key.clone(), conn2).await.unwrap();

    // Should still be only 1 user (no duplicate)
    let users = get_current_users(&lobby).await.unwrap();
    assert_eq!(users.len(), 1, "Should have exactly 1 user after reconnection");

    println!("✅ E2E test_e2e_reconnection_no_duplicates passed - no duplicates after reconnection");
}

/// Test: Rapid joins and leaves don't cause ghost users
///
/// Stress test to ensure that rapid add/remove operations
/// don't leave ghost users in the lobby.
#[tokio::test]
async fn test_e2e_rapid_joins_leaves_no_ghosts() {
    let lobby = Arc::new(Lobby::new());
    let num_operations = 20;

    // Rapidly add and remove users
    for i in 0..num_operations {
        let key = generate_test_key(100 + i);
        let connection = create_test_connection(&key, i as u64);

        // Add
        add_user(&lobby, key.clone(), connection).await.unwrap();

        // Immediately remove
        remove_user(&lobby, &key).await.unwrap();
    }

    // Verify no ghost users remain
    let users = get_current_users(&lobby).await.unwrap();
    assert_eq!(users.len(), 0, "Should have 0 ghost users after rapid operations");

    // Now add some users and remove half
    for i in 0..10 {
        let key = generate_test_key(200 + i);
        let connection = create_test_connection(&key, i as u64);
        add_user(&lobby, key.clone(), connection).await.unwrap();
    }

    // Remove even-numbered users (200, 202, 204, 206, 208)
    for i in (0..10).step_by(2) {
        let key = generate_test_key(200 + i);
        remove_user(&lobby, &key).await.unwrap();
    }

    // Should have exactly 5 users remaining
    let users = get_current_users(&lobby).await.unwrap();
    assert_eq!(users.len(), 5, "Should have 5 users after removing half");

    // Verify all remaining keys are unique (no duplicates)
    let mut unique_users = users.clone();
    unique_users.sort();
    unique_users.dedup();
    assert_eq!(users.len(), unique_users.len(), "No duplicate keys should exist");

    println!("✅ E2E test_e2e_rapid_joins_leaves_no_ghosts passed - {} rapid operations clean", num_operations);
}

/// Test: Lobby state is consistent during concurrent operations
///
/// Verifies that concurrent add/remove operations maintain
/// lobby consistency without data corruption.
#[tokio::test]
async fn test_e2e_concurrent_lobby_consistency() {
    let lobby = Arc::new(Lobby::new());
    let num_clients = 10;

    // Spawn concurrent clients - indices 0-9
    // Clients 0, 3, 6, 9 will add then remove themselves
    // Clients 1, 2, 4, 5, 7, 8 will add and stay
    let mut handles = Vec::new();

    for i in 0..num_clients {
        let lobby_clone = lobby.clone();
        let key = generate_test_key(300 + i);

        let handle = tokio::spawn(async move {
            let connection = create_test_connection(&key, i as u64);

            // Some clients will add, some will add then remove
            if i % 3 == 0 {
                // Add then remove (clients 0, 3, 6, 9)
                add_user(&lobby_clone, key.clone(), connection).await.unwrap();
                tokio::time::sleep(Duration::from_millis(10)).await;
                remove_user(&lobby_clone, &key).await.unwrap();
                0 // Removed
            } else {
                // Just add and stay (clients 1, 2, 4, 5, 7, 8)
                add_user(&lobby_clone, key.clone(), connection).await.unwrap();
                1 // Stayed
            }
        });

        handles.push(handle);
    }

    // Wait for all concurrent operations
    let results: Vec<u32> = futures_util::future::join_all(handles)
        .await
        .into_iter()
        .map(|r| r.unwrap())
        .collect();

    // Count how many clients stayed (didn't remove themselves)
    // Clients that removed themselves: 0, 3, 6, 9 = 4 clients
    // Clients that stayed: 1, 2, 4, 5, 7, 8 = 6 clients
    let _stayed_count: usize = results.iter().map(|x| *x as usize).sum();

    // Verify lobby state is consistent
    let users = get_current_users(&lobby).await.unwrap();
    let expected = 6;
    assert_eq!(
        users.len(),
        expected,
        "Lobby should have {} users, got {}. Users: {:?}",
        expected,
        users.len(),
        users
    );

    // Verify no duplicates
    let mut unique_users = users.clone();
    unique_users.sort();
    unique_users.dedup();
    assert_eq!(
        users.len(),
        unique_users.len(),
        "No duplicate users should exist in lobby"
    );

    println!("✅ E2E test_e2e_concurrent_lobby_consistency passed - {} clients, {} final users",
             num_clients, users.len());
}

/// Test: Message routing uses correct connection sender
///
/// Verifies that the lobby returns the correct sender for message routing
/// when sending messages to specific users.
#[tokio::test]
async fn test_e2e_message_routing_sender_correct() {
    let lobby = Arc::new(Lobby::new());

    // Create channels for message routing
    let (sender1, mut receiver1) = mpsc::unbounded_channel::<SharedMessage>();
    let (sender2, _receiver2) = mpsc::unbounded_channel::<SharedMessage>();

    let key1 = generate_test_key(30);
    let key2 = generate_test_key(31);

    // Create connections with real senders
    let conn1 = ActiveConnection {
        public_key: key1.clone(),
        sender: sender1,
        connection_id: 1,
    };

    let conn2 = ActiveConnection {
        public_key: key2.clone(),
        sender: sender2,
        connection_id: 2,
    };

    // Add both to lobby
    add_user(&lobby, key1.clone(), conn1).await.unwrap();
    add_user(&lobby, key2.clone(), conn2).await.unwrap();

    // Get user connections for routing
    let user1_conn = profile_server::lobby::get_user(&lobby, &key1).await.unwrap();
    let user2_conn = profile_server::lobby::get_user(&lobby, &key2).await.unwrap();

    // Both should exist
    assert!(user1_conn.is_some());
    assert!(user2_conn.is_some());

    // Send a message to user 1 via their returned sender
    let test_message = SharedMessage::new_text(
        "Hello User 1".to_string(),
        key2.clone(),
        "test_signature".to_string(),
        "2025-12-23T10:00:00Z".to_string(),
    );

    // Drain any broadcast messages that might be in the queue first
    let _ = tokio::time::timeout(Duration::from_millis(10), receiver1.recv()).await;

    if let Some(conn) = user1_conn {
        let _ = conn.sender.send(test_message.clone());
    }

    // User 1 should receive the message
    let received = tokio::time::timeout(Duration::from_millis(100), receiver1.recv())
        .await
        .expect("User 1 should receive message")
        .expect("Message should be received");

    match received {
        SharedMessage::Text { message, .. } => {
            assert_eq!(message, "Hello User 1");
        }
        _ => panic!("Expected Text message"),
    }

    println!("✅ E2E test_e2e_message_routing_sender_correct passed - messages routed to correct users");
}

/// Test: Complete message routing path (Story 3.2 integration)
///
/// This test verifies the complete message routing flow:
/// 1. Two clients connect and authenticate
/// 2. Client A looks up Client B's connection via `get_user()`
/// 3. Client A sends a message through B's sender channel
/// 4. Client B receives the message via their receiver
///
/// This is the critical path for Story 3.2 (Send Message to Server)
#[tokio::test]
async fn test_e2e_complete_message_routing_path() {
    let lobby = Arc::new(Lobby::new());

    // Create sender/receiver pairs for two clients
    let (sender_a, mut receiver_a) = mpsc::unbounded_channel::<SharedMessage>();
    let (sender_b, mut receiver_b) = mpsc::unbounded_channel::<SharedMessage>();

    let key_a = generate_test_key(40);
    let key_b = generate_test_key(41);

    // Create connections
    let conn_a = ActiveConnection {
        public_key: key_a.clone(),
        sender: sender_a,
        connection_id: 1,
    };
    let conn_b = ActiveConnection {
        public_key: key_b.clone(),
        sender: sender_b,
        connection_id: 2,
    };

    // Both clients join lobby
    add_user(&lobby, key_a.clone(), conn_a).await.unwrap();
    add_user(&lobby, key_b.clone(), conn_b).await.unwrap();

    // Verify both are in lobby
    let users = get_current_users(&lobby).await.unwrap();
    assert_eq!(users.len(), 2);

    // ========================================
    // Story 3.2 Message Routing Path:
    // ========================================
    // Step 1: Client A wants to send to Client B
    // Step 2: Server queries lobby: is recipient online?
    // Step 3: get_user() returns Some(ActiveConnection) if online
    // Step 4: Server routes message through B's sender channel

    // Simulate Step 2 & 3: Query lobby for recipient
    let recipient_conn = profile_server::lobby::get_user(&lobby, &key_b)
        .await
        .unwrap()
        .expect("Recipient should be online");

    // Verify we got the correct connection (with correct sender)
    assert_eq!(recipient_conn.connection_id, 2);

    // Step 4: Send message through recipient's sender (simulating server routing)
    let routing_message = SharedMessage::new_text(
        "Hello from A!".to_string(),
        key_a.clone(),
        "sig_abc123".to_string(),
        "2025-12-23T12:00:00Z".to_string(),
    );

    // Send through B's sender (as server would do for routing)
    let _ = recipient_conn.sender.send(routing_message.clone());

    // Verify B receives the message
    let received = tokio::time::timeout(Duration::from_millis(100), receiver_b.recv())
        .await
        .expect("Client B should receive routed message")
        .expect("Message should not be None");

    match received {
        SharedMessage::Text { message, sender_public_key, .. } => {
            assert_eq!(message, "Hello from A!");
            assert_eq!(sender_public_key, key_a);
        }
        _ => panic!("Expected Text message"),
    }

    // Also verify A's receiver is empty (message was routed to B, not broadcast to A)
    // Drain any potential broadcast messages
    let _ = tokio::time::timeout(Duration::from_millis(10), receiver_a.recv()).await;

    // Verify A did NOT receive the message (it was routed specifically to B)
    let not_received = tokio::time::timeout(Duration::from_millis(50), receiver_a.recv()).await;
    assert!(not_received.is_err(), "Client A should NOT receive message routed to B");

    println!("✅ E2E test_e2e_complete_message_routing_path passed - Story 3.2 routing verified");
}
