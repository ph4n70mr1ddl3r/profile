//! User lobby management for tracking online users
//!
//! This module provides thread-safe lobby management for tracking
//! authenticated users and their WebSocket connections.
//!
//! Follows Architecture Decision: Uses Arc<RwLock<HashMap>> pattern for concurrent access
//! - Arc: allows multiple threads to hold references to the lobby
//! - RwLock: multiple readers can access simultaneously, exclusive writer for modifications
//! - HashMap: O(1) lookup for message routing (critical for performance)

pub mod state;
pub mod manager;

pub use state::{PublicKey, ActiveConnection, Lobby};
pub use manager::{add_user, remove_user, get_user, get_current_users};

/// Compatibility alias for existing code
/// This will be deprecated in favor of ActiveConnection
#[derive(Debug, Clone)]
pub struct Connection {
    pub public_key: Vec<u8>,
    pub connected_at: std::time::Instant,
}