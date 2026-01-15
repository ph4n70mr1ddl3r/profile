//! Authentication handler module

pub mod handler;

pub use handler::{
    create_error_message, create_success_message, handle_authentication, AuthResult,
};
