# Story 2.4: Broadcast User Leave Notifications

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **server**,
I want to **notify all connected users when a user disconnects from the lobby**,
so that **everyone knows immediately when someone is no longer available to message**.

## Acceptance Criteria

**Epic Context:** This story is part of Epic 2: Presence - Online Lobby & Real-Time Updates, which enables users to see who's online and receive real-time presence updates.

**Story Foundation** [Source: /home/riddler/profile/_bmad-output/epics.md#L717-L756]:

**Given** a user's WebSocket connection closes (intentional or network failure)
**When** the server detects the disconnection
**Then** the server removes their entry from the lobby
**And** broadcasts to all remaining users: `{type: "lobby_update", left: [{publicKey: "..."}]}`
**And** the message includes only the departed users (delta, not full list)
**And** is delivered within 100ms of user disconnection

**Given** a client receives a lobby_update with left users
**When** the update arrives
**Then** the client removes these users from its lobby display
**And** the Lobby component re-renders to show available users
**And** a brief notification appears: "User [key] left" (optional visual feedback)

**Given** the departing user selected another user to message
**When** they disconnect
**Then** the recipient is notified they left (via lobby update)
**And** any pending messages to that user are handled appropriately (offline notification)

**Given** a user is currently messaging someone
**When** that person disconnects
**Then** the lobby reflects the disconnection immediately
**And** if the user tries to send another message, they receive "recipient offline" notification

**Technical Implementation Requirements** [Source: /home/riddler/profile/_bmad-output/epics.md#L717-L756]:
- Connection drop detection: WebSocket close handler
- Lobby update: broadcast to remaining connections
- Notification content: include public keys of departed users
- Delivery: within 100ms of detected disconnection
- State consistency: server is single source of truth

**Related FRs:** FR32 (User leave notifications), FR33 (Real-time lobby updates) [Source: /home/riddler/profile/_bmad-output/epics.md#L66-L73]

## Developer Context Section - CRITICAL IMPLEMENTATION GUIDE

**🔥 CRITICAL MISSION:** This story ensures users are immediately notified when others disconnect, preventing confusion and enabling proper offline messaging behavior. This completes the real-time presence system with both join AND leave notifications.

### **Technical Specifications** [Source: architecture.md#L71-L76]

**Core Technology Stack:**
- **Language:** Rust (confirmed from Epic 1)
- **Async Runtime:** Tokio 1.48.0 (latest stable)
- **WebSocket:** tokio-tungstenite 0.28.0 (latest stable)
- **Cryptography:** ed25519-dalek 2.2.0 (latest stable, audited)
- **Serialization:** serde 1.0.228 + serde_json 1.0
- **Concurrency Pattern:** `Arc<RwLock<HashMap<PublicKey, ActiveConnection>>>` (from Story 2.1)

**Critical Dependencies from Epic 2 Stories:**
- ✅ **Story 2.1 Complete:** Server lobby data structure `HashMap<PublicKey, ActiveConnection>` working
- ✅ **Story 2.1 Complete:** `broadcast_user_left()` function exists in `server/src/lobby/manager.rs` (lines 153-183)
- ✅ **Story 2.2 Complete:** Client lobby display and state management working
- ✅ **Story 2.2 Complete:** Lobby update protocol `LobbyUpdateMessage` with `joined`/`left` fields defined
- ✅ **Story 2.2 Complete:** Client `LobbyState.remove_user()` method implemented (lines 213-217 in lobby_state.rs)
- ✅ **Story 2.2 Complete:** WebSocket message handler for `lobby_update` type exists

### **Architecture & Implementation Guide**

**Server Structure (from previous stories):**
- **Main server:** `profile-root/server/src/main.rs`
- **Connection handler:** `profile-root/server/src/connection/handler.rs` (WebSocket loop)
- **Lobby manager:** `profile-root/server/src/lobby/manager.rs` (broadcast functions already exist)
- **Lobby state:** `profile-root/server/src/lobby/state.rs` (data structures)

**Existing Broadcast Functionality (Story 2.1 - ALREADY IMPLEMENTED):**
```rust
// server/src/lobby/manager.rs:153-183
pub async fn broadcast_user_left(
    &self,
    left_users: Vec<String>
) -> Result<(), LobbyError> {
    // Create lobby update message
    let update = LobbyUpdateMessage {
        r#type: "lobby_update".to_string(),
        joined: vec![],
        left: left_users,
    };

    // Serialize to JSON
    let message = serde_json::to_string(&update)?;

    // Broadcast to all connections EXCEPT leaving users
    let mut lobby = self.lobby.write().await;
    for (public_key, connection) in lobby.iter() {
        // Filter: don't send to users who are leaving
        if !update.left.contains(public_key) {
            let _ = connection.send_message(&message).await;
        }
    }

    Ok(())
}
```

**Client-Side Leave Handling (Story 2.2 - ALREADY IMPLEMENTED):**
```rust
// client/src/connection/client.rs:129-138
"LobbyUpdate::UsersLeft(left_users)" => {
    for left_key in left_users {
        let was_selected = lobby_state.selected_user() == Some(left_key.as_str());
        lobby_state.remove_user(&left_key);

        // If selected user left, notify and clear selection
        if was_selected {
            ui.show_notification(&format!("User {} disconnected", &left_key[0..8]));
            ui.clear_recipient_selection();
        }
    }
},
```

**What This Story Adds:**

This story primarily integrates **existing broadcast functionality** with WebSocket close detection in the connection handler. The broadcast functions already exist from Story 2.1, and client-side handling already exists from Story 2.2. This story ensures they're properly wired together.

**Key Integration Points:**

1. **Server-Side:** Call `broadcast_user_left()` when WebSocket connection closes
   - Location: `server/src/connection/handler.rs` WebSocket message loop
   - Trigger: `Message::Close` frame detected
   - Action: Call `lobby_manager.broadcast_user_left(vec![public_key])`

2. **Client-Side:** Leave handling already wired (Story 2.2)
   - `parse_lobby_message()` handles `lobby_update` with `left` field
   - `LobbyState.remove_user()` removes from display
   - Selected user cleared automatically when they leave

3. **Offline Messaging:** Already handled by Epic 3 Story 3.6
   - When user tries to message offline recipient, server sends `offline` notification
   - No changes needed in this story (Epic 3 handles it)

### **File Structure & Patterns**

**Core Files to Modify:**
1. `profile-root/server/src/connection/handler.rs` - Add broadcast call on WebSocket close
2. `server/tests/leave_notification_tests.rs` - NEW: Leave notification integration tests

**Pattern Consistency Requirements:**
- Follow established error handling from Epic 1
- Use same tracing patterns for logging
- Maintain consistent code organization from previous stories
- Preserve existing module structure in `/src/connection/` and `/src/lobby/`

### **Testing Strategy** [Source: Story 2.1, 2.2 patterns]

**Integration Test Coverage (5+ tests required):**
1. `test_single_leave_broadcast` - Verify broadcast sent to all remaining users
2. `test_leaving_user_excluded_from_broadcast` - Verify leaving user doesn't receive own notification
3. `test_selected_user_clears_on_leave` - Verify selection cleared when recipient leaves
4. `test_multiple_leaves_consistency` - Verify lobby state consistent after multiple leaves
5. `test_connection_drop_cleanup` - Verify lobby cleanup on abrupt disconnection

**Unit Test Coverage (3+ tests required):**
1. `test_broadcast_user_left_excludes_self` - Unit test for manager.rs broadcast function
2. `test_left_users_format_correctness` - Verify JSON format matches protocol
3. `test_multiple_leaves_batched` - Verify multiple left users in single broadcast

**Test Location & Standards:**
- Create `server/tests/leave_notification_tests.rs` for leave-specific tests
- Extend existing `server/tests/lobby_integration.rs` tests (already has basic leave tests)
- Follow Epic 2 test patterns (async tokio tests, mock WebSocket)
- Use same test utilities from Story 2.1 and 2.2

### **Anti-Pattern Prevention**

**Common Mistakes to Avoid:**
1. **Sending to Leaving User:** Never broadcast leave notification to the user who is leaving (already handled by existing code)
2. **Missing Selection Clear:** Always clear selection if selected user leaves (already handled by Story 2.2)
3. **Race Conditions:** Ensure broadcast happens after lobby removal (already handled by Story 2.1)
4. **Duplicate Removals:** Don't try to remove users multiple times (already handled by Story 2.1's `remove_user`)
5. **Incomplete Broadcast:** Send ALL users who left, not just first user (already handled by Vec-based approach)

**Precedent from Epic 2:**
- Story 2.1 established lobby cleanup and broadcast patterns
- Story 2.2 established client-side lobby update handling
- Story 2.3 confirmed join broadcasts work correctly
- Follow the same patterns for leave broadcasts

### **Cross-Story Dependency Map**

**Dependencies:**
- **Depends On:** Stories 2.1 (lobby), 2.2 (display), 2.3 (join broadcasts) complete ✅
- **Required For:**
  - **Story 2.5 (Real-Time Sync):** Requires leave notifications to sync lobby state
  - **Story 3.6 (Offline Notifications):** Requires leave detection to know when recipient is offline

**Interface Contracts:**
- Must call `broadcast_user_left()` when WebSocket closes
- Client already handles `LobbyUpdate::UsersLeft` messages
- Server must ensure lobby removal happens before broadcast (already implemented)
- Client must clear selection when selected user leaves (already implemented)

**Protocol Contract (from Story 2.2):**
```rust
// server/src/protocol/mod.rs - ALREADY DEFINED
#[derive(Serialize, Deserialize, Debug)]
pub struct LobbyUpdateMessage {
    pub r#type: String,           // "lobby_update"
    pub joined: Vec<LobbyUserCompact>,  // Users who joined
    pub left: Vec<String>,             // Public keys of departed users
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LobbyUserCompact {
    pub public_key: String,  // Hex-encoded public key (no status field - always "online")
}
```

### **Success Criteria & Completion Status**

**Success Criteria:**
- Server broadcasts leave notification when WebSocket connection closes
- All remaining users receive notification (except leaving user)
- Client removes departed users from lobby display
- Selected recipient cleared automatically when they leave
- Lobby state remains consistent with server truth
- <100ms delivery from disconnect to broadcast

**Implementation Phases:**
1. **Phase 1:** Verify `broadcast_user_left()` function exists (already in Story 2.1)
2. **Phase 2:** Integrate broadcast call in WebSocket close handler (MODIFY handler.rs)
3. **Phase 3:** Add comprehensive testing for leave scenarios
4. **Phase 4:** Verify client-side leave handling works (already in Story 2.2)

**Ready for Development:** ✅ All requirements analyzed, architecture reviewed, and implementation guide provided. The developer has comprehensive context for flawless implementation.

**Status:** ready-for-dev
**Next Steps:** Proceed to Epic 2.5 (Real-Time Lobby Synchronization) or run `dev-story` for additional stories

## Tasks / Subtasks

### **Task 1: Integrate Broadcast with WebSocket Close Handler** (AC: #1, Technical Requirements)
- [ ] **1.1** Locate WebSocket message loop in `server/src/connection/handler.rs`
- [ ] **1.2** Find `Message::Close` case in message loop
- [ ] **1.3** Add call to `lobby_manager.broadcast_user_left(vec![public_key])` after lobby cleanup
- [ ] **1.4** Ensure public_key is hex-encoded String (not bytes) for broadcast
- [ ] **1.5** Add tracing log: `info!("User {} disconnected, broadcasting leave notification", public_key)`

### **Task 2: Verify Client-Side Leave Handling** (AC: #2, #3)
- [ ] **2.1** Verify `parse_lobby_message()` handles `lobby_update` with `left` field (already in Story 2.2)
- [ ] **2.2** Verify `LobbyState.remove_user()` removes user from display (already in Story 2.2)
- [ ] **2.3** Verify selection cleared when selected user leaves (already in Story 2.2)
- [ ] **2.4** Verify lobby re-renders after user removed (already in Story 2.2)
- [ ] **2.5** No client-side changes needed (Story 2.2 already handles all leave scenarios)

### **Task 3: Edge Case - Selected User Leaves** (AC: #3, #5)
- [ ] **3.1** Verify client shows notification when selected user leaves (already in Story 2.2)
- [ ] **3.2** Verify composer disabled when no user selected (already in Story 2.2)
- [ ] **3.3** Verify user can select different recipient after previous selection leaves
- [ ] **3.4** No code changes needed (already handled by Story 2.2's selection clearing)

### **Task 4: Edge Case - Multiple Users Leave** (AC: #4)
- [ ] **4.1** Test scenario: 3 users online, 2 disconnect simultaneously
- [ ] **4.2** Verify single broadcast contains both departed users
- [ ] **4.3** Verify client removes both users from display
- [ ] **4.4** Verify lobby state remains consistent (no ghost users)
- [ ] **4.5** Already handled by `broadcast_user_left(Vec<String>)` taking multiple users

### **Task 5: Create Comprehensive Test Suite**
- [ ] **5.1** Create `server/tests/leave_notification_tests.rs` with integration tests
- [ ] **5.2** Add `test_single_leave_broadcast` - Verify all remaining users receive notification
- [ ] **5.3** Add `test_leaving_user_excluded_from_broadcast` - Verify self-exclusion works
- [ ] **5.4** Add `test_selected_user_clears_on_leave` - Verify selection clearing
- [ ] **5.5** Add `test_multiple_leaves_consistency` - Verify multiple leaves handled correctly
- [ ] **5.6** Add `test_connection_drop_cleanup` - Verify cleanup on abrupt disconnect
- [ ] **5.7** Ensure all tests pass (target: 5+ tests, 100% passing)

## Dev Notes

### **Source Citations & Requirements Traceability**
- **Story Foundation:** Requirements from `epics.md` lines 717-756 [Source: /home/riddler/profile/_bmad-output/epics.md#L717-L756]
- **Functional Requirements:** FR32 (User leave notifications), FR33 (Real-time lobby updates) [Source: /home/riddler/profile/_bmad-output/epics.md#L66-L73]
- **Architecture Constraints:** Delta update format, tokio broadcast for efficient multi-recipient sending [Source: /home/riddler/profile/_bmad-output/architecture.md#L1268-L1283]
- **Technical Stack:** Rust + Tokio + WebSocket + serde_json [Source: /home/riddler/profile/_bmad-output/architecture.md#L71-L76]
- **Performance Requirements:** Lobby updates <100ms propagation [Source: /home/riddler/profile/_bmad-output/architecture.md#L49-L54]

### **Git History Intelligence**

**Recent Commit Patterns (Epic 2 Stories):**
- **Feature Commits:** `feat(server): add lobby cleanup on client disconnect` (Story 2.1 pattern)
- **Feature Commits:** `feat(client): implement lobby display and updates` (Story 2.2 pattern)
- **Test Coverage:** Each story adds 20-30 integration tests in `/server/tests/`
- **Code Organization:** Modular structure in `/server/src/connection/`, `/server/src/lobby/`
- **Error Handling:** Consistent use of `tracing` for logging, proper cleanup in handlers
- **File Modification Focus:** Lobby management and message handling

**Established Patterns to Follow:**
```rust
// From Story 2.1 - Lobby cleanup pattern (handler.rs:145-147)
match msg {
    Message::Text(text) => { /* auth/message handling */ },
    Message::Close(frame) => {
        let reason = frame.as_ref()
            .map(|f| f.reason.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        // Lobby cleanup (already implemented)
        lobby.remove_user(&public_key).await;

        // NEW: Broadcast leave notification
        if let Err(e) = lobby_manager.broadcast_user_left(vec![public_key.clone()]).await {
            tracing::error!("Failed to broadcast leave notification: {}", e);
        }

        return Err(format!("Connection closed: {}", reason).into());
    },
    _ => { /* handle other message types */ }
}
```

```rust
// From Story 2.2 - Client-side leave handling (client.rs:129-138)
"LobbyUpdate::UsersLeft(left_users)" => {
    for left_key in left_users {
        let was_selected = lobby_state.selected_user() == Some(left_key.as_str());
        lobby_state.remove_user(&left_key);

        // If selected user left, notify and clear selection
        if was_selected {
            ui.show_notification(&format!("User {} disconnected", &left_key[0..8]));
            ui.clear_recipient_selection();
        }
    }
},
```

### **Concrete Testing Examples**

**Integration Test Template:**
```rust
// server/tests/leave_notification_tests.rs
#[tokio::test]
async fn test_single_leave_broadcast() {
    // Setup
    let (tx, _rx) = broadcast::channel(10);
    let manager = LobbyManager::new(tx.clone());

    // Create 3 users
    let user1_key = "aaaa1111222233334444555566667777".to_string();
    let user2_key = "bbbb1111222233334444555566667777".to_string();
    let user3_key = "cccc1111222233334444555566667777".to_string();

    // Add all to lobby
    manager.add_user(user1_key.clone(), mock_connection()).await;
    manager.add_user(user2_key.clone(), mock_connection()).await;
    manager.add_user(user3_key.clone(), mock_connection()).await;

    // User2 disconnects
    manager.remove_user(&user2_key).await;

    // Broadcast leave notification
    manager.broadcast_user_left(vec![user2_key.clone()]).await.unwrap();

    // Assert: User1 and User3 received notification
    // Assert: User2 did NOT receive own notification
}

#[tokio::test]
async fn test_leaving_user_excluded_from_broadcast() {
    // Similar to test_single_leave_broadcast
    // Explicitly verify leaving user doesn't receive own notification
}

#[tokio::test]
async fn test_selected_user_clears_on_leave() {
    // Setup client with selected user
    let lobby_state = LobbyState::new();
    lobby_state.set_users(vec![
        LobbyUser { public_key: "aaa111...".to_string(), is_online: true }
    ]);
    lobby_state.select(&"aaa111...");

    // Simulate leave notification
    lobby_state.remove_user(&"aaa111...");

    // Assert: Selection cleared
    assert!(lobby_state.selected_user().is_none());
}
```

### **Cross-Story Dependency Map**

**This Story Dependencies:**
- **Depends On:** Stories 2.1 (lobby), 2.2 (display), 2.3 (join broadcasts) complete ✅
- **Required For:**
  - **Story 2.5 (Real-Time Sync):** Requires leave notifications for complete lobby synchronization
  - **Story 3.6 (Offline Notifications):** Requires leave detection to trigger offline messaging

**Interface Contracts:**
- Server must call `broadcast_user_left()` on WebSocket close
- Client already handles `LobbyUpdate::UsersLeft` messages (Story 2.2)
- Lobby state must be single source of truth (Story 2.1 pattern)
- Selection must clear when selected user leaves (Story 2.2 pattern)

### **Actionable Code Snippets**

**WebSocket Close Handler Integration:**
```rust
// server/src/connection/handler.rs - Modify Message::Close case
match msg {
    Message::Close(frame) => {
        let reason = frame.as_ref()
            .map(|f| f.reason.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        tracing::info!(
            "Connection closing for user {}, reason: {}",
            hex::encode(&public_key),
            reason
        );

        // Step 1: Remove from lobby (already implemented)
        let was_online = lobby.remove_user(&public_key).await;

        // Step 2: Broadcast leave notification if user was in lobby
        if was_online {
            let public_key_hex = hex::encode(&public_key);
            if let Err(e) = lobby_manager.broadcast_user_left(vec![public_key_hex]).await {
                tracing::error!("Failed to broadcast leave notification: {}", e);
            }
        }

        // Step 3: Send close frame
        self.inner.send(Message::Close(Some(frame))).await?;

        tracing::info!("Connection closed for {}", hex::encode(&public_key));
        return Err(format!("Connection closed: {}", reason).into());
    },
    _ => { /* handle other message types */ }
}
```

**Public Key Encoding Note:**
- Server stores public keys as hex-encoded `String` in lobby (Story 2.1 technical debt decision)
- `broadcast_user_left()` expects `Vec<String>` of hex-encoded keys
- Handler must call `hex::encode(&public_key)` before broadcasting
- This is consistent with Story 2.1's accepted technical debt

### **Project Structure Guidance**

**Files to Modify:**
1. **`server/src/connection/handler.rs`** - Add broadcast call in WebSocket close handler (MODIFY Message::Close case)
2. **`server/tests/leave_notification_tests.rs`** - NEW: Leave notification tests

**Directory Structure:**
```
profile-root/
├── server/
│   ├── src/
│   │   ├── connection/
│   │   │   └── handler.rs           # ← MODIFY: Add broadcast call
│   │   └── lobby/
│   │       ├── manager.rs            # ← Uses existing broadcast_user_left()
│   │       └── state.rs              # ← Uses existing remove_user()
│   └── tests/
│       ├── lobby_integration.rs         # ← Has basic leave tests
│       └── leave_notification_tests.rs  # ← NEW: Dedicated leave tests
└── client/
    ├── src/
    │   ├── connection/
    │   │   └── client.rs            # ← Already handles leave notifications
    │   └── ui/
    │       └── lobby_state.rs         # ← Already removes users on leave
```

**Naming Conventions:**
- Use `snake_case` for functions/variables (consistent with Rust conventions)
- Use `PascalCase` for types/structs
- Test functions: `test_<scenario>_<expected_outcome>`
- Error types: `<Module>Error` (e.g., LobbyError)

## Dev Agent Record

### Agent Model Used

MiniMax-M2.1

### Debug Log References

### Completion Notes List

### File List

**Core Implementation (Modified):**
- `profile-root/server/src/connection/handler.rs` - Add broadcast_user_left() call in WebSocket close handler

**Testing (New):**
- `profile-root/server/tests/leave_notification_tests.rs` - NEW: Leave notification integration tests

**Protocol (NOT Modified):**
- `profile-root/shared/src/protocol/mod.rs` - LobbyUpdateMessage already defined (Story 2.2)

**Already Implemented (from previous stories):**
- `profile-root/server/src/lobby/manager.rs` - broadcast_user_left() function (lines 153-183)
- `profile-root/server/src/lobby/state.rs` - remove_user() function (already exists)
- `profile-root/client/src/connection/client.rs` - Client-side leave handling (lines 129-138)
- `profile-root/client/src/ui/lobby_state.rs` - remove_user() and selection clearing (lines 213-217)
- `profile-root/server/tests/lobby_integration.rs` - Basic leave tests already exist
