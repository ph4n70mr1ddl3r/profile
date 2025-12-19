//! UI error display for connection errors
//!
//! Maps technical error codes to user-friendly messages

/// Display user-friendly connection error message
pub fn display_connection_error(reason: &str) -> String {
    match reason {
        "auth_failed" => {
            "Authentication failed. Your signature could not be verified. Try again or check your key.".to_string()
        }
        "server_shutdown" => {
            "Server maintenance. Reconnect to continue.".to_string()
        }
        "timeout" => {
            "Connection timeout. Check your network and try reconnecting.".to_string()
        }
        "client_disconnect" => {
            // Intentional disconnect - no user message needed
            "".to_string()
        }
        _ => {
            // Unknown or network issue
            "Connection lost. Check your network and try reconnecting.".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_message_mapping() {
        // Test auth_failed
        let msg = display_connection_error("auth_failed");
        assert!(msg.contains("Authentication failed"));
        assert!(msg.contains("signature"));

        // Test server_shutdown
        let msg = display_connection_error("server_shutdown");
        assert!(msg.contains("Server maintenance"));
        
        // Test timeout
        let msg = display_connection_error("timeout");
        assert!(msg.contains("timeout"));
        assert!(msg.contains("network"));
        
        // Test client_disconnect (no message)
        let msg = display_connection_error("client_disconnect");
        assert!(msg.is_empty());
        
        // Test unknown reason (default message)
        let msg = display_connection_error("unknown_error");
        assert!(msg.contains("Connection lost"));
        assert!(msg.contains("network"));
    }
}
