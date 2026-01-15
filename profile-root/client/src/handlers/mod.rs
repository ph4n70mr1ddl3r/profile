//! UI event handlers for key generation and management

pub mod compose;
pub mod composer;
pub mod edge_cases;
pub mod key_generation;
pub mod key_import;
pub mod lobby;
pub mod offline;
pub mod verify;

pub use crate::state::composer::{
    clear_all_ephemeral_data, format_connection_notification, ConnectionState,
};
pub use compose::{compose_and_send_message, compose_message_draft, ComposeError};
pub use composer::{
    create_composer_with_state, get_send_result_message, handle_composer_can_send,
    handle_composer_clear, handle_composer_get_draft, handle_composer_set_send_callback,
    handle_composer_set_status_callback, handle_composer_text_change, handle_send_message,
};
pub use key_generation::handle_generate_new_key;
pub use key_import::handle_import_key;
pub use lobby::{
    clear_lobby_selection, get_lobby_selected_user, get_lobby_user_count,
    handle_lobby_navigate_down, handle_lobby_navigate_up, handle_lobby_state_update,
    handle_lobby_user_joined, handle_lobby_user_left, handle_lobby_user_select,
};
pub use offline::{
    add_undelivered_message, clear_undelivered_for_recipient, create_offline_notification,
    create_shared_undelivered_messages, create_undelivered_display_message, dismiss_notification,
    format_notification_message, get_undelivered_for_recipient, parse_offline_notification,
    OfflineNotification, SharedUndeliveredMessages, UndeliveredMessage,
};
pub use verify::{
    create_invalid_signature_notification, format_public_key, verify_chat_message, verify_message,
    VerificationResult,
};
