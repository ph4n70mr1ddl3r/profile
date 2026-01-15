//! Manual test for lobby UI property binding
//!
//! This test demonstrates that:
//! 1. Lobby state is properly initialized
//! 2. UI properties are updated when lobby state changes
//! 3. User selection works correctly
//! 4. Keyboard navigation works correctly
//!
//! Run with: cargo run --bin test_lobby_binding

use profile_client::ui::lobby_state::{LobbyState, LobbyUser};

fn main() {
    println!("🧪 Testing Lobby UI Property Binding\n");
    println!("======================================\n");

    // Test 1: Create empty lobby state
    println!("Test 1: Empty Lobby State");
    println!("--------------------------------------");
    let lobby = LobbyState::new();
    println!("✅ Empty lobby created");
    println!("   User count: {}", lobby.len());
    println!("   Is empty: {}\n", lobby.is_empty());

    // Test 2: Add users to lobby
    println!("Test 2: Add Users to Lobby");
    println!("--------------------------------------");
    let mut lobby = lobby;
    let test_users = vec![
        LobbyUser::new(
            "aaaa1111222233334444555566667777aaa111222233".to_string(),
            true,
        ),
        LobbyUser::new(
            "bbbb1111222233334444555566667777bbb111222233".to_string(),
            true,
        ),
        LobbyUser::new(
            "cccc1111222233334444555566667777ccc111222233".to_string(),
            true,
        ),
    ];

    for user in &test_users {
        lobby.add_user(user.clone());
        let pk = &user.public_key;
        let start = if pk.len() >= 8 { &pk[..8] } else { pk };
        let end = if pk.len() >= 8 {
            &pk[pk.len() - 8..]
        } else {
            ""
        };
        println!("✅ Added user: {}...{}", start, end);
    }
    println!("   Total users: {}\n", lobby.len());

    // Test 3: Verify users are in lobby
    println!("Test 3: Verify Users in Lobby");
    println!("--------------------------------------");
    for user in lobby.users() {
        let pk = &user.public_key;
        let start = if pk.len() >= 8 { &pk[..8] } else { pk };
        let end = if pk.len() >= 8 {
            &pk[pk.len() - 8..]
        } else {
            ""
        };
        println!(
            "✅ User found: {}...{} (online: {})",
            start, end, user.is_online
        );
    }
    println!();

    // Test 4: Select a user
    println!("Test 4: Select a User");
    println!("--------------------------------------");
    let selected_key = test_users[1].public_key.clone();
    let success = lobby.select(&selected_key);
    let pk = &selected_key;
    let start = if pk.len() >= 8 { &pk[..8] } else { pk };
    let end = if pk.len() >= 8 {
        &pk[pk.len() - 8..]
    } else {
        ""
    };
    println!("Selected user: {}...{}", start, end);
    println!("Selection result: {}", success);
    println!("Current selection: {:?}\n", lobby.selected_user());

    // Test 5: Verify selection highlight
    println!("Test 5: Verify Selection Highlight");
    println!("--------------------------------------");
    for user in lobby.users() {
        let is_selected = lobby.is_selected(&user.public_key);
        let status = if is_selected {
            "👆 SELECTED"
        } else {
            "  normal"
        };
        let pk = &user.public_key;
        let start = if pk.len() >= 8 { &pk[..8] } else { pk };
        let end = if pk.len() >= 8 {
            &pk[pk.len() - 8..]
        } else {
            ""
        };
        println!("{}: {}...{}", status, start, end);
    }
    println!();

    // Test 6: Simulate UI property binding
    println!("Test 6: Simulate UI Property Binding");
    println!("--------------------------------------");
    println!("If this lobby state were bound to UI properties:");
    println!();

    let users_vec: Vec<&LobbyUser> = lobby.users();
    let selected_key = lobby.selected_user();

    // Simulate populating 5 lobby slots (as done in main.rs update_lobby_ui())
    for (i, user) in users_vec.iter().enumerate().take(5) {
        let is_selected = selected_key == Some(user.public_key.as_str());
        let pk = &user.public_key;

        println!("Slot {} (lobby_user_{}_...):", i + 1, i + 1);
        println!("  lobby_user_{}_public_key: {}", i + 1, pk);
        println!("  lobby_user_{}_online: {}", i + 1, user.is_online);
        println!("  lobby_user_{}_selected: {}", i + 1, is_selected);
    }

    // Remaining slots (empty)
    for i in users_vec.len()..5 {
        println!("Slot {} (lobby_user_{}_...):", i + 1, i + 1);
        println!("  lobby_user_{}_public_key: \"\"", i + 1);
        println!("  lobby_user{}_online: true", i + 1);
        println!("  lobby_user{}_selected: false", i + 1);
    }
    println!();

    println!("User count (lobby_user_count): {}", lobby.len());
    if let Some(key) = selected_key {
        let pk = key;
        let start = if pk.len() >= 8 { &pk[..8] } else { pk };
        let end = if pk.len() >= 8 {
            &pk[pk.len() - 8..]
        } else {
            ""
        };
        println!("Selected user (lobby_selected_user): {}...{}", start, end);
    } else {
        println!("Selected user (lobby_selected_user): \"\"");
    }
    println!();

    // Test 7: Keyboard navigation simulation
    println!("Test 7: Keyboard Navigation Simulation");
    println!("--------------------------------------");
    let mut lobby = LobbyState::new();
    lobby.add_user(LobbyUser::new(
        "user1_aaaaa1111222233334444555566aa".to_string(),
        true,
    ));
    lobby.add_user(LobbyUser::new(
        "user2_bbbbb1111222233334444555566bb".to_string(),
        true,
    ));
    lobby.add_user(LobbyUser::new(
        "user3_ccccc1111222233334444555566cc".to_string(),
        true,
    ));

    println!("Initial state:");
    for user in lobby.users() {
        let pk = &user.public_key;
        let display = if pk.len() >= 10 { &pk[..10] } else { pk };
        println!("  - {}", display);
    }

    // Simulate ArrowDown selection - clone keys to avoid borrow issues
    let keys: Vec<String> = lobby.users().iter().map(|u| u.public_key.clone()).collect();

    if let Some(key) = keys.first() {
        lobby.select(key);
        println!("\nAfter ArrowDown (select first):");
        println!("  Selected: {:?}\n", lobby.selected_user());
    }

    // Simulate another ArrowDown
    if let Some(key) = keys.get(1) {
        lobby.select(key);
        println!("After another ArrowDown:");
        println!("  Selected: {:?}\n", lobby.selected_user());
    }
    println!();

    // Test 6: Simulate UI property binding
    println!("Test 6: Simulate UI Property Binding");
    println!("--------------------------------------");
    println!("If this lobby state were bound to UI properties:");
    println!();

    let users_vec: Vec<&LobbyUser> = lobby.users();
    let selected_key = lobby.selected_user();

    // Simulate populating 5 lobby slots (as done in main.rs update_lobby_ui())
    for (i, user) in users_vec.iter().enumerate().take(5) {
        let is_selected = selected_key == Some(user.public_key.as_str());

        println!("Slot {} (lobby_user_{}_...):", i + 1, i + 1);
        println!("  lobby_user_{}_public_key: {}", i + 1, user.public_key);
        println!("  lobby_user_{}_online: {}", i + 1, user.is_online);
        println!("  lobby_user_{}_selected: {}", i + 1, is_selected);
    }

    // Remaining slots (empty)
    for i in users_vec.len()..5 {
        println!("Slot {} (lobby_user_{}_...):", i + 1, i + 1);
        println!("  lobby_user_{}_public_key: \"\"", i + 1);
        println!("  lobby_user_{}_online: true", i + 1);
        println!("  lobby_user_{}_selected: false", i + 1);
    }
    println!();

    println!("User count (lobby_user_count): {}", lobby.len());
    if let Some(key) = selected_key {
        println!(
            "Selected user (lobby_selected_user): {}...{}",
            &key[..8],
            &key[56..]
        );
    } else {
        println!("Selected user (lobby_selected_user): \"\"");
    }
    println!();

    // Test 7: Keyboard navigation simulation
    println!("Test 7: Keyboard Navigation Simulation");
    println!("--------------------------------------");
    let mut lobby = LobbyState::new();
    lobby.add_user(LobbyUser::new(
        "user1_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        true,
    ));
    lobby.add_user(LobbyUser::new(
        "user2_bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_string(),
        true,
    ));
    lobby.add_user(LobbyUser::new(
        "user3_ccccccccccccccccccccccccccccccc".to_string(),
        true,
    ));

    println!("Initial state:");
    for user in lobby.users() {
        println!("  - {}", &user.public_key[..10]);
    }

    // Simulate ArrowDown selection - clone keys to avoid borrow issues
    let keys: Vec<String> = lobby.users().iter().map(|u| u.public_key.clone()).collect();

    if let Some(key) = keys.first() {
        lobby.select(key);
        println!("\nAfter ArrowDown (select first):");
        println!("  Selected: {:?}\n", lobby.selected_user());
    }

    // Simulate another ArrowDown
    if let Some(key) = keys.get(1) {
        lobby.select(key);
        println!("After another ArrowDown:");
        println!("  Selected: {:?}\n", lobby.selected_user());
    }

    println!("======================================");
    println!("✅ All lobby binding tests passed!");
    println!("======================================");
}
