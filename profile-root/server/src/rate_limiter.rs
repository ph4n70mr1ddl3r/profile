//! Rate limiting for authentication attempts
//!
//! This module provides rate limiting to prevent brute force attacks
//! on authentication endpoints using a per-client counter approach with
//! automatic cleanup of expired entries.

use profile_shared::config;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Per-client rate limiter for authentication attempts
pub struct AuthRateLimiter {
    state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    client_attempts: HashMap<String, ClientState>,
    max_attempts_per_window: u32,
    window_duration: Duration,
}

/// Multiplier for cleanup threshold (entries older than this * window_duration are removed)
const CLEANUP_MULTIPLIER: u8 = 2;

struct ClientState {
    attempts: u32,
    window_start: Instant,
}

impl AuthRateLimiter {
    /// Create a new rate limiter for authentication
    ///
    /// Configuration:
    /// - 5 attempts per minute per client (identified by IP or connection ID)
    /// - Window resets after 1 minute
    /// - Expired entries are automatically cleaned up
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimitState {
                client_attempts: HashMap::new(),
                max_attempts_per_window: config::connection::rate_limit::MAX_AUTH_ATTEMPTS,
                window_duration: config::connection::rate_limit::AUTH_WINDOW_DURATION,
            })),
        }
    }

    /// Check if an authentication attempt is allowed for a specific client
    ///
    /// Returns `true` if the attempt should be allowed, `false` if rate limited
    pub async fn check_auth_allowed(&self, client_id: &str) -> bool {
        let mut state = self.state.lock().await;
        let now = Instant::now();

        // Clean up expired entries (older than CLEANUP_MULTIPLIER * window duration)
        let window_duration = state.window_duration;
        state.client_attempts.retain(|_, client_state| {
            now.duration_since(client_state.window_start)
                < window_duration * CLEANUP_MULTIPLIER as u32
        });

        // Get or create client state
        let window_duration = state.window_duration;
        let max_attempts = state.max_attempts_per_window;
        let client_state = state
            .client_attempts
            .entry(client_id.to_string())
            .or_insert(ClientState {
                attempts: 0,
                window_start: now,
            });

        // Reset window if expired
        if now.duration_since(client_state.window_start) >= window_duration {
            client_state.attempts = 0;
            client_state.window_start = now;
        }

        // Check if under limit
        if client_state.attempts < max_attempts {
            client_state.attempts += 1;
            true
        } else {
            false
        }
    }

    /// Get the number of remaining attempts for a specific client before rate limiting
    pub async fn remaining_attempts(&self, client_id: &str) -> u32 {
        let state = self.state.lock().await;
        if let Some(client_state) = state.client_attempts.get(client_id) {
            state
                .max_attempts_per_window
                .saturating_sub(client_state.attempts)
        } else {
            state.max_attempts_per_window
        }
    }

    /// Get the time until the next attempt is allowed for a specific client
    pub async fn wait_time(&self, client_id: &str) -> Duration {
        let state = self.state.lock().await;
        if let Some(client_state) = state.client_attempts.get(client_id) {
            if client_state.attempts < state.max_attempts_per_window {
                Duration::ZERO
            } else {
                let elapsed = client_state.window_start.elapsed();
                if elapsed >= state.window_duration {
                    Duration::ZERO
                } else {
                    state.window_duration - elapsed
                }
            }
        } else {
            Duration::ZERO
        }
    }
}

impl Default for AuthRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic_functionality() {
        let limiter = AuthRateLimiter::new();

        // First few attempts should be allowed for client 1
        assert!(limiter.check_auth_allowed("client1").await);
        assert!(limiter.check_auth_allowed("client1").await);
        assert!(limiter.check_auth_allowed("client1").await);

        // After 5 attempts, should be rate limited for client 1
        assert!(limiter.check_auth_allowed("client1").await);
        assert!(limiter.check_auth_allowed("client1").await);
        assert!(!limiter.check_auth_allowed("client1").await); // 6th should fail

        // But client 2 should still be allowed (per-client limiting)
        assert!(limiter.check_auth_allowed("client2").await);
    }

    #[tokio::test]
    async fn test_rate_limiter_default() {
        let limiter = AuthRateLimiter::default();
        assert!(limiter.check_auth_allowed("client_default").await);
    }

    #[tokio::test]
    async fn test_remaining_attempts() {
        let limiter = AuthRateLimiter::new();
        let remaining = limiter.remaining_attempts("client_remaining").await;
        assert!(remaining > 0);

        // Use some attempts
        limiter.check_auth_allowed("client_remaining").await;
        limiter.check_auth_allowed("client_remaining").await;

        let remaining_after = limiter.remaining_attempts("client_remaining").await;
        assert!(remaining_after < remaining);
    }

    #[tokio::test]
    async fn test_wait_time() {
        let limiter = AuthRateLimiter::new();
        // Initially should not have wait time
        let wait_time: Duration = limiter.wait_time("client_wait").await;
        assert!(wait_time.is_zero());

        // Use all attempts
        for _ in 0..5 {
            limiter.check_auth_allowed("client_wait").await;
        }

        // Should have wait time when rate limited
        let wait_time_after: Duration = limiter.wait_time("client_wait").await;
        assert!(!wait_time_after.is_zero());
    }

    #[tokio::test]
    async fn test_expired_entries_cleanup() {
        let limiter = AuthRateLimiter::new();

        // Use all attempts for client_expired
        for _ in 0..5 {
            limiter.check_auth_allowed("client_expired").await;
        }

        // Should be rate limited
        assert!(!limiter.check_auth_allowed("client_expired").await);

        // Manually manipulate state to simulate expired entry
        let mut state = limiter.state.lock().await;
        let window_duration = state.window_duration;
        if let Some(client_state) = state.client_attempts.get_mut("client_expired") {
            client_state.window_start = Instant::now() - window_duration * 3;
        }
        drop(state);

        // After cleanup, should be allowed again
        assert!(limiter.check_auth_allowed("client_expired").await);
    }
}
