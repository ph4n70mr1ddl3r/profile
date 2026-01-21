//! Profile server binary entry point
//!
//! This server handles WebSocket connections and user authentication
//! for Profile secure messaging application.
//!
//! TODO: Add HTTP health check endpoint at /health for monitoring

use profile_server::connection;
use profile_server::lobby::Lobby;
use profile_shared::config;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing subscriber - logs go to stderr with structured format
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("Profile Server starting...");

    // Create shared lobby instance
    let lobby = Arc::new(Lobby::new());

    // Bind to WebSocket port
    let listener = TcpListener::bind(config::server::BIND_ADDRESS).await?;
    println!(
        "✅ Server listening on ws://{}",
        config::server::BIND_ADDRESS
    );

    // Accept and handle WebSocket connections
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("🔗 New connection from: {}", addr);

                // Clone lobby for each connection
                let lobby_clone = Arc::clone(&lobby);

                // Spawn task to handle this connection
                tokio::spawn(async move {
                    if let Err(e) =
                        connection::handler::handle_connection(stream, lobby_clone).await
                    {
                        println!("❌ Connection handling error: {}", e);
                    }
                });
            }
            Err(e) => {
                println!("❌ Failed to accept connection: {}", e);
            }
        }
    }
}
