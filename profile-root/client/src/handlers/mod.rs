//! UI event handlers for key generation and management

pub mod key_generation;
pub mod key_import;
pub mod lobby;

pub use key_generation::handle_generate_new_key;
pub use key_import::handle_import_key;
pub use lobby::{
    handle_lobby_user_select,
    handle_lobby_navigate_up,
    handle_lobby_navigate_down,
    handle_lobby_user_joined,
    handle_lobby_user_left,
    handle_lobby_state_update,
    clear_lobby_selection,
    get_lobby_user_count,
    get_lobby_selected_user,
};

