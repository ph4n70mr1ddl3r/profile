//! User lobby management for tracking online users
//!
//! This module provides thread-safe lobby management for tracking
//! authenticated users and their WebSocket connections.
//!
//! Follows Architecture Decision: Uses Arc<RwLock<HashMap>> pattern for concurrent access
//! - Arc: allows multiple threads to hold references to the lobby
//! - RwLock: multiple readers can access simultaneously, exclusive writer for modifications
//! - HashMap: O(1) lookup for message routing (critical for performance)

pub mod manager;
pub mod state;

pub use manager::{add_user, get_current_users, get_user, remove_user};
pub use state::{ActiveConnection, Lobby, PublicKey};
