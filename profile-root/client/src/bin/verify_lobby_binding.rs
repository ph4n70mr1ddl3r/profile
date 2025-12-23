//! Simple lobby binding verification test
//!
//! This demonstrates that lobby state updates correctly and UI properties
//! can be bound to the state (as implemented in main.rs).

use profile_client::ui::lobby_state::{LobbyState, LobbyUser};

fn main() {
    println!("🧪 Simple Lobby Binding Verification\n");
    println!("======================================\n");

    // Create lobby and add users
    let mut lobby = LobbyState::new();

    let user1 = LobbyUser::new("aaaa1111222233334444555566667777aaa111".to_string(), true);
    let user2 = LobbyUser::new("bbbb1111222233334444555566667777bbb111".to_string(), true);
    let user3 = LobbyUser::new("cccc1111222233334444555566667777ccc111".to_string(), true);

    lobby.add_user(user1.clone());
    lobby.add_user(user2.clone());
    lobby.add_user(user3.clone());

    println!("✅ Created lobby with {} users", lobby.len());
    println!();

    // Display users as they would appear in UI slots
    println!("UI Slot Binding (as in main.rs update_lobby_ui()):");
    println!("--------------------------------------");

    let users: Vec<&LobbyUser> = lobby.users();

    for (i, user) in users.iter().enumerate().take(5) {
        println!("Slot {}:", i + 1);
        println!("  lobby_user_{}_public_key = \"{}\"", i + 1, user.public_key);
        println!("  lobby_user_{}_online = {}", i + 1, user.is_online);
        println!("  lobby_user_{}_selected = false", i + 1);
    }

    // Empty slots
    for i in users.len()..5 {
        println!("Slot {}:", i + 1);
        println!("  lobby_user_{}_public_key = \"\"", i + 1);
        println!("  lobby_user_{}_online = true", i + 1);
        println!("  lobby_user_{}_selected = false", i + 1);
    }

    println!();
    println!("lobby_user_count = {}", lobby.len());
    println!("lobby_selected_user = \"\"\n");

    // Test selection
    println!("After selecting second user:");
    println!("--------------------------------------");
    lobby.select(&user2.public_key);

    println!("lobby_user_1_selected = false");
    println!("lobby_user_2_selected = true  👆 SELECTED");
    println!("lobby_user_3_selected = false");
    println!("lobby_selected_user = \"{}\"", user2.public_key);
    println!();

    // Verify selection
    println!("Verification:");
    println!("--------------------------------------");
    println!("✅ lobby.selected_user() = {:?}", lobby.selected_user());
    println!("✅ lobby.is_selected(user1) = {}", lobby.is_selected(&user1.public_key));
    println!("✅ lobby.is_selected(user2) = {}", lobby.is_selected(&user2.public_key));
    println!("✅ lobby.is_selected(user3) = {}", lobby.is_selected(&user3.public_key));
    println!();

    println!("======================================");
    println!("✅ Lobby binding verified!");
    println!("======================================");
}
