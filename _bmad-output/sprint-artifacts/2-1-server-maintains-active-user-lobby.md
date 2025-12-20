# Story 2.1: Server Maintains Active User Lobby

**Epic:** 2 - Presence - Online Lobby & Real-Time Updates  
**Story ID:** 2.1  
**Status:** ready-for-dev  
**Priority:** High - Foundation for Epic 2  
**Estimated Complexity:** Medium  
**Dependencies:** Epic 1 completion (authentication system, stories 1.5 & 1.6)

## 🎯 Story Overview

This story establishes the server-side foundation for presence tracking by maintaining an in-memory data structure that tracks all currently authenticated users. Without this critical infrastructure, the lobby cannot function, users cannot see who's available to message, and real-time presence updates are impossible.

**Business Value:** Enables the core presence infrastructure that allows users to see who's online and receive real-time updates when people connect or disconnect - essential for effective messaging functionality.

**Technical Value:** Creates the authoritative server state for presence tracking, providing the data foundation for all subsequent Epic 2 stories (2.2, 2.3, 2.4) and enabling message routing based on online status (story 3.2).

### Cross-Story Dependencies

**Depends On:**
- Story 1.5 (Authenticate to Server) - Authentication success triggers lobby add
- Story 1.6 (Handle Authentication Failure) - WebSocket close frame detection required for cleanup

**Blocks:**
- Story 2.2 (Query & Display Lobby) - Needs lobby state to display
- Story 2.3 (Broadcast User Join) - Needs join notifications
- Story 2.4 (Broadcast User Leave) - Needs leave notifications  
- Story 3.2 (Send Message to Server) - Message routing requires recipient lobby lookup

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a server application,
I want to maintain an in-memory list of all currently authenticated users with their public keys,
so that I can inform clients who is available to message and route messages to online recipients.

## Acceptance Criteria

### AC1: User Lobby Entry Creation on Successful Authentication

**Given** a user successfully authenticates with a valid signature (from story 1.5)  
**When** the server validates their authentication signature  
**Then** the server adds their entry to the active lobby with structure: `{publicKey: "...", activeConnection: WebSocketHandle}`  
**And** the user entry remains in the lobby as long as the WebSocket connection is open  
**And** each user appears exactly once in the lobby (no duplicate entries)

---

### AC2: Handle User Reconnection (Same Key, New Connection)

**Given** a user already exists in the lobby (previous connection)  
**When** they authenticate again with the same public key from a different WebSocket connection  
**Then** the server updates their connection reference (replaces old with new)  
**And** treats it as a single user in the lobby (no duplicate)  
**And** broadcasts the reconnection as a leave→join delta (old connection removed, new connection added)

---

### AC3: User Removal on Connection Close

**Given** a user's WebSocket connection closes (intentional or network failure)  
**When** the server detects the disconnection via `Message::Close` frame  
**Then** the server removes their entry from the lobby  
**And** the lobby becomes the source of truth for who is online  
**And** the user no longer receives messages or appears in other clients' lobby lists

**Critical:** This depends on story 1.6 properly capturing WebSocket close frames. See Story 1.6 close frame contract before implementing.

---

### AC4: Lobby Query for Message Routing

**Given** a message sender wants to send to a specific recipient  
**When** the message routing logic (story 3.2) queries the lobby  
**Then** the server checks: is `recipient_public_key` in the lobby?  
**And** returns `Some(&ActiveConnection)` if online or `None` if offline  
**And** routes message accordingly (deliver if online, offline notification if not)

---

### AC5: Lobby Consistency Under Concurrent Operations

**Given** multiple users are joining and leaving the lobby simultaneously  
**When** concurrent add/remove operations occur  
**Then** the lobby state remains consistent with no race conditions  
**And** no ghost users remain after disconnection  
**And** no duplicate users appear after reconnection  
**And** all clients receive consistent lobby state eventually

---

## Tasks / Subtasks

### Task 1: Define Lobby Data Structure (AC1, AC2, AC5)
- [ ] Create `server/src/lobby/state.rs` with:
  - [ ] `pub type PublicKey = String;` type alias (exported for use in routing)
  - [ ] `pub struct ActiveConnection { pub_key: PublicKey, sender: mpsc::UnboundedSender<Message> }`
  - [ ] `type Lobby = Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>;`
  - [ ] Add inline unit tests for struct construction
- [ ] Implement thread-safe access pattern using `Arc<RwLock<T>>`
- [ ] **CRITICAL:** Ensure `ActiveConnection.sender` is `mpsc::UnboundedSender<Message>` (not TODO comment)

### Task 2: Implement Lobby Add Operation (AC1, AC2)
- [ ] Create `server/src/lobby/manager.rs` with:
  - [ ] `pub async fn add_user(lobby: &Lobby, key: PublicKey, conn: ActiveConnection) -> Result<(), LobbyError>`
  - [ ] **CRITICAL:** Check for existing user (reconnection case from AC2)
  - [ ] If exists: call `broadcast_user_left()` for old connection, then replace entry
  - [ ] If new: simply add to HashMap
  - [ ] Call `broadcast_user_joined()` to notify all other users
  - [ ] Return `LobbyError::InvalidPublicKey` for malformed keys
  - [ ] Add unit tests: `test_add_user_new_entry`, `test_add_user_reconnection_replaces`
- [ ] Integrate with `server/src/connection/auth.rs` to call after signature validation succeeds
- [ ] Use typed errors (`LobbyError` enum, not string errors)

### Task 3: Implement Lobby Remove Operation (AC3, AC5)
- [ ] Create `server/src/lobby/manager.rs` (continued):
  - [ ] `pub async fn remove_user(lobby: &Lobby, key: &PublicKey) -> Result<(), LobbyError>`
  - [ ] Remove entry from HashMap
  - [ ] Call `broadcast_user_left()` to notify all remaining users
  - [ ] Return `Ok(())` if key not found (idempotent operation)
  - [ ] Add unit tests: `test_remove_user_deletes_entry`, `test_remove_nonexistent_user_safe`
- [ ] Integrate with `server/src/connection/handler.rs` message loop:
  - [ ] **CRITICAL:** Capture `Message::Close` frame (don't ignore it - see Story 1.6 learnings)
  - [ ] Call `lobby.remove_user(&public_key).await` on close frame
  - [ ] Add explicit test: `test_close_frame_triggers_lobby_removal()`
  - [ ] Prevents ghost users from remaining in lobby after disconnection

### Task 4: Implement Lobby Query Operation (AC4)
- [ ] Create in `server/src/lobby/manager.rs`:
  - [ ] `pub async fn get_user(lobby: &Lobby, key: &PublicKey) -> Option<&ActiveConnection>`
  - [ ] **CRITICAL:** Return `Option<&ActiveConnection>` (NOT clone, NOT Arc) for zero-copy access
  - [ ] Used by story 3.2 for message routing (recipient online check)
  - [ ] Add unit tests: `test_get_user_returns_existing`, `test_get_user_returns_none_for_missing`
- [ ] Create snapshot method for story 2.2:
  - [ ] `pub async fn get_current_users(lobby: &Lobby) -> Vec<PublicKey>`
  - [ ] Returns list of all currently online public keys
  - [ ] Pre-allocate Vec with capacity: `Vec::with_capacity(lobby.read().await.len())`
  - [ ] Used by story 2.2 for initial lobby display

### Task 5: Implement Broadcast Helpers (AC1, AC2, AC3)
- [ ] Create in `server/src/lobby/manager.rs`:
  - [ ] `async fn broadcast_user_joined(lobby: &Lobby, key: &PublicKey) -> Result<(), LobbyError>`
  - [ ] Construct message: `{"type": "lobby_update", "joined": [{"publicKey": "..."}]}`
  - [ ] Iterate through all lobby entries, send to each WebSocket sender
  - [ ] Skip failed sends (user may have disconnected during broadcast)
  - [ ] `async fn broadcast_user_left(lobby: &Lobby, key: &PublicKey) -> Result<(), LobbyError>`
  - [ ] Construct message: `{"type": "lobby_update", "left": [{"publicKey": "..."}]}`
  - [ ] Send delta (only changed user), not full lobby snapshot
  - [ ] Add unit test: `test_broadcast_sends_delta_format()` verifying JSON structure
  - [ ] Add integration test: `test_message_routing_uses_sender()` verifying WebSocket delivery

### Task 6: Integration with Connection Handler (AC1, AC3)
- [ ] Update `server/src/connection/handler.rs`:
  - [ ] After auth success: call `lobby.add_user(public_key, connection)`
  - [ ] In message loop: check for `Message::Close` frame
  - [ ] On close: call `lobby.remove_user(&public_key)`
  - [ ] Ensure both operations complete before connection fully closes
  - [ ] Add error handling for lobby operations

### Task 7: Error Handling & Validation (AC5)
- [ ] Define error types in `shared/src/errors/`:
  - [ ] `LobbyError::DuplicateUser`
  - [ ] `LobbyError::UserNotFound`
  - [ ] `LobbyError::InvalidPublicKey`
- [ ] Validate all operations:
  - [ ] Public key format validation (hex string, correct length)
  - [ ] Duplicate user handling (AC2: replace pattern)
  - [ ] Concurrent access safety (no panics)
- [ ] Return user-friendly error messages

### Task 8: Testing Suite
- [ ] Unit tests in `server/src/lobby/manager.rs`:
  - [ ] `test_add_user_new_entry` - Basic addition
  - [ ] `test_add_user_reconnection_replaces` - AC2: Same key replaces old connection
  - [ ] `test_remove_user_deletes_entry` - Basic removal
  - [ ] `test_remove_nonexistent_user_safe` - Idempotent removal
  - [ ] `test_get_user_returns_existing` - Query returns Some
  - [ ] `test_get_user_returns_none_for_missing` - Query returns None
  - [ ] `test_concurrent_add_remove_safe` - AC5: 10+ concurrent operations, no race conditions
  - [ ] `test_close_frame_triggers_lobby_removal` - AC3: Close frame detection (Story 1.6)
  - [ ] `test_broadcast_sends_delta_format` - Verify delta JSON structure
  - [ ] `test_ghost_user_prevention` - Disconnect → user removed, doesn't linger
- [ ] Integration tests in `server/tests/lobby_sync.rs`:
  - [ ] `test_multiple_clients_lobby_consistency` - AC5: Eventual consistency
  - [ ] `test_user_add_triggers_broadcast` - AC1: Join notification sent
  - [ ] `test_user_remove_triggers_broadcast` - AC3: Leave notification sent
  - [ ] `test_reconnection_updates_not_duplicates` - AC2: No duplicate entries
  - [ ] `test_message_routing_uses_sender` - AC4: WebSocket sender works for routing
- [ ] E2E test in `server/tests/integration_multiclient.rs` (from architecture.md):
  - [ ] Spawn real server
  - [ ] Spawn 3 real client processes with different keys
  - [ ] All authenticate → verify all appear in each other's lobby
  - [ ] One disconnects → verify removed from all lobbies
  - [ ] One reconnects → verify not duplicated

## Test Specification

### Close Frame Detection Test (AC3, Story 1.6 Integration)
```rust
#[tokio::test]
async fn test_close_frame_triggers_lobby_removal() {
    // Setup: Create lobby, add user with WebSocket connection
    let lobby = Arc::new(RwLock::new(HashMap::new()));
    let public_key = "test_key_123".to_string();
    // Add user to lobby
    
    // Action: Simulate Message::Close frame received in connection handler
    // Verify: lobby.remove_user() called
    // Verify: User no longer in lobby
    
    // Expected: User entry removed, no ghost user remains
}
```

### Reconnection Test (AC2)
```rust
#[tokio::test]
async fn test_add_user_reconnection_replaces() {
    // Setup: Add user once to lobby
    let lobby = Arc::new(RwLock::new(HashMap::new()));
    let public_key = "reconnect_user".to_string();
    
    // Action: Add same public_key with different connection
    // Verify: Old connection replaced (not duplicated)
    // Verify: Lobby size remains 1
    // Verify: broadcast_user_left() called for old
    // Verify: broadcast_user_joined() called for new
}
```

### Concurrent Operations Test (AC5)
```rust
#[tokio::test]
async fn test_concurrent_add_remove_safe() {
    // Setup: Create lobby, spawn 20 tasks
    // Action: 10 tasks add users, 10 tasks remove users concurrently
    // Verify: No panics, no race conditions
    // Verify: Final lobby state consistent with operations
}
```

---

## Dev Notes

### Architecture Patterns & Constraints

**Primary Pattern: Thread-Safe Shared State** [Source: architecture.md#Pattern-5-State-Management]

```rust
// server/src/lobby/state.rs
pub type PublicKey = String;  // Type alias for clarity

#[must_use]
pub struct ActiveConnection {
    public_key: PublicKey,
    sender: mpsc::UnboundedSender<Message>,  // CRITICAL: Must be actual sender, not TODO
    // For Phase 2: last_activity: Instant (timeout detection)
}

pub type Lobby = Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>;
```

**Why this pattern:**
- `Arc` = multiple threads can hold reference to same lobby
- `RwLock` = multiple readers (queries) can run simultaneously, exclusive writer (add/remove)
- `HashMap` = O(1) lookup for message routing (story 3.2 critical path)
- `mpsc::UnboundedSender` = efficient broadcast to specific client
- `PublicKey` type alias = improves readability throughout codebase

**Do NOT use:**
- ❌ Flags or booleans for state (use enums) [Source: architecture.md#Anti-Pattern]
- ❌ Direct `Mutex` if you have many readers (use `RwLock`)
- ❌ `Vec` for user list (use HashMap for O(1) lookup)
- ❌ Polling for lobby changes (use push notifications)
- ❌ String errors (use typed `LobbyError` enum)
- ❌ println! debugging (use `tracing` crate with structured logging)

### Source Tree Components to Modify

**Files to Create:**
- `server/src/lobby/state.rs` - Data structures (ActiveConnection, Lobby type, PublicKey alias)
- `server/src/lobby/manager.rs` - Operations (add, remove, get, broadcast)
- `server/src/lobby/mod.rs` - Module exports

**Files to Modify:**
- `server/src/connection/auth.rs` - Call `lobby.add_user()` after signature validation (story 1.5 integration)
- `server/src/connection/handler.rs` - **CRITICAL:** Capture `Message::Close` frame and call `lobby.remove_user()` (see Story 1.6 close frame contract)
- `server/src/connection/manager.rs` - Ensure lobby is passed to all connection handlers
- `server/src/lib.rs` - Export lobby module publicly

**Files to Reference (Don't Modify):**
- `shared/src/protocol/types.rs` - Message definitions for `lobby_update` format
- `shared/src/errors/` - Error type definitions (add `LobbyError` enum here)

**Integration Points:**
- **From Story 1.5:** After `auth::validate_signature()` succeeds → call `lobby.add_user()`
- **From Story 1.6:** When `Message::Close` received → call `lobby.remove_user()` (prevents ghost users)
- **For Story 2.2:** Provide `get_current_users()` for initial lobby snapshot
- **For Story 3.2:** Provide `get_user()` for message routing recipient lookup

### Testing Standards Summary

**Concurrency Testing:** [Source: architecture.md#Testing-Architecture]

This story MUST handle concurrent operations safely. Test with:
- 10+ simultaneous add operations
- 10+ simultaneous remove operations  
- Mix of add/remove/query operations
- No race conditions, no panics, no data corruption

**Metrics & Observability:**

Add basic metrics for Phase 2 monitoring:
- Counter: `lobby_users_total` (current count)
- Counter: `lobby_joins_total` (incremented on add)
- Counter: `lobby_leaves_total` (incremented on remove)
- Histogram: `lobby_operation_duration_ms` (add/remove/get timing)

Use `tracing` crate for structured logging:
```rust
tracing::info!(user_count = lobby.len(), "User added to lobby");
tracing::warn!(public_key = %key, "Reconnection detected, replacing old connection");
```

**Determinism Testing:**

While this story doesn't involve cryptography, ensure:
- Same sequence of operations → same final lobby state (always)
- Multiple clients see consistent lobby state (eventually)

**Edge Cases:**

Test these specific scenarios:
- User authenticates, immediately disconnects (close frame) → removed (no ghost user)
- User authenticates twice from different connections → replaces, not duplicates (AC2)
- 100+ rapid joins/leaves → no ghost users, no duplicates, stable state
- Query during add/remove operations → consistent results (RwLock guarantees)
- Broadcast during user disconnect → skip failed sends, don't panic

### Project Structure Notes

**Module Organization:** Snake_case, nested by responsibility [Source: architecture.md#Pattern-1-Rust-Module-Conventions]

```
server/src/lobby/
  ├── mod.rs         (exports add_user, remove_user, get_user, broadcast_*)
  ├── state.rs       (ActiveConnection struct, Lobby type, constants)
  └── manager.rs     (implementation of add/remove/get/broadcast)
```

**Naming Conventions:**
- Functions: `add_user()`, `remove_user()`, `get_user()`, `broadcast_user_joined()`
- Types: `ActiveConnection`, `Lobby`, `PublicKey` (type alias)
- Variables: `public_key`, `ws_sender`, `lobby_state`
- Errors: `LobbyError::DuplicateUser`, `LobbyError::UserNotFound`, `LobbyError::InvalidPublicKey`

**No Conflicts:** This is new functionality (no existing files to refactor)

### Critical Implementation Warnings

**See Story 1.6 for Close Frame Contract:** This story depends on proper WebSocket close frame detection. Review [Story 1.6 - Handle Authentication Failure/Disconnection] for close frame handling patterns before implementing Task 3.

---

#### ⚠️ WARNING 1: WebSocket Close Frame Handling (AC3)

The connection handler MUST capture `Message::Close` frames to remove users from lobby. Ignoring close frames creates "ghost users" who appear online but can't receive messages.

**Implementation Location:** `server/src/connection/handler.rs` message loop

```rust
// CORRECT implementation:
match msg {
    Message::Text(text) => { /* process */ },
    Message::Close(frame) => {
        tracing::info!(public_key = %public_key, "Close frame received");
        lobby.remove_user(&public_key).await?;
        return Ok(()); // Exit handler cleanly
    },
    _ => {}
}
```

**Required Test:** `test_close_frame_triggers_lobby_removal()` - See Task 8

---

#### ⚠️ WARNING 2: Reconnection Must Replace, Not Duplicate (AC2)

When a user connects with a public key already in the lobby, you MUST replace the old connection (not create duplicate). This handles network interruptions and prevents multiple entries for the same user.

**Implementation Location:** `server/src/lobby/manager.rs::add_user()`

```rust
// CORRECT reconnection handling:
pub async fn add_user(lobby: &Lobby, key: PublicKey, new_conn: ActiveConnection) -> Result<(), LobbyError> {
    let mut guard = lobby.write().await;
    
    if let Some(old_conn) = guard.remove(&key) {
        tracing::warn!(public_key = %key, "Reconnection: replacing old connection");
        drop(guard); // Release lock before broadcast
        broadcast_user_left(&lobby, &key).await?;
    }
    
    lobby.write().await.insert(key.clone(), new_conn);
    broadcast_user_joined(&lobby, &key).await?;
    Ok(())
}
```

**Required Test:** `test_add_user_reconnection_replaces()` - Verify lobby size remains 1

---

#### ⚠️ WARNING 3: Broadcast Deltas, Not Full Lobby (AC1, AC3)

Performance Requirement: When a user joins/leaves, broadcast ONLY the changed user (delta), not the entire lobby. Full lobby broadcasts don't scale beyond ~50 users.

**Implementation Location:** `server/src/lobby/manager.rs::broadcast_*()`

```rust
// CORRECT delta broadcast:
async fn broadcast_user_joined(lobby: &Lobby, joined_key: &PublicKey) -> Result<(), LobbyError> {
    let update = json!({
        "type": "lobby_update",
        "joined": [{"publicKey": joined_key}]  // Only the new user
    });
    
    let guard = lobby.read().await;
    let recipients: Vec<_> = guard.iter()
        .filter(|(k, _)| *k != joined_key)  // Don't send to the user who just joined
        .map(|(_, conn)| &conn.sender)
        .collect();
    drop(guard);  // Release lock before network I/O
    
    for sender in recipients {
        let _ = sender.send(Message::Text(update.to_string()));  // Ignore send failures
    }
    Ok(())
}
```

**Scalability:** Delta broadcasts = O(n) messages total. Full lobby = O(n²) messages.

**Required Test:** `test_broadcast_sends_delta_format()` - Verify JSON contains only `joined`/`left` array

### References

For complete architectural context and implementation patterns, see:
- **Functional Requirements:** `_bmad-output/epics.md` - Story 2.1 specification (lines 593-628)
- **Architecture Decisions:** `_bmad-output/architecture.md` - Lobby state management patterns (lines 301-333), Component boundaries (lines 1287-1307), Implementation patterns (lines 850-925)
- **Previous Stories:** Story 1.5 (auth integration point), Story 1.6 (close frame contract)
- **WebSocket Protocol:** `_bmad-output/architecture.md` - Message format specification (lines 366-418)

## Dev Agent Record

### Agent Model Used

Claude 3.7 Sonnet (or later)

### Debug Log References

Not yet generated - populate after implementation

### Completion Notes List

- [ ] Lobby data structure (`Arc<RwLock<HashMap>>`) implemented
- [ ] Add user operation tested with unit + integration tests
- [ ] Remove user operation tested with concurrent scenarios
- [ ] Close frame detection integrated with connection handler
- [ ] Reconnection race condition handled properly
- [ ] Delta broadcasts working (verified with multi-client E2E test)
- [ ] All 5 acceptance criteria verified
- [ ] Code review completed (0 high/medium issues)
- [ ] Ready to unblock story 2.2 (display lobby)

### File List

**Created:**
- `server/src/lobby/mod.rs`
- `server/src/lobby/state.rs`
- `server/src/lobby/manager.rs`
- `server/tests/lobby_sync.rs`

**Modified:**
- `server/src/connection/auth.rs` - Add lobby integration
- `server/src/connection/handler.rs` - Add close frame handling
- `server/src/lib.rs` - Export lobby module

**Test Files:**
- `server/tests/integration_multilient.rs` - E2E multi-client test
