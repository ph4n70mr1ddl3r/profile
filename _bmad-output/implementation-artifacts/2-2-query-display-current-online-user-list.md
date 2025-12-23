# Story 2.2: Query & Display Current Online User List

Status: in-progress  # UI structure fixed, but Rust-side property binding still needed to populate lobby slots

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **user**,
I want to **see the list of all currently online users with their public keys**,
So that **I can choose who to send a message to**.

## Acceptance Criteria

**Epic Context:** This story is part of Epic 2: Presence - Online Lobby & Real-Time Updates, which enables users to see who's online and receive real-time presence updates.

**Story Foundation** [Source: /home/riddler/profile/_bmad-output/epics.md#L632-L673]:

**Given** a user is authenticated and connected
**When** the client loads or reconnects
**Then** the server immediately sends the current lobby state: `{type: "lobby", users: [{publicKey: "..."}, ...]}`
**And** the client receives this lobby snapshot
**And** displays a list of online users in the Lobby component

**Given** the lobby is displayed
**When** the user views the list
**Then** each user is shown with:
   - Public key in monospace, blue, full (not truncated)
   - Online indicator (● green dot)
   - Selection highlight (if selected, blue background)
**And** users can be scrolled if more than fit on screen
**And** the list shows "No users online" if lobby is empty

**Given** a user is viewing the lobby
**When** they click on a user
**Then** that user is selected (highlighted in blue)
**And** the chat area activates for messaging that user
**And** the composer field receives focus

**Keyboard Navigation (Tab + Arrow Keys):**
```rust
/// Tab focus handling:
/// - Tab from previous component → moves focus to first lobby item
/// - Tab from last lobby item → moves focus to next component (composer)
/// - Shift+Tab → reverse navigation
/// - Arrow keys: move selection up/down when lobby has focus
/// - Enter: confirms selection when lobby item has focus
/// ```

**Given** multiple users are available
**When** the user uses keyboard navigation
**Then** arrow keys move selection up/down
**And** Enter key confirms selection

**Technical Implementation Requirements** [Source: /home/riddler/profile/_bmad-output/epics.md#L632-L673]:
- Component: `LobbyComponent` (custom Slint component)
- Data: Display `{publicKey: String, isOnline: bool}`
- Selection state: tracked separately from display
- Keyboard nav: standard Tab/Arrow keys, Enter to select
- Click handler: select user and activate chat for that user

**Related FRs:** FR26, FR27, FR28, FR29, FR30 [Source: /home/riddler/profile/_bmad-output/epics.md#L66-L73]

## Developer Context Section - CRITICAL IMPLEMENTATION GUIDE

**🔥 CRITICAL MISSION:** This story creates the client-side lobby display that users interact with to select message recipients. Get this wrong and users can't choose who to message, breaking the entire messaging flow.

### **Technical Specifications** [Source: architecture.md#L71-L76]

**Core Technology Stack:**
- **Language:** Rust (confirmed from Epic 1)
- **UI Framework:** Slint 1.5+ (latest stable)
- **Async Runtime:** Tokio 1.48.0 (for WebSocket client)
- **WebSocket:** tokio-tungstenite 0.28.0 (client)
- **Serialization:** serde 1.0.228 + serde_json 1.0

**Critical Dependencies from Epic 2 Story 2.1:**
- ✅ Server lobby data structure: `HashMap<PublicKey, ActiveConnection>` (Story 2.1 complete)
- ✅ Lobby broadcast protocol: `{type: "lobby", users: [{publicKey: "..."}]}`
- ✅ Server `lobby.get_all_users()` method available for sending initial snapshot
- ✅ Public key type: hex-encoded String (from Story 2.1, accepted technical debt)

### **Architecture & Implementation Guide**

**Client Structure (from previous stories):**
- **Main client:** `profile-root/client/src/main.rs`
- **Connection handler:** `profile-root/client/src/connection/client.rs` (WebSocket client)
- **Lobby component:** `profile-root/client/src/ui/lobby.rs` (NEW - this story)
- **Lobby state:** `profile-root/client/src/ui/lobby_state.rs` (NEW - this story)
- **UI state:** `profile-root/client/src/ui/state.rs` (manage lobby selection state)
- **UI handler:** `profile-root/client/src/ui/handler.rs` (connect UI to state)

**WebSocket Integration:**
- Extend existing `WebSocketStream<TcpStream>` handling from Epic 1
- Add lobby message handler in connection client
- Follow established message parsing patterns from Story 1.5

**Lobby Display Pattern (from UX Design):**
```rust
// Lobby component states from UX Design
enum LobbyItemState {
    Default,      // Idle, no user selected - Gray background, neutral text
    Hover,        // Mouse over user - Slightly elevated background (#374151)
    Selected,     // User is current recipient - Blue highlight (#0066CC), bold text
    Online,       // User indicator - Green dot (●) #22c55e
    Offline,      // User indicator - Gray dot (○) #6b7280
}
```

**Lobby Protocol (from Architecture Decision 4):**
```json
// Server sends on successful auth
{
  "type": "lobby",
  "users": [
    {"publicKey": "3a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb", "status": "online"},
    {"publicKey": "7b4d9c2a3e8f1d4c5a6b7e8f9a0b1c2d", "status": "online"}
  ]
}

// Lobby update (delta) - NOTE: joined users don't include status (always "online")
{
  "type": "lobby_update",
  "joined": [{"publicKey": "3a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb"}],
  "left": ["7b4d9c2a3e8f1d4c5a6b7e8f9a0b1c2d"]
}
```

**Error Handling for Lobby Messages:**
```rust
/// Lobby error handling scenarios:
/// 1. Malformed JSON: Log error with details, show "Connection error" toast to user
/// 2. Timeout: Retry lobby request up to 3 times with exponential backoff
/// 3. Disconnect during sync: Show "Reconnecting..." state, auto-retry on reconnect
/// 4. Duplicate keys: Deduplicate users by public_key before rendering
/// 5. Empty users array: Display "No users online" message (not an error)
/// 6. Invalid status field: Default to "online", log warning for debugging
```

**Performance Requirements (from PRD):**
```rust
/// Performance targets:
/// - Initial lobby render: < 200ms for 100+ users
/// - Delta update render (join/leave): < 50ms
/// - Selection response: < 16ms (60fps)
/// - Lobby update propagation: < 100ms end-to-end (from server broadcast to client render)
/// - Memory: < 1KB per lobby user (public_key string + bool)
```

**Scroll Behavior & Selection Interaction Specification:**
```rust
/// Scroll behavior for lobby:
/// - Maintain scroll position during delta updates (users joining/leaving)
/// - Auto-scroll to new users only if user was already at bottom
/// - No virtual scrolling needed for MVP (under 100 users typical)
/// - If lobby grows beyond viewport, ScrollView enables scrolling
///
/// Scroll-Selection Interaction:
/// - Selection does NOT change scroll position (user maintains their view)
/// - If selected item is scrolled out of view, NO auto-scroll (user can scroll manually)
/// - Selection highlight renders regardless of scroll position
/// - Click on off-screen item: scrolls item into view AND selects it
```

**Selection Edge Case Handling:**
```rust
/// Selection edge cases:
/// - Selected user leaves: Clear selection automatically, notify user "User disconnected"
/// - Empty lobby + keyboard nav: No-op (no selection change), optionally play subtle error sound
/// - Selection wrap: Yes (ArrowUp from first → last, ArrowDown from last → first)
/// - Rapid selection changes: Debounce chat composer activation (50ms)
```

**File Naming Convention (SINGLE AUTHORITATIVE CONVENTION):**
- `lobby.rs` - LobbyComponent UI implementation
- `lobby_state.rs` - Lobby state management (single source of truth)
- `lobby_item.slint` - Slint component for single lobby item

### **File Structure & Patterns**

**Core Files to Create/Modify:**
1. `profile-root/client/src/ui/lobby.rs` - NEW: LobbyComponent implementation
2. `profile-root/client/src/ui/lobby_state.rs` - NEW: Lobby state management
3. `profile-root/client/src/ui/state.rs` - MODIFY: Add lobby selection state management
4. `profile-root/client/src/connection/client.rs` - MODIFY: Handle lobby message types
5. `profile-root/client/ui/main.slint` - MODIFY: Add Lobby component to layout
6. `profile-root/client/src/ui/chat_screen.rs` - MODIFY: Integrate lobby with chat (verify screens/ subdirectory exists, otherwise use: `profile-root/client/src/ui/chat_screen.rs`)
7. `profile-root/client/tests/lobby_display_tests.rs` - NEW: Lobby display tests
8. `profile-root/client/tests/lobby_navigation_tests.rs` - NEW: Keyboard/mouse navigation tests

**Pattern Consistency Requirements:**
- Follow established error handling from Epic 1
- Use same tracing patterns for logging
- Maintain consistent code organization from previous stories
- Preserve existing module structure in `/src/ui/` and `/src/connection/`

**UI Design System (from UX Design):**

**Color Constants:**
```rust
// From UX Design color system
const LOBBY_ONLINE_INDICATOR: Color = Color::from_rgb_u32(0x22c55e);  // Green #22c55e
const LOBBY_OFFLINE_INDICATOR: Color = Color::from_rgb_u32(0x6b7280); // Gray #6b7280
const LOBBY_SELECTED_BG: Color = Color::from_rgb_u32(0x0066CC);       // Blue #0066CC
const LOBBY_KEY_COLOR: Color = Color::from_rgb_u32(0x0066CC);         // Blue #0066CC
const LOBBY_DEFAULT_BG: Color = Color::from_rgb_u32(0x111827);        // Surface dark
const LOBBY_HOVER_BG: Color = Color::from_rgb_u32(0x374151);          // Surface lighter
```

**Typography (from UX Design):**
- Public key: Monospace font (Consolas, Monaco, or platform default), 12px, blue color
- Online indicator: 12px, green (#22c55e) for online, gray (#6b7280) for offline
- Spacing: 8px base unit, 8px padding for user items, 8px gap between items

### **Testing Strategy** [Source: Story 2.1 patterns]

**Unit Test Coverage (8+ tests required):**
1. `test_lobby_component_renders_users` - Verify lobby displays all users from state
2. `test_lobby_empty_state` - Verify "No users online" message when lobby is empty
3. `test_lobby_keyboard_navigation` - Verify arrow keys move selection up/down
4. `test_lobby_mouse_selection` - Verify click selects user and activates chat
5. `test_lobby_online_indicator` - Verify green dot shown for online users
6. `test_lobby_selected_highlight` - Verify blue highlight when user is selected
7. `test_lobby_handles_malformed_json` - Verify malformed lobby message doesn't crash
8. `test_lobby_handles_duplicate_users` - Verify deduplication logic when same user appears twice

**Integration Test Coverage (5+ tests required):**
1. `test_lobby_receives_initial_state` - Verify client parses and displays initial lobby
2. `test_lobby_updates_on_join` - Verify lobby updates when user joins (from Story 2.3)
3. `test_lobby_updates_on_leave` - Verify lobby updates when user leaves (from Story 2.4)
4. `test_lobby_selection_activates_chat` - Verify selecting user enables chat composer
5. `test_lobby_key_copy` - Verify public key can be copied from lobby

**Test Location & Standards:**
- Create `profile-root/client/tests/lobby_display_tests.rs`
- Follow Epic 1 test patterns (integrated tests, mock server)
- Use same mocking and async testing patterns
- Include both unit and integration test coverage

### **Anti-Pattern Prevention**

**Common Mistakes to Avoid:**
1. **Truncated Keys:** Never show abbreviated public keys - always show full key
2. **Missing Online Indicator:** Always show green dot for online users
3. **Keyboard Incomplete:** Support both arrow keys AND Enter, not just one
4. **State Desync:** Lobby selection state must match actual selected recipient
5. **Empty State Missing:** Handle empty lobby with "No users online" message
6. **Focus Issues:** Composer should receive focus when user selects recipient

**Precedent from Previous Stories:**
- Story 2.1 established server lobby data structure (HashMap)
- Story 1.5 established WebSocket authentication flow
- Story 1.3 established public key display patterns (monospace, blue)

### **Cross-Story Dependency Map**

**Dependencies:**
- **Depends On:** Story 2.1 (server lobby) complete ✅
- **Required For:** All Epic 2 stories (2.3-2.5)
  - **2.3 (Join Notifications):** Requires lobby to display joined users
  - **2.4 (Leave Notifications):** Requires lobby to remove departed users
  - **2.5 (Real-Time Sync):** Requires lobby state management for updates

**Interface Contracts for Future Stories:**
- Must expose `lobby.selection()` for chat composer activation
- Must trigger `on_user_selected(public_key)` callback when user selects
- Must accept `lobby.update(users: Vec<LobbyUser>)` for initial state
- Must accept `lobby.add_user(public_key)` for join updates
- Must accept `lobby.remove_user(public_key)` for leave updates

**Client-Server Protocol (structs in shared/protocol/mod.rs - may need additions):**
```rust
// Protocol structs reference: profile-root/shared/src/protocol/mod.rs
// NOTE: LobbyUserCompact may need to be ADDED to shared/protocol/mod.rs if not present

// Existing structs (verify in shared/protocol/mod.rs):
#[derive(Serialize, Deserialize, Debug)]
pub struct LobbyMessage {
    pub r#type: String,  // "lobby"
    pub users: Vec<LobbyUser>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LobbyUser {
    pub public_key: String,  // Hex-encoded public key
    pub status: String,      // "online" (always "online" in initial lobby)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LobbyUpdateMessage {
    pub r#type: String,  // "lobby_update"
    pub joined: Vec<LobbyUserCompact>,  // Users who joined (no status field - always "online")
    pub left: Vec<String>,       // Public keys of departed users
}

/// Compact struct for lobby_update joined users (ADD to shared/protocol/mod.rs if not present)
#[derive(Serialize, Deserialize, Debug)]
pub struct LobbyUserCompact {
    pub public_key: String,  // Hex-encoded public key
}
```

### **Success Criteria & Completion Status**

**Success Criteria:**
- Client receives and parses initial lobby from server
- Lobby displays all online users with full public keys
- Green online indicator shown for each user
- Click selects user and activates chat composer
- Keyboard navigation (arrows + Enter) works correctly
- Empty lobby shows "No users online" message
- Public keys displayed in monospace, blue, untruncated

**Implementation Phases:**
1. **Phase 1:** Create LobbyComponent UI structure in Slint
2. **Phase 2:** Implement lobby state management (selection, display)
3. **Phase 3:** Handle WebSocket lobby messages from server
4. **Phase 4:** Implement keyboard/mouse selection
5. **Phase 5:** Integrate with chat composer activation
6. **Phase 6:** Add comprehensive testing

**Ready for Development:** ✅ All requirements analyzed, architecture reviewed, and implementation guide provided. The developer has comprehensive context for flawless implementation.

**Status:** ready-for-dev
**Next Steps:** Proceed to Epic 2.3 (Join Notifications) or run `dev-story` for additional stories

## Tasks / Subtasks

### **Task 1: Create LobbyComponent UI Structure** (AC: #1, #2, UI Requirements)
- [x] **1.1** Create `client/src/ui/lobby.rs` with LobbyComponent definition
- [x] **1.2** Define LobbyItem struct with public_key and selection state
- [x] **1.3** Create `client/src/ui/lobby_item.slint` - Slint component for single lobby item
- [x] **1.4** Implement LobbyComponent with ScrollView containing lobby items
- [x] **1.5** Handle empty lobby state with "No users online" message

### **Task 2: Implement Lobby State Management** (AC: #2, #3, State)
- [x] **2.1** Create `client/src/state/lobby.rs` with shared lobby state
- [x] **2.2** Add `lobby_users: Vec<LobbyUser>` field to UI state
- [x] **2.3** Add `selected_user: Option<String>` field for selection tracking
- [x] **2.4** Implement lobby state update methods (set_users, add_user, remove_user)
- [x] **2.5** Implement selection state management (select, deselect, is_selected)

### **Task 3: Handle WebSocket Lobby Messages** (AC: #1, Protocol Integration)
- [x] **3.1** Extend `client/src/connection/client.rs` to handle `lobby` message type
- [x] **3.2** Parse lobby message: `{type: "lobby", users: [{publicKey, status}]}`
- [x] **3.3** Call UI state update to populate lobby with received users
- [x] **3.4** Handle lobby message deserialization with serde_json
- [x] **3.5** Add error handling for malformed lobby messages
- [x] **3.6** Fix: Handle ALL users in lobby updates (not just first) - BUG FIX

### **Task 4: Implement Keyboard & Mouse Selection** (AC: #3, #4, Interaction)
- [x] **4.1** Add keyboard event handler for ArrowUp/ArrowDown in lobby
- [x] **4.2** Implement selection cycling (wrap around at top/bottom)
- [x] **4.3** Add Enter key handler to confirm selection
- [x] **4.4** Add mouse click handler for selecting users
- [x] **4.5** Update visual state (highlight) on selection change

### **Task 5: Integrate with Chat Composer** (AC: #3, Cross-Component)
- [x] **5.1** Add `on_user_selected` callback to LobbyComponent
- [x] **5.2** Trigger chat activation when user selects a recipient
- [x] **5.3** Focus composer field when user is selected
- [x] **5.4** Disable composer when no user is selected
- [x] **5.5** Update selection state across components

### **Task 6: Public Key Display & Styling** (AC: #2, UI Requirements)
- [x] **6.1** Display public key in monospace font (Consolas/Monaco)
- [x] **6.2** Apply blue color (#0066CC) to public key text
- [x] **6.3** Ensure public key is NOT truncated (full 64-char hex display)
- [x] **6.4** Add online indicator (green dot #22c55e) for each user
- [x] **6.5** Apply selection highlight (blue background #0066CC) when selected

### **Task 7: Create Comprehensive Test Suite**
- [x] **7.1** Create `client/tests/lobby_display_tests.rs` with unit tests
- [x] **7.2** Add `test_lobby_renders_users` verification
- [x] **7.3** Add `test_lobby_empty_state` verification
- [x] **7.4** Add `test_lobby_keyboard_navigation` verification
- [x] **7.5** Add `test_lobby_mouse_selection` verification
- [x] **7.6** Create `client/tests/lobby_integration_tests.rs` for integration tests
- [x] **7.7** Add `test_lobby_receives_initial_state` integration test
- [ ] **7.8** Ensure all tests pass (target: 10+ tests, 100% passing) - PENDING VERIFICATION

### **Review Follow-ups (AI)**
- [x] **FIX** Add `ui/mod.rs` to story File List
- [x] **FIX** Add `protocol/mod.rs` to story File List (was incorrectly in "Already Implemented")
- [x] **FIX** Create missing `lobby_integration_tests.rs` file
- [x] **FIX** Create `client/src/state/lobby.rs` for state integration
- [x] **FIX** Integrate lobby into `main.slint` with properties and callbacks
- [x] **FIX** Create `handlers/lobby.rs` for event handling
- [x] **BUG FIX** `parse_lobby_message` now handles ALL users in updates (not just first)
- [x] **CODE REVIEW FIX** Fixed CRITICAL issue: lobby list UI now actually renders LobbyItem components in main.slint (was just placeholder text)
- [x] **CODE REVIEW FIX** Removed unused `std::thread` import from client.rs:621
- [x] **CODE REVIEW FIX** Added `Default` trait implementation for `LobbyEventHandler`
- [x] **CODE REVIEW FIX** Fixed unnecessary reference borrowing in lobby_state.rs:344
- [x] **CODE REVIEW FIX** Updated story File List with correct file paths (fixed 10 discrepancies)
- [ ] **TODO - REMAINING** Implement Rust-side property binding for lobby slot properties (`lobby_user_1_public_key`, `lobby_user_1_online`, `lobby_user_1_selected`, etc.) in main.rs callback setup. Current Slint UI renders slots correctly but properties are not populated from Rust, so no users will be visible until binding is implemented.

## Dev Notes

### **Source Citations & Requirements Traceability**
- **Story Foundation:** Requirements from `epics.md` lines 632-673 [Source: /home/riddler/profile/_bmad-output/epics.md#L632-L673]
- **Functional Requirements:** FR26 (server maintains lobby), FR27 (user listing), FR28 (lobby query), FR29 (lobby display), FR30 (recipient selection) [Source: /home/riddler/profile/_bmad-output/epics.md#L66-L73]
- **UX Design:** LobbyComponent states, keyboard/mouse navigation [Source: /home/riddler/profile/_bmad-output/ux-design-specification.md#L502-L506]
- **UI States:** Lobby item states (default, hover, selected, online, offline) [Source: /home/riddler/profile/_bmad-output/ux-design-specification.md#L16413-L16449]
- **Architecture Protocol:** Lobby protocol definition [Source: /home/riddler/profile/_bmad-output/architecture.md#L399-L421]
- **Lobby Protocol:** Initial lobby + delta updates [Source: /home/riddler/profile/_bmad-output/architecture.md#L1268-L1283]

### **Git History Intelligence**
**Recent Commit Patterns (Story 2.1):**
- **Feature Commits:** `feat(server): add lobby cleanup on client disconnect`
- **Test Coverage:** Each story adds 20-30 integration tests in `/server/tests/`
- **Code Organization:** Modular structure in `/server/src/connection/`, `/server/src/lobby/`
- **Error Handling:** Consistent use of `tracing` for logging, proper cleanup in `Drop` impls
- **File Modification Focus:** Changes in `handler.rs` WebSocket message loop

**Established Patterns to Follow (Client Side):**
```rust
// From Story 1.3 - Public key display pattern
pub fn display_public_key(key: &PublicKey) -> Text {
    Text {
        text: hex::encode(key),
        font_family: "Consolas",
        font_size: 12,
        color: Color::from_rgb_u32(0x0066CC),  // Blue
    }
}

// From Story 1.5 - WebSocket message handling
match message.type_field.as_str() {
    "auth_success" => { /* handle auth success */ },
    "lobby" => { /* NEW: handle lobby state */ },
    "message" => { /* handle incoming message */ },
    _ => { tracing::warn!("Unknown message type: {}", message.type_field) }
}
```

### **Concrete Testing Examples**
**Unit Test Template:**
```rust
// client/tests/lobby_display_tests.rs
#[tokio::test]
async fn test_lobby_renders_users() {
    // Setup
    let lobby_state = LobbyState::new();
    let test_users = vec![
        LobbyUser { public_key: "3a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb".to_string(), is_online: true },
        LobbyUser { public_key: "7b4d9c2a3e8f1d4c5a6b7e8f9a0b1c2d".to_string(), is_online: true },
    ];
    
    // Set lobby users
    lobby_state.set_users(test_users);
    
    // Assert
    assert_eq!(lobby_state.users().len(), 2);
    assert!(lobby_state.users()[0].public_key.starts_with("3a8f"));
}

#[tokio::test]
async fn test_lobby_keyboard_navigation() {
    // Setup lobby with 3 users
    let lobby_state = LobbyState::new();
    let users = vec![
        LobbyUser { public_key: "aaaa1111222233334444555566667777".to_string(), is_online: true },
        LobbyUser { public_key: "bbbb1111222233334444555566667777".to_string(), is_online: true },
        LobbyUser { public_key: "cccc1111222233334444555566667777".to_string(), is_online: true },
    ];
    lobby_state.set_users(users);
    
    // Initial selection
    assert_eq!(lobby_state.selected_user(), None);
    
    // Press ArrowDown - should select first user
    lobby_state.handle_keydown(Key::ArrowDown);
    assert_eq!(lobby_state.selected_user(), Some("aaaa1111222233334444555566667777".to_string()));
    
    // Press ArrowDown again - should select second user
    lobby_state.handle_keydown(Key::ArrowDown);
    assert_eq!(lobby_state.selected_user(), Some("bbbb1111222233334444555566667777".to_string()));
    
    // Press ArrowUp - should go back to first user
    lobby_state.handle_keydown(Key::ArrowUp);
    assert_eq!(lobby_state.selected_user(), Some("aaaa1111222233334444555566667777".to_string()));
}

#[tokio::test]
async fn test_lobby_empty_state() {
    let lobby_state = LobbyState::new();
    lobby_state.set_users(vec![]);
    
    assert!(lobby_state.is_empty());
    // Verify empty state message is displayed
}
```

**Integration Test Template:**
```rust
// client/tests/lobby_integration_tests.rs
#[tokio::test]
async fn test_lobby_receives_initial_state() {
    // Setup mock server with lobby state
    let mock_server = MockServer::start().await;
    mock_server.set_lobby_state(vec![
        "3a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb".to_string(),
        "7b4d9c2a3e8f1d4c5a6b7e8f9a0b1c2d".to_string(),
    ]);
    
    // Connect client
    let client = TestClient::connect(mock_server.url()).await;
    
    // Authenticate
    client.authenticate().await;
    
    // Verify lobby message received
    let lobby_msg = client.receive_message().await;
    assert_eq!(lobby_msg.type_field, "lobby");
    assert_eq!(lobby_msg.users.len(), 2);
}
```

### **Cross-Story Dependency Map**
**This Story Dependencies:**
- **Depends On:** Story 2.1 (server lobby infrastructure)
- **Required For:** All Epic 2 stories (2.3-2.5)
  - **2.3 (Join Notifications):** Requires lobby to display joined users
  - **2.4 (Leave Notifications):** Requires lobby to remove departed users
  - **2.5 (Real-Time Sync):** Requires lobby state management for updates

**Interface Contracts:**
- Must expose `lobby.users()` for rendering users in UI
- Must expose `lobby.selected_user()` for chat activation
- Must expose `lobby.select(public_key)` for user selection
- Must expose `lobby.add_user(user)` for join updates
- Must expose `lobby.remove_user(public_key)` for leave updates

### **Actionable Code Snippets**
**Lobby Component (Slint):**
```slint
// client/ui/lobby_item.slint
component LobbyItem {
    in property <string> public_key;
    in property <bool> is_online;
    in property <bool> is_selected;

    Rectangle {
        background: is_selected ? #0066CC : (hover ? #374151 : #111827);
        height: 36px;

        // Online indicator
        Rectangle {
            x: 8px;
            y: 14px;
            width: 8px;
            height: 8px;
            border-radius: 4px;
            background: is_online ? #22c55e : #6b7280;
        }

        // Public key
        Text {
            x: 24px;
            y: 10px;
            text: public_key;
            font-family: "Consolas";
            font-size: 12px;
            color: is_selected ? #ffffff : #0066CC;
        }
    }

    // Click handler
    callback clicked(string);
}
```

**Lobby State Management (Rust - UPDATED):**
```rust
// client/src/ui/lobby_state.rs
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct LobbyState {
    users: HashMap<String, LobbyUser>,
    selected_user: Option<String>,
}

#[derive(Debug, Clone)]
pub struct LobbyUser {
    pub public_key: String,
    pub is_online: bool,
}

impl LobbyState {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            selected_user: None,
        }
    }

    /// Check if user exists in lobby (for deduplication)
    pub fn has_user(&self, public_key: &str) -> bool {
        self.users.contains_key(public_key)
    }

    pub fn set_users(&mut self, users: Vec<LobbyUser>) {
        self.users.clear();
        for user in users {
            // Deduplicate: only insert if not already present
            if !self.users.contains_key(&user.public_key) {
                self.users.insert(user.public_key.clone(), user);
            }
        }
    }

    pub fn add_user(&mut self, user: LobbyUser) {
        // Deduplicate before adding
        if !self.users.contains_key(&user.public_key) {
            self.users.insert(user.public_key.clone(), user);
        }
    }

    pub fn remove_user(&mut self, public_key: &str) -> bool {
        let was_present = self.users.remove(public_key).is_some();
        if self.selected_user.as_deref() == Some(public_key) {
            self.selected_user = None;
        }
        was_present
    }

    pub fn users(&self) -> Vec<&LobbyUser> {
        self.users.values().collect()
    }

    pub fn select(&mut self, public_key: &str) -> bool {
        if self.users.contains_key(public_key) {
            self.selected_user = Some(public_key.to_string());
            true
        } else {
            false
        }
    }

    pub fn selected_user(&self) -> Option<&str> {
        self.selected_user.as_deref()
    }

    pub fn clear_selection(&mut self) {
        self.selected_user = None;
    }

    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    pub fn len(&self) -> usize {
        self.users.len()
    }
}
```

**WebSocket Message Handler (UPDATED with error handling):**
```rust
// client/src/connection/client.rs
async fn handle_message(
    message: WebSocketMessage,
    state: &mut ClientState,
) -> Result<(), ClientError> {
    match message.type_field.as_str() {
        "auth_success" => {
            // Authentication successful, lobby will be sent next
            tracing::info!("Authentication successful");
        }
        "lobby" => {
            // Parse lobby message
            let lobby_msg: LobbyMessage = match serde_json::from_str(&message.json) {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("Failed to parse lobby message: {}", e);
                    // Show error to user, don't crash
                    state.ui.show_error("Connection error. Please retry.");
                    return Ok(());
                }
            };

            // Convert to LobbyUser structs
            let users: Vec<LobbyUser> = lobby_msg.users
                .into_iter()
                .map(|u| LobbyUser {
                    public_key: u.public_key,
                    is_online: u.status == "online",  // status always "online" in lobby message
                })
                .collect();

            // Update lobby state
            state.lobby.set_users(users);
            tracing::info!("Lobby updated with {} users", state.lobby.users().len());
        }
        "lobby_update" => {
            // Handle join/leave updates
            let update: LobbyUpdateMessage = match serde_json::from_str(&message.json) {
                Ok(msg) => msg,
                Err(e) => {
                    tracing::error!("Failed to parse lobby_update: {}", e);
                    return Ok(());
                }
            };

            // Handle joined users (no status field - always "online")
            for joined in update.joined {
                // Deduplicate: only add if not already present
                if !state.lobby.has_user(&joined.public_key) {
                    state.lobby.add_user(LobbyUser {
                        public_key: joined.public_key,
                        is_online: true,  // Joined users are always online
                    });
                }
            }

            // Handle left users
            for left_key in update.left {
                let was_selected = state.lobby.selected_user() == Some(left_key.as_str());
                state.lobby.remove_user(&left_key);

                // If selected user left, notify and clear selection
                if was_selected {
                    state.ui.show_notification(&format!("User {} disconnected", &left_key[0..8]));
                    state.ui.clear_recipient_selection();
                }
            }

            tracing::info!("Lobby updated: +{} -{}", update.joined.len(), update.left.len());
        }
        "message" => {
            // Handle incoming message
            handle_incoming_message(message, state).await?;
        }
        "notification" => {
            // Handle notifications (offline, etc.)
            handle_notification(message, state).await?;
        }
        "error" => {
            // Handle errors
            let error: ErrorMessage = match serde_json::from_str(&message.json) {
                Ok(e) => e,
                Err(_) => return Ok(()),
            };
            tracing::error!("Server error: {}", error.reason);
            state.ui.show_error(&error.reason);
        }
        _ => {
            tracing::warn!("Unknown message type: {}", message.type_field);
        }
    }
    Ok(())
}
```

### **Project Structure Guidance**
**Files to Create:**
1. **`client/src/ui/lobby.rs`** - NEW: LobbyComponent implementation
2. **`client/src/ui/lobby_state.rs`** - NEW: Lobby state management
3. **`client/ui/lobby_item.slint`** - NEW: Slint component definition
4. **`client/tests/lobby_display_tests.rs`** - NEW: Unit tests
5. **`client/tests/lobby_integration_tests.rs`** - NEW: Integration tests

**Files to Modify:**
1. **`client/src/ui/state.rs`** - Integrate lobby state
2. **`client/src/connection/client.rs`** - Handle lobby messages
3. **`client/ui/main.slint`** - Add lobby to layout
4. **`client/src/ui/handler.rs`** - Connect UI events to state

**Directory Structure:**
```
profile-root/
├── client/
│   ├── src/
│   │   ├── ui/
│   │   │   ├── lobby.rs              # ← NEW: Lobby component
│   │   │   ├── lobby_state.rs        # ← NEW: Lobby state management
│   │   │   ├── state.rs              # ← MODIFY: Integrate lobby
│   │   │   └── handler.rs            # ← MODIFY: Connect events
│   │   ├── connection/
│   │   │   └── client.rs             # ← MODIFY: Handle lobby messages
│   │   └── main.rs                   # ← Initialize lobby state
│   ├── ui/
│   │   ├── lobby_item.slint          # ← NEW: Slint component
│   │   └── main.slint                # ← MODIFY: Add lobby to layout
│   └── tests/
│       ├── lobby_display_tests.rs    # ← NEW: Unit tests
│       └── lobby_integration_tests.rs # ← NEW: Integration tests
```

**Naming Conventions:**
- Use `snake_case` for functions/variables (consistent with Rust conventions)
- Use `PascalCase` for types/structs
- Test functions: `test_<scenario>_<expected_outcome>`
- Slint components: `PascalCase` (LobbyItem, LobbyComponent)

## Dev Agent Record

### Agent Model Used

MiniMax-M1.5

### Debug Log References

### Completion Notes List

**2025-12-23 - Story Context Created:**
- ✅ Story requirements extracted from epics.md (lines 632-673)
- ✅ UX design requirements extracted (LobbyComponent states, navigation)
- ✅ Architecture requirements extracted (lobby protocol)
- ✅ Cross-story dependencies documented (2.1 complete, 2.3-2.5 depend on this)
- ✅ Technical implementation guide provided with code snippets
- ✅ Testing strategy with 10+ test cases defined
- ✅ File structure and naming conventions documented
- ✅ Anti-pattern prevention guidance included

**2025-12-24 - All Validations Applied:**
- ✅ Fixed protocol: lobby_update joined users now match architecture (no status field)
- ✅ Removed duplicate Performance Requirements section
- ✅ Added Tab navigation specification (focus in/out, Shift+Tab)
- ✅ Consolidated file naming to single convention (lobby.rs, not user_list.rs)
- ✅ Added scroll-selection interaction specification
- ✅ Clarified protocol struct ownership (LobbyUserCompact may need addition)
- ✅ Added error scenario tests to test strategy (malformed JSON, duplicates)
- ✅ Standardized JSON placeholders (full hex format)
- ✅ Removed duplicate directory structure visualization
- ✅ Standardized test placeholders to full hex format
- ✅ Updated WebSocket handler comment about status field
- ✅ Clarified chat_screen.rs path (verify screens/ subdirectory)
- ✅ Consolidated completion notes into single validation section

### File List

**Core Implementation (New):**
- `profile-root/client/src/ui/lobby.rs` - LobbyComponent UI implementation
- `profile-root/client/src/ui/lobby_state.rs` - Lobby state management
- `profile-root/client/src/ui/lobby_item.slint` - Slint lobby item component
- `profile-root/client/src/state/lobby.rs` - Lobby state async wrapper
- `profile-root/client/src/handlers/lobby.rs` - Lobby event handlers
- `profile-root/client/tests/lobby_display_tests.rs` - Unit tests (8+ tests)
- `profile-root/client/tests/lobby_integration_tests.rs` - Integration tests (5+ tests)

**Core Implementation (Modified):**
- `profile-root/client/src/ui/main.slint` - Add lobby to layout (FIXED: now renders actual LobbyItems)
- `profile-root/client/src/ui/mod.rs` - Export lobby modules
- `profile-root/client/src/state/mod.rs` - Integrate lobby state
- `profile-root/client/src/connection/client.rs` - Handle lobby messages
- `profile-root/client/src/handlers/mod.rs` - Export lobby handlers

**Protocol Definition (Modified):**
- `profile-root/shared/src/protocol/mod.rs` - Added LobbyMessage, LobbyUserCompact, LobbyUpdateMessage types

**Documentation:**
- `_bmad-output/implementation-artifacts/2-2-query-display-current-online-user-list.md` - This story file (FIXED: File List updated)

**Documentation:**
- `_bmad-output/implementation-artifacts/2-2-query-display-current-online-user-list.md` - This story file

**NOT Modified (no changes needed):**
- `profile-root/server/src/` - Server lobby already implemented in Story 2.1
- `profile-root/shared/src/crypto/` - Cryptographic operations already complete
