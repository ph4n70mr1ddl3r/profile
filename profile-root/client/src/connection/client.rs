use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use crate::state::session::SharedKeyState;

/// Authentication response from server
#[derive(Debug, Clone, PartialEq)]
pub enum AuthResponse {
    /// Successful authentication with list of online users
    Success { users: Vec<String> },
    /// Authentication failed with reason and details
    Failed { reason: String, details: String },
}

/// Internal message types for parsing server responses
#[derive(Debug, Deserialize)]
struct ServerMessage {
    r#type: String,
}

#[derive(Debug, Deserialize)]
struct AuthSuccessMessage {
    r#type: String,
    users: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AuthErrorMessage {
    r#type: String,
    reason: String,
    details: String,
}

/// Parse authentication response from server
fn parse_auth_response(text: &str) -> Result<AuthResponse, Box<dyn std::error::Error + Send + Sync>> {
    // First, determine message type
    let msg: ServerMessage = serde_json::from_str(text)?;
    
    match msg.r#type.as_str() {
        "auth_success" => {
            let success: AuthSuccessMessage = serde_json::from_str(text)?;
            Ok(AuthResponse::Success { 
                users: success.users 
            })
        }
        "error" => {
            let error: AuthErrorMessage = serde_json::from_str(text)?;
            Ok(AuthResponse::Failed { 
                reason: error.reason, 
                details: error.details 
            })
        }
        other => Err(format!("Unknown message type: {}", other).into())
    }
}

/// WebSocket client for connecting to the profile server
#[derive(Debug)]
pub struct WebSocketClient {
    connection: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    key_state: SharedKeyState,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(key_state: SharedKeyState) -> Self {
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
    pub async fn authenticate(&mut self) -> Result<AuthResponse, Box<dyn std::error::Error + Send + Sync>> {
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
        
        // Send auth message and wait for response
        if let Some(connection) = &mut self.connection {
            // Send auth message
            connection.send(Message::Text(auth_json)).await?;
            
            // Wait for server response
            if let Some(msg) = connection.next().await {
                match msg? {
                    Message::Text(text) => {
                        return parse_auth_response(&text);
                    }
                    Message::Close(_) => {
                        return Err("Server closed connection during authentication".into());
                    }
                    _ => {
                        return Err("Unexpected message type from server".into());
                    }
                }
            } else {
                return Err("No response from server".into());
            }
        }
        
        Err("No connection available".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::session::create_shared_key_state;

    // Note: Real connection test requires running server
    // Integration test would be: spawn server, connect client, verify handshake
    // This is covered by Task 11 (E2E test) which is deferred to future story

    #[test]
    fn test_parse_auth_success_response() {
        let json = r#"{"type":"auth_success","users":["abc123","def456"]}"#;
        let result = parse_auth_response(json).unwrap();
        
        match result {
            AuthResponse::Success { users } => {
                assert_eq!(users.len(), 2);
                assert_eq!(users[0], "abc123");
                assert_eq!(users[1], "def456");
            }
            _ => panic!("Expected Success response"),
        }
    }

    #[test]
    fn test_parse_auth_error_response() {
        let json = r#"{"type":"error","reason":"auth_failed","details":"Signature did not verify"}"#;
        let result = parse_auth_response(json).unwrap();
        
        match result {
            AuthResponse::Failed { reason, details } => {
                assert_eq!(reason, "auth_failed");
                assert_eq!(details, "Signature did not verify");
            }
            _ => panic!("Expected Failed response"),
        }
    }

    #[test]
    fn test_parse_unknown_message_type() {
        let json = r#"{"type":"unknown_type"}"#;
        let result = parse_auth_response(json);
        
        assert!(result.is_err(), "Should fail for unknown message type");
    }

    #[test]
    fn test_parse_invalid_json() {
        let json = "not valid json";
        let result = parse_auth_response(json);
        
        assert!(result.is_err(), "Should fail for invalid JSON");
    }
}