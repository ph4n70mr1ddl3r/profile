//! Leave notification integration tests
//!
//! Tests that verify leave broadcasts are sent to all remaining users
//! when a user disconnects from the WebSocket connection.

use profile_server::lobby::{ActiveConnection, Lobby};
use profile_shared::Message as SharedMessage;

use tokio::sync::mpsc;

fn create_test_connection_with_sender(
    key: &str,
) -> (ActiveConnection, mpsc::UnboundedReceiver<SharedMessage>) {
    let (sender, receiver) = mpsc::unbounded_channel::<SharedMessage>();

    // Ensure key is exactly 64 characters (32 bytes hex-encoded) for validation
    let padded_key = if key.len() >= 64 {
        key[..64].to_string()
    } else {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        format!(
            "{:016x}{:016x}{:016x}{:016x}",
            hash,
            hash >> 16,
            hash >> 32,
            hash >> 48
        )
    };

    let connection = ActiveConnection {
        public_key: padded_key,
        sender,
        connection_id: 1,
    };

    (connection, receiver)
}

#[tokio::test]
async fn test_single_leave_broadcast() {
    let lobby = Lobby::new();

    // Create 3 users with receivers
    let (conn1, mut rx1) = create_test_connection_with_sender("user1");
    let (conn2, _rx2) = create_test_connection_with_sender("user2");
    let (conn3, mut rx3) = create_test_connection_with_sender("user3");

    let key1 = conn1.public_key.clone();
    let key2 = conn2.public_key.clone();
    let key3 = conn3.public_key.clone();

    // Add all to lobby
    profile_server::lobby::add_user(&lobby, key1.clone(), conn1)
        .await
        .unwrap();
    profile_server::lobby::add_user(&lobby, key2.clone(), conn2)
        .await
        .unwrap();
    profile_server::lobby::add_user(&lobby, key3.clone(), conn3)
        .await
        .unwrap();

    // Drain initial join messages (from adding users)
    // User1 receives join broadcasts for User2 and User3
    for _ in 0..3 {
        let _ = tokio::time::timeout(std::time::Duration::from_millis(100), rx1.recv()).await;
    }
    // User3 receives join broadcast for User2
    let _ = tokio::time::timeout(std::time::Duration::from_millis(100), rx3.recv()).await;

    // User2 disconnects - this should trigger broadcast to users 1 and 3
    profile_server::lobby::remove_user(&lobby, &key2)
        .await
        .unwrap();

    // Assert: User1 received leave notification
    let msg1 = tokio::time::timeout(std::time::Duration::from_millis(100), rx1.recv())
        .await
        .expect("Timeout - User1 should have received notification")
        .expect("User1 should have received leave notification");

    match msg1 {
        SharedMessage::LobbyUpdate { joined, left } => {
            assert!(
                joined.is_none(),
                "Expected no joined users in leave notification"
            );
            assert!(left.is_some(), "Expected left users in notification");
            if let Some(left_users) = left {
                assert_eq!(left_users.len(), 1, "Expected 1 user to leave");
                assert_eq!(left_users[0], key2, "Expected user2 to leave");
            }
        }
        _ => panic!("Expected LobbyUpdate message, got: {:?}", msg1),
    }

    // Assert: User3 also received leave notification
    let msg3 = tokio::time::timeout(std::time::Duration::from_millis(100), rx3.recv())
        .await
        .expect("Timeout - User3 should have received notification")
        .expect("User3 should have received leave notification");

    match msg3 {
        SharedMessage::LobbyUpdate { joined, left } => {
            assert!(joined.is_none());
            assert!(left.is_some());
            if let Some(left_users) = left {
                assert_eq!(left_users.len(), 1);
                assert_eq!(left_users[0], key2);
            }
        }
        _ => panic!("Expected LobbyUpdate message, got: {:?}", msg3),
    }

    println!("✅ Single leave broadcast works - all remaining users notified");
}

#[tokio::test]
async fn test_leaving_user_excluded_from_broadcast() {
    let lobby = Lobby::new();

    // Create 2 users
    let (conn1, mut rx1) = create_test_connection_with_sender("user1");
    let (conn2, mut rx2) = create_test_connection_with_sender("user2");

    let key1 = conn1.public_key.clone();
    let key2 = conn2.public_key.clone();

    // Add both to lobby
    profile_server::lobby::add_user(&lobby, key1.clone(), conn1)
        .await
        .unwrap();
    profile_server::lobby::add_user(&lobby, key2.clone(), conn2)
        .await
        .unwrap();

    // User2 disconnects
    profile_server::lobby::remove_user(&lobby, &key2)
        .await
        .unwrap();

    // Assert: User1 received notification
    let _msg1 = tokio::time::timeout(std::time::Duration::from_millis(100), rx1.recv())
        .await
        .expect("Timeout - User1 should have received notification")
        .expect("User1 should have received leave notification");

    // Assert: User2 did NOT receive own notification (receiver is closed)
    let result = tokio::time::timeout(std::time::Duration::from_millis(100), rx2.recv()).await;

    assert!(
        result.is_err() || result.unwrap().is_none(),
        "User2 should NOT receive their own leave notification"
    );

    println!("✅ Leaving user correctly excluded from broadcast");
}

#[tokio::test]
async fn test_multiple_leaves_consistency() {
    let lobby = Lobby::new();

    // Create 4 users
    let (conn1, mut rx1) = create_test_connection_with_sender("user1");
    let (conn2, mut _rx2) = create_test_connection_with_sender("user2");
    let (conn3, mut _rx3) = create_test_connection_with_sender("user3");
    let (conn4, mut rx4) = create_test_connection_with_sender("user4");

    let key1 = conn1.public_key.clone();
    let key2 = conn2.public_key.clone();
    let key3 = conn3.public_key.clone();
    let key4 = conn4.public_key.clone();

    // Add all to lobby
    profile_server::lobby::add_user(&lobby, key1.clone(), conn1)
        .await
        .unwrap();
    profile_server::lobby::add_user(&lobby, key2.clone(), conn2)
        .await
        .unwrap();
    profile_server::lobby::add_user(&lobby, key3.clone(), conn3)
        .await
        .unwrap();
    profile_server::lobby::add_user(&lobby, key4.clone(), conn4)
        .await
        .unwrap();

    // Drain initial join messages for all remaining users
    // User1 receives joins for User2, User3, User4
    for _ in 0..3 {
        let _ = tokio::time::timeout(std::time::Duration::from_millis(100), rx1.recv()).await;
    }
    // User4 receives joins for User2, User3
    for _ in 0..2 {
        let _ = tokio::time::timeout(std::time::Duration::from_millis(100), rx4.recv()).await;
    }

    // User2 and User3 disconnect simultaneously
    profile_server::lobby::remove_user(&lobby, &key2)
        .await
        .unwrap();
    profile_server::lobby::remove_user(&lobby, &key3)
        .await
        .unwrap();

    // Assert: User1 received two separate leave notifications (one per user)
    let msg1a = tokio::time::timeout(std::time::Duration::from_millis(100), rx1.recv())
        .await
        .expect("Timeout - should have received first leave notification")
        .expect("Should have received first leave notification");

    let msg1b = tokio::time::timeout(std::time::Duration::from_millis(100), rx1.recv())
        .await
        .expect("Timeout - should have received second leave notification")
        .expect("Should have received second leave notification");

    // Verify each message has ONE left user (per-departure notification)
    match &msg1a {
        SharedMessage::LobbyUpdate { left, .. } => {
            if let Some(users) = left {
                assert_eq!(users.len(), 1, "Expected 1 user in first leave notification");
            }
        }
        _ => panic!("Expected LobbyUpdate"),
    }

    match &msg1b {
        SharedMessage::LobbyUpdate { left, .. } => {
            if let Some(users) = left {
                assert_eq!(users.len(), 1, "Expected 1 user in second leave notification");
            }
        }
        _ => panic!("Expected LobbyUpdate"),
    }

    // Track which users left
    let mut left_users = Vec::new();

    for msg in [msg1a, msg1b] {
        match msg {
            SharedMessage::LobbyUpdate { left, .. } => {
                if let Some(users) = left {
                    for u in users {
                        left_users.push(u.clone());
                    }
                }
            }
            _ => panic!("Expected LobbyUpdate"),
        }
    }

    // Assert: Both user2 and user3 left
    assert_eq!(left_users.len(), 2, "Expected 2 users to leave total");
    assert!(left_users.contains(&key2), "Expected user2 to have left");
    assert!(left_users.contains(&key3), "Expected user3 to have left");

    // Assert: User4 also received notifications (verify consistency)
    let msg4a = tokio::time::timeout(std::time::Duration::from_millis(100), rx4.recv())
        .await
        .expect("Timeout - User4 should have received notification")
        .expect("User4 should have received notification");

        match msg4a {
            SharedMessage::LobbyUpdate { left, .. } => {
                if let Some(users) = left {
                    assert!(
                        users.iter().any(|u| u == &key2)
                            || users.iter().any(|u| u == &key3)
                    );
                }
        }
        _ => panic!("Expected LobbyUpdate"),
    }

    println!("✅ Multiple leaves handled correctly - lobby state consistent");
}

#[tokio::test]
async fn test_connection_drop_cleanup() {
    let lobby = Lobby::new();

    // Create user
    let (conn1, _rx1) = create_test_connection_with_sender("user1");
    let (conn2, mut rx2) = create_test_connection_with_sender("user2");

    let key1 = conn1.public_key.clone();
    let key2 = conn2.public_key.clone();

    // Add both to lobby
    profile_server::lobby::add_user(&lobby, key1.clone(), conn1)
        .await
        .unwrap();
    profile_server::lobby::add_user(&lobby, key2.clone(), conn2)
        .await
        .unwrap();

    // User1 disconnects abruptly (simulated via remove_user)
    profile_server::lobby::remove_user(&lobby, &key1)
        .await
        .unwrap();

    // Assert: User2 received notification
    let msg = tokio::time::timeout(std::time::Duration::from_millis(100), rx2.recv())
        .await
        .expect("Timeout - notification should have been sent")
        .expect("Notification should have been received");

        match msg {
            SharedMessage::LobbyUpdate { left, .. } => {
                assert!(left.is_some());
                if let Some(users) = left {
                    assert_eq!(users.len(), 1);
                    assert_eq!(users[0], key1);
                }
            }
        _ => panic!("Expected LobbyUpdate"),
    }

    // Assert: Lobby cleanup complete - no ghost user
    let current_users = profile_server::lobby::get_current_users(&lobby)
        .await
        .unwrap();
    assert_eq!(current_users.len(), 1);
    assert!(!current_users.contains(&key1));
    assert!(current_users.contains(&key2));

    println!("✅ Connection drop cleanup works - no ghost users remain");
}
