//! Centralized configuration for the Profile application
//!
//! This module provides constants and configuration used across
//! all crates to ensure consistency and maintainability.

/// Lobby configuration
pub mod lobby {
    /// Maximum number of users allowed in the lobby
    /// Note: Client uses this for UI display, server enforces this limit
    pub const MAX_LOBBY_SIZE: usize = 100;

    /// Maximum number of users to display in client UI
    /// This should be less than or equal to MAX_LOBBY_SIZE
    pub const MAX_DISPLAY_USERS: usize = 100;
}

/// Message configuration
pub mod message {
    /// Maximum number of messages to retain in memory
    /// Used for both client display and server history
    pub const MAX_MESSAGE_HISTORY: usize = 50;

    /// Maximum message size in bytes
    pub const MAX_MESSAGE_SIZE: usize = 4096;
}

/// Connection configuration
pub mod connection {
    use std::time::Duration;

    /// WebSocket connection timeout
    pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

    /// Keep-alive ping interval
    pub const PING_INTERVAL: Duration = Duration::from_secs(25);

    /// Rate limiting configuration
    pub mod rate_limit {
        /// Maximum authentication attempts per time window
        pub const MAX_AUTH_ATTEMPTS: u32 = 5;

        /// Time window for rate limiting
        pub const AUTH_WINDOW_DURATION: std::time::Duration = std::time::Duration::from_secs(60);

        /// Burst allowance for rate limiting
        pub const BURST_ALLOWANCE: u32 = 3;
    }
}

/// Client UI configuration
pub mod ui {
    /// Maximum number of lobby users to display
    pub const MAX_LOBBY_USERS_DISPLAY: usize = 100;

    /// Maximum number of chat messages to display
    pub const MAX_CHAT_MESSAGES_DISPLAY: usize = 50;

    /// UI refresh rate in milliseconds
    pub const REFRESH_INTERVAL_MS: u64 = 100;
}

/// Server configuration
pub mod server {
    use std::time::Duration;

    /// Server bind address
    pub const BIND_ADDRESS: &str = "127.0.0.1:8080";

    /// Maximum concurrent connections
    pub const MAX_CONCURRENT_CONNECTIONS: usize = 1000;

    /// Graceful shutdown timeout
    pub const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configuration_consistency() {
        // Ensure display limits don't exceed actual limits
        const {
            assert!(
                ui::MAX_LOBBY_USERS_DISPLAY <= lobby::MAX_LOBBY_SIZE,
                "Display limit should not exceed actual lobby size limit"
            )
        };

        const {
            assert!(
                ui::MAX_CHAT_MESSAGES_DISPLAY <= message::MAX_MESSAGE_HISTORY,
                "Display message limit should not exceed actual message history limit"
            )
        };
    }

    #[test]
    fn test_rate_limit_configuration() {
        // Ensure rate limit configuration is reasonable
        const {
            assert!(
                connection::rate_limit::MAX_AUTH_ATTEMPTS > 0,
                "Should allow at least one auth attempt"
            )
        };

        assert!(
            connection::rate_limit::AUTH_WINDOW_DURATION.as_secs() > 0,
            "Rate limit window should be positive"
        );
    }
}
