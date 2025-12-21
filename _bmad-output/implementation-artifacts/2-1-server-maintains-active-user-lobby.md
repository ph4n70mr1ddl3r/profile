# Story 2.1: Server Maintains Active User Lobby

Status: in-progress

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a server application,
I want to maintain an in-memory list of all currently authenticated users with their public keys,
so that I can inform clients who is available to message and route messages to online recipients.

## Acceptance Criteria

**Epic Context:** This story is part of Epic 2: Presence - Online Lobby & Real-Time Updates, which enables users to see who's online and receive real-time presence updates.

**Story Foundation** [Source: /home/riddler/profile/_bmad-output/epics.md#L593-L630]:

**Given** a user successfully authenticates
**When** the server validates their authentication signature
**Then** the server adds their entry to the active lobby: `{publicKey: "...", activeConnection}`
**And** the lobby entry remains active as long as the WebSocket connection is open
**And** each user appears exactly once in the lobby (no duplicates)

**Given** a user already exists in the lobby
**When** they authenticate again (reconnection from same key)
**Then** the server updates their connection reference
**And** treats it as a single user (replaces previous entry, broadcasts leave/join delta)

**Given** a user's WebSocket connection closes
**When** the connection is terminated (intentional or network failure)
**Then** the server removes their entry from the lobby
**And** the lobby becomes the source of truth for who is online

**Given** the server needs to route a message
**When** a recipient is specified
**Then** the server queries the lobby: is recipient online?
**And** routes accordingly (deliver if online, offline notification if not)

**Technical Implementation Requirements** [Source: /home/riddler/profile/_bmad-output/epics.md#L593-L630]:
- Data structure: `HashMap<PublicKey, ActiveConnection>`
- Per-connection handler: manages lobby add/remove
- Atomic operations: ensure no race conditions with concurrent connections
- Broadcast mechanism: efficient delta updates (don't retransmit entire lobby each time)

**Related FRs:** FR26 (User presence display), FR8 (Message routing), FR9 (Online status) [Source: /home/riddler/profile/_bmad-output/epics.md#L66-L73]

## Developer Context Section - CRITICAL IMPLEMENTATION GUIDE

**🔥 CRITICAL MISSION:** This story creates the foundational lobby infrastructure that ALL Epic 2 stories depend on. Get this wrong and the entire presence system fails.

### **Technical Specifications** [Source: architecture.md#L71-L76]

**Core Technology Stack:**
- **Language:** Rust (confirmed from Epic 1)
- **Async Runtime:** Tokio 1.48.0 (latest stable)
- **WebSocket:** tokio-tungstenite 0.28.0 (latest stable)
- **Cryptography:** ed25519-dalek 2.2.0 (latest stable, audited)
- **Serialization:** serde 1.0.228 + serde_json 1.0
- **Concurrency Pattern:** `Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>`

**Critical Dependencies from Epic 1:**
- ✅ WebSocket authentication system (Story 1.5 completed)
- ✅ Ed25519 signature verification established
- ✅ PublicKey type defined and working
- ✅ Connection handling patterns proven

### **Architecture & Implementation Guide**

**Server Structure (from previous stories):**
- **Main server:** `profile-root/server/src/main.rs`
- **Connection handler:** `profile-root/server/src/connection/handler.rs` (WebSocket loop at lines 72-89)
- **Lobby module:** `profile-root/server/src/lobby/mod.rs` (extend existing)
- **Authentication:** Extend existing auth flow in connection handler

**Concurrency Pattern (proven from Epic 1):**
```rust
// Use same pattern as Epic 1 authentication
use std::sync::Arc;
use tokio::sync::RwLock;

// Lobby structure following established patterns
type Lobby = Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>;

// Add to existing handler initialization
let lobby: Lobby = Arc::new(RwLock::new(HashMap::new()));
```

**WebSocket Integration:**
- Extend existing `WebSocketStream<TcpStream>` handling from Story 1.6
- Add lobby management to existing message loop at lines 72-89 in `handler.rs`
- Follow established lobby cleanup patterns from Story 1.6 (lines 504-514)

### **File Structure & Patterns**

**Core Files to Modify/Create:**
1. `profile-root/server/src/lobby/mod.rs` - Extend lobby data structure
2. `profile-root/server/src/connection/handler.rs` - Add lobby management to WebSocket loop
3. `profile-root/server/tests/lobby_integration.rs` - New integration tests
4. `profile-root/server/src/main.rs` - Initialize lobby in server startup

**Pattern Consistency Requirements:**
- Follow established error handling from Epic 1
- Use same tracing patterns for logging
- Maintain consistent code organization from previous stories
- Preserve existing module structure in `/src/connection/` and `/src/lobby/`

### **Testing Strategy** [Source: Story 1.6 patterns]

**Integration Test Coverage (5+ tests required):**
1. `test_lobby_adds_user_on_auth` - Verify successful auth adds to lobby
2. `test_lobby_removes_user_on_disconnect` - Verify cleanup on connection close  
3. `test_lobby_handles_reconnection` - Verify duplicate key handling
4. `test_lobby_prevents_duplicates` - Verify single user per key
5. `test_lobby_thread_safety` - Verify concurrent access safety

**Test Location & Standards:**
- Extend existing `profile-root/server/tests/auth_integration.rs` (patterns from Story 1.6)
- Create new `profile-root/server/tests/lobby_integration.rs` for lobby-specific tests
- Follow Epic 1 test patterns (139 tests passing standard)
- Use same mocking and async testing patterns
- Include both unit and integration test coverage

### **Anti-Pattern Prevention**

**Common Mistakes to Avoid:**
1. **Duplicate Users:** Never allow same PublicKey in lobby twice
2. **Race Conditions:** Always use RwLock for concurrent access  
3. **Memory Leaks:** Clean up lobby entries on disconnect (proven pattern from Story 1.6)
4. **Error Handling:** Don't ignore WebSocket errors - clean up lobby on any failure
5. **Thread Safety:** Always clone Arc and use proper async patterns

**Precedent from Epic 1:**
- Story 1.6 established lobby cleanup patterns (lines 504-514 in handler.rs)
- Authentication flow is proven and working
- WebSocket message handling is stable

### **Cross-Story Dependency Map**

**Dependencies:**
- **Depends On:** Epic 1 complete (authentication system working)
- **Required For:** All Epic 2 stories (2.2-2.5)
  - **2.2 (Lobby Display):** Requires lobby data structure to query
  - **2.3 (Join Notifications):** Requires lobby to track who joined  
  - **2.4 (Leave Notifications):** Requires lobby cleanup on disconnect
  - **2.5 (Real-Time Sync):** Requires lobby as source of truth

**Interface Contracts for Future Stories:**
- Must expose `lobby.is_online(public_key)` for message routing (Epic 3)
- Must support `lobby.get_all_users()` for lobby display (Story 2.2)
- Must trigger events on add/remove for broadcast (Stories 2.3, 2.4)

### **Success Criteria & Completion Status**

**Success Criteria:**
- Server maintains authoritative lobby state
- No race conditions with concurrent connections  
- Proper cleanup on disconnection
- Foundation ready for Epic 2.2 (lobby display) and Epic 2.4 (broadcast updates)

**Implementation Phases:**
1. **Phase 1:** Extend lobby data structure with HashMap<PublicKey, ActiveConnection>
2. **Phase 2:** Integrate lobby management into existing WebSocket handler
3. **Phase 3:** Add comprehensive testing for all scenarios
4. **Phase 4:** Verify cleanup patterns and thread safety

**Ready for Development:** ✅ All requirements analyzed, architecture reviewed, and implementation guide provided. The developer has comprehensive context for flawless implementation.

**Status:** ready-for-dev  
**Next Steps:** Run `dev-story` for implementation, then `code-review` for validation

## Tasks / Subtasks

### **Task 1: Implement Lobby Data Structure** (AC: #1, #2, #3, Technical Requirements)
- [ ] **1.1** Define `Lobby` type as `Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>` in `server/src/lobby/mod.rs`
- [ ] **1.2** Create `ActiveConnection` struct wrapping `WebSocketStream<TcpStream>` and metadata
- [ ] **1.3** Implement lobby CRUD operations: `add_user()`, `remove_user()`, `update_connection()`, `is_online()`
- [ ] **1.4** Ensure thread safety with proper `RwLock` usage patterns

### **Task 2: Integrate Lobby with Authentication Flow** (AC: #1, #2)
- [ ] **2.1** Extend authentication handler in `server/src/connection/handler.rs` to call `lobby.add_user()` on successful auth
- [ ] **2.2** Handle reconnection scenario: check for existing key, update connection reference
- [ ] **2.3** Add lobby reference to connection handler state (clone Arc)
- [ ] **2.4** Implement duplicate prevention: ensure same PublicKey appears only once

### **Task 3: Implement Connection Cleanup** (AC: #3)
- [ ] **3.1** Add lobby cleanup in WebSocket close handler (`Message::Close` branch)
- [ ] **3.2** Ensure `lobby.remove_user()` is called on any connection termination
- [ ] **3.3** Follow established patterns from Story 1.6 (lines 504-514 in handler.rs)
- [ ] **3.4** Add error recovery: clean up lobby entry on WebSocket errors

### **Task 4: Create Lobby Query Interface** (AC: #4)
- [ ] **4.1** Implement `lobby.is_online(public_key: &PublicKey) -> bool` method
- [ ] **4.2** Add `lobby.get_connection()` for message routing (future use)
- [ ] **4.3** Ensure query operations use read lock for performance
- [ ] **4.4** Add lobby state inspection for debugging/tests

### **Task 5: Comprehensive Testing Suite**
- [ ] **5.1** Create `server/tests/lobby_integration.rs` with 5+ integration tests
- [ ] **5.2** Extend existing `auth_integration.rs` tests to verify lobby integration
- [ ] **5.3** Add unit tests for lobby data structure in `server/src/lobby/mod.rs`
- [ ] **5.4** Verify thread safety with concurrent access tests
- [ ] **5.5** Ensure test coverage matches Epic 1 standards (139+ tests passing)

### **Review Follow-ups (AI)**
- [ ] **[AI-Review][HIGH]** Story File List section is completely empty despite 11 files being modified - Update Dev Agent Record with accurate file list [story:380-381]
- [ ] **[AI-Review][HIGH]** AC2 reconnection logic missing in state.rs add_user() - Implementation should check for existing users before insert [lobby/state.rs:39-43]
- [ ] **[AI-Review][HIGH]** WebSocket sender in connection handler creates dead-end channel - Messages won't reach client [connection/handler.rs:45]
- [ ] **[AI-Review][MEDIUM]** Inconsistent error handling between state.rs (String errors) and manager.rs (LobbyError) - Use consistent error types [lobby/state.rs:39, manager.rs:26]
- [ ] **[AI-Review][LOW]** Test duplication between state.rs and manager.rs - Consolidate overlapping test coverage [lobby/state.rs:125, manager.rs:184]
- [ ] **[AI-Review][LOW]** Connection ID generation TODO in handler - Implement unique connection ID generation [connection/handler.rs:49]

## Dev Notes

### **Source Citations & Requirements Traceability**
- **Story Foundation:** Requirements from `epics.md` lines 593-630 [Source: /home/riddler/profile/_bmad-output/epics.md#L593-L630]
- **Functional Requirements:** FR26 (server maintains lobby), FR8 (message routing), FR9 (online status) [Source: /home/riddler/profile/_bmad-output/epics.md#L66-L73]
- **Architecture Constraints:** HashMap<PublicKey, ActiveConnection> pattern [Source: /home/riddler/profile/_bmad-output/architecture.md#L70-L85]
- **Technical Stack:** Rust + Tokio + ed25519-dalek [Source: /home/riddler/profile/_bmad-output/architecture.md#L71-L76]
- **Performance Requirements:** Lobby updates <100ms propagation [Source: /home/riddler/profile/_bmad-output/architecture.md#L49-L54]

### **Git History Intelligence**
**Recent Commit Patterns (Epic 1):**
- **Feature Commits:** `feat(server): add lobby cleanup on client disconnect` (Story 1.6 pattern)
- **Test Coverage:** Each story adds 20-30 integration tests in `/server/tests/`
- **Code Organization:** Modular structure in `/server/src/connection/`, `/server/src/lobby/`
- **Error Handling:** Consistent use of `tracing` for logging, proper cleanup in `Drop` impls
- **File Modification Focus:** Majority of changes in `handler.rs` WebSocket message loop

**Established Patterns to Follow:**
```rust
// From Story 1.6 - WebSocket close handling pattern
match msg {
    Message::Text(text) => { /* auth handling */ },
    Message::Close(frame) => {
        let reason = frame.as_ref()
            .map(|f| f.reason.to_string())
            .unwrap_or_else(|| "Unknown".to_string());
        lobby.remove_user(&public_key).await; // ← NEW: lobby cleanup
        return Err(format!("Connection closed: {}", reason).into());
    },
    _ => { /* handle other message types */ }
}
```

### **Concrete Testing Examples**
**Integration Test Template:**
```rust
// server/tests/lobby_integration.rs
#[tokio::test]
async fn test_lobby_adds_user_on_auth() {
    // Setup
    let lobby: Lobby = Arc::new(RwLock::new(HashMap::new()));
    let (client_stream, server_stream) = tokio::io::duplex(1024);
    
    // Simulate authentication
    let public_key = PublicKey::from_bytes(&[0u8; 32]).unwrap();
    let result = authenticate_user(&public_key, server_stream, lobby.clone()).await;
    
    // Assert
    assert!(result.is_ok());
    assert!(lobby.read().await.contains_key(&public_key));
}

#[tokio::test]
async fn test_lobby_removes_user_on_disconnect() {
    // Similar pattern to Story 1.6 test_disconnection_cleanup()
    // Verify lobby entry removed when WebSocket closes
}

#[tokio::test]
async fn test_lobby_handles_reconnection() {
    // Add user, simulate disconnect, reconnect with same key
    // Verify connection reference updated, not duplicate entry
}
```

### **Cross-Story Dependency Map**
**This Story Dependencies:**
- **Depends On:** Epic 1 complete (authentication system working)
- **Required For:** All Epic 2 stories (2.2-2.5)
  - **2.2 (Lobby Display):** Requires lobby data structure to query
  - **2.3 (Join Notifications):** Requires lobby to track who joined  
  - **2.4 (Leave Notifications):** Requires lobby cleanup on disconnect
  - **2.5 (Real-Time Sync):** Requires lobby as source of truth

**Interface Contracts:**
- Must expose `lobby.is_online(public_key)` for message routing (Epic 3)
- Must support `lobby.get_all_users()` for lobby display (Story 2.2)
- Must trigger events on add/remove for broadcast (Stories 2.3, 2.4)

### **Actionable Code Snippets**
**Lobby Type Definition:**
```rust
// server/src/lobby/mod.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type PublicKey = [u8; 32];
pub struct ActiveConnection {
    pub stream: WebSocketStream<TcpStream>,
    pub connected_at: std::time::Instant,
}

pub type Lobby = Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>;

impl Lobby {
    pub async fn add_user(&self, public_key: PublicKey, connection: ActiveConnection) -> Result<(), LobbyError> {
        let mut lobby = self.write().await;
        if lobby.contains_key(&public_key) {
            return Err(LobbyError::DuplicateUser);
        }
        lobby.insert(public_key, connection);
        Ok(())
    }
    
    pub async fn is_online(&self, public_key: &PublicKey) -> bool {
        let lobby = self.read().await;
        lobby.contains_key(public_key)
    }
}
```

**Integration with Authentication Handler:**
```rust
// server/src/connection/handler.rs (extend existing auth flow)
async fn handle_authentication(
    auth_message: AuthMessage, 
    stream: WebSocketStream<TcpStream>,
    lobby: Lobby,
) -> Result<(), HandlerError> {
    // Existing signature verification
    let public_key = auth_message.public_key;
    
    // Add to lobby
    let connection = ActiveConnection {
        stream,
        connected_at: std::time::Instant::now(),
    };
    
    lobby.add_user(public_key, connection).await?;
    
    // Continue with existing success flow
    Ok(())
}
```

### **Project Structure Guidance**
**Files to Modify:**
1. **`server/src/lobby/mod.rs`** - Core lobby implementation (NEW/EXTEND)
2. **`server/src/connection/handler.rs`** - Integrate lobby with auth (MODIFY lines 72-89)
3. **`server/tests/lobby_integration.rs`** - New test file (CREATE)
4. **`server/src/main.rs`** - Initialize lobby at startup (MODIFY)

**Directory Structure:**
```
profile-root/
├── server/
│   ├── src/
│   │   ├── lobby/           # ← Extend this module
│   │   │   ├── mod.rs       # Lobby type & operations
│   │   │   └── error.rs     # Lobby-specific errors
│   │   ├── connection/
│   │   │   └── handler.rs   # ← Modify WebSocket loop
│   │   └── main.rs          # ← Initialize lobby
│   └── tests/
│       ├── auth_integration.rs  # ← Extend existing
│       └── lobby_integration.rs # ← Create new
```

**Naming Conventions:**
- Use `snake_case` for functions/variables (consistent with Rust conventions)
- Use `PascalCase` for types/structs
- Test functions: `test_<scenario>_<expected_outcome>`
- Error types: `<Module>Error` (e.g., `LobbyError`)

## Dev Agent Record

### Agent Model Used

{{agent_model_name_version}}

### Debug Log References

### Completion Notes List

### File List

**Core Implementation:**
- `profile-root/server/src/lobby/mod.rs` - Extended lobby module with exports
- `profile-root/server/src/lobby/state.rs` - Core lobby data structures and basic operations  
- `profile-root/server/src/lobby/manager.rs` - High-level lobby operations with reconnection handling
- `profile-root/server/src/connection/handler.rs` - Integrated lobby management with WebSocket handling

**Error Handling:**
- `profile-root/shared/src/errors/lobby_error.rs` - Lobby-specific error types
- `profile-root/shared/src/errors/mod.rs` - Updated error module exports
- `profile-root/shared/src/protocol/mod.rs` - Updated protocol exports

**Testing:**
- `profile-root/server/tests/lobby_state_isolated_test.rs` - Isolated lobby state tests

**Documentation:**
- `_bmad-output/sprint-artifacts/2-1-server-maintains-active-user-lobby.md` - This story file
- `_bmad-output/sprint-artifacts/sprint-status.yaml` - Sprint tracking updates

