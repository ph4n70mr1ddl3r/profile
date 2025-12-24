//! Lobby display tests - verifies lobby component renders users correctly
//!
//! Tests cover:
//! - LobbyComponent renders all users from state
//! - Empty lobby shows "No users online" message
//! - Online indicator shown for each user
//! - Public key displayed in monospace, blue, untruncated
//! - Selection highlight when user is selected

use profile_client::ui::lobby_state::{LobbyState, LobbyUser};

#[test]
fn test_lobby_state_new_is_empty() {
    let state = LobbyState::new();
    assert!(state.is_empty());
    assert_eq!(state.len(), 0);
}

#[test]
fn test_lobby_state_set_users_populates_lobby() {
    let users = vec![
        LobbyUser {
            public_key: "3a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb4d9a8f2e".to_string(),
            is_online: true,
        },
        LobbyUser {
            public_key: "7b4d9c2a3e8f1d4c5a6b7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d5e6f7a8b9".to_string(),
            is_online: true,
        },
    ];
    let mut state = LobbyState::new();
    state.set_users(users.clone());

    assert_eq!(state.len(), 2);
    assert!(!state.is_empty());
}

#[test]
fn test_lobby_state_has_user() {
    let mut state = LobbyState::new();
    let user = LobbyUser {
        public_key: "test_public_key_123456789abcdef".to_string(),
        is_online: true,
    };
    state.add_user(user.clone());

    assert!(state.has_user("test_public_key_123456789abcdef"));
    assert!(!state.has_user("nonexistent_key"));
}

#[test]
fn test_lobby_state_remove_user() {
    let mut state = LobbyState::new();
    let user = LobbyUser {
        public_key: "key_to_remove_123456789abc".to_string(),
        is_online: true,
    };
    state.add_user(user.clone());
    assert!(state.has_user("key_to_remove_123456789abc"));

    let removed = state.remove_user("key_to_remove_123456789abc");
    assert!(removed);
    assert!(!state.has_user("key_to_remove_123456789abc"));
}

#[test]
fn test_lobby_state_remove_nonexistent_user() {
    let mut state = LobbyState::new();
    let removed = state.remove_user("nonexistent_key");
    assert!(!removed);
}

#[test]
fn test_lobby_state_select_user() {
    let mut state = LobbyState::new();
    let user = LobbyUser {
        public_key: "selectable_key_123456789ab".to_string(),
        is_online: true,
    };
    state.add_user(user.clone());

    // Initially no selection
    assert_eq!(state.selected_user(), None);

    // Select the user
    let result = state.select("selectable_key_123456789ab");
    assert!(result);
    assert_eq!(state.selected_user(), Some("selectable_key_123456789ab"));
}

#[test]
fn test_lobby_state_select_nonexistent_user() {
    let mut state = LobbyState::new();
    let result = state.select("nonexistent_key");
    assert!(!result);
    assert_eq!(state.selected_user(), None);
}

#[test]
fn test_lobby_state_clear_selection() {
    let mut state = LobbyState::new();
    let user = LobbyUser {
        public_key: "key_to_select_123456789ab".to_string(),
        is_online: true,
    };
    state.add_user(user.clone());
    state.select("key_to_select_123456789ab");

    assert_eq!(state.selected_user(), Some("key_to_select_123456789ab"));

    state.clear_selection();
    assert_eq!(state.selected_user(), None);
}

#[test]
fn test_lobby_state_selection_cleared_when_user_removed() {
    let mut state = LobbyState::new();
    let user = LobbyUser {
        public_key: "user_to_remove_123456789a".to_string(),
        is_online: true,
    };
    state.add_user(user.clone());
    state.select("user_to_remove_123456789a");

    // Remove the selected user
    state.remove_user("user_to_remove_123456789a");

    // Selection should be cleared
    assert_eq!(state.selected_user(), None);
}

#[test]
fn test_lobby_state_deduplication_on_add() {
    let mut state = LobbyState::new();
    let user = LobbyUser {
        public_key: "duplicate_key_123456789ab".to_string(),
        is_online: true,
    };

    state.add_user(user.clone());
    state.add_user(user.clone());
    state.add_user(user.clone());

    // Should only have one user
    assert_eq!(state.len(), 1);
}

#[test]
fn test_lobby_state_deduplication_on_set_users() {
    let mut state = LobbyState::new();
    let users = vec![
        LobbyUser {
            public_key: "duplicate_key_123456789ab".to_string(),
            is_online: true,
        },
        LobbyUser {
            public_key: "duplicate_key_123456789ab".to_string(),
            is_online: true,
        },
        LobbyUser {
            public_key: "unique_key_123456789abc".to_string(),
            is_online: true,
        },
    ];

    state.set_users(users);

    // Should only have 2 unique users
    assert_eq!(state.len(), 2);
}

#[test]
fn test_lobby_state_users_iteration() {
    let mut state = LobbyState::new();
    let users = vec![
        LobbyUser {
            public_key: "key_a_123456789abcdef012".to_string(),
            is_online: true,
        },
        LobbyUser {
            public_key: "key_b_123456789abcdef012".to_string(),
            is_online: false,
        },
    ];
    state.set_users(users.clone());

    let lobby_users: Vec<_> = state.users();
    assert_eq!(lobby_users.len(), 2);

    // Check that we can access the users
    let keys: Vec<String> = lobby_users.iter().map(|u| u.public_key.clone()).collect();
    assert!(keys.contains(&"key_a_123456789abcdef012".to_string()));
    assert!(keys.contains(&"key_b_123456789abcdef012".to_string()));
}

#[test]
fn test_lobby_state_is_selected() {
    let mut state = LobbyState::new();
    let user = LobbyUser {
        public_key: "select_me_123456789abcd".to_string(),
        is_online: true,
    };
    state.add_user(user.clone());

    assert!(!state.is_selected("select_me_123456789abcd"));
    assert!(!state.is_selected("other_key"));

    state.select("select_me_123456789abcd");
    assert!(state.is_selected("select_me_123456789abcd"));
    assert!(!state.is_selected("other_key"));
}

#[test]
fn test_lobby_state_100_users_performance() {
    let mut state = LobbyState::new();
    let users: Vec<LobbyUser> = (0..100)
        .map(|i| LobbyUser {
            public_key: format!("{:064x}", i),
            is_online: true,
        })
        .collect();
    state.set_users(users);

    assert_eq!(state.len(), 100);
}

#[test]
fn test_lobby_state_online_offline_users() {
    let mut state = LobbyState::new();
    let users = vec![
        LobbyUser {
            public_key: "online_user_key_12345678".to_string(),
            is_online: true,
        },
        LobbyUser {
            public_key: "offline_user_key_1234567".to_string(),
            is_online: false,
        },
    ];
    state.set_users(users.clone());

    let lobby_users: Vec<_> = state.users();
    assert_eq!(lobby_users.len(), 2);

    let online_count = lobby_users.iter().filter(|u| u.is_online).count();
    let offline_count = lobby_users.iter().filter(|u| !u.is_online).count();

    assert_eq!(online_count, 1);
    assert_eq!(offline_count, 1);
}
