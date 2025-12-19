//! Authentication handler module

pub mod handler;

pub use handler::{handle_authentication, create_success_message, create_error_message, AuthResult};