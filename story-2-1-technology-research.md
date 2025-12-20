# Story 2.1: Server Maintains Active User Lobby - Technology Research

## Executive Summary

This document provides critical technology research for implementing Story 2.1: "Server Maintains Active User Lobby". Based on current stable releases and security best practices as of December 2025, this research identifies the latest versions, security considerations, and implementation guidelines for the key technologies.

## Story 2.1 Context

**Objective**: Implement a server-side WebSocket lobby system that maintains active user connections with cryptographic authentication.

**Technical Requirements**:
- Async WebSocket server handling concurrent connections
- Ed25519 signature verification for authentication
- In-memory lobby: `HashMap<PublicKey, ActiveConnection>`
- Atomic operations for thread-safe concurrent access
- WebSocket security and TLS implementation

## Technology Stack Analysis

### 1. Tokio 1.48.0 (Latest Stable)

**Current Version**: 1.48.0 (Released October 14, 2025)
**Previous Version Mentioned**: 1.35 (Significantly outdated - 13 major versions behind)

#### Key Changes Since 1.35:
- **MSRV Update**: Minimum Rust version now 1.71 (was likely 1.63 in 1.35)
- **New Features**:
  - `fs::File::max_buf_size()` for optimized file operations
  - `TcpStream::quickack()` and `TcpStream::set_quickack()` for low-latency networking
  - `LocalKey::try_get()` for better task-local storage
  - `sync::SetOnce` - similar to `std::sync::OnceLock`
  - `sync::Notify::notified_owned()` for owned notifications

#### Critical Security Fixes:
- **Broadcast Channel Soundness** (v1.42.1): Fixed race condition in `clone()` calls for Send but !Sync types
- **Runtime Stability**: Improved wake ordering and task scheduling

#### Performance Improvements:
- Reduced generated code size for `Timeout<T>::poll`
- Enhanced `AtomicWaker` performance
- Better memory allocation in broadcast channels

#### Migration Considerations:
```toml
# Update Cargo.toml
tokio = { version = "1.48", features = ["full"] }

# Key API changes:
- Ensure Rust 1.71+ toolchain
- Review task spawning patterns for new cooperative scheduling
- Update any custom runtime configurations
```

### 2. tokio-tungstenite 0.28.0 (Latest Stable)

**Current Version**: 0.28.0 (Released September 25, 2025)
**Previous Version Mentioned**: 0.21 (7 major versions behind)

#### Key Improvements:
- **Rust Version**: Now requires Rust 1.63+
- **TLS Support**: Enhanced rustls integration with `rustls-pki-types`
- **Performance**: Optimized connection handling and memory usage
- **API Stability**: Mature API with proven WebSocket implementation

#### Security Enhancements:
- Built-in support for modern TLS configurations
- Better error handling for malformed connections
- Enhanced protocol compliance (RFC 6455)

#### Recommended Configuration:
```toml
tokio-tungstenite = { version = "0.28", features = ["rustls-tls-webpki-roots"] }
```

#### Migration from 0.21:
```rust
// New secure connection pattern
use tokio_tungstenite::{connect_async_tls_with_config, Connector};

let connector = Connector::Rustls;
let (socket, response) = connect_async_tls_with_config(
    url,
    None, // No custom headers
    Some(connector),
    false // Disable Nagle for low latency
).await?;
```

### 3. ed25519-dalek 2.2.0 (Latest Stable)

**Current Version**: 2.2.0 (Released July 9, 2025)
**Previous Version Mentioned**: 2.1 (Minor version difference)

#### Security Status:
- **No known vulnerabilities** in current version
- **Active development** with 3.0.0-pre.3 available (use caution with pre-release)
- **Audited codebase** with strong cryptographic implementation

#### Key Features:
- **Default Features**: `["fast", "std", "zeroize"]` for security and performance
- **Memory Safety**: Zeroize trait for secure memory cleanup
- **Batch Verification**: Support for verifying multiple signatures efficiently
- **Serde Support**: Built-in serialization/deserialization

#### Recommended Configuration:
```toml
ed25519-dalek = { version = "2.2", features = ["serde", "zeroize"] }
```

#### Security Best Practices:
```rust
use ed25519_dalek::{SigningKey, Verifier, Signer};
use rand::rngs::OsRng;

// Secure key generation
let mut csprng = OsRng;
let signing_key = SigningKey::generate(&mut csprng);

// Message signing with zeroize protection
let message = b"Authentication challenge";
let signature = signing_key.sign(message);

// Verification
let verifying_key = signing_key.verifying_key();
assert!(verifying_key.verify(message, &signature).is_ok());
```

### 4. serde 1.0.228 (Latest Stable)

**Current Version**: 1.0.228 (Released September 27, 2025)
**Previous Version Mentioned**: 1.0 (Version range acceptable)

#### Recent Improvements:
- **Documentation**: Enhanced derive macro documentation
- **Diagnostics**: Better error messages for serialization issues
- **Performance**: Optimized code generation for derived implementations

#### Recommended Features:
```toml
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0" # For JSON message serialization
```

### 5. WebSocket Security Best Practices (Critical)

#### Transport Security
```rust
// NEVER use ws:// in production
const INSECURE_URL: &str = "ws://example.com/socket"; // ❌ DANGEROUS

const SECURE_URL: &str = "wss://example.com/socket"; // ✅ SECURE
```

#### Origin Validation (Critical for CSWSH Prevention)
```rust
use tokio_tungstenite::accept_async_with_config;
use tungstenite::protocol::WebSocketConfig;

async fn handle_connection(stream: TcpStream) -> Result<()> {
    let config = WebSocketConfig {
        max_message_size: Some(64 * 1024), // 64KB limit
        max_frame_size: Some(64 * 1024),
        ..Default::default()
    };

    // CRITICAL: Validate origin to prevent Cross-Site WebSocket Hijacking
    let mut socket = accept_async_with_config(stream, Some(config))
        .await
        .expect("Failed to accept connection");

    // Additional security: implement rate limiting
    // Additional security: validate message structure
    // Additional security: implement proper authentication
    
    Ok(())
}
```

#### Message-Level Authorization
```rust
#[derive(Debug, Deserialize)]
struct WebSocketMessage {
    message_type: String,
    // ... other fields
}

async fn process_message(socket: &mut WebSocketStream<TcpStream>, msg: WebSocketMessage) {
    match msg.message_type.as_str() {
        "auth_challenge" => {
            // Handle authentication
        }
        "lobby_query" => {
            // Check if user is authorized for lobby access
        }
        _ => {
            // Reject unknown message types
            let _ = socket.send(Message::Text("Unknown message type".to_string())).await;
        }
    }
}
```

#### Input Validation
```rust
use serde_json;

// Safe JSON parsing (never use eval())
fn parse_message(data: &str) -> Result<WebSocketMessage, serde_json::Error> {
    serde_json::from_str(data)
}

// Size limits (prevent DoS)
const MAX_MESSAGE_SIZE: usize = 64 * 1024; // 64KB

fn validate_message_size(data: &[u8]) -> Result<(), &'static str> {
    if data.len() > MAX_MESSAGE_SIZE {
        Err("Message too large")
    } else {
        Ok(())
    }
}
```

## Implementation Architecture

### Recommended Cargo Dependencies
```toml
[dependencies]
tokio = { version = "1.48", features = ["full"] }
tokio-tungstenite = { version = "0.28", features = ["rustls-tls-webpki-roots"] }
ed25519-dalek = { version = "2.2", features = ["serde", "zeroize"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8" # For cryptographic key generation
```

### Core Implementation Pattern
```rust
use std::collections::HashMap;
use tokio::sync::RwLock;
use ed25519_dalek::{PublicKey, Signature, SigningKey};
use tokio_tungstenite::{WebSocketStream, accept_async};
use futures_util::{StreamExt, SinkExt};

type Lobby = RwLock<HashMap<PublicKey, WebSocketStream<TcpStream>>>;

struct ServerState {
    lobby: Lobby,
    signing_key: SigningKey, // For server signatures
}

impl ServerState {
    async fn add_to_lobby(&self, public_key: PublicKey, connection: WebSocketStream<TcpStream>) {
        let mut lobby = self.lobby.write().await;
        lobby.insert(public_key, connection);
    }

    async fn remove_from_lobby(&self, public_key: &PublicKey) {
        let mut lobby = self.lobby.write().await;
        lobby.remove(public_key);
    }

    async fn broadcast_to_lobby(&self, message: &str) {
        let lobby = self.lobby.read().await;
        let mut connections_to_remove = Vec::new();

        for (public_key, connection) in lobby.iter() {
            if let Err(e) = connection.send(Message::Text(message.to_string())).await {
                log::warn!("Failed to send to {}: {}", public_key, e);
                connections_to_remove.push(*public_key);
            }
        }

        // Clean up failed connections
        drop(lobby);
        let mut lobby = self.lobby.write().await;
        for public_key in connections_to_remove {
            lobby.remove(&public_key);
        }
    }
}
```

## Security Checklist

### Transport Layer
- [ ] Use `wss://` URLs only (never `ws://`)
- [ ] Configure proper TLS certificates
- [ ] Disable weak cipher suites
- [ ] Enable HSTS headers

### Authentication
- [ ] Implement Ed25519 signature verification
- [ ] Validate origin headers
- [ ] Use secure session management
- [ ] Implement token rotation for long connections

### Authorization
- [ ] Message-level access control
- [ ] Rate limiting per connection
- [ ] Size limits on all messages
- [ ] Input validation with allowlists

### Monitoring
- [ ] Log connection events with user identity
- [ ] Monitor authentication failures
- [ ] Track security violations
- [ ] Alert on unusual connection patterns

## Migration Timeline

### Phase 1: Dependency Updates
1. Update Tokio from 1.35 → 1.48 (requires Rust 1.71+)
2. Update tokio-tungstenite from 0.21 → 0.28
3. Update ed25519-dalek from 2.1 → 2.2
4. Update serde to latest (1.0.228)

### Phase 2: Security Hardening
1. Implement origin validation
2. Add message size limits
3. Implement rate limiting
4. Add comprehensive logging

### Phase 3: Testing & Validation
1. Security testing for CSWSH vulnerabilities
2. Performance testing under load
3. Authentication flow testing
4. Connection resilience testing

## Critical Security Warnings

1. **Origin Validation**: MUST implement origin validation to prevent Cross-Site WebSocket Hijacking (CSWSH)
2. **WSS Only**: NEVER use unencrypted WebSocket connections in production
3. **Input Validation**: ALL WebSocket messages must be validated and size-limited
4. **Memory Safety**: Use ed25519-dalek's zeroize feature for secure memory cleanup
5. **Rate Limiting**: Implement per-connection rate limiting to prevent DoS

## Performance Considerations

- Use `RwLock` for lobby access (read-heavy workload)
- Implement connection pooling for database connections
- Use batch signature verification where possible
- Monitor memory usage for long-lived connections
- Implement proper backpressure handling

## Conclusion

The recommended technology stack provides a secure, performant foundation for Story 2.1. The key focus areas are:

1. **Security First**: Implement all OWASP WebSocket security recommendations
2. **Latest Stable Versions**: All identified versions are mature and secure
3. **Migration Path**: Clear upgrade path from current versions
4. **Monitoring**: Comprehensive logging and monitoring capabilities

This implementation will provide a robust, secure WebSocket lobby system suitable for production use with proper security controls.