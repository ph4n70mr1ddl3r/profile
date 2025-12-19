use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use crate::state::session::SharedKeyState;

/// WebSocket client for connecting to the profile server
#[derive(Debug)]
pub struct WebSocketClient {
    connection: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    key_state: Arc<SharedKeyState>,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(key_state: Arc<SharedKeyState>) -> Self {
        Self {
            connection: None,
            key_state,
        }
    }
    
    /// Connect to the profile server
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = "ws://127.0.0.1:8080";
        
        let (ws_stream, _) = connect_async(url).await?;
        self.connection = Some(ws_stream);
        
        Ok(())
    }
    
    /// Perform authentication handshake
    pub async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get keys from shared state
        let (public_key, private_key) = {
            let key_state = self.key_state.lock().await;
            let public_key = key_state.public_key()
                .ok_or("No public key available. Generate or import a key first.")?
                .clone();
            let private_key = key_state.private_key()
                .ok_or("No private key available. Generate or import a key first.")?
                .clone();
            (public_key, private_key)
        };
        
        // Create authentication message using auth.rs module
        let auth_msg = super::auth::ClientAuthMessage::new(public_key, private_key)?;
        let auth_json = auth_msg.to_json()?;
        
        // Send auth message
        if let Some(connection) = &mut self.connection {
            // Send auth message
            connection.send(Message::Text(auth_json)).await?;
            
            // For now, just return success (would parse response in full implementation)
            println!("✅ Authentication message sent successfully");
            return Ok(());
        }
        
        Err("No connection available".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::session::create_shared_key_state;

    #[tokio::test]
    async fn test_websocket_client_connection() {
        // This test should fail initially - we're implementing the RED phase
        
        let key_state = create_shared_key_state();
        let mut client = WebSocketClient::new(key_state);
        
        // This should fail because we haven't implemented the connection logic yet
        let result = client.connect().await;
        
        // In RED phase, this should fail
        assert!(result.is_err(), "WebSocket connection should fail until implemented");
    }

    #[tokio::test]
    async fn test_websocket_client_authentication() {
        // Test authentication flow
        
        let result = test_authentication_flow().await;
        
        // This should pass once we have the minimal implementation
        assert!(result.is_ok(), "Authentication flow should work: {:?}", result.err());
    }

    async fn test_authentication_flow() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Test the authentication flow - this will fail until implemented
        
        let key_state = create_shared_key_state();
        let mut client = WebSocketClient::new(key_state);
        
        // Try to connect (will fail in RED phase)
        let _ = client.connect().await?;
        
        // Try to authenticate (will fail in RED phase)
        let _ = client.authenticate().await?;
        
        Ok(())
    }
}