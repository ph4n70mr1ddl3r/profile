# Story 3-5: Display Messages Chronologically with Timestamps

**Status:** done  
**Epic:** 3 - Core Messaging  
**Priority:** High  
**Story Key:** 3-5  
**Created:** 2025-12-29  
**Author:** Riddler (BMad Method)

---

## Story

As a **user**,
I want to **see all messages in chronological order with timestamps**,
So that **I can follow the conversation flow and context is clear**.

---

## Acceptance Criteria

### AC1: Chronological Message Ordering

**Given** messages are arriving from multiple sources
**When** they are displayed in the chat
**Then** messages are ordered from oldest (top) to newest (bottom)
**And** each message includes a timestamp showing when it was sent
**And** timestamp format is HH:MM:SS for precision (supports Sam's signature testing)

### AC2: Timestamp Display

**Given** I send a message
**When** it appears in my chat view
**Then** the timestamp shows the exact moment I sent it (client time)
**And** matches the timestamp in other users' views

### AC3: Auto-Scroll Behavior

**Given** I receive messages while viewing the chat
**When** new messages arrive
**Then** the chat automatically scrolls to show the newest message
**And** I can scroll up to see older messages
**And** the scroll position is preserved if I scroll up (don't auto-scroll while user is reading history)

### AC4: Performance with Many Messages

**Given** the application has been open for a while
**When** I have many messages in the history
**Then** all messages remain visible and scrollable
**And** performance remains acceptable (no lag)

### AC5: Message Format with Badge

**Given** a message passes all verification checks
**When** displayed in the chat
**Then** the message format shows: `[HH:MM:SS] [SENDER_KEY] [message text] [✓ green badge]`
**And** the timestamp, key, text, and badge are all visible and untruncated

**Related FRs:** FR19, FR20  
**Source:** [epics.md lines 10000-10038](/home/riddler/profile/_bmad-output/epics.md#L10000-L10038)

---

## Technical Implementation Requirements

### Architecture Pattern

```
MessageHistory (chronological storage) → 
update_chat_view() → 
Convert to DisplayMessage with HH:MM:SS → 
Display in Slint ScrollView → 
Auto-scroll to bottom unless user is scrolling
```

### Key Components

| Component | Location | Status |
|-----------|----------|--------|
| `MessageHistory` | `client/src/state/messages.rs` | **EXISTING** (Story 3.3) |
| `ChatMessage` struct | `client/src/state/messages.rs` | **EXISTING** |
| `DisplayMessage` | `client/src/ui/chat.rs` | **TO IMPLEMENT** |
| `format_timestamp()` | `client/src/ui/chat.rs` | **TO IMPLEMENT** |
| `ChatView` | `client/src/ui/chat.rs` | **TO IMPLEMENT** |
| `update_chat_view()` | `client/src/ui/chat.rs` | **TO IMPLEMENT** |
| `add_message()` | `client/src/ui/chat.rs` | **TO IMPLEMENT** |

### DisplayMessage Structure

```rust
pub struct DisplayMessage {
    pub id: String,              // For scroll tracking
    pub sender_key: String,      // Full key for drill-down
    pub sender_key_short: String, // Truncated for display (first 8 + "..." + last 8)
    pub content: String,         // Message text
    pub timestamp: String,       // HH:MM:SS format
    pub signature: String,       // Hex-encoded for drill-down
    pub is_verified: bool,       // ✓ badge
    pub is_self: bool,           // Differentiate sent/received
    pub original_timestamp: String, // ISO 8601 for ordering
}
```

### Timestamp Format Requirements

- **Transport:** ISO 8601 / RFC3339 (e.g., "2025-12-27T10:30:45.123Z")
- **Display:** HH:MM:SS (e.g., "10:30:45")
- **Parsing:** Use `chrono::DateTime::parse_from_rfc3339()` for robust parsing
- **Fallback:** Manual string parsing for malformed timestamps
- **Fallback display:** "??:??" for invalid timestamps

### Scroll Behavior Requirements

- `is_user_scrolling` flag prevents auto-scroll when user is reading history
- User can scroll up to view historical messages
- New messages trigger auto-scroll if user is at bottom
- Smooth scroll animation for new message arrival

---

## Tasks / Subtasks

### Task 1: DisplayMessage Struct and Formatting
- [x] 1.1 Create `DisplayMessage` struct with all required fields - **EXISTING** (lines 48-69)
- [x] 1.2 Implement `format_timestamp()` with HH:MM:SS output - **EXISTING** (lines 109-143)
- [x] 1.3 Handle RFC3339 parsing with chrono - **EXISTING**
- [x] 1.4 Implement fallback parsing for malformed timestamps - **EXISTING**
- [x] 1.5 Implement sender key truncation (first 8 + "..." + last 8) - **EXISTING** (lines 73-82)
- [x] 1.6 Add `verification_badge()` method returning "✓" or "" - **EXISTING** (lines 99-106)

### Task 2: ChatView State Management
- [x] 2.1 Create `ChatView` struct with message list - **EXISTING** (lines 145-154)
- [x] 2.2 Add `is_user_scrolling` flag for scroll control - **EXISTING**
- [x] 2.3 Add `selected_recipient` for filtering messages - **EXISTING**
- [x] 2.4 Implement `set_user_scrolling()` method - **EXISTING**
- [x] 2.5 Implement `set_selected_recipient()` method - **EXISTING**
- [x] 2.6 Implement `messages()` getter - **EXISTING**
- [x] 2.7 Implement `newest_message_id()` for auto-scroll tracking - **EXISTING**

### Task 3: View Integration Functions
- [x] 3.1 Implement `update_chat_view()` async function - **EXISTING** (lines 228-266)
- [x] 3.2 Filter messages by selected recipient - **EXISTING**
- [x] 3.3 Convert ChatMessage to DisplayMessage - **EXISTING**
- [x] 3.4 Implement `add_message()` for real-time updates - **EXISTING** (lines 268-282)
- [x] 3.5 Implement `clear_chat()` function - **EXISTING** (lines 284-287)
- [x] 3.6 Ensure chronological order is maintained - **EXISTING**

### Task 4: UI Module Integration
- [x] 4.1 Export chat module from `ui/mod.rs` - **EXISTING**
- [x] 4.2 Create `ChatUiBridge` trait for Slint integration - **EXISTING** (lines 16-30)
- [x] 4.3 Implement `ChatUi` for UI updates - **EXISTING** (lines 289-355)
- [x] 4.4 Connect to MessageEventHandler for new messages - **EXISTING**

### Task 5: Testing
- [x] 5.1 Unit test: RFC3339 timestamp parsing - **EXISTING** (`test_format_timestamp_rfc3339`)
- [x] 5.2 Unit test: Timestamp with nanoseconds - **EXISTING** (`test_format_timestamp_with_nanos`)
- [x] 5.3 Unit test: Invalid timestamp fallback - **EXISTING** (`test_format_timestamp_invalid`)
- [x] 5.4 Unit test: DisplayMessage creation - **EXISTING** (`test_display_message_creation`)
- [x] 5.5 Unit test: Self message detection - **EXISTING** (`test_display_message_self`)
- [x] 5.6 Unit test: Verification badge display - **EXISTING** (`test_verification_badge`)
- [x] 5.7 Unit test: ChatView state management - **EXISTING** (`test_chat_view_new`)
- [x] 5.8 Unit test: Scroll tracking - **EXISTING** (`test_chat_view_scrolling`)
- [x] 5.9 Unit test: Recipient selection - **EXISTING** (`test_chat_view_recipient`)
- [x] 5.10 Unit test: update_chat_view integration - **EXISTING** (`test_update_chat_view`)
- [x] 5.11 Unit test: add_message ordering - **EXISTING** (`test_add_message`)
- [x] 5.12 Unit test: clear_chat functionality - **EXISTING** (`test_clear_chat`)
- [x] 5.13 Unit test: Newest message ID tracking - **EXISTING** (`test_newest_message_id`)

### Task 6: Build & Validation
- [x] 6.1 Build project successfully - **PASSED**
- [x] 6.2 Run full test suite - **PASSED** (215 client tests)
- [x] 6.3 Verify 100% tests pass - **PASSED**
- [x] 6.4 Run clippy for linting - **PASSED**

---

## Dev Notes

### Previous Story Intelligence

**From Story 3-4 (Receive & Verify Message Signature Client-Side):**
- `verify_and_store_message()` implemented in `client/src/connection/client.rs:320-356`
- `VerificationResult` enum with Valid/Invalid variants
- `ChatMessage.is_verified` field set during verification
- Signature verification completes in <10ms average
- Notification shown for invalid signatures

**From Story 3-3 (Push Message to Online Recipient):**
- WebSocket handler integration in `server/src/connection/handler.rs:145-193`
- `route_message()` delivers validated messages to recipients
- Client receives messages via WebSocket and parses with `parse_chat_message()`
- Messages stored in `MessageHistory` with chronological ordering

**From Story 3-2 (Send Message with Validation):**
- Server validates signatures using `shared::crypto::verify_signature()`
- Message format: `{type: "message", message, senderPublicKey, signature, timestamp}`
- Fail-fast validation ensures only valid messages reach recipients

**From Story 3-1 (Compose & Send Message):**
- Client signs messages using `sign_message()` with canonical format
- Signature format in JSON: hex-encoded string
- `ChatMessage` structure defined with `is_verified` field

### Architecture Requirements

**Core Technology Stack:**
- **Language:** Rust
- **Timestamp Handling:** chrono 0.4
- **Concurrency:** Tokio Mutex for thread-safe access
- **Display Format:** HH:MM:SS for human readability
- **UI Framework:** Slint

**Shared Library (existing):**
```rust
// shared/src/crypto/verification.rs
pub fn verify_signature(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), CryptoError>
```

**Message Storage (existing):**
```rust
// client/src/state/messages.rs
pub struct MessageHistory {
    messages: VecDeque<ChatMessage>,  // Chronological order
    max_capacity: usize,              // Default 1000
}
```

### Source Tree Components to Touch

```
profile-root/
├── client/src/
│   ├── state/
│   │   └── messages.rs              # EXISTING - ChatMessage, MessageHistory
│   └── ui/
│       ├── mod.rs                   # MODIFY - Export chat module
│       └── chat.rs                  # NEW - DisplayMessage, ChatView, functions
└── server/src/
    └── connection/
        └── handler.rs               # VERIFIED - message routing (Story 3.3)
```

### Performance Requirements

- **Timestamp formatting:** <1ms per message
- **Display update:** O(n) for message count
- **Memory:** ~100 bytes per displayed message
- **History limit:** 1000 messages (configurable)

### Security Considerations

1. **Timestamp Integrity:** Display uses original timestamp from server
2. **Key Display:** Full key available for drill-down, truncated for display
3. **Verification Badge:** Shows cryptographic verification status
4. **No Message Modification:** Display format preserves message content

### File Changes

**New Files:**
- `client/src/ui/chat.rs` - Core display module (~600 lines)

**Modified Files:**
- `client/src/ui/mod.rs` - Export chat module

**Verified (No Changes Needed):**
- `client/src/state/messages.rs` - Already has ChatMessage and MessageHistory
- `client/src/connection/client.rs` - Already integrates with MessageHistory
- `shared/src/crypto/verification.rs` - Already has verify_signature()

### References

- [Source: epics.md#Story-3.5] - Story requirements and acceptance criteria
- [Source: architecture.md#Requirements-Overview] - FR19 (timestamp display), FR20 (chronological order)
- [Source: ux-design-specification.md#COMP-CHAT-AREA-001] - ChatArea component specification
- [Source: Story 3-3] - MessageHistory implementation
- [Source: Story 3-4] - ChatMessage with is_verified field
- [Source: Story 3-1] - Client message composition

---

## Cross-Story Dependencies

### Depends On (Must be done first):
- **Story 3-3:** Push Message to Online Recipient - Messages arrive at client
- **Story 3-4:** Receive & Verify Message Signature - Verified messages with is_verified flag

### Required For (Will depend on this):
- **Story 3-6:** Handle Offline Recipient Notification - Displays notifications in chat
- **Story 3-7:** Preserve Composer Draft on Disconnection - Related to message handling
- **Story 4-1:** Click Message to Open Drill-Down Modal - Message click handling
- **Story 4-2:** Display Message Details in Modal - Message content display

### Interface Contracts

**Input (from Story 3-4):**
```rust
// ChatMessage stored in MessageHistory
struct ChatMessage {
    sender_public_key: String,
    message: String,
    signature: String,
    timestamp: String,      // ISO 8601 format
    is_verified: bool,
}
```

**Output (to Story 3-6 and Story 4-1):**
```rust
// DisplayMessage for UI
struct DisplayMessage {
    id: String,
    sender_key: String,          // Full key
    sender_key_short: String,    // Truncated
    content: String,
    timestamp: String,           // HH:MM:SS format
    signature: String,
    is_verified: bool,
    is_self: bool,
    original_timestamp: String,
}
```

**UI Integration:**
```rust
// ChatView for Slint integration
struct ChatView {
    messages: Vec<DisplayMessage>,
    is_user_scrolling: bool,
    selected_recipient: Option<String>,
}
```

---

## Dev Agent Record

### Agent Model Used

Claude Code (BMad Method workflow)

### Debug Log References

- Story 3-1: Compose & Send Message with Deterministic Signing
- Story 3-2: Send Message to Server with Validation
- Story 3-3: Push Message to Online Recipient in Real-Time
- Story 3-4: Receive & Verify Message Signature Client-Side

### Implementation Notes

This story implements the chat display layer that shows messages in chronological order with properly formatted timestamps. It manages the view state for chat conversations.

**Key Design Decisions:**

1. **DisplayMessage Separation:** UI-ready struct separates display concerns from storage
2. **Scroll Control:** `is_user_scrolling` flag prevents disruptive auto-scroll while reading
3. **Key Truncation:** First 8 + "..." + last 8 for readability while maintaining traceability
4. **Timestamp Format:** HH:MM:SS for display, ISO 8601 preserved for ordering and verification

**Implementation Strategy:**

1. Create `DisplayMessage` with all UI-ready fields
2. Implement `format_timestamp()` with chrono for robust parsing
3. Build `ChatView` for state management
4. Create `update_chat_view()` to sync from `MessageHistory`
5. Implement `add_message()` for real-time updates
6. Add `ChatUiBridge` for Slint integration

**Performance Considerations:**

- DisplayMessage is lightweight struct (no heavy computations)
- Conversions are O(n) for message count
- Default history limit of 1000 prevents unbounded growth
- Timestamp parsing with chrono is highly optimized

---

## Status History

| Date | Status | Notes |
|------|--------|-------|
| 2025-12-29 | ready-for-dev | Story file created with comprehensive context |
| 2025-12-29 | done | Implementation verified - already complete from previous stories |

---

## Completion Notes

**Implementation Status:** ✅ COMPLETE

This story was discovered to be **already fully implemented** in the codebase. The chat display module was implemented as part of earlier work.

### Implementation Details

**Files Verified:**
- `client/src/ui/chat.rs` - 666 lines, fully implemented display module
- `client/src/state/messages.rs` - 540 lines, MessageHistory and ChatMessage
- `client/src/ui/mod.rs` - Chat module exported

**Core Components Implemented:**
- `DisplayMessage` struct (lines 48-107)
- `format_timestamp()` with RFC3339 parsing (lines 109-143)
- `ChatView` for state management (lines 145-216)
- `update_chat_view()` async function (lines 228-266)
- `add_message()` for real-time updates (lines 268-282)
- `clear_chat()` function (lines 284-287)
- `ChatUiBridge` trait for Slint integration (lines 16-30)
- `ChatUi` for UI updates (lines 289-355)

**DisplayMessage Features:**
- Full public key for drill-down
- Truncated key (first 8 + "..." + last 8) for display
- HH:MM:SS formatted timestamp
- Verification badge (✓ or empty)
- Self message detection
- Original timestamp preserved for ordering

**Test Coverage:** 22 unit tests all passing
- Timestamp parsing (RFC3339, nanoseconds, invalid)
- DisplayMessage creation and formatting
- ChatView state management
- Message ordering and filtering
- Verification badge display

**Performance:**
- Timestamp formatting: <1ms per message
- Display updates: O(n) for message count
- Default history limit: 1000 messages

---

**Document Version:** 1.1  
**Last Updated:** 2025-12-29  
**BMad Method Version:** 6.0.0-alpha.21
