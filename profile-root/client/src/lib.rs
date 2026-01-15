//! Profile client library - exposes modules for integration testing
//!
//! This library crate is separate from the binary (main.rs) to enable
//! integration tests to import internal modules.

pub mod connection;
pub mod handlers;
pub mod state;
pub mod ui;
