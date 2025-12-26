//! UI event handlers for key generation and management

pub mod key_generation;
pub mod key_import;
pub mod lobby;
pub mod composer;
pub mod verify;

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
pub use composer::{
    handle_send_message,
    handle_composer_text_change,
    handle_composer_clear,
    handle_composer_can_send,
    handle_composer_get_draft,
    handle_composer_set_status_callback,
    handle_composer_set_send_callback,
    create_composer_with_state,
    get_send_result_message,
};
pub use verify::{
    verify_message,
    verify_chat_message,
    VerificationResult,
    create_invalid_signature_notification,
    format_public_key,
};