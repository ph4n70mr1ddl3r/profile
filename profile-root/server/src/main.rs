//! Profile server binary entry point
//!
//! This server handles WebSocket connections and user authentication
//! for Profile secure messaging application.
//!
//! TODO: Add HTTP health check endpoint at /health for monitoring

use profile_server::connection;
use profile_server::lobby::Lobby;
use profile_server::rate_limiter::AuthRateLimiter;
use profile_shared::config;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Profile Server starting...");

    let lobby = Arc::new(Lobby::new());
    let rate_limiter = Arc::new(AuthRateLimiter::new());

    let listener = TcpListener::bind(config::server::BIND_ADDRESS).await?;
    tracing::info!(
        bind_address = config::server::BIND_ADDRESS,
        "Server listening"
    );

    let mut accept_errors = 0u32;
    const MAX_CONSECUTIVE_ACCEPT_ERRORS: u32 = 10;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Shutdown signal received, exiting gracefully");
                break;
            }
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        accept_errors = 0;
                        tracing::info!(client_ip = %addr, "New connection");

                        let lobby_clone = Arc::clone(&lobby);
                        let rate_limiter_clone = Arc::clone(&rate_limiter);

                        tokio::spawn(async move {
                            if let Err(e) = connection::handler::handle_connection(
                                stream,
                                lobby_clone,
                                rate_limiter_clone,
                            )
                            .await
                            {
                                tracing::error!(error = %e, "Connection handling error");
                            }
                        });
                    }
                    Err(e) => {
                        accept_errors += 1;
                        tracing::error!(
                            error = %e,
                            consecutive_errors = accept_errors,
                            "Failed to accept connection"
                        );
                        if accept_errors >= MAX_CONSECUTIVE_ACCEPT_ERRORS {
                            tracing::error!(
                                "Too many consecutive accept errors, shutting down"
                            );
                            return Err(format!(
                                "Consecutive accept errors ({}) exceeded threshold",
                                accept_errors
                            )
                            .into());
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }

    Ok(())
}
