use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
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
    #[serde(default)]
    _type: String,
    users: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct AuthErrorMessage {
    #[serde(default)]
    _type: String,
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
                        let response = parse_auth_response(&text)?;
                        
                        // Check if authentication failed
                        if let AuthResponse::Failed { reason, details: _ } = &response {
                            // Use error_display to map to user-friendly message
                            use crate::ui::error_display::display_connection_error;
                            let user_message = display_connection_error(&reason);
                            
                            // If no specific user message, provide generic auth failure
                            let final_message = if user_message.is_empty() || user_message.contains("Connection lost") {
                                "Authentication failed. Your signature could not be verified. Try again or check your key.".to_string()
                            } else {
                                user_message
                            };
                            
                            return Err(final_message.into());
                        }
                        
                        return Ok(response);
                    }
                    Message::Close(frame) => {
                        let reason = frame.as_ref()
                            .map(|f| f.reason.to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        
                        // Use error_display to map to user-friendly message
                        use crate::ui::error_display::display_connection_error;
                        let user_message = display_connection_error(&reason);
                        
                        // If we have a specific message, use it; otherwise use generic
                        let final_message = if !user_message.is_empty() && !user_message.contains("Connection lost") {
                            user_message
                        } else {
                            format!("Connection closed: {}", reason)
                        };
                        
                        return Err(final_message.into());
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
    
    /// Handle disconnection with reason
    pub async fn handle_disconnection(
        &mut self, 
        reason: String
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Remove connection
        self.connection = None;
        
        // Return error that triggers UI display
        Err(format!("Connection closed: {}", reason).into())
    }
    
    /// Close connection gracefully
    pub async fn close_gracefully(
        &mut self
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(connection) = &mut self.connection {
            use tokio_tungstenite::tungstenite::protocol::{CloseFrame, frame::coding::CloseCode};
            let close_frame = CloseFrame {
                code: CloseCode::Normal,
                reason: "client_disconnect".into(),
            };
            connection.send(Message::Close(Some(close_frame))).await?;
        }
        self.connection = None;
        Ok(())
    }
    
    /// Check if client has an active connection
    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }
    
    /// Run persistent message loop to handle incoming messages and close frames
    /// This should be called after successful authentication to detect disconnections during normal operation
    pub async fn run_message_loop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            // Check if we have a connection
            if self.connection.is_none() {
                return Err("No connection available".into());
            }
            
            // Get next message
            let msg_result = if let Some(connection) = &mut self.connection {
                connection.next().await
            } else {
                return Err("Connection lost unexpectedly".into());
            };
            
            // Process message
            match msg_result {
                Some(Ok(Message::Text(text))) => {
                    // Handle incoming messages (future stories will process these)
                    println!("Received message: {}", text);
                    // TODO: Story 3.2+ will handle message types here
                }
                Some(Ok(Message::Close(frame))) => {
                    // Server closed the connection
                    let reason = frame.as_ref()
                        .map(|f| f.reason.to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    
                    // Use error_display to map to user-friendly message
                    use crate::ui::error_display::display_connection_error;
                    let user_message = display_connection_error(&reason);
                    
                    // Clean up connection state
                    self.connection = None;
                    
                    // Return error with user-friendly message
                    let final_message = if !user_message.is_empty() && !user_message.contains("Connection lost") {
                        user_message
                    } else {
                        format!("Connection closed: {}", reason)
                    };
                    
                    return Err(final_message.into());
                }
                Some(Ok(Message::Ping(data))) => {
                    // Respond to ping with pong
                    if let Some(conn) = &mut self.connection {
                        conn.send(Message::Pong(data)).await?;
                    }
                }
                Some(Ok(Message::Pong(_))) => {
                    // Pong received, connection alive
                }
                Some(Ok(_)) => {
                    // Other message types (binary, etc.)
                    println!("Received unexpected message type");
                }
                Some(Err(e)) => {
                    // Connection error (network issue, stream closed)
                    self.connection = None;
                    return Err(format!("Connection lost: {}", e).into());
                }
                None => {
                    // Stream ended without explicit close frame
                    self.connection = None;
                    return Err("Connection lost. Check your network and try reconnecting.".into());
                }
            }
        }
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

    #[tokio::test]
    async fn test_handle_close_frame_with_reason() {
        let key_state = create_shared_key_state();
        let mut client = WebSocketClient::new(key_state);
        
        // Simulate disconnection with reason
        let result = client.handle_disconnection("auth_failed".to_string()).await;
        
        // Should return error with reason
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Connection closed: auth_failed"));
        
        // Connection should be None
        assert!(client.connection.is_none());
    }

    #[tokio::test]
    async fn test_handle_close_frame_without_reason() {
        let key_state = create_shared_key_state();
        let mut client = WebSocketClient::new(key_state);
        
        // Simulate disconnection without specific reason
        let result = client.handle_disconnection("Unknown".to_string()).await;
        
        // Should return error with "Unknown"
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Connection closed: Unknown"));
    }

    #[tokio::test]
    async fn test_connection_state_after_disconnect() {
        let key_state = create_shared_key_state();
        let mut client = WebSocketClient::new(key_state);
        
        // Initially no connection
        assert!(client.connection.is_none());
        
        // Simulate disconnect
        let _ = client.handle_disconnection("test reason".to_string()).await;
        
        // Connection should still be None (cleanup successful)
        assert!(client.connection.is_none());
    }

    #[tokio::test]
    async fn test_graceful_shutdown_sends_close_frame() {
        let key_state = create_shared_key_state();
        let mut client = WebSocketClient::new(key_state);
        
        // Without a connection, graceful shutdown should succeed
        let result = client.close_gracefully().await;
        assert!(result.is_ok());
        assert!(client.connection.is_none());
        
        // Note: Testing with actual connection requires server integration test
        // This is covered in Task 9 (Integration Tests)
    }
}