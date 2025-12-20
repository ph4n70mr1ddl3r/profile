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

**Implementation Detail:** Use `HashMap<PublicKey, ActiveConnection>` stored in `Arc<RwLock<>>` for thread-safe access [Source: architecture.md#Pattern-5-State-Management]

---

### AC2: Handle User Reconnection (Same Key, New Connection)

**Given** a user already exists in the lobby (previous connection)  
**When** they authenticate again with the same public key from a different WebSocket connection  
**Then** the server updates their connection reference (replaces old with new)  
**And** treats it as a single user in the lobby (no duplicate)  
**And** broadcasts the reconnection as a leave→join delta (old connection removed, new connection added)  

**Implementation Detail:** Use `lobby.update_or_insert()` pattern. Emit leave notification for old connection, join notification for new connection [Source: epics.md#Story-2.1-AC2]

---

### AC3: User Removal on Connection Close

**Given** a user's WebSocket connection closes (intentional or network failure)  
**When** the server detects the disconnection via `Message::Close` frame  
**Then** the server removes their entry from the lobby  
**And** the lobby becomes the source of truth for who is online  
**And** the user no longer receives messages or appears in other clients' lobby lists  

**Critical:** This depends on story 1.6 properly capturing WebSocket close frames [Source: 1-6-handle-authentication-failure-disconnection.md#Critical-Implementation-Warning-1]

**Implementation Detail:** In connection handler's message loop, when `Message::Close` received, call `lobby.remove(&public_key)` and broadcast leave notification [Source: architecture.md#Integration-Points]

---

### AC4: Lobby Query for Message Routing

**Given** a message sender wants to send to a specific recipient  
**When** the message routing logic (story 3.2) queries the lobby  
**Then** the server checks: is `recipient_public_key` in the lobby?  
**And** returns `Some(ActiveConnection)` if online or `None` if offline  
**And** routes message accordingly (deliver if online, offline notification if not)  

**Implementation Detail:** Provide query interface `lobby.get(&public_key) -> Option<&ActiveConnection>` [Source: epics.md#Story-2.1-AC4]

---

### AC5: Lobby Consistency Under Concurrent Operations

**Given** multiple users are joining and leaving the lobby simultaneously  
**When** concurrent add/remove operations occur  
**Then** the lobby state remains consistent with no race conditions  
**And** no ghost users remain after disconnection  
**And** no duplicate users appear after reconnection  
**And** all clients receive consistent lobby state eventually  

**Implementation Detail:** Thread-safe data structure (`Arc<RwLock<HashMap>>`) with atomic operations [Source: architecture.md#Implementation-Patterns]

---

## Tasks / Subtasks

### Task 1: Define Lobby Data Structure (AC1, AC2, AC5)
- [ ] Create `server/src/lobby/state.rs` with:
  - [ ] `type PublicKey = String;` type alias
  - [ ] `pub struct ActiveConnection { websocket_sender: mpsc::UnboundedSender<Message>, ... }`
  - [ ] `type Lobby = Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>;`
  - [ ] Add inline unit tests for struct construction
- [ ] Implement thread-safe access pattern using `Arc<RwLock<T>>`
- [ ] Ensure no Clone issues with WebSocket sender

### Task 2: Implement Lobby Add Operation (AC1, AC2)
- [ ] Create `server/src/lobby/manager.rs` with:
  - [ ] `pub async fn add_user(lobby: &Lobby, key: PublicKey, conn: ActiveConnection) -> Result<()>`
  - [ ] Check for existing user (reconnection case from AC2)
  - [ ] If exists: broadcast leave for old, then add new
  - [ ] If new: simply add to HashMap
  - [ ] Call broadcast helper to notify all other users
  - [ ] Add unit tests for add operation
- [ ] Integrate with `server/src/connection/auth.rs` to call after signature validation succeeds
- [ ] Handle errors gracefully (malformed keys, etc.)

### Task 3: Implement Lobby Remove Operation (AC3, AC5)
- [ ] Create `server/src/lobby/manager.rs` (continued):
  - [ ] `pub async fn remove_user(lobby: &Lobby, key: &PublicKey) -> Result<()>`
  - [ ] Remove entry from HashMap
  - [ ] Call broadcast helper to notify all remaining users
  - [ ] Handle errors (key not found, etc.)
  - [ ] Add unit tests for remove operation
- [ ] Integrate with `server/src/connection/handler.rs` message loop:
  - [ ] **CRITICAL:** Capture `Message::Close` frame (don't ignore it)
  - [ ] Call `lobby.remove_user(&public_key)` on close
  - [ ] Prevents ghost users from remaining in lobby
  - [ ] Add comment: "Close frame handling required by story 1.6 learnings"

### Task 4: Implement Lobby Query Operation (AC4)
- [ ] Create in `server/src/lobby/manager.rs`:
  - [ ] `pub async fn get_user(lobby: &Lobby, key: &PublicKey) -> Option<Arc<ActiveConnection>>`
  - [ ] Used by story 3.2 for message routing (recipient online check)
  - [ ] Add unit tests for query operation
- [ ] Create snapshot method for story 2.2:
  - [ ] `pub async fn get_current_users(lobby: &Lobby) -> Vec<PublicKey>`
  - [ ] Returns list of all currently online public keys
  - [ ] Used by story 2.2 for initial lobby display

### Task 5: Implement Broadcast Helpers (AC1, AC2, AC3)
- [ ] Create in `server/src/lobby/manager.rs`:
  - [ ] `pub async fn broadcast_user_joined(lobby: &Lobby, key: &PublicKey)`
  - [ ] Sends `{type: "lobby_update", joined: [{publicKey: "..."}]}` to all others
  - [ ] `pub async fn broadcast_user_left(lobby: &Lobby, key: &PublicKey)`
  - [ ] Sends `{type: "lobby_update", left: [{publicKey: "..."}]}` to all others
  - [ ] Send delta (only changed users), not full lobby each time
  - [ ] Add unit tests for broadcast functions

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
  - [ ] `test_add_user_new_entry`
  - [ ] `test_add_user_reconnection_replaces`
  - [ ] `test_remove_user_deletes_entry`
  - [ ] `test_remove_nonexistent_user_safe`
  - [ ] `test_get_user_returns_existing`
  - [ ] `test_get_user_returns_none_for_missing`
  - [ ] `test_concurrent_add_remove_safe` (10 concurrent operations)
- [ ] Integration tests in `server/tests/lobby_sync.rs`:
  - [ ] `test_multiple_clients_lobby_consistency`
  - [ ] `test_user_add_triggers_broadcast`
  - [ ] `test_user_remove_triggers_broadcast`
  - [ ] `test_reconnection_updates_not_duplicates`
- [ ] E2E test in `server/tests/integration_multiclient.rs` (from architecture.md):
  - [ ] Spawn real server
  - [ ] Spawn 3 real client processes with different keys
  - [ ] All authenticate → verify all appear in each other's lobby
  - [ ] One disconnects → verify removed from all lobbies
  - [ ] One reconnects → verify not duplicated

---

## Dev Notes

### Architecture Patterns & Constraints

**Primary Pattern: Thread-Safe Shared State** [Source: architecture.md#Pattern-5-State-Management]

```rust
// server/src/lobby/state.rs
pub type PublicKey = String;

pub struct ActiveConnection {
    public_key: PublicKey,
    sender: mpsc::UnboundedSender<Message>,
    // For future: last_activity timestamp for Phase 2 timeouts
}

pub type Lobby = Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>;
```

**Why this pattern:**
- `Arc` = multiple threads can hold reference to same lobby
- `RwLock` = multiple readers (queries) can run simultaneously, exclusive writer (add/remove)
- `HashMap` = O(1) lookup for message routing (story 3.2 critical path)
- `mpsc::UnboundedSender` = efficient broadcast to specific client

**Do NOT use:**
- ❌ Flags or booleans for state (use enums) [Source: architecture.md#Anti-Pattern]
- ❌ Direct `Mutex` if you have many readers (use `RwLock`)
- ❌ `Vec` for user list (use HashMap for O(1) lookup)
- ❌ Polling for lobby changes (use push notifications)

### Source Tree Components to Modify

**Files to Create:**
- `server/src/lobby/state.rs` - Data structures (ActiveConnection, Lobby type)
- `server/src/lobby/manager.rs` - Operations (add, remove, get, broadcast)

**Files to Modify:**
- `server/src/connection/auth.rs` - Call `lobby.add_user()` after signature validation (story 1.5 integration)
- `server/src/connection/handler.rs` - Capture `Message::Close` and call `lobby.remove_user()`
- `server/src/connection/manager.rs` - Ensure lobby is passed to all connection handlers

**Files to Reference (Don't Modify):**
- `shared/src/protocol/types.rs` - Message definitions for `lobby_update` format
- `shared/src/errors/` - Error type definitions

### Testing Standards Summary

**Concurrency Testing:** [Source: architecture.md#Testing-Architecture]

This story MUST handle concurrent operations safely. Test with:
- 10+ simultaneous add operations
- 10+ simultaneous remove operations  
- Mix of add/remove/query operations
- No race conditions, no panics, no data corruption

**Determinism Testing:**

While this story doesn't involve cryptography, ensure:
- Same sequence of operations → same final lobby state (always)
- Multiple clients see consistent lobby state (eventually)

**Edge Cases:**

Test these specific scenarios:
- User authenticates, immediately disconnects (close frame) → removed
- User authenticates twice from different connections → replaces, not duplicates
- 100+ rapid joins/leaves → no ghost users, no duplicates
- Query during add/remove operations → consistent results

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
- Types: `ActiveConnection`, `Lobby`, `PublicKey`
- Variables: `public_key`, `ws_sender`, `lobby_state`

**No Conflicts:** This is new functionality (no existing files to refactor)

### Critical Implementation Warnings

#### ⚠️ WARNING 1: WebSocket Close Frame Handling

**WRONG:** Ignoring close frames
```rust
match msg {
    Message::Text(text) => { /* handle */ },
    _ => { /* ignore close frames */ }  // ❌ GHOST USERS!
}
```

**CORRECT:** Capture close frames
```rust
match msg {
    Message::Text(text) => { /* process */ },
    Message::Close(frame) => {
        lobby.remove_user(&public_key).await?;
        return Ok(()); // Exit connection handler cleanly
    },
    _ => {}
}
```

**Where:** `server/src/connection/handler.rs` message loop  
**Why:** Missing this creates ghost users that can't receive messages (AC3 failure)

---

#### ⚠️ WARNING 2: Reconnection Race Condition

**WRONG:** Blindly adding without checking duplicates
```rust
lobby.insert(public_key.clone(), new_connection);  // ❌ Could be duplicate
```

**CORRECT:** Handle reconnection explicitly (AC2)
```rust
if let Some(old_conn) = lobby.remove(&public_key) {
    // Old user is being replaced
    // Broadcast leave for old connection
    broadcast_user_left(&lobby, &public_key).await;
}
// Now add the new connection
lobby.insert(public_key.clone(), new_connection);
broadcast_user_joined(&lobby, &public_key).await;
```

**Where:** `server/src/lobby/manager.rs::add_user()`  
**Why:** Without this, rapid reconnects could create duplicates or race conditions (AC2, AC5 failure)

---

#### ⚠️ WARNING 3: Delta Broadcasts (Not Full Lobby)

**WRONG:** Sending entire lobby on every change
```rust
async fn broadcast_lobby_update(lobby: &Lobby) {
    let all_users: Vec<_> = lobby.read().await.keys().collect();
    // Send all 1000 users to all 1000 clients
    // = 1,000,000 user entries transmitted per update
    send_to_all_clients(all_users).await;  // ❌ SCALES TERRIBLY
}
```

**CORRECT:** Send only changed users (delta)
```rust
async fn broadcast_user_joined(lobby: &Lobby, joined_key: &PublicKey) {
    let update = json!({
        "type": "lobby_update",
        "joined": [{"publicKey": joined_key}]
    });
    // Send only the new user, not all 1000
    send_to_all_clients(update).await;  // ✅ EFFICIENT
}
```

**Where:** `server/src/lobby/manager.rs::broadcast_*()` functions  
**Why:** Delta updates scale to thousands of users (AC1, AC3 efficiency)

### References

- **Functional Requirements:** [Source: epics.md#Story-2.1-Server-Maintains-Active-User-Lobby] lines 593-628
- **Architecture Decisions:** [Source: architecture.md#Decision-2-Server-Side-Validation-Routing] lines 301-333 (lobby state management)
- **Component Boundaries:** [Source: architecture.md#Component-Boundaries-Server-Architecture] lines 1287-1307 (Lobby Manager component)
- **Project Structure:** [Source: architecture.md#Project-Structure-Boundaries] lines 1106-1109 (lobby/ directory structure)
- **Implementation Patterns:** [Source: architecture.md#Pattern-5-State-Management] lines 850-925 (enum-based state, thread safety)
- **Previous Stories:** [Source: 1-5-authenticate-to-server-with-signature-proof.md] (integration point), [Source: 1-6-handle-authentication-failure-disconnection.md#Critical-Implementation-Warning-1] (close frame handling)
- **Data Format:** [Source: architecture.md#WebSocket-Protocol-Definition] lines 366-418 (lobby message format)

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
