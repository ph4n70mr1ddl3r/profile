//! Lobby synchronization integration tests for the server
//!
//! Tests cover:
//! - Real-time broadcast delivery! - Lobby (AC1)
// consistency with multiple clients (AC2, AC4)
//! - Network resilience (AC4)
//! - Selection-aware broadcasts (AC5)

use profile_shared::Message as SharedMessage;
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

use profile_server::lobby::manager::{add_user, get_current_users, get_user, remove_user};
use profile_server::lobby::state::{ActiveConnection, Lobby};

/// Create a test lobby with default configuration
fn create_test_lobby() -> Lobby {
    Lobby::new()
}

/// Create a test connection with the given public key
fn create_test_connection(key: &str) -> ActiveConnection {
    static CONNECTION_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    let (sender, _) = mpsc::unbounded_channel::<SharedMessage>();

    // Generate a valid 64-char hex key
    let padded_key = generate_valid_key(key);

    let connection_id = CONNECTION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    ActiveConnection {
        public_key: padded_key,
        sender,
        connection_id,
    }
}

/// Generate a valid 64-character hex public key from input string
fn generate_valid_key(input: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();
    format!(
        "{:016x}{:016x}{:016x}{:016x}",
        hash,
        hash >> 16,
        hash >> 32,
        hash >> 48
    )
}

/// Test AC1: Broadcast delivery within 100ms
///
/// Verifies that lobby broadcasts are delivered within required
/// latency threshold.
#[tokio::test]
async fn test_broadcast_delivery_within_100ms() {
    let lobby = create_test_lobby();

    // Create a test receiver
    let (test_sender, mut test_receiver) = mpsc::unbounded_channel::<SharedMessage>();

    // Create a mock connection
    let mock_connection = ActiveConnection {
        public_key: generate_valid_key("existing_user"),
        sender: test_sender,
        connection_id: 999,
    };

    // Add existing user to lobby
    add_user(&lobby, mock_connection.public_key.clone(), mock_connection)
        .await
        .unwrap();

    // Drain any broadcast messages
    let _ = timeout(Duration::from_millis(10), test_receiver.recv()).await;

    // Measure time for add_user + broadcast (FIX: No artificial delay before measurement)
    let start = std::time::Instant::now();

    // Add a new user (triggers broadcast)
    let connection = create_test_connection("new_user");
    let connection_key = connection.public_key.clone();
    let result = add_user(&lobby, connection_key.clone(), connection).await;

    assert!(result.is_ok());

    // Receive broadcast
    let received = timeout(Duration::from_millis(100), test_receiver.recv())
        .await
        .expect("Timeout waiting for broadcast")
        .expect("No message received");

    let elapsed = start.elapsed();

    // Verify it's a lobby update
    match received {
        SharedMessage::LobbyUpdate { joined, left } => {
            assert!(!joined.is_empty());
            assert!(left.is_empty());
            assert_eq!(joined[0].public_key, connection_key);
        }
        _ => panic!("Expected LobbyUpdate message"),
    }

    // Assert latency is within 100ms
    assert!(
        elapsed.as_millis() < 100,
        "Broadcast took {}ms, required <100ms",
        elapsed.as_millis()
    );
}

/// Test AC2: Multi-client lobby consistency
///
/// Verifies that multiple clients receive consistent lobby state
/// through multiple join/leave events.
#[tokio::test]
async fn test_multi_client_lobby_consistency() {
    let lobby = create_test_lobby();

    // Create receivers for 3 clients
    let (sender1, mut receiver1) = mpsc::unbounded_channel::<SharedMessage>();
    let (sender2, mut receiver2) = mpsc::unbounded_channel::<SharedMessage>();
    let (sender3, mut receiver3) = mpsc::unbounded_channel::<SharedMessage>();

    // Create connections with distinct keys
    let conn1 = ActiveConnection {
        public_key: generate_valid_key("client_1"),
        sender: sender1,
        connection_id: 1,
    };
    let conn2 = ActiveConnection {
        public_key: generate_valid_key("client_2"),
        sender: sender2,
        connection_id: 2,
    };
    let conn3 = ActiveConnection {
        public_key: generate_valid_key("client_3"),
        sender: sender3,
        connection_id: 3,
    };

    let key1 = conn1.public_key.clone();
    let key2 = conn2.public_key.clone();
    let key3 = conn3.public_key.clone();

    // Add all clients
    add_user(&lobby, key1.clone(), conn1).await.unwrap();
    add_user(&lobby, key2.clone(), conn2).await.unwrap();
    add_user(&lobby, key3.clone(), conn3).await.unwrap();

    // Drain all broadcasts
    let _ = timeout(Duration::from_millis(10), receiver1.recv()).await;
    let _ = timeout(Duration::from_millis(10), receiver2.recv()).await;
    let _ = timeout(Duration::from_millis(10), receiver3.recv()).await;

    // Add a 4th client
    let (sender4, mut receiver4) = mpsc::unbounded_channel::<SharedMessage>();
    let conn4 = ActiveConnection {
        public_key: generate_valid_key("client_4"),
        sender: sender4,
        connection_id: 4,
    };
    let key4 = conn4.public_key.clone();

    add_user(&lobby, key4.clone(), conn4).await.unwrap();

    // All existing clients should receive the broadcast
    let msg1 = timeout(Duration::from_millis(100), receiver1.recv())
        .await
        .expect("Client 1 didn't receive broadcast")
        .expect("Client 1 got no message");

    let msg2 = timeout(Duration::from_millis(100), receiver2.recv())
        .await
        .expect("Client 2 didn't receive broadcast")
        .expect("Client 2 got no message");

    let msg3 = timeout(Duration::from_millis(100), receiver3.recv())
        .await
        .expect("Client 3 didn't receive broadcast")
        .expect("Client 3 got no message");

    // All messages should be lobby updates with one joined user
    let joined_key = match (msg1, msg2, msg3) {
        (
            SharedMessage::LobbyUpdate {
                joined: joined1,
                left: _,
            },
            SharedMessage::LobbyUpdate {
                joined: joined2,
                left: _,
            },
            SharedMessage::LobbyUpdate {
                joined: joined3,
                left: _,
            },
        ) => {
            // Verify all have one joined user
            assert_eq!(joined1.len(), 1, "Client 1 should have 1 joined user");
            assert_eq!(joined2.len(), 1, "Client 2 should have 1 joined user");
            assert_eq!(joined3.len(), 1, "Client 3 should have 1 joined user");
            joined1[0].public_key.clone()
        }
        _ => panic!("All clients should receive LobbyUpdate"),
    };

    // Verify the joined key is present in the lobby
    let users = get_current_users(&lobby).await.unwrap();
    assert!(
        users.contains(&joined_key),
        "Joined user should be in lobby"
    );

    // Client 4 should NOT receive the broadcast (they're the one joining)
    let unexpected = timeout(Duration::from_millis(50), receiver4.recv()).await;
    assert!(
        unexpected.is_err(),
        "Client 4 should not receive their own join broadcast"
    );

    // Verify final lobby state
    assert_eq!(users.len(), 4);
    assert!(users.contains(&key1));
    assert!(users.contains(&key2));
    assert!(users.contains(&key3));
    assert!(users.contains(&key4));
}

/// Test AC4: Reconnection preserves lobby consistency
///
/// Verifies that when a user reconnects, the lobby remains consistent
/// and broadcasts are sent correctly.
#[tokio::test]
async fn test_reconnection_preserves_lobby_consistency() {
    let lobby = create_test_lobby();

    // Create a receiver for another client to observe broadcasts
    let (observer_sender, mut observer_receiver) = mpsc::unbounded_channel::<SharedMessage>();
    let observer = ActiveConnection {
        public_key: generate_valid_key("observer"),
        sender: observer_sender,
        connection_id: 99,
    };
    add_user(&lobby, observer.public_key.clone(), observer)
        .await
        .unwrap();

    // Drain initial broadcast
    let _ = timeout(Duration::from_millis(10), observer_receiver.recv()).await;

    // User connects
    let (user_sender, _) = mpsc::unbounded_channel::<SharedMessage>();
    let user_conn = ActiveConnection {
        public_key: generate_valid_key("reconnecting_user"),
        sender: user_sender,
        connection_id: 1,
    };
    let user_key = user_conn.public_key.clone();
    add_user(&lobby, user_key.clone(), user_conn).await.unwrap();

    // Drain join broadcast
    let _ = timeout(Duration::from_millis(10), observer_receiver.recv()).await;

    // User disconnects
    remove_user(&lobby, &user_key).await.unwrap();

    // Drain leave broadcast
    let leave_msg = timeout(Duration::from_millis(100), observer_receiver.recv())
        .await
        .expect("Observer didn't receive leave broadcast")
        .expect("No leave message");

    match leave_msg {
        SharedMessage::LobbyUpdate { joined, left } => {
            assert!(joined.is_empty());
            assert!(!left.is_empty());
            assert_eq!(left[0], user_key);
        }
        _ => panic!("Expected LobbyUpdate with left users"),
    }

    // User reconnects with new connection
    let (user_sender2, _) = mpsc::unbounded_channel::<SharedMessage>();
    let user_conn2 = ActiveConnection {
        public_key: generate_valid_key("reconnecting_user"),
        sender: user_sender2,
        connection_id: 2,
    };
    let user_key2 = user_conn2.public_key.clone();
    assert_eq!(user_key, user_key2); // Same public key

    add_user(&lobby, user_key2.clone(), user_conn2)
        .await
        .unwrap();

    // Should receive join broadcast (user re-joined)
    let join_msg = timeout(Duration::from_millis(100), observer_receiver.recv())
        .await
        .expect("Observer didn't receive rejoin broadcast")
        .expect("No rejoin message");

    match join_msg {
        SharedMessage::LobbyUpdate { joined, left } => {
            assert!(!joined.is_empty());
            assert!(joined[0].public_key == user_key);
            assert!(left.is_empty());
        }
        _ => panic!("Expected LobbyUpdate with joined users"),
    }

    // Verify only one instance of user in lobby
    let users = get_current_users(&lobby).await.unwrap();
    let count = users.iter().filter(|k| *k == &user_key).count();
    assert_eq!(count, 1, "User should only appear once in lobby");
}

/// Test AC4: Network resilience with rapid disconnects
///
/// Verifies lobby handles rapid disconnect/reconnect scenarios gracefully.
#[tokio::test]
async fn test_network_resilience_rapid_changes() {
    let lobby = create_test_lobby();

    // Observer to track broadcasts
    let (observer_sender, mut observer_receiver) = mpsc::unbounded_channel::<SharedMessage>();
    let observer = ActiveConnection {
        public_key: generate_valid_key("observer"),
        sender: observer_sender,
        connection_id: 99,
    };
    add_user(&lobby, observer.public_key.clone(), observer)
        .await
        .unwrap();
    let _ = timeout(Duration::from_millis(10), observer_receiver.recv()).await;

    // Rapid connect/disconnect cycles
    for i in 0..10 {
        let (sender, _) = mpsc::unbounded_channel::<SharedMessage>();
        let temp_conn = ActiveConnection {
            public_key: generate_valid_key(&format!("temp_user_{}", i)),
            sender,
            connection_id: 1000 + i,
        };
        let temp_key = temp_conn.public_key.clone();

        add_user(&lobby, temp_key.clone(), temp_conn).await.unwrap();
        remove_user(&lobby, &temp_key).await.unwrap();
    }

    // Should have handled all events without errors
    let users = get_current_users(&lobby).await.unwrap();
    assert_eq!(users.len(), 1, "Only observer should remain in lobby");

    // Verify observer received broadcasts for all joins and leaves
    let mut join_count = 0;
    let mut leave_count = 0;

    while let Ok(Some(msg)) = timeout(Duration::from_millis(50), observer_receiver.recv()).await {
        if let SharedMessage::LobbyUpdate { joined, left } = msg {
            join_count += joined.len();
            leave_count += left.len();
        }
    }

    assert_eq!(join_count, 10, "Observer should receive 10 join broadcasts");
    assert_eq!(
        leave_count, 10,
        "Observer should receive 10 leave broadcasts"
    );
}

/// Test AC4: Lobby state query after network issues
///
/// Verifies that get_current_users returns accurate state after
/// various network conditions.
#[tokio::test]
async fn test_lobby_state_accuracy_after_network_issues() {
    let lobby = create_test_lobby();

    // Add 5 users
    for i in 0..5 {
        let conn = create_test_connection(&format!("user_{}", i));
        let key = conn.public_key.clone();
        add_user(&lobby, key.clone(), conn).await.unwrap();
    }

    assert_eq!(get_current_users(&lobby).await.unwrap().len(), 5);

    // Simulate network partition - remove users 1 and 3
    remove_user(&lobby, &generate_valid_key("user_1"))
        .await
        .unwrap();
    remove_user(&lobby, &generate_valid_key("user_3"))
        .await
        .unwrap();

    assert_eq!(get_current_users(&lobby).await.unwrap().len(), 3);

    // Add back user 1
    let conn = create_test_connection("user_1_readd");
    let key = conn.public_key.clone();
    add_user(&lobby, key.clone(), conn).await.unwrap();

    assert_eq!(get_current_users(&lobby).await.unwrap().len(), 4);

    // Verify get_user works correctly
    let user = get_user(&lobby, &key).await.unwrap();
    assert!(user.is_some());

    let nonexistent_key =
        "nonexistent_key_1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcd"
            .to_string();
    let nonexistent = get_user(&lobby, &nonexistent_key).await.unwrap();
    assert!(nonexistent.is_none());
}

/// Test: Broadcast excludes sender (user doesn't receive their own join)
#[tokio::test]
async fn test_broadcast_excludes_sender() {
    let lobby = create_test_lobby();

    let (sender, mut receiver) = mpsc::unbounded_channel::<SharedMessage>();
    let conn = ActiveConnection {
        public_key: generate_valid_key("new_user"),
        sender,
        connection_id: 1,
    };
    let key = conn.public_key.clone();

    add_user(&lobby, key.clone(), conn).await.unwrap();

    // User should NOT receive their own join broadcast
    let unexpected = timeout(Duration::from_millis(50), receiver.recv()).await;
    assert!(
        unexpected.is_err(),
        "User should not receive their own join broadcast"
    );

    // Lobby should have the user
    let users = get_current_users(&lobby).await.unwrap();
    assert!(users.contains(&key));
}

/// Test: Leave broadcast to remaining users
///
/// Verifies that when a user leaves, the remaining users receive
/// the leave notification. The broadcast goes to remaining users only.
#[tokio::test]
async fn test_leave_broadcast_excludes_leaving_user() {
    let lobby = create_test_lobby();

    // Add a user who will leave
    let (sender, _receiver) = mpsc::unbounded_channel::<SharedMessage>();
    let conn = ActiveConnection {
        public_key: generate_valid_key("leaving_user"),
        sender,
        connection_id: 1,
    };
    let key = conn.public_key.clone();
    add_user(&lobby, key.clone(), conn).await.unwrap();

    // Add an observer to receive the leave broadcast
    let (observer_sender, mut observer_receiver) = mpsc::unbounded_channel::<SharedMessage>();
    let observer_key = generate_valid_key("observer");
    let observer = ActiveConnection {
        public_key: observer_key.clone(),
        sender: observer_sender,
        connection_id: 2,
    };
    add_user(&lobby, observer_key.clone(), observer)
        .await
        .unwrap();

    // Drain any broadcasts
    let _ = timeout(Duration::from_millis(10), observer_receiver.recv()).await;

    // Remove the leaving user
    remove_user(&lobby, &key).await.unwrap();

    // Observer should receive the leave broadcast
    let leave_msg = timeout(Duration::from_millis(100), observer_receiver.recv())
        .await
        .expect("Observer didn't receive leave broadcast")
        .expect("No leave message");

    match leave_msg {
        SharedMessage::LobbyUpdate { joined, left } => {
            assert!(joined.is_empty());
            assert!(!left.is_empty());
            assert_eq!(left[0], key);
        }
        _ => panic!("Expected LobbyUpdate with left users"),
    }

    // Verify lobby state is correct
    let users = get_current_users(&lobby).await.unwrap();
    assert_eq!(users.len(), 1);
    assert!(users.contains(&observer_key));
}
