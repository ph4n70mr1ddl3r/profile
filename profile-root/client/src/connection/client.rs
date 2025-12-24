use serde::Deserialize;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};
use crate::state::session::SharedKeyState;
use crate::ui::lobby_state::{LobbyState, LobbyUser};
use std::rc::Rc;
use std::cell::RefCell;

/// Authentication response from server
#[derive(Debug, Clone, PartialEq)]
pub enum AuthResponse {
    /// Successful authentication with list of online users
    Success { users: Vec<String> },
    /// Authentication failed with reason and details
    Failed { reason: String, details: String },
}

/// Callback handler for lobby events
#[derive(Clone)]
pub struct LobbyEventHandler {
    /// Called when initial lobby state is received
    pub on_lobby_received: Rc<RefCell<dyn Fn(LobbyState)>>,
    /// Called when a user joins the lobby
    pub on_user_joined: Rc<RefCell<dyn Fn(LobbyUser)>>,
    /// Called when a user leaves the lobby
    pub on_user_left: Rc<RefCell<dyn Fn(String)>>,
}

impl LobbyEventHandler {
    /// Create a new lobby event handler with no-op callbacks
    #[inline]
    pub fn new() -> Self {
        Self {
            on_lobby_received: Rc::new(RefCell::new(|_: LobbyState| {})),
            on_user_joined: Rc::new(RefCell::new(|_: LobbyUser| {})),
            on_user_left: Rc::new(RefCell::new(|_: String| {})),
        }
    }

    /// Create with custom callbacks
    #[inline]
    pub fn with_callbacks(
        on_lobby_received: impl Fn(LobbyState) + 'static,
        on_user_joined: impl Fn(LobbyUser) + 'static,
        on_user_left: impl Fn(String) + 'static,
    ) -> Self {
        Self {
            on_lobby_received: Rc::new(RefCell::new(on_lobby_received)),
            on_user_joined: Rc::new(RefCell::new(on_user_joined)),
            on_user_left: Rc::new(RefCell::new(on_user_left)),
        }
    }

    /// Emit lobby received event
    #[inline]
    pub fn lobby_received(&self, state: &LobbyState) {
        (self.on_lobby_received.borrow())(state.clone());
    }

    /// Emit user joined event
    #[inline]
    pub fn user_joined(&self, user: &LobbyUser) {
        (self.on_user_joined.borrow())(user.clone());
    }

    /// Emit user left event
    #[inline]
    pub fn user_left(&self, public_key: &str) {
        (self.on_user_left.borrow())(public_key.to_string());
    }
}

impl Default for LobbyEventHandler {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Response from the lobby message parser
#[derive(Debug, Clone, PartialEq)]
pub enum LobbyResponse {
    /// Initial lobby state with all users
    LobbyState { users: Vec<LobbyUser> },
    /// One or more users joined the lobby
    UsersJoined { public_keys: Vec<String> },
    /// One or more users left the lobby
    UsersLeft { public_keys: Vec<String> },
    /// Unknown or unhandled message type
    Ignored,
}

/// Parse a lobby message from the server
pub fn parse_lobby_message(text: &str) -> Result<LobbyResponse, Box<dyn std::error::Error + Send + Sync>> {
    // First, determine message type
    let msg: ServerMessage = serde_json::from_str(text)?;

    match msg.r#type.as_str() {
        "lobby" => {
            // Parse lobby message with full user list
            let lobby_msg: profile_shared::protocol::LobbyMessage = serde_json::from_str(text)?;

            // Convert to LobbyUser structs
            let users: Vec<LobbyUser> = lobby_msg
                .users
                .into_iter()
                .map(|u| LobbyUser {
                    public_key: u.public_key,
                    is_online: u.status == "online",
                })
                .collect();

            Ok(LobbyResponse::LobbyState { users })
        }
        "lobby_update" => {
            // Parse lobby update (delta)
            let update: profile_shared::protocol::LobbyUpdateMessage = serde_json::from_str(text)?;

            // Handle all joined users (FIX: was only returning first)
            if !update.joined.is_empty() {
                let joined_keys: Vec<String> = update.joined
                    .into_iter()
                    .map(|u| u.public_key)
                    .collect();
                return Ok(LobbyResponse::UsersJoined {
                    public_keys: joined_keys,
                });
            }

            // Handle all left users (FIX: was only returning first)
            if !update.left.is_empty() {
                return Ok(LobbyResponse::UsersLeft {
                    public_keys: update.left,
                });
            }

            // Empty update
            Ok(LobbyResponse::Ignored)
        }
        // Other message types are not lobby messages
        _ => Ok(LobbyResponse::Ignored),
    }
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
pub struct WebSocketClient {
    connection: Option<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    key_state: SharedKeyState,
    lobby_event_handler: Option<LobbyEventHandler>,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(key_state: SharedKeyState) -> Self {
        Self {
            connection: None,
            key_state,
            lobby_event_handler: None,
        }
    }

    /// Set the lobby event handler
    ///
    /// The handler will be called when lobby messages arrive from the server.
    pub fn set_lobby_event_handler(&mut self, handler: LobbyEventHandler) {
        self.lobby_event_handler = Some(handler);
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
                    // Try to parse as lobby message first (Story 2.2)
                    if let Ok(lobby_response) = parse_lobby_message(&text) {
                        println!("Received lobby message: {:?}", lobby_response);

                        // Handle lobby responses
                        if let Some(ref handler) = self.lobby_event_handler {
                            match lobby_response {
                                LobbyResponse::LobbyState { users } => {
                                    // Update lobby state with initial user list
                                    let mut lobby_state = LobbyState::new();
                                    lobby_state.set_users(users);
                                    handler.lobby_received(&lobby_state);
                                }
                                LobbyResponse::UsersJoined { public_keys } => {
                                    // Users joined - notify handler for each
                                    for key in public_keys {
                                        handler.user_joined(&LobbyUser::new(key, true));
                                    }
                                }
                                LobbyResponse::UsersLeft { public_keys } => {
                                    // Users left - notify handler for each
                                    for key in public_keys {
                                        handler.user_left(&key);
                                    }
                                }
                                LobbyResponse::Ignored => {
                                    // Non-lobby message, ignore
                                }
                            }
                        }
                    } else {
                        // Not a lobby message - may be future story type (message, notification, etc.)
                        println!("Received non-lobby message: {}", text);
                        // Future stories (3.2+) will handle other message types
                    }
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

    // ========== Lobby Message Tests ==========

    #[test]
    fn test_parse_lobby_message_full_state() {
        let json = r#"{"type":"lobby","users":[{"publicKey":"key1","status":"online"},{"publicKey":"key2","status":"offline"}]}"#;
        let result = parse_lobby_message(json).unwrap();

        match result {
            LobbyResponse::LobbyState { users } => {
                assert_eq!(users.len(), 2);
                assert_eq!(users[0].public_key, "key1");
                assert!(users[0].is_online);
                assert_eq!(users[1].public_key, "key2");
                assert!(!users[1].is_online);
            }
            _ => panic!("Expected LobbyState response"),
        }
    }

    #[test]
    fn test_parse_lobby_message_empty() {
        let json = r#"{"type":"lobby","users":[]}"#;
        let result = parse_lobby_message(json).unwrap();

        match result {
            LobbyResponse::LobbyState { users } => {
                assert!(users.is_empty());
            }
            _ => panic!("Expected LobbyState response"),
        }
    }

    #[test]
    fn test_parse_lobby_update_user_joined() {
        let json = r#"{"type":"lobby_update","joined":[{"publicKey":"new_user"}],"left":[]}"#;
        let result = parse_lobby_message(json).unwrap();

        match result {
            LobbyResponse::UsersJoined { public_keys } => {
                assert_eq!(public_keys.len(), 1);
                assert_eq!(public_keys[0], "new_user");
            }
            _ => panic!("Expected UsersJoined response"),
        }
    }

    #[test]
    fn test_parse_lobby_update_user_left() {
        let json = r#"{"type":"lobby_update","joined":[],"left":["departed_user"]}"#;
        let result = parse_lobby_message(json).unwrap();

        match result {
            LobbyResponse::UsersLeft { public_keys } => {
                assert_eq!(public_keys.len(), 1);
                assert_eq!(public_keys[0], "departed_user");
            }
            _ => panic!("Expected UsersLeft response"),
        }
    }

    #[test]
    fn test_parse_lobby_update_multiple_users() {
        let json = r#"{"type":"lobby_update","joined":[{"publicKey":"user1"},{"publicKey":"user2"}],"left":[]}"#;
        let result = parse_lobby_message(json).unwrap();

        // Should now return ALL joined users (FIX: was only returning first)
        match result {
            LobbyResponse::UsersJoined { public_keys } => {
                assert_eq!(public_keys.len(), 2);
                assert_eq!(public_keys[0], "user1");
                assert_eq!(public_keys[1], "user2");
            }
            _ => panic!("Expected UsersJoined response"),
        }
    }

    #[test]
    fn test_parse_lobby_update_empty() {
        let json = r#"{"type":"lobby_update","joined":[],"left":[]}"#;
        let result = parse_lobby_message(json).unwrap();

        assert_eq!(result, LobbyResponse::Ignored);
    }

    #[test]
    fn test_parse_non_lobby_message() {
        let json = r#"{"type":"text","message":"hello"}"#;
        let result = parse_lobby_message(json).unwrap();

        assert_eq!(result, LobbyResponse::Ignored);
    }

    #[test]
    fn test_parse_invalid_lobby_json() {
        let json = "not valid json";
        let result = parse_lobby_message(json);

        assert!(result.is_err());
    }

    #[test]
    fn test_lobby_event_handler_creation() {
        let handler = LobbyEventHandler::new();
        // Handler should be created without panicking (no-op callbacks)
        // Just verify the fields exist and can be called
        let test_state = LobbyState::new();
        handler.lobby_received(&test_state); // Should not panic (no-op)
        let test_user = LobbyUser::new("key".to_string(), true);
        handler.user_joined(&test_user); // Should not panic (no-op)
        handler.user_left("key"); // Should not panic (no-op)
    }

    #[test]
    fn test_lobby_event_handler_with_callbacks() {
        // Test that callbacks are actually called using Arc<Mutex<>> pattern
        use std::sync::{Arc, Mutex};

        let state_count = Arc::new(Mutex::new(0));
        let joined_count = Arc::new(Mutex::new(0));
        let left_count = Arc::new(Mutex::new(0));

        let state_count_clone = state_count.clone();
        let joined_count_clone = joined_count.clone();
        let left_count_clone = left_count.clone();

        let handler = LobbyEventHandler::with_callbacks(
            move |_state| *state_count_clone.lock().unwrap() += 1,
            move |_user| *joined_count_clone.lock().unwrap() += 1,
            move |_key| *left_count_clone.lock().unwrap() += 1,
        );

        // Test lobby received callback
        let test_state = LobbyState::new();
        handler.lobby_received(&test_state);
        assert_eq!(*state_count.lock().unwrap(), 1);

        // Test user joined callback
        let test_user = LobbyUser::new("test_key".to_string(), true);
        handler.user_joined(&test_user);
        assert_eq!(*joined_count.lock().unwrap(), 1);

        // Test user left callback
        handler.user_left("departed_key");
        assert_eq!(*left_count.lock().unwrap(), 1);
    }
}