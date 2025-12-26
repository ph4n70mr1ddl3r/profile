# Story 3.5: Display Messages Chronologically with Timestamps

Status: in-progress

## Story

As a **user**,
I want to **see all messages in chronological order with timestamps**,
So that **I can follow the conversation flow and context is clear**.

## Acceptance Criteria

**Story Foundation** [Source: /home/riddler/profile/_bmad-output/epics.md#L10000-L10038]:

**Given** messages are arriving from multiple sources
**When** they are displayed in the chat
**Then** messages are ordered from oldest (top) to newest (bottom)
**And** each message includes a timestamp showing when it was sent
**And** timestamp format is HH:MM:SS for precision (supports Sam's signature testing)

**Given** I send a message
**When** it appears in my chat view
**Then** the timestamp shows the exact moment I sent it (client time)
**And** matches the timestamp in other users' views

**Given** I receive messages while viewing the chat
**When** new messages arrive
**Then** the chat automatically scrolls to show the newest message
**And** I can scroll up to see older messages
**And** the scroll position is preserved if I scroll up (don't auto-scroll while user is reading history)

**Given** the application has been open for a while
**When** I have many messages in the history
**Then** all messages remain visible and scrollable
**And** performance remains acceptable (no lag)

**Technical Implementation Requirements** [Source: /home/riddler/profile/_bmad-output/epics.md#L10030-L10035]:
- Message storage: in-memory list, ordered by timestamp
- Display: Slint ScrollView with auto-scroll to bottom
- Timestamp format: ISO 8601 on transport, HH:MM:SS in display
- Scroll behavior: auto-scroll to newest unless user is scrolling
- Memory management: clear on app close (ephemeral storage)

**Related FRs:** FR19, FR20 [Source: /home/riddler/profile/_bmad-output/epics.md#L57-L58]

---

## Developer Context Section - CRITICAL IMPLEMENTATION GUIDE

**CRITICAL MISSION:** This story implements the chat display layer that shows messages in chronological order with properly formatted timestamps. It manages the view state for chat conversations.

### Technical Specifications

**Core Technology Stack:**
- **Language:** Rust
- **Timestamp Handling:** chrono 0.4
- **Concurrency:** Tokio Mutex for thread-safe access
- **Display Format:** HH:MM:SS for human readability

**Dependencies from Previous Stories:**
- ✅ Story 3.3: MessageHistory (message storage)
- ✅ Story 3.4: ChatMessage with is_verified field
- ✅ Story 3.1: Client message composer

### Architecture & Implementation Guide

**Client Structure:**
- **Chat view:** `profile-root/client/src/ui/chat.rs` - Display components
- **Message storage:** `profile-root/client/src/state/messages.rs` - MessageHistory
- **Integration:** ChatView connects history to display

**Display Flow:**
```
MessageHistory (chronological storage) → 
update_chat_view() → 
Convert to DisplayMessage with HH:MM:SS → 
Display in Slint ScrollView → 
Auto-scroll to bottom unless user is scrolling
```

**DisplayMessage Structure:**
```rust
pub struct DisplayMessage {
    pub id: String,              // For scroll tracking
    pub sender_key_short: String, // Truncated for display
    pub content: String,          // Message text
    pub timestamp: String,        // HH:MM:SS format
    pub is_verified: bool,        // ✓ badge
    pub is_self: bool,            // Differentiate sent/received
}
```

### Implementation Details

**1. format_timestamp() (chat.rs:57-115)**
- Converts ISO 8601 (RFC3339) to HH:MM:SS
- Uses chrono for reliable parsing
- Fallback for malformed timestamps

**2. DisplayMessage (chat.rs:17-66)**
- Wraps ChatMessage for display
- Formats sender key (first 8 + "..." + last 8)
- Includes verification badge state
- Tracks if message is from self

**3. ChatView (chat.rs:118-185)**
- Manages message display state
- Tracks user scrolling (for auto-scroll behavior)
- Stores selected recipient
- Maintains message list

**4. update_chat_view() (chat.rs:188-213)**
- Syncs message history to display
- Filters by selected recipient
- Converts to DisplayMessage format

**5. add_message() (chat.rs:215-224)**
- Adds new message in real-time
- Maintains chronological order
- Used for live message updates

### Cross-Story Dependency Map

**Dependencies:**
- **Depends On:** Story 3.3 (MessageHistory), Story 3.4 (ChatMessage)
- **Required For:** Story 3.6 (Offline notifications in chat)

**Interface Contracts:**
- ChatView works with SharedMessageHistory
- DisplayMessage provides UI-ready data
- Auto-scroll controlled via is_user_scrolling flag

---

## Implementation Analysis

### Features Implemented

| Feature | Location | Status |
|---------|----------|--------|
| format_timestamp() | ui/chat.rs | ✅ Complete |
| DisplayMessage struct | ui/chat.rs | ✅ Complete |
| ChatView state | ui/chat.rs | ✅ Complete |
| update_chat_view() | ui/chat.rs | ✅ Complete |
| add_message() | ui/chat.rs | ✅ Complete |
| clear_chat() | ui/chat.rs | ✅ Complete |
| Scroll tracking | ui/chat.rs | ✅ Complete |
| UI module export | ui/mod.rs | ✅ Complete |

### Tests Implemented

| Test | Location | Status |
|------|----------|--------|
| RFC3339 timestamp | ui/chat.rs | ✅ 1 test |
| Timestamp with nanos | ui/chat.rs | ✅ 1 test |
| Invalid timestamp | ui/chat.rs | ✅ 1 test |
| Display message creation | ui/chat.rs | ✅ 1 test |
| Self message | ui/chat.rs | ✅ 1 test |
| Verification badge | ui/chat.rs | ✅ 1 test |
| Chat view new | ui/chat.rs | ✅ 1 test |
| Scrolling state | ui/chat.rs | ✅ 1 test |
| Recipient selection | ui/chat.rs | ✅ 1 test |
| Update chat view | ui/chat.rs | ✅ 1 test |
| Add message | ui/chat.rs | ✅ 1 test |
| Clear chat | ui/chat.rs | ✅ 1 test |
| Newest message ID | ui/chat.rs | ✅ 1 test |

---

## Tasks / Subtasks

### Task 1: Chat Display Module
- [x] 1.1 Create DisplayMessage struct
- [x] 1.2 Implement format_timestamp() with HH:MM:SS
- [x] 1.3 Create ChatView for display state
- [x] 1.4 Add scroll tracking (is_user_scrolling)
- [x] 1.5 Add tests (13 tests)

### Task 2: Integration
- [x] 2.1 Export chat module from ui
- [x] 2.2 Implement update_chat_view()
- [x] 2.3 Implement add_message() for real-time
- [x] 2.4 Add clear_chat() function

### Task 3: Testing & Validation
- [x] 3.1 Build project successfully
- [x] 3.2 Run full test suite
- [x] 3.3 Verify 100% tests pass

---

## Dev Notes

### Source Citations & Requirements Traceability
- **Story Foundation:** Requirements from epics.md lines 10000-10038
- **Functional Requirements:** FR19 (timestamp display), FR20 (chronological order)

### Key Implementation Notes

**Timestamp Format:**
- Transport: ISO 8601 / RFC3339 (e.g., "2025-12-27T10:30:45.123Z")
- Display: HH:MM:SS (e.g., "10:30:45")
- Supports Sam's signature testing with second precision

**Chronological Ordering:**
- Messages stored oldest → newest in Vec
- MessageHistory maintains order
- Display shows in insertion order

**Auto-Scroll Behavior:**
- is_user_scrolling flag prevents auto-scroll when reading
- User can scroll up to view history
- New messages trigger auto-scroll if not scrolling

**Performance:**
- DisplayMessage is lightweight struct
- Conversions are O(n) for message count
- Default history limit of 1000 messages

**Self vs Other Messages:**
- is_self flag differentiates sent/received
- Used for UI styling (right-align sent messages)
- Helps identify which conversation to show

### File List

**Core Implementation:**
- `profile-root/client/src/ui/chat.rs` - Chat display module (NEW)

**Module Exports:**
- `profile-root/client/src/ui/mod.rs` - Export chat module

**Tests:**
- 13 new tests in ui/chat.rs

### Completion Notes

**2025-12-27 - Story 3.5 Implementation Complete:**

This story implements the chat display layer for showing messages chronologically with formatted timestamps. Key features:

1. **Chronological Display**: Messages ordered oldest → newest
2. **Timestamp Formatting**: ISO 8601 → HH:MM:SS
3. **Auto-Scroll**: Smart scrolling that respects user reading
4. **Verification Badge**: Shows ✓ for verified messages
5. **Self/Other Differentiation**: Clear visual distinction

**Next Steps:**
- Story 3.6: Handle Offline Recipient Notification
- Story 3.7: Preserve Composer Draft on Disconnection
- Story 3.8: Handle Message Composition Edge Cases

---

## Testing Summary

### Unit Tests (Client Chat Display)
- 13 tests covering all display scenarios
- Tests for: timestamp formatting, message creation, scroll state, chat view updates

### Integration Tests
- update_chat_view() syncs from MessageHistory
- add_message() maintains order
- clear_chat() resets view

### Performance Requirements
- Timestamp formatting: <1ms per message
- Display update: O(n) for message count
- Memory: ~100 bytes per displayed message

---

## Status: in-progress

**Next Steps:**
- Story 3.6: Handle Offline Recipient Notification
- Story 3.7: Preserve Composer Draft on Disconnection
- Story 3.8: Handle Message Composition Edge Cases
- Integrate chat display with Slint UI components
