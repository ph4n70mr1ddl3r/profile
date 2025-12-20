# Dependency Migration Guide for Story 2.1

## Current vs. Recommended Versions

Based on the technology research for Story 2.1, here are the required dependency updates:

| Library | Current Version | Recommended Version | Security Impact |
|---------|----------------|-------------------|-----------------|
| tokio | 1.35 | 1.48 | **HIGH** - 13 major versions behind, contains critical fixes |
| tokio-tungstenite | 0.21 | 0.28 | **HIGH** - 7 major versions behind, security improvements |
| ed25519-dalek | 2.1 | 2.2 | **MEDIUM** - Minor update with stability improvements |
| serde | 1.0 | 1.0.228 | **LOW** - More specific version for consistency |

## Critical Security Updates Required

### 1. Tokio 1.35 → 1.48 (CRITICAL)

**Why Update**: 13 major versions behind with critical security fixes
- Fixed broadcast channel soundness issue (CVE-level impact)
- Improved task scheduling and memory safety
- Enhanced async runtime stability

**Changes Required**:
```toml
# In Cargo.toml workspace.dependencies
tokio = { version = "1.48", features = ["full"] }
```

**Migration Notes**:
- Requires Rust 1.71+ (check `rustc --version`)
- No breaking API changes for basic usage
- Review any custom runtime configurations

### 2. tokio-tungstenite 0.21 → 0.28 (CRITICAL)

**Why Update**: 7 major versions behind with important security enhancements
- Better TLS configuration support
- Improved WebSocket protocol compliance
- Enhanced error handling and security

**Changes Required**:
```toml
# In Cargo.toml workspace.dependencies
tokio-tungstenite = "0.28"
```

**Security Improvements**:
- Better origin validation support
- Enhanced TLS configuration
- Improved frame handling security

### 3. ed25519-dalek 2.1 → 2.2 (RECOMMENDED)

**Why Update**: Latest stable with security hardening
- Enhanced zeroization support
- Improved batch verification
- Better memory safety guarantees

**Changes Required**:
```toml
# In Cargo.toml workspace.dependencies
ed25519-dalek = { version = "2.2", features = ["serde", "zeroize"] }
```

### 4. serde Version Specificity (RECOMMENDED)

**Why Update**: Pin to latest stable for consistency
```toml
# In Cargo.toml workspace.dependencies
serde = { version = "1.0.228", features = ["derive"] }
```

## Updated Cargo.toml

Replace your workspace dependencies with:

```toml
[workspace]
resolver = "2"
members = ["server", "client", "shared"]

# Shared dependencies across all crates - UPDATED VERSIONS
[workspace.dependencies]
ed25519-dalek = { version = "2.2", features = ["serde", "zeroize"] }
zeroize = { version = "1.6", features = ["derive"] }
tokio = { version = "1.48", features = ["full"] }
tokio-tungstenite = "0.28"
futures-util = "0.3"
slint = "1.5"
slint-build = "1.5"
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0"
hex = "0.4"
rand = "0.8"
arboard = "3.4"
```

## Story 2.1 Implementation Pattern

With the updated dependencies, implement the lobby system:

```rust
// In server/src/main.rs
use tokio::sync::RwLock;
use tokio_tungstenite::{accept_async, WebSocketStream};
use ed25519_dalek::{PublicKey, Verifier};
use std::collections::HashMap;

type Lobby = RwLock<HashMap<PublicKey, WebSocketStream<TcpStream>>>;

pub struct ServerState {
    lobby: Lobby,
    verifying_key: VerifyingKey, // For client auth verification
}

impl ServerState {
    pub async fn handle_connection(
        &self,
        stream: TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Secure WebSocket acceptance with size limits
        let ws_stream = accept_async(stream).await?;
        
        // Add connection to lobby after authentication
        // Implementation details for Story 2.1
        
        Ok(())
    }
    
    pub async fn broadcast_lobby_update(&self) {
        let lobby = self.lobby.read().await;
        let update = format!("Current users: {}", lobby.len());
        
        for (_public_key, connection) in lobby.iter() {
            if let Err(e) = connection.send(tungstenite::Message::Text(update.clone())).await {
                log::warn!("Failed to send update: {}", e);
            }
        }
    }
}
```

## Security Implementation Checklist

### Pre-Implementation (After Dependencies Update)
- [ ] Test all existing functionality with updated dependencies
- [ ] Verify Rust toolchain is 1.71+
- [ ] Run `cargo audit` to check for vulnerabilities
- [ ] Update any custom error handling for new API changes

### Story 2.1 Security Requirements
- [ ] **Origin Validation**: Implement origin checking for WebSocket connections
- [ ] **Message Size Limits**: Configure 64KB maximum message size
- [ ] **Authentication**: Verify Ed25519 signatures for all client messages
- [ ] **Rate Limiting**: Implement per-connection rate limiting
- [ ] **Input Validation**: Validate all WebSocket message structure

### Example Security Configuration
```rust
use tokio_tungstenite::accept_async_with_config;
use tungstenite::protocol::WebSocketConfig;

let config = WebSocketConfig {
    max_message_size: Some(64 * 1024), // 64KB limit
    max_frame_size: Some(64 * 1024),
    ..Default::default()
};

let mut socket = accept_async_with_config(stream, Some(config))
    .await
    .expect("Failed to accept WebSocket connection");
```

## Testing Strategy

1. **Unit Tests**: Test all Ed25519 signature verification
2. **Integration Tests**: Test WebSocket connection lifecycle
3. **Security Tests**: Test origin validation and CSWSH prevention
4. **Load Tests**: Test concurrent connection handling

## Migration Commands

```bash
# 1. Update dependencies
cargo update

# 2. Check for compilation errors
cargo check

# 3. Run tests to verify functionality
cargo test

# 4. Security audit
cargo audit

# 5. Build release version
cargo build --release
```

## Breaking Changes Assessment

- **tokio 1.35 → 1.48**: Minimal breaking changes for basic usage
- **tokio-tungstenite 0.21 → 0.28**: API improvements, some method signatures may have changed
- **ed25519-dalek 2.1 → 2.2**: Minor updates, mostly additive

## Post-Migration Verification

After updating dependencies, verify:

1. ✅ All tests pass
2. ✅ No compilation warnings
3. ✅ Performance benchmarks remain stable
4. ✅ Security features work correctly
5. ✅ WebSocket connections establish properly

## Emergency Rollback Plan

If issues occur, temporarily revert in `Cargo.toml`:

```toml
# Emergency rollback versions
tokio = { version = "1.35", features = ["full"] }
tokio-tungstenite = "0.21"
ed25519-dalek = "2.1"
```

Then run `cargo update` to restore previous working state while investigating issues.

## Next Steps

1. **Immediate**: Update workspace dependencies
2. **Testing**: Comprehensive testing of updated dependencies
3. **Story 2.1**: Implement secure WebSocket lobby system
4. **Security Review**: OWASP WebSocket security validation
5. **Performance Testing**: Load testing with concurrent connections