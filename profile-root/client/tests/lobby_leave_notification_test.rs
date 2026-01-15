//! Leave notification integration tests for Story 2.4
//!
//! Integration tests that verify the full leave notification flow:
//! - Server broadcasts leave notification via WebSocket
//! - Client receives and parses lobby_update with left field
//! - Client removes departed users from lobby display
//! - Client clears selection when selected user leaves
//!
//! These tests complement the server-side tests in leave_notification_tests.rs
//! by verifying the client-side handling of leave notifications.

use profile_client::connection::client::{parse_lobby_message, LobbyResponse};
use profile_client::ui::lobby_state::{LobbyState, LobbyUser};

/// Test: Client parses lobby_update with left users correctly
#[tokio::test]
async fn test_parse_lobby_update_with_left_users() {
    let json = r#"{"type":"lobby_update","joined":[],"left":["abc123def456","xyz789abc012"]}"#;

    let result = parse_lobby_message(json).unwrap();

    match result {
        LobbyResponse::UsersLeft { public_keys } => {
            assert_eq!(public_keys.len(), 2);
            assert_eq!(public_keys[0], "abc123def456");
            assert_eq!(public_keys[1], "xyz789abc012");
        }
        _ => panic!("Expected UsersLeft response"),
    }
}

/// Test: Client removes departed users from lobby display (AC#2)
#[tokio::test]
async fn test_client_removes_departed_users_from_display() {
    let mut state = LobbyState::new();

    // Initial state with 3 users
    let initial_users = vec![
        LobbyUser::new("user1_1234567890123456".to_string(), true),
        LobbyUser::new("user2_1234567890123456".to_string(), true),
        LobbyUser::new("user3_1234567890123456".to_string(), true),
    ];
    state.set_users(initial_users.clone());
    assert_eq!(state.len(), 3);
    assert!(state.has_user("user1_1234567890123456"));
    assert!(state.has_user("user2_1234567890123456"));
    assert!(state.has_user("user3_1234567890123456"));

    // Simulate receiving lobby_update for user2 leaving
    let leave_json = r#"{"type":"lobby_update","joined":[],"left":["user2_1234567890123456"]}"#;
    let result = parse_lobby_message(leave_json).unwrap();

    match result {
        LobbyResponse::UsersLeft { public_keys } => {
            assert_eq!(public_keys.len(), 1);
            assert_eq!(public_keys[0], "user2_1234567890123456");

            // Remove user from state
            for key in public_keys {
                state.remove_user(&key);
            }
        }
        _ => panic!("Expected UsersLeft response"),
    }

    // Verify user2 was removed from display
    assert_eq!(state.len(), 2);
    assert!(state.has_user("user1_1234567890123456"));
    assert!(!state.has_user("user2_1234567890123456"));
    assert!(state.has_user("user3_1234567890123456"));

    println!("✅ Client removes departed users from lobby display - verified");
}

/// Test: Selection is cleared when selected user leaves (AC#3)
#[tokio::test]
async fn test_selection_cleared_when_selected_user_leaves() {
    let mut state = LobbyState::new();

    // Initial state with 2 users
    let initial_users = vec![
        LobbyUser::new("staying_user_12345678".to_string(), true),
        LobbyUser::new("leaving_user_87654321".to_string(), true),
    ];
    state.set_users(initial_users);

    // Select the user who's about to leave
    state.select("leaving_user_87654321");
    assert_eq!(state.selected_user(), Some("leaving_user_87654321"));

    // Simulate receiving lobby_update for selected user leaving
    let leave_json = r#"{"type":"lobby_update","joined":[],"left":["leaving_user_87654321"]}"#;
    let result = parse_lobby_message(leave_json).unwrap();

    match result {
        LobbyResponse::UsersLeft { public_keys } => {
            assert_eq!(public_keys.len(), 1);

            // Remove user and verify selection is cleared
            for key in public_keys {
                let was_selected = state.selected_user() == Some(key.as_str());
                state.remove_user(&key);

                if was_selected {
                    // Selection should be cleared automatically by remove_user
                    assert_eq!(state.selected_user(), None);
                }
            }
        }
        _ => panic!("Expected UsersLeft response"),
    }

    println!("✅ Selection cleared when selected user leaves - verified");
}

/// Test: Non-selected user removal doesn't affect selection (AC#3 edge case)
#[tokio::test]
async fn test_non_selected_removal_preserves_selection() {
    let mut state = LobbyState::new();

    // Initial state with 3 users
    let initial_users = vec![
        LobbyUser::new("user_a_123456789012".to_string(), true),
        LobbyUser::new("user_b_123456789012".to_string(), true),
        LobbyUser::new("user_c_123456789012".to_string(), true),
    ];
    state.set_users(initial_users);

    // Select user_a
    state.select("user_a_123456789012");
    assert_eq!(state.selected_user(), Some("user_a_123456789012"));

    // Simulate receiving lobby_update for user_b leaving (not selected)
    let leave_json = r#"{"type":"lobby_update","joined":[],"left":["user_b_123456789012"]}"#;
    let result = parse_lobby_message(leave_json).unwrap();

    match result {
        LobbyResponse::UsersLeft { public_keys } => {
            assert_eq!(public_keys.len(), 1);
            for key in public_keys {
                state.remove_user(&key);
            }
        }
        _ => panic!("Expected UsersLeft response"),
    }

    // Verify user_b was removed but selection preserved
    assert_eq!(state.selected_user(), Some("user_a_123456789012"));
    assert!(!state.has_user("user_b_123456789012"));

    println!("✅ Non-selected user removal preserves selection - verified");
}

/// Test: Multiple simultaneous leaves handled correctly (AC#4)
#[tokio::test]
async fn test_multiple_simultaneous_leaves() {
    let mut state = LobbyState::new();

    // Initial state with 5 users
    let initial_users = vec![
        LobbyUser::new("user1_aaaaaaaaaaaaaa".to_string(), true),
        LobbyUser::new("user2_bbbbbbbbbbbbbb".to_string(), true),
        LobbyUser::new("user3_cccccccccccccc".to_string(), true),
        LobbyUser::new("user4_dddddddddddddd".to_string(), true),
        LobbyUser::new("user5_eeeeeeeeeeeeee".to_string(), true),
    ];
    state.set_users(initial_users.clone());
    assert_eq!(state.len(), 5);

    // Select user3 (who will be in the middle of the leave batch)
    state.select("user3_cccccccccccccc");
    assert_eq!(state.selected_user(), Some("user3_cccccccccccccc"));

    // Simulate receiving lobby_update with multiple users leaving
    // Per Story 2.4 design, this is received as separate messages
    // but we also handle multiple in a single message for robustness
    let leave_json = r#"{"type":"lobby_update","joined":[],"left":["user2_bbbbbbbbbbbbbb","user3_cccccccccccccc","user4_dddddddddddddd"]}"#;
    let result = parse_lobby_message(leave_json).unwrap();

    match result {
        LobbyResponse::UsersLeft { public_keys } => {
            assert_eq!(public_keys.len(), 3);

            // Remove all departed users
            for key in public_keys.clone() {
                let was_selected = state.selected_user() == Some(key.as_str());
                state.remove_user(&key);
                if was_selected {
                    assert_eq!(state.selected_user(), None);
                }
            }

            // Verify all 3 users removed
            assert_eq!(state.len(), 2);
            assert!(!state.has_user("user2_bbbbbbbbbbbbbb"));
            assert!(!state.has_user("user3_cccccccccccccc"));
            assert!(!state.has_user("user4_dddddddddddddd"));
            assert!(state.has_user("user1_aaaaaaaaaaaaaa"));
            assert!(state.has_user("user5_eeeeeeeeeeeeee"));
        }
        _ => panic!("Expected UsersLeft response"),
    }

    println!("✅ Multiple simultaneous leaves handled correctly - verified");
}

/// Test: Lobby re-renders after user removal (AC#2)
#[tokio::test]
async fn test_lobby_renders_after_user_removal() {
    let mut state = LobbyState::new();

    // Initial state with 2 users
    let initial_users = vec![
        LobbyUser::new("render_test_user1_123".to_string(), true),
        LobbyUser::new("render_test_user2_456".to_string(), true),
    ];
    state.set_users(initial_users);

    // Verify initial state
    assert_eq!(state.len(), 2);

    // Remove one user
    state.remove_user("render_test_user1_123");

    // Verify lobby state reflects removal (would trigger re-render in UI)
    assert_eq!(state.len(), 1);
    assert!(!state.has_user("render_test_user1_123"));
    assert!(state.has_user("render_test_user2_456"));

    println!("✅ Lobby state correctly reflects user removal - verified for re-render");
}

/// Test: Protocol format matches server broadcast (AC#1)
#[tokio::test]
async fn test_protocol_format_matches_server_broadcast() {
    // This test verifies the client correctly interprets the server's
    // lobby_update message format as specified in AC#1:
    // {type: "lobby_update", left: ["publicKey1", "publicKey2"]}

    // Test single user leave
    let single_leave = r#"{"type":"lobby_update","joined":[],"left":["single_user_key_123456"]}"#;
    let result1 = parse_lobby_message(single_leave).unwrap();
    match result1 {
        LobbyResponse::UsersLeft { public_keys } => {
            assert_eq!(public_keys.len(), 1);
            assert_eq!(public_keys[0], "single_user_key_123456");
        }
        _ => panic!("Expected UsersLeft"),
    }

    // Test empty left (should be Ignored - no users to process)
    let empty_left = r#"{"type":"lobby_update","joined":[],"left":[]}"#;
    let result2 = parse_lobby_message(empty_left).unwrap();
    match result2 {
        LobbyResponse::Ignored => {
            // Empty update is correctly ignored
        }
        _ => panic!("Expected Ignored for empty update"),
    }

    println!("✅ Protocol format matches server broadcast - verified");
}

/// Test: Edge case - removing non-existent user is safe
#[tokio::test]
async fn test_removing_nonexistent_user_is_safe() {
    let mut state = LobbyState::new();

    // Initial state with 1 user
    let initial_users = vec![LobbyUser::new("existing_user_123456".to_string(), true)];
    state.set_users(initial_users);

    // Try to remove a user that doesn't exist
    state.remove_user("nonexistent_user_123456");

    // Should not panic, existing user should still be there
    assert_eq!(state.len(), 1);
    assert!(state.has_user("existing_user_123456"));

    println!("✅ Removing non-existent user is safe - verified");
}

/// Test: Integration - Full leave notification flow simulation
#[tokio::test]
async fn test_full_leave_notification_flow() {
    let mut state = LobbyState::new();

    // Setup: 3 users in lobby, user1 is selected
    let initial_users = vec![
        LobbyUser::new("alice_12345678901234".to_string(), true),
        LobbyUser::new("bob_12345678901234".to_string(), true),
        LobbyUser::new("charlie_1234567890".to_string(), true),
    ];
    state.set_users(initial_users.clone());
    state.select("bob_12345678901234");
    assert_eq!(state.selected_user(), Some("bob_12345678901234"));

    // Step 1: Server broadcasts leave notification
    // Server sends: {type: "lobby_update", left: ["bob_12345678901234"]}
    let leave_json = r#"{"type":"lobby_update","joined":[],"left":["bob_12345678901234"]}"#;

    // Step 2: Client receives and parses message
    let response = parse_lobby_message(leave_json).unwrap();

    // Step 3: Client updates lobby state
    match response {
        LobbyResponse::UsersLeft { public_keys } => {
            for key in public_keys {
                let was_selected = state.selected_user() == Some(key.as_str());
                state.remove_user(&key);

                // AC#3: If selected user leaves, selection is cleared
                if was_selected {
                    assert_eq!(state.selected_user(), None);
                }
            }
        }
        _ => panic!("Expected UsersLeft"),
    }

    // Verify final state
    assert_eq!(state.len(), 2); // alice and charlie remain
    assert!(state.has_user("alice_12345678901234"));
    assert!(!state.has_user("bob_12345678901234")); // bob left
    assert!(state.has_user("charlie_1234567890"));
    assert_eq!(state.selected_user(), None); // selection cleared

    println!("✅ Full leave notification flow works end-to-end - verified");
}
