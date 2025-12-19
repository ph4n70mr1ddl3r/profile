//! UI event handlers for key generation and management

pub mod key_generation;
pub mod key_import;

pub use key_generation::handle_generate_new_key;
pub use key_import::handle_import_key;

