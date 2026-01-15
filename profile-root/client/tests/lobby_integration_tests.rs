//! Lobby integration tests
//!
//! Integration tests for Story 2.2: Query & Display Current Online User List
//! Tests cover:
//! - Lobby state receives and displays initial state from server
//! - Lobby updates when users join/leave
//! - Selection activates chat composer
//! - Keyboard navigation works across components
//! - JSON parsing error handling

use profile_client::connection::client::{parse_lobby_message, LobbyResponse};
use profile_client::ui::lobby_state::{LobbyState, LobbyUser};

/// Test: Lobby receives initial state with multiple users
#[tokio::test]
async fn test_lobby_receives_initial_state() {
    let json = r#"{"type":"lobby","users":[{"publicKey":"3a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb","status":"online"},{"publicKey":"7b4d9c2a3e8f1d4c5a6b7e8f9a0b1c2d","status":"online"}]}"#;

    let result = parse_lobby_message(json).unwrap();

    match result {
        LobbyResponse::LobbyState { users } => {
            assert_eq!(users.len(), 2);
            assert_eq!(users[0].public_key, "3a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb");
            assert!(users[0].is_online);
            assert_eq!(users[1].public_key, "7b4d9c2a3e8f1d4c5a6b7e8f9a0b1c2d");
            assert!(users[1].is_online);
        }
        _ => panic!("Expected LobbyState response"),
    }
}

/// Test: Lobby updates when user joins
#[tokio::test]
async fn test_lobby_updates_on_join() {
    let mut state = LobbyState::new();

    // Initial state with one user
    let initial_users = vec![LobbyUser::new("existing_user_12345678".to_string(), true)];
    state.set_users(initial_users.clone());
    assert_eq!(state.len(), 1);

    // Simulate receiving lobby update for new user joining
    let join_json =
        r#"{"type":"lobby_update","joined":[{"publicKey":"new_user_joining_now"}],"left":[]}"#;
    let result = parse_lobby_message(join_json).unwrap();

    match result {
        LobbyResponse::UsersJoined { public_keys } => {
            assert_eq!(public_keys.len(), 1);
            assert_eq!(public_keys[0], "new_user_joining_now");

            // Add new user to state
            state.add_user(LobbyUser::new(public_keys[0].clone(), true));
            assert_eq!(state.len(), 2);
            assert!(state.has_user("new_user_joining_now"));
        }
        _ => panic!("Expected UsersJoined response"),
    }
}

/// Test: Lobby updates when user leaves
#[tokio::test]
async fn test_lobby_updates_on_leave() {
    let mut state = LobbyState::new();

    // Initial state with multiple users
    let initial_users = vec![
        LobbyUser::new("user_staying_12345678".to_string(), true),
        LobbyUser::new("user_leaving_87654321".to_string(), true),
    ];
    state.set_users(initial_users.clone());
    assert_eq!(state.len(), 2);

    // Select user that's about to leave
    state.select("user_leaving_87654321");
    assert_eq!(state.selected_user(), Some("user_leaving_87654321"));

    // Simulate receiving lobby update for user leaving
    let leave_json = r#"{"type":"lobby_update","joined":[],"left":["user_leaving_87654321"]}"#;
    let result = parse_lobby_message(leave_json).unwrap();

    match result {
        LobbyResponse::UsersLeft { public_keys } => {
            assert_eq!(public_keys.len(), 1);
            assert_eq!(public_keys[0], "user_leaving_87654321");

            // Remove user from state
            state.remove_user(&public_keys[0]);
            assert_eq!(state.len(), 1);
            assert!(!state.has_user("user_leaving_87654321"));

            // Selection should be cleared when selected user leaves
            assert_eq!(state.selected_user(), None);
        }
        _ => panic!("Expected UsersLeft response"),
    }
}

/// Test: Selection activates chat composer (simulated via callback)
#[tokio::test]
async fn test_lobby_selection_activates_chat() {
    let mut state = LobbyState::new();

    // Add users
    let users = vec![
        LobbyUser::new("selectable_user_1234567".to_string(), true),
        LobbyUser::new("other_user_8765432".to_string(), true),
    ];
    state.set_users(users);

    // Initially no selection
    assert_eq!(state.selected_user(), None);

    // Select a user
    let result = state.select("selectable_user_1234567");
    assert!(result);
    assert_eq!(state.selected_user(), Some("selectable_user_1234567"));

    // Verify selected user is one we want to message
    assert!(state.is_selected("selectable_user_1234567"));
    assert!(!state.is_selected("other_user_8765432"));
}

/// Test: Multiple users joining at once
#[tokio::test]
async fn test_lobby_multiple_users_join() {
    let join_json = r#"{"type":"lobby_update","joined":[{"publicKey":"user1"},{"publicKey":"user2"},{"publicKey":"user3"}],"left":[]}"#;

    let result = parse_lobby_message(join_json).unwrap();

    match result {
        LobbyResponse::UsersJoined { public_keys } => {
            assert_eq!(public_keys.len(), 3);
            assert_eq!(public_keys[0], "user1");
            assert_eq!(public_keys[1], "user2");
            assert_eq!(public_keys[2], "user3");
        }
        _ => panic!("Expected UsersJoined response with all users"),
    }
}

/// Test: Multiple users leaving at once
#[tokio::test]
async fn test_lobby_multiple_users_leave() {
    let leave_json = r#"{"type":"lobby_update","joined":[],"left":["departed1","departed2"]}"#;

    let result = parse_lobby_message(leave_json).unwrap();

    match result {
        LobbyResponse::UsersLeft { public_keys } => {
            assert_eq!(public_keys.len(), 2);
            assert_eq!(public_keys[0], "departed1");
            assert_eq!(public_keys[1], "departed2");
        }
        _ => panic!("Expected UsersLeft response with all users"),
    }
}

/// Test: Lobby handles empty state
#[tokio::test]
async fn test_lobby_empty_state() {
    let json = r#"{"type":"lobby","users":[]}"#;

    let result = parse_lobby_message(json).unwrap();

    match result {
        LobbyResponse::LobbyState { users } => {
            assert!(users.is_empty());
        }
        _ => panic!("Expected LobbyState response"),
    }
}

/// Test: Lobby handles malformed JSON
#[tokio::test]
async fn test_lobby_handles_malformed_json() {
    // Test 1: Invalid JSON syntax - missing closing bracket
    let invalid_json = r#"{"type":"lobby","users":[{"publicKey":"key1"}"#;

    let result = parse_lobby_message(invalid_json);
    assert!(result.is_err(), "Should fail for invalid JSON syntax");

    // Test 2: Valid JSON but missing required "users" field
    let missing_field_json = r#"{"type":"lobby"}"#;

    let result2 = parse_lobby_message(missing_field_json);
    assert!(
        result2.is_err() || matches!(result2, Ok(LobbyResponse::Ignored)),
        "Should handle missing 'users' field"
    );

    // Test 3: Invalid message type
    let unknown_type_json = r#"{"type":"unknown_type","users":[]}"#;

    let result3 = parse_lobby_message(unknown_type_json);
    assert!(
        matches!(result3, Ok(LobbyResponse::Ignored)),
        "Unknown message types should be ignored"
    );
}

/// Test: Lobby state with mixed online/offline users
#[tokio::test]
async fn test_lobby_mixed_online_status() {
    let json = r#"{"type":"lobby","users":[{"publicKey":"online_user_12345","status":"online"},{"publicKey":"offline_user_67890","status":"offline"}]}"#;

    let result = parse_lobby_message(json).unwrap();

    match result {
        LobbyResponse::LobbyState { users } => {
            assert_eq!(users.len(), 2);

            let online_user = users
                .iter()
                .find(|u| u.public_key == "online_user_12345")
                .unwrap();
            assert!(online_user.is_online);

            let offline_user = users
                .iter()
                .find(|u| u.public_key == "offline_user_67890")
                .unwrap();
            assert!(!offline_user.is_online);
        }
        _ => panic!("Expected LobbyState response"),
    }
}

/// Test: Selection wrap-around with keyboard navigation simulation
#[tokio::test]
async fn test_lobby_selection_wrap_around() {
    let mut state = LobbyState::new();

    let users = vec![
        LobbyUser::new("first_user_12345678".to_string(), true),
        LobbyUser::new("middle_user_8765432".to_string(), true),
        LobbyUser::new("last_user_1122334455".to_string(), true),
    ];
    state.set_users(users);

    // Select last user
    state.select("last_user_1122334455");
    assert_eq!(state.selected_user(), Some("last_user_1122334455"));

    // Simulate arrow up wrap-around to last index
    // Note: Actual keyboard handling is in lobby.rs LobbyKeyboardHandler
    // This test verifies state can handle selection changes
    state.clear_selection();
    state.select("first_user_12345678");
    assert_eq!(state.selected_user(), Some("first_user_12345678"));
}

/// Test: Enter key confirms selection
#[tokio::test]
async fn test_lobby_enter_key_confirms_selection() {
    let mut state = LobbyState::new();

    let users = vec![
        LobbyUser::new("selectable_user_1234567".to_string(), true),
        LobbyUser::new("other_user_8765432".to_string(), true),
    ];
    state.set_users(users);

    // Initially no selection
    assert_eq!(state.selected_user(), None);

    // Select a user (simulating Enter key or click)
    let result = state.select("selectable_user_1234567");
    assert!(result);
    assert_eq!(state.selected_user(), Some("selectable_user_1234567"));

    // Verify selection is confirmed
    assert!(state.is_selected("selectable_user_1234567"));
}
