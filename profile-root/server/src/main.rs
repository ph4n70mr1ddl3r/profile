//! Profile server implementation
//!
//! This server handles WebSocket connections and user authentication
//! for the Profile secure messaging application.

pub mod protocol;
pub mod lobby;
pub mod auth;
pub mod connection;

use std::sync::Arc;
use tokio::net::TcpListener;
use crate::lobby::Lobby;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Profile Server starting...");
    
    // Create shared lobby instance
    let lobby = Arc::new(Lobby::new());
    
    // Bind to WebSocket port
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("✅ Server listening on ws://127.0.0.1:8080");
    
    // Accept and handle WebSocket connections
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                println!("🔗 New connection from: {}", addr);
                
                // Clone lobby for each connection
                let lobby_clone = Arc::clone(&lobby);
                
                // Spawn task to handle this connection
                tokio::spawn(async move {
                    if let Err(e) = connection::handler::handle_connection(stream, lobby_clone).await {
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
