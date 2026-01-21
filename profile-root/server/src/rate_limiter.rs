//! Rate limiting for authentication attempts
//!
//! This module provides rate limiting to prevent brute force attacks
//! on authentication endpoints using a simple in-memory counter approach.

use profile_shared::config;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Simple in-memory rate limiter for authentication attempts
pub struct AuthRateLimiter {
    state: Arc<Mutex<RateLimitState>>,
}

struct RateLimitState {
    attempts: u32,
    window_start: Instant,
    max_attempts_per_window: u32,
    window_duration: Duration,
}

impl AuthRateLimiter {
    /// Create a new rate limiter for authentication
    ///
    /// Configuration:
    /// - 5 attempts per minute
    /// - Window resets after 1 minute
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RateLimitState {
                attempts: 0,
                window_start: Instant::now(),
                max_attempts_per_window: config::connection::rate_limit::MAX_AUTH_ATTEMPTS,
                window_duration: config::connection::rate_limit::AUTH_WINDOW_DURATION,
            })),
        }
    }

    /// Check if an authentication attempt is allowed
    ///
    /// Returns `true` if the attempt should be allowed, `false` if rate limited
    pub async fn check_auth_allowed(&self) -> bool {
        let mut state = self.state.lock().await;
        let now = Instant::now();

        // Reset window if expired
        if now.duration_since(state.window_start) >= state.window_duration {
            state.attempts = 0;
            state.window_start = now;
        }

        // Check if under limit
        if state.attempts < state.max_attempts_per_window {
            state.attempts += 1;
            true
        } else {
            false
        }
    }

    /// Get the number of remaining attempts before rate limiting
    pub async fn remaining_attempts(&self) -> u32 {
        let state = self.state.lock().await;
        state.max_attempts_per_window.saturating_sub(state.attempts)
    }

    /// Get the time until the next attempt is allowed
    pub async fn wait_time(&self) -> Duration {
        let state = self.state.lock().await;
        if state.attempts < state.max_attempts_per_window {
            Duration::ZERO
        } else {
            let elapsed = state.window_start.elapsed();
            if elapsed >= state.window_duration {
                Duration::ZERO
            } else {
                state.window_duration - elapsed
            }
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

        // First few attempts should be allowed
        assert!(limiter.check_auth_allowed().await);
        assert!(limiter.check_auth_allowed().await);
        assert!(limiter.check_auth_allowed().await);

        // After 5 attempts, should be rate limited
        assert!(limiter.check_auth_allowed().await);
        assert!(limiter.check_auth_allowed().await);
        assert!(!limiter.check_auth_allowed().await); // 6th should fail
    }

    #[tokio::test]
    async fn test_rate_limiter_default() {
        let limiter = AuthRateLimiter::default();
        assert!(limiter.check_auth_allowed().await);
    }

    #[tokio::test]
    async fn test_remaining_attempts() {
        let limiter = AuthRateLimiter::new();
        let remaining = limiter.remaining_attempts().await;
        assert!(remaining > 0);

        // Use some attempts
        limiter.check_auth_allowed().await;
        limiter.check_auth_allowed().await;

        let remaining_after = limiter.remaining_attempts().await;
        assert!(remaining_after < remaining);
    }

    #[tokio::test]
    async fn test_wait_time() {
        let limiter = AuthRateLimiter::new();
        // Initially should not have wait time
        let wait_time: Duration = limiter.wait_time().await;
        assert!(wait_time.is_zero());

        // Use all attempts
        for _ in 0..5 {
            limiter.check_auth_allowed().await;
        }

        // Should have wait time when rate limited
        let wait_time_after: Duration = limiter.wait_time().await;
        assert!(!wait_time_after.is_zero());
    }
}
