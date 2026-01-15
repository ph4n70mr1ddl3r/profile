//! Isolated unit tests for lobby state data structures
//!
//! These tests verify the core lobby data structures work correctly
//! in isolation, without requiring the full server infrastructure.
//!
//! Uses the real types from profile_server::lobby module.

mod test_utils;

use profile_server::lobby::{get_current_users, get_user, Lobby};
use test_utils::create_test_connection;

#[tokio::test]
async fn test_lobby_state_implementation() {
    let lobby = Lobby::new();

    // Test initial state
    assert_eq!(lobby.user_count().await.unwrap(), 0);
    assert!(lobby.get_full_lobby_state().await.unwrap().is_empty());

    // Create a test user
    let user_key = "test_user_123".to_string();
    let connection = create_test_connection(&user_key, 1);

    // Add user
    lobby.add_user(connection).await.unwrap();
    assert_eq!(lobby.user_count().await.unwrap(), 1);

    let state = lobby.get_full_lobby_state().await.unwrap();
    assert_eq!(state.len(), 1);
    assert!(state.contains(&user_key));

    // Remove user
    lobby.remove_user(&user_key).await.unwrap();
    assert_eq!(lobby.user_count().await.unwrap(), 0);
    assert!(lobby.get_full_lobby_state().await.unwrap().is_empty());
}

#[tokio::test]
async fn test_public_key_type_alias() {
    use profile_server::lobby::PublicKey;

    let key: PublicKey = "test_key".to_string();
    assert_eq!(key, "test_key");
}

#[tokio::test]
async fn test_active_connection_creation() {
    let public_key = "test_connection_key_12345678901234".to_string();
    let connection_id = 42;

    // Use the shared test helper
    let connection = create_test_connection(&public_key, connection_id);

    assert_eq!(connection.public_key, public_key);
    assert_eq!(connection.connection_id, connection_id);
}

#[tokio::test]
async fn test_lobby_multiple_users_sequential() {
    let lobby = Lobby::new();

    // Add 5 users sequentially
    for i in 0..5 {
        let key = format!("sequential_user_{:02}", i);
        let connection = create_test_connection(&key, i as u64);
        lobby.add_user(connection).await.unwrap();
    }

    assert_eq!(lobby.user_count().await.unwrap(), 5);

    // Remove 2 users
    for i in 0..2 {
        let key = format!("sequential_user_{:02}", i);
        lobby.remove_user(&key).await.unwrap();
    }

    assert_eq!(lobby.user_count().await.unwrap(), 3);

    // Verify remaining users
    for i in 2..5 {
        let key = format!("sequential_user_{:02}", i);
        assert!(lobby.user_exists(&key).await.unwrap());
    }

    // Verify removed users
    for i in 0..2 {
        let key = format!("sequential_user_{:02}", i);
        assert!(!lobby.user_exists(&key).await.unwrap());
    }
}

#[tokio::test]
async fn test_lobby_get_connection() {
    // NOTE: This test previously used state.rs get_connection() method
    // which was consolidated into manager.rs get_user() function.
    // Using get_user() from manager.rs instead (same functionality).
    let lobby = Lobby::new();
    let test_key = "get_connection_test_12345678901234".to_string();
    let connection = create_test_connection(&test_key, 99);

    // Get before adding (using consolidated get_user API)
    let result = get_user(&lobby, &test_key).await.unwrap();
    assert!(result.is_none());

    // Add user
    lobby.add_user(connection).await.unwrap();

    // Get after adding
    let result = get_user(&lobby, &test_key).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().connection_id, 99);
}

#[tokio::test]
async fn test_lobby_get_all_connections() {
    let lobby = Lobby::new();

    // Add 3 users
    for i in 0..3 {
        let key = format!("all_connections_user_{:02}", i);
        let connection = create_test_connection(&key, i as u64);
        lobby.add_user(connection).await.unwrap();
    }

    // Get all connections
    let connections = lobby.get_all_connections().await.unwrap();
    assert_eq!(connections.len(), 3);

    // Verify all connection IDs
    let mut ids: Vec<u64> = connections.iter().map(|c| c.connection_id).collect();
    ids.sort();
    assert_eq!(ids, vec![0, 1, 2]);
}

#[tokio::test]
async fn test_lobby_user_exists() {
    let lobby = Lobby::new();
    let test_key = "exists_test_12345678901234567890".to_string();

    // Initially doesn't exist
    assert!(!lobby.user_exists(&test_key).await.unwrap());

    // Add user
    let connection = create_test_connection(&test_key, 1);
    lobby.add_user(connection).await.unwrap();

    // Now exists
    assert!(lobby.user_exists(&test_key).await.unwrap());

    // Remove and check again
    lobby.remove_user(&test_key).await.unwrap();
    assert!(!lobby.user_exists(&test_key).await.unwrap());
}

#[tokio::test]
async fn test_lobby_get_user_function() {
    let lobby = Lobby::new();
    let test_key = "get_user_function_test_12345678901234".to_string();

    // Get before adding
    let result = get_user(&lobby, &test_key).await.unwrap();
    assert!(result.is_none());

    // Add user
    let connection = create_test_connection(&test_key, 42);
    lobby.add_user(connection).await.unwrap();

    // Get after adding
    let result = get_user(&lobby, &test_key).await.unwrap();
    assert!(result.is_some());
    assert_eq!(result.unwrap().connection_id, 42);
}

#[tokio::test]
async fn test_lobby_get_current_users_function() {
    let lobby = Lobby::new();

    // Add 3 users
    for i in 0..3 {
        let key = format!("get_current_users_{:02}", i);
        let connection = create_test_connection(&key, i as u64);
        lobby.add_user(connection).await.unwrap();
    }

    // Get all users using function
    let users = get_current_users(&lobby).await.unwrap();
    assert_eq!(users.len(), 3);
}
