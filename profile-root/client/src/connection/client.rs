use crate::state::messages::{
    create_shared_message_history, create_shared_message_history_with_capacity, ChatMessage,
    SharedMessageHistory,
};
use crate::state::session::SharedKeyState;
use crate::ui::lobby_state::{LobbyState, LobbyUser};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, info, warn};

/// Type alias for recipient offline callback
type RecipientOfflineCallback = Rc<RefCell<dyn Fn(String) + 'static>>;

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
    /// Called when the selected user leaves the lobby (AC5)
    pub on_selection_lost: Rc<RefCell<dyn Fn(String)>>,
}

/// Callback handler for chat message events
#[derive(Clone)]
pub struct MessageEventHandler {
    /// Called when a new message is received and verified
    pub on_message_received: Rc<RefCell<dyn Fn(ChatMessage)>>,
    /// Called when a message has an invalid signature
    pub on_invalid_signature: Rc<RefCell<dyn Fn(String)>>,
    /// Called when an error occurs
    pub on_error: Rc<RefCell<dyn Fn(String)>>,
    /// Called for general notifications (e.g., offline status, info messages)
    pub on_notification: Rc<RefCell<dyn Fn(String)>>,
}

impl MessageEventHandler {
    /// Create a new message event handler with no-op callbacks
    #[inline]
    pub fn new() -> Self {
        Self {
            on_message_received: Rc::new(RefCell::new(|_: ChatMessage| {})),
            on_invalid_signature: Rc::new(RefCell::new(|_: String| {})),
            on_error: Rc::new(RefCell::new(|_: String| {})),
            on_notification: Rc::new(RefCell::new(|_: String| {})),
        }
    }

    /// Create with custom callbacks
    #[inline]
    pub fn with_callbacks(
        on_message_received: impl Fn(ChatMessage) + 'static,
        on_invalid_signature: impl Fn(String) + 'static,
        on_error: impl Fn(String) + 'static,
        on_notification: impl Fn(String) + 'static,
    ) -> Self {
        Self {
            on_message_received: Rc::new(RefCell::new(on_message_received)),
            on_invalid_signature: Rc::new(RefCell::new(on_invalid_signature)),
            on_error: Rc::new(RefCell::new(on_error)),
            on_notification: Rc::new(RefCell::new(on_notification)),
        }
    }

    /// Emit message received event (for verified messages)
    #[inline]
    pub fn message_received(&self, message: &ChatMessage) {
        (self.on_message_received.borrow())(message.clone());
    }

    /// Emit invalid signature event
    #[inline]
    pub fn invalid_signature(&self, notification: &str) {
        (self.on_invalid_signature.borrow())(notification.to_string());
    }

    /// Emit error event
    #[inline]
    pub fn error(&self, error: &str) {
        (self.on_error.borrow())(error.to_string());
    }

    /// Emit notification event (for general notifications like offline status)
    #[inline]
    pub fn notification(&self, message: &str) {
        (self.on_notification.borrow())(message.to_string());
    }
}

impl Default for MessageEventHandler {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl LobbyEventHandler {
    /// Create a new lobby event handler with no-op callbacks
    #[inline]
    pub fn new() -> Self {
        Self {
            on_lobby_received: Rc::new(RefCell::new(|_: LobbyState| {})),
            on_user_joined: Rc::new(RefCell::new(|_: LobbyUser| {})),
            on_user_left: Rc::new(RefCell::new(|_: String| {})),
            on_selection_lost: Rc::new(RefCell::new(|_: String| {})),
        }
    }

    /// Create with custom callbacks
    #[inline]
    pub fn with_callbacks(
        on_lobby_received: impl Fn(LobbyState) + 'static,
        on_user_joined: impl Fn(LobbyUser) + 'static,
        on_user_left: impl Fn(String) + 'static,
        on_selection_lost: impl Fn(String) + 'static,
    ) -> Self {
        Self {
            on_lobby_received: Rc::new(RefCell::new(on_lobby_received)),
            on_user_joined: Rc::new(RefCell::new(on_user_joined)),
            on_user_left: Rc::new(RefCell::new(on_user_left)),
            on_selection_lost: Rc::new(RefCell::new(on_selection_lost)),
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

    /// Emit selection lost event (AC5)
    #[inline]
    pub fn selection_lost(&self, public_key: &str) {
        (self.on_selection_lost.borrow())(public_key.to_string());
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

/// Response from the chat message parser
#[derive(Debug, Clone, PartialEq)]
pub enum ChatResponse {
    /// A new message was received
    Message(ChatMessage),
    /// Message was ignored (e.g., already verified by server)
    Ignored,
}

/// Parse a lobby message from the server
pub fn parse_lobby_message(
    text: &str,
) -> Result<LobbyResponse, Box<dyn std::error::Error + Send + Sync>> {
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
                    is_online: u.status.as_deref() == Some("online"),
                })
                .collect();

            Ok(LobbyResponse::LobbyState { users })
        }
        "lobby_update" => {
            // Parse lobby update (delta)
            let update: profile_shared::protocol::LobbyUpdateMessage = serde_json::from_str(text)?;

            // Handle joined users (all users in delta)
            if !update.joined.is_empty() {
                let joined_keys: Vec<String> =
                    update.joined.into_iter().map(|u| u.public_key).collect();
                return Ok(LobbyResponse::UsersJoined {
                    public_keys: joined_keys,
                });
            }

            // Handle left users (all users in delta)
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

/// Parse a chat message from the server
///
/// This handles the "message" type sent when another user sends a message.
/// The message has already been validated by the server, so we trust it.
/// The client will do its own verification for defense in depth.
pub fn parse_chat_message(
    text: &str,
) -> Result<ChatResponse, Box<dyn std::error::Error + Send + Sync>> {
    // First, determine message type
    let msg: ServerMessage = serde_json::from_str(text)?;

    match msg.r#type.as_str() {
        "message" => {
            // Parse the text message from protocol module
            let text_msg: profile_shared::protocol::Message = serde_json::from_str(text)?;

            // Extract the message data using pattern matching
            let (message, sender_public_key, signature, timestamp) = match text_msg {
                profile_shared::protocol::Message::Text {
                    message,
                    sender_public_key,
                    signature,
                    timestamp,
                } => (message, sender_public_key, signature, timestamp),
                _ => return Ok(ChatResponse::Ignored),
            };

            // Create a ChatMessage (initially unverified, client will verify)
            let chat_msg = ChatMessage::new(sender_public_key, message, signature, timestamp);
            Ok(ChatResponse::Message(chat_msg))
        }
        // Other message types are not chat messages
        _ => Ok(ChatResponse::Ignored),
    }
}

/// Notification response from server
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationResponse {
    /// Recipient is offline
    RecipientOffline {
        recipient_key: String,
        message: Option<String>,
    },
    /// User came back online
    UserBackOnline { public_key: String },
    /// Unknown notification type
    Unknown,
}

/// Parse a notification from the server
pub fn parse_notification(
    text: &str,
) -> Result<NotificationResponse, Box<dyn std::error::Error + Send + Sync>> {
    use crate::handlers::offline::OfflineNotification;

    // First, determine message type
    let msg: ServerMessage = serde_json::from_str(text)?;

    match msg.r#type.as_str() {
        "notification" => {
            let notification: OfflineNotification = serde_json::from_str(text)?;
            match notification.event.as_str() {
                "recipient_offline" => Ok(NotificationResponse::RecipientOffline {
                    recipient_key: notification.recipient,
                    message: notification.message,
                }),
                "user_online" => Ok(NotificationResponse::UserBackOnline {
                    public_key: notification.recipient,
                }),
                _ => Ok(NotificationResponse::Unknown),
            }
        }
        _ => Ok(NotificationResponse::Unknown),
    }
}

/// Verify and store a received chat message
///
/// This function performs client-side signature verification and stores
/// valid messages in the message history. Invalid messages are rejected.
///
/// # Arguments
/// * `chat_msg` - The parsed but unverified chat message
/// * `message_history` - Shared message history for storage
/// * `handler` - Message event handler for callbacks
///
/// # Returns
/// Ok(()) if message was verified and stored, Err(reason) otherwise
pub async fn verify_and_store_message(
    chat_msg: &ChatMessage,
    message_history: &SharedMessageHistory,
    handler: &Option<MessageEventHandler>,
) {
    use crate::handlers::verify::{
        create_invalid_signature_notification, format_public_key, verify_chat_message,
    };

    // Verify the signature
    match verify_chat_message(chat_msg) {
        crate::handlers::verify::VerificationResult::Valid(verified_msg) => {
            // Store in message history
            let mut history = message_history.lock().await;
            history.add_message(verified_msg.clone());

            // Notify handler
            if let Some(ref h) = handler {
                h.message_received(&verified_msg);
            }
        }
        crate::handlers::verify::VerificationResult::Invalid {
            sender_public_key,
            reason,
        } => {
            // Log warning and notify user
            warn!(
                key = %format_public_key(&sender_public_key),
                reason = %reason,
                "Invalid signature received"
            );

            // Create notification
            let notification = create_invalid_signature_notification(&sender_public_key, &reason);

            // Notify handler
            if let Some(ref h) = handler {
                h.invalid_signature(&notification);
            }
        }
    }
}

/// Parse any server message (lobby or chat)
///
/// Returns the appropriate response type based on message content.
/// Useful for handling messages in the WebSocket loop.
pub fn parse_server_message(
    text: &str,
) -> Result<ServerMessageResponse, Box<dyn std::error::Error + Send + Sync>> {
    // First, determine message type
    let msg: ServerMessage = serde_json::from_str(text)?;

    match msg.r#type.as_str() {
        "lobby" | "lobby_update" => {
            // Try to parse as lobby message
            match parse_lobby_message(text) {
                Ok(response) => Ok(ServerMessageResponse::Lobby(response)),
                Err(e) => Err(e),
            }
        }
        "message" => {
            // Try to parse as chat message
            match parse_chat_message(text) {
                Ok(response) => Ok(ServerMessageResponse::Chat(response)),
                Err(e) => Err(e),
            }
        }
        "error" => {
            // Parse error message
            let error_msg: ServerErrorMessage = serde_json::from_str(text)?;
            Ok(ServerMessageResponse::Error(error_msg))
        }
        _ => Ok(ServerMessageResponse::Unknown),
    }
}

/// Unified server message response
#[derive(Debug, Clone, PartialEq)]
pub enum ServerMessageResponse {
    /// Lobby-related message
    Lobby(LobbyResponse),
    /// Chat message
    Chat(ChatResponse),
    /// Error from server
    Error(ServerErrorMessage),
    /// Unknown message type
    Unknown,
}

/// Server error message structure
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ServerErrorMessage {
    pub r#type: String,
    pub reason: String,
    pub details: Option<String>,
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
fn parse_auth_response(
    text: &str,
) -> Result<AuthResponse, Box<dyn std::error::Error + Send + Sync>> {
    // First, determine message type
    let msg: ServerMessage = serde_json::from_str(text)?;

    match msg.r#type.as_str() {
        "auth_success" => {
            let success: AuthSuccessMessage = serde_json::from_str(text)?;
            Ok(AuthResponse::Success {
                users: success.users,
            })
        }
        "error" => {
            let error: AuthErrorMessage = serde_json::from_str(text)?;
            Ok(AuthResponse::Failed {
                reason: error.reason,
                details: error.details,
            })
        }
        other => Err(format!("Unknown message type: {}", other).into()),
    }
}

/// Connection state for the WebSocket client
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionState {
    /// Not connected
    Disconnected,
    /// Currently attempting to connect or authenticate
    Connecting,
    /// Fully connected and authenticated
    Connected,
    /// Temporarily disconnected, attempting to reconnect
    Reconnecting { attempts: u32 },
}

/// WebSocket client for connecting to the profile server
pub struct WebSocketClient {
    connection: Option<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    key_state: SharedKeyState,
    message_history: SharedMessageHistory,
    lobby_event_handler: Option<LobbyEventHandler>,
    message_event_handler: Option<MessageEventHandler>,
    /// Track currently selected recipient for selection loss detection (AC5)
    selected_recipient: Option<String>,
    /// Current connection state (AC4 - Network Resilience)
    connection_state: ConnectionState,
    /// Maximum reconnection attempts before giving up (AC4)
    max_reconnect_attempts: u32,
    /// Backoff multiplier for exponential backoff (AC4)
    reconnect_backoff_ms: u64,
    /// Queue for messages to send after reconnection (AC4 - race handling)
    /// Maps recipient public key -> list of pending messages for that recipient
    pending_messages: std::sync::Arc<tokio::sync::Mutex<HashMap<String, Vec<String>>>>,
    /// Notification when recipient goes offline during message composition (AC4)
    recipient_offline_handler: Option<RecipientOfflineCallback>,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    pub fn new(key_state: SharedKeyState) -> Self {
        Self {
            connection: None,
            key_state,
            message_history: create_shared_message_history(),
            lobby_event_handler: None,
            message_event_handler: None,
            selected_recipient: None,
            connection_state: ConnectionState::Disconnected,
            max_reconnect_attempts: 5,
            reconnect_backoff_ms: 1000,
            pending_messages: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            recipient_offline_handler: None,
        }
    }

    /// Create with custom message history capacity
    pub fn with_history_capacity(key_state: SharedKeyState, capacity: usize) -> Self {
        Self {
            connection: None,
            key_state,
            message_history: create_shared_message_history_with_capacity(capacity),
            lobby_event_handler: None,
            message_event_handler: None,
            selected_recipient: None,
            connection_state: ConnectionState::Disconnected,
            max_reconnect_attempts: 5,
            reconnect_backoff_ms: 1000,
            pending_messages: std::sync::Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            recipient_offline_handler: None,
        }
    }

    /// Set the selected recipient for selection loss tracking (AC5)
    pub fn set_selected_recipient(&mut self, public_key: Option<String>) {
        self.selected_recipient = public_key;
    }

    /// Get the currently selected recipient
    pub fn selected_recipient(&self) -> Option<&str> {
        self.selected_recipient.as_deref()
    }

    /// Get the message history
    pub fn message_history(&self) -> SharedMessageHistory {
        self.message_history.clone()
    }

    /// Get the current connection state (AC4)
    pub fn connection_state(&self) -> ConnectionState {
        self.connection_state.clone()
    }

    /// Set handler for recipient offline notifications (AC4)
    pub fn set_recipient_offline_handler(&mut self, handler: impl Fn(String) + 'static) {
        self.recipient_offline_handler = Some(Rc::new(RefCell::new(handler)));
    }

    /// Attempt automatic reconnection with exponential backoff (AC4)
    ///
    /// This implements Task 5.1: "Add reconnection logic for temporary disconnects"
    async fn attempt_reconnect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut attempts = 0;

        while attempts < self.max_reconnect_attempts {
            self.connection_state = ConnectionState::Reconnecting { attempts };

            // Exponential backoff: 1s, 2s, 4s, 8s, 16s
            let backoff = self.reconnect_backoff_ms * 2u64.pow(attempts);
            debug!(
                backoff_ms = backoff,
                attempt = attempts + 1,
                max_attempts = self.max_reconnect_attempts,
                "Reconnecting"
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(backoff)).await;

            // Try to reconnect
            match self.connect().await {
                Ok(_) => {
                    info!("Reconnected successfully");
                    return self.reconnection_flow().await;
                }
                Err(e) => {
                    warn!(attempt = attempts + 1, error = %e, "Reconnection attempt failed");
                    attempts += 1;
                }
            }
        }

        // Give up after max attempts
        let err_msg = format!(
            "Failed to reconnect after {} attempts. Please reconnect manually.",
            self.max_reconnect_attempts
        );
        self.connection_state = ConnectionState::Disconnected;

        if let Some(ref handler) = self.message_event_handler {
            handler.error(&err_msg);
        }

        Err(err_msg.into())
    }

    /// Complete reconnection flow after connection established (AC4)
    ///
    /// This implements Task 5.2: "On reconnect, request full lobby state from server"
    async fn reconnection_flow(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Authenticate with server
        match self.authenticate().await {
            Ok(_) => {
                info!("Re-authenticated successfully");
                self.connection_state = ConnectionState::Connected;

                // Send any pending messages (Task 5.3: Handle race)
                let messages_to_send: Vec<String> = {
                    let mut pending = self.pending_messages.lock().await;
                    if !pending.is_empty() {
                        let total_count: usize = pending.values().map(|v| v.len()).sum();
                        info!(
                            count = total_count,
                            "Sending pending messages from reconnection"
                        );
                        // Flatten all pending messages into a single vector
                        let messages: Vec<String> = pending.values().flatten().cloned().collect();
                        // Clear the HashMap
                        pending.clear();
                        messages
                    } else {
                        vec![]
                    }
                };

                // Send messages after releasing lock (avoid borrow checker conflict)
                for msg in messages_to_send {
                    self.send_message_internal(&msg).await?;
                }

                Ok(())
            }
            Err(e) => {
                warn!(error = %e, "Authentication after reconnect failed");
                self.connection_state = ConnectionState::Disconnected;
                Err(e)
            }
        }
    }

    /// Send a message to the server (internal helper)
    async fn send_message_internal(
        &mut self,
        message: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(connection) = &mut self.connection {
            connection.send(Message::Text(message.to_string())).await?;
            Ok(())
        } else {
            Err("No connection available".into())
        }
    }

    /// Send a message to the server (public API)
    ///
    /// # Arguments
    /// * `message` - The JSON message to send
    ///
    /// # Returns
    /// Ok(()) if message was sent successfully
    ///
    /// # Errors
    /// Returns error if connection is not available or send fails
    pub async fn send_message(
        &mut self,
        message: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.send_message_internal(&message).await
    }

    /// Set the lobby event handler
    ///
    /// The handler will be called when lobby messages arrive from the server.
    pub fn set_lobby_event_handler(&mut self, handler: LobbyEventHandler) {
        self.lobby_event_handler = Some(handler);
    }

    /// Set the message event handler
    ///
    /// The handler will be called when chat messages arrive from the server.
    pub fn set_message_event_handler(&mut self, handler: MessageEventHandler) {
        self.message_event_handler = Some(handler);
    }

    /// Connect to the profile server
    pub async fn connect(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use environment variable PROFILE_SERVER_URL if set, otherwise default to localhost
        let url = std::env::var("PROFILE_SERVER_URL")
            .unwrap_or_else(|_| "ws://127.0.0.1:8080".to_string());

        let (ws_stream, _) = connect_async(&url).await?;
        self.connection = Some(ws_stream);

        Ok(())
    }

    /// Perform authentication handshake
    pub async fn authenticate(
        &mut self,
    ) -> Result<AuthResponse, Box<dyn std::error::Error + Send + Sync>> {
        // Get keys from shared state
        // Create authentication message using auth.rs module within the lock scope
        let auth_msg = {
            let key_state = self.key_state.lock().await;
            let public_key = key_state
                .public_key()
                .ok_or("No public key available. Generate or import a key first.")?
                .clone();
            let private_key = key_state
                .private_key()
                .ok_or("No private key available. Generate or import a key first.")?;

            super::auth::ClientAuthMessage::new_with_ref(public_key, private_key)?
        };
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
                            let user_message = display_connection_error(reason);

                            // If no specific user message, provide generic auth failure
                            let final_message = if user_message.is_empty()
                                || user_message.contains("Connection lost")
                            {
                                "Authentication failed. Your signature could not be verified. Try again or check your key.".to_string()
                            } else {
                                user_message
                            };

                            return Err(final_message.into());
                        }

                        return Ok(response);
                    }
                    Message::Close(frame) => {
                        let reason = frame
                            .as_ref()
                            .map(|f| f.reason.to_string())
                            .unwrap_or_else(|| "Unknown".to_string());

                        // Use error_display to map to user-friendly message
                        use crate::ui::error_display::display_connection_error;
                        let user_message = display_connection_error(&reason);

                        // If we have a specific message, use it; otherwise use generic
                        let final_message = if !user_message.is_empty()
                            && !user_message.contains("Connection lost")
                        {
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

    /// Handle disconnection with reason (AC4 - Network Resilience)
    ///
    /// If this is a temporary disconnect, attempt automatic reconnection.
    pub async fn handle_disconnection(
        &mut self,
        reason: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Remove connection
        self.connection = None;
        self.connection_state = ConnectionState::Disconnected;

        // Check if this is a recoverable disconnection (network-level, not application-level)
        // Application-level reasons like "server_shutdown", "timeout", "auth_failed" are permanent
        let is_temporary = matches!(
            reason.as_str(),
            "connection closed" | "Connection reset by peer" | "Broken pipe"
        );

        if is_temporary {
            warn!(reason = %reason, "Temporary disconnect - attempting reconnection");
            return self.attempt_reconnect().await;
        }

        // Permanent disconnect - return error that triggers UI display
        Err(format!("Connection closed: {}", reason).into())
    }

    /// Close connection gracefully
    pub async fn close_gracefully(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(connection) = &mut self.connection {
            use tokio_tungstenite::tungstenite::protocol::{frame::coding::CloseCode, CloseFrame};
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
    pub async fn run_message_loop(
        &mut self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
                        debug!(?lobby_response, "Received lobby message");

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
                                    // Check if selected user left (AC5)
                                    let selected_left = self
                                        .selected_recipient
                                        .as_ref()
                                        .map(|sel_key| public_keys.contains(sel_key))
                                        .unwrap_or(false);

                                    // Users left - notify handler for each
                                    for key in &public_keys {
                                        handler.user_left(key);
                                    }

                                    // If selected user left, notify (AC5)
                                    if selected_left {
                                        if let Some(ref sel_key) = self.selected_recipient {
                                            handler.selection_lost(sel_key);
                                        }
                                        self.selected_recipient = None;
                                    }
                                }
                                LobbyResponse::Ignored => {
                                    // Non-lobby message, ignore
                                }
                            }
                        }
                    } else if let Ok(chat_response) = parse_chat_message(&text) {
                        // Handle chat message with verification (Story 3.3 + 3.4)
                        match chat_response {
                            ChatResponse::Message(message) => {
                                debug!(sender = %message.sender_public_key.chars().take(16).collect::<String>(), "Received chat message - verifying");

                                // Verify and store the message
                                verify_and_store_message(
                                    &message,
                                    &self.message_history,
                                    &self.message_event_handler,
                                )
                                .await;
                            }
                            ChatResponse::Ignored => {
                                // Message was ignored
                            }
                        }
                    } else {
                        // Try to parse as error or other message
                        if let Ok(server_msg) = parse_server_message(&text) {
                            match server_msg {
                                ServerMessageResponse::Error(error) => {
                                    warn!(reason = %error.reason, details = %error.details.clone().unwrap_or_default(), "Server error");
                                    if let Some(ref handler) = self.message_event_handler {
                                        let details = error.details.unwrap_or_default();
                                        handler.error(&format!("{}: {}", error.reason, details));
                                    }
                                }
                                ServerMessageResponse::Unknown => {
                                    debug!(message = %text, "Received unknown message type");
                                }
                                _ => {
                                    // Lobby and chat already handled above
                                }
                            }
                        } else if let Ok(notification) = parse_notification(&text) {
                            // Handle notification (Story 3.6)
                            match notification {
                                NotificationResponse::RecipientOffline {
                                    recipient_key,
                                    message,
                                } => {
                                    debug!(recipient = %recipient_key.chars().take(16).collect::<String>(), "Recipient is offline");

                                    // Format notification message (AC4 - User Notification)
                                    let notification_msg = format!(
                                        "User {} is offline. Message not delivered.",
                                        &recipient_key[..std::cmp::min(16, recipient_key.len())]
                                    );

                                    // Queue message for delivery when recipient comes online (AC4)
                                    if let Some(msg_content) = message {
                                        let mut pending = self.pending_messages.lock().await;
                                        pending
                                            .entry(recipient_key.clone())
                                            .or_insert_with(Vec::new)
                                            .push(msg_content.clone());
                                        debug!(recipient = %recipient_key, message = %msg_content, "Message queued for delivery when recipient comes online");
                                    }

                                    // Notify recipient_offline_handler (AC4)
                                    if let Some(ref handler) = self.recipient_offline_handler {
                                        handler.borrow()(recipient_key.clone());
                                    }

                                    // Notify message event handler of notification (not invalid signature!)
                                    if let Some(ref handler) = self.message_event_handler {
                                        handler.notification(&notification_msg);
                                    }
                                }
                                NotificationResponse::UserBackOnline { public_key } => {
                                    info!(user = %public_key.chars().take(16).collect::<String>(), "User is back online");

                                    // Send pending messages for this user (AC4 - Deliver queued messages)
                                    let user_messages: Vec<String> = {
                                        let mut pending = self.pending_messages.lock().await;
                                        // Remove and return messages for this recipient
                                        pending.remove(&public_key).unwrap_or_default()
                                    };

                                    // Send messages after releasing lock
                                    for msg in &user_messages {
                                        if let Err(e) = self.send_message_internal(msg).await {
                                            warn!(user = %public_key, error = %e, "Failed to send queued message");
                                        }
                                    }
                                    info!(user = %public_key, count = user_messages.len(), "Delivered queued messages");
                                }
                                NotificationResponse::Unknown => {
                                    debug!(message = %text, "Received unknown notification");
                                }
                            }
                        } else {
                            debug!(message = %text, "Received unparseable message");
                        }
                    }
                }
                Some(Ok(Message::Close(frame))) => {
                    // Server closed the connection
                    let reason = frame
                        .as_ref()
                        .map(|f| f.reason.to_string())
                        .unwrap_or_else(|| "Unknown".to_string());

                    // Use error_display to map to user-friendly message
                    use crate::ui::error_display::display_connection_error;
                    let user_message = display_connection_error(&reason);

                    // Clean up connection state
                    self.connection = None;

                    // Check if we should attempt reconnection (AC4)
                    let is_temporary = matches!(
                        reason.as_str(),
                        "connection closed"
                            | "Connection reset by peer"
                            | "Broken pipe"
                            | "timeout"
                    );

                    if is_temporary {
                        warn!(reason = %reason, "Connection closed (temporary) - attempting reconnection");
                        return self.attempt_reconnect().await;
                    }

                    // Permanent disconnect - return error with user-friendly message
                    let final_message =
                        if !user_message.is_empty() && !user_message.contains("Connection lost") {
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
                    debug!("Received unexpected message type");
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
        let json =
            r#"{"type":"error","reason":"auth_failed","details":"Signature did not verify"}"#;
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
        use std::sync::{Arc, Mutex};

        let state_count = Arc::new(Mutex::new(0));
        let joined_count = Arc::new(Mutex::new(0));
        let left_count = Arc::new(Mutex::new(0));
        let selection_lost_count = Arc::new(Mutex::new(0));

        let state_count_clone = state_count.clone();
        let joined_count_clone = joined_count.clone();
        let left_count_clone = left_count.clone();
        let selection_lost_count_clone = selection_lost_count.clone();

        let handler = LobbyEventHandler::with_callbacks(
            move |_state| *state_count_clone.lock().unwrap() += 1,
            move |_user| *joined_count_clone.lock().unwrap() += 1,
            move |_key| *left_count_clone.lock().unwrap() += 1,
            move |_key| *selection_lost_count_clone.lock().unwrap() += 1,
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

    #[test]
    fn test_lobby_event_handler_selection_lost_noop() {
        let handler = LobbyEventHandler::new();
        // Should not panic (no-op)
        handler.selection_lost("key");
    }

    #[tokio::test]
    async fn test_client_selected_recipient_tracking() {
        use crate::state::session::create_shared_key_state;

        let key_state = create_shared_key_state();
        let mut client = WebSocketClient::new(key_state);

        // Initially no selection
        assert!(client.selected_recipient().is_none());

        // Set selected recipient
        client.set_selected_recipient(Some("test_key".to_string()));
        assert_eq!(client.selected_recipient(), Some("test_key"));

        // Clear selection
        client.set_selected_recipient(None);
        assert!(client.selected_recipient().is_none());
    }
}
