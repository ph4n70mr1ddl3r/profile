# Story 3.3: Push Message to Online Recipient in Real-Time

Status: in-progress

## Story

As a **server**,
I want to **immediately push received messages to online recipients via WebSocket**,
So that **messages arrive instantly with no polling or delays**.

## Acceptance Criteria

**Story Foundation** [Source: /home/riddler/profile/_bmad-output/epics.md#L5917-L5955]:

**Given** a message has passed all validation checks
**When** the recipient is online (in the lobby)
**Then** the server finds the recipient's WebSocket connection
**And** pushes the message immediately: `{type: "message", message: "...", senderPublicKey: "...", signature: "...", timestamp: "..."}`
**And** the message includes the sender's public key and signature intact (not modified)
**And** delivery happens within 500ms end-to-end latency (sender → server → recipient)

**Given** the recipient's client receives the message
**When** the message arrives via WebSocket
**Then** the client adds it to the message history
**And** the message appears in the chat area
**And** the chat auto-scrolls to show the newest message

**Given** messages are being sent frequently
**When** the recipient has many messages arriving
**Then** all messages are delivered in order (chronological, by timestamp)
**And** the chat displays messages in order (oldest at top, newest at bottom)

**Given** the recipient is actively viewing the chat
**When** a new message arrives
**Then** they see it immediately (real-time push, not polling)

**Technical Implementation Requirements** [Source: /home/riddler/profile/_bmad-output/epics.md#L5947-L5952]:
- Push mechanism: use tokio broadcast or per-connection send
- Message ordering: use timestamps as tiebreaker
- Delivery latency: target <500ms end-to-end
- Message forwarding: forward original message as-is (don't modify)
- Client handling: WebSocket message handler receives push

**Related FRs:** FR17, FR18, FR20 [Source: /home/riddler/profile/_bmad-output/epics.md#L55-L58]

---

## Developer Context Section - CRITICAL IMPLEMENTATION GUIDE

**CRITICAL MISSION:** This story implements real-time message delivery from server to client using WebSocket push. Messages are pushed immediately upon successful validation.

### Technical Specifications

**Core Technology Stack:**
- **Language:** Rust
- **Async Runtime:** Tokio 1.48.0
- **WebSocket:** tokio-tungstenite 0.28.0
- **Serialization:** serde/serde_json 1.0+

**Dependencies from Previous Stories:**
- ✅ Story 3.1: Client message composer (message format)
- ✅ Story 3.2: Server validation (route_message function)

### Architecture & Implementation Guide

**Server Structure:**
- **Routing:** `profile-root/server/src/message/mod.rs` - Already implements route_message()
- **Connection:** Uses lobby HashMap for O(1) recipient lookup

**Client Structure:**
- **Message history:** `profile-root/client/src/state/messages.rs` - Thread-safe message storage
- **Message parsing:** `profile-root/client/src/connection/client.rs` - parse_chat_message()
- **Event handling:** `profile-root/client/src/connection/client.rs` - MessageEventHandler

**Real-Time Push Flow:**
```
Server validates message → route_message() → 
Find recipient in lobby HashMap → 
Send via mpsc::UnboundedSender → 
Client WebSocket receives → 
parse_chat_message() → 
Store in MessageHistory → 
Notify MessageEventHandler → 
UI updates reactively
```

**Message History Storage:**
- Thread-safe via Arc<Mutex<MessageHistory>>
- Chronological ordering by timestamp
- Automatic capacity management (FIFO eviction)
- JSON serialization for persistence

### Implementation Details

**1. MessageHistory (messages.rs:17-280)**
- VecDeque for O(1) append and efficient iteration
- Chronological ordering maintained automatically
- Capacity limit with automatic eviction
- Thread-safe via Arc<Mutex<>>

**2. ChatMessage (messages.rs:17-50)**
- Stores: sender_public_key, message, signature, timestamp, is_verified
- Serializable for persistence
- Supports verification status

**3. MessageEventHandler (client.rs:37-80)**
- on_message_received: Called when message arrives
- on_error: Called for server errors

**4. parse_chat_message (client.rs:206-243)**
- Parses `{type: "message", ...}` format
- Extracts sender, message, signature, timestamp
- Returns ChatMessage for history storage

**5. WebSocketClient Integration (client.rs:341-470)**
- Added message_history: SharedMessageHistory
- Added message_event_handler: Option<MessageEventHandler>
- Updated run_message_loop() to handle incoming messages
- Messages stored in history and handlers notified

### Cross-Story Dependency Map

**Dependencies:**
- **Depends On:** Story 3.2 (server validation and routing)
- **Required For:** Story 3.4 (client-side signature verification)

**Interface Contracts:**
- Server sends `{type: "message", ...}` JSON via WebSocket
- Client parses and stores in MessageHistory
- Chronological ordering by timestamp maintained

---

## Implementation Analysis

### Features Implemented

| Feature | Location | Status |
|---------|----------|--------|
| ChatMessage struct | state/messages.rs | ✅ Complete |
| MessageHistory with ordering | state/messages.rs | ✅ Complete |
| SharedMessageHistory type | state/messages.rs | ✅ Complete |
| Chronological message storage | state/messages.rs | ✅ Complete |
| MessageEventHandler | connection/client.rs | ✅ Complete |
| parse_chat_message() | connection/client.rs | ✅ Complete |
| WebSocketClient integration | connection/client.rs | ✅ Complete |
| State module exports | state/mod.rs | ✅ Complete |

### Tests Implemented

| Test | Location | Status |
|------|----------|--------|
| Empty history | state/messages.rs | ✅ 1 test |
| Single message | state/messages.rs | ✅ 1 test |
| Chronological order | state/messages.rs | ✅ 1 test |
| Add multiple messages | state/messages.rs | ✅ 1 test |
| Clear history | state/messages.rs | ✅ 1 test |
| Capacity limit | state/messages.rs | ✅ 1 test |
| Messages from sender | state/messages.rs | ✅ 1 test |
| Has messages check | state/messages.rs | ✅ 1 test |
| Serialization roundtrip | state/messages.rs | ✅ 1 test |
| Newest/oldest | state/messages.rs | ✅ 1 test |

---

## Tasks / Subtasks

### Task 1: Message History Module
- [x] 1.1 Create ChatMessage struct
- [x] 1.2 Create MessageHistory with chronological ordering
- [x] 1.3 Implement capacity management (FIFO eviction)
- [x] 1.4 Add serialization for persistence
- [x] 1.5 Add tests (10 tests)

### Task 2: Client Message Parsing
- [x] 2.1 Add MessageEventHandler callback struct
- [x] 2.2 Implement parse_chat_message() function
- [x] 2.3 Add ServerMessageResponse enum
- [x] 2.4 Update WebSocketClient with message history
- [x] 2.5 Update run_message_loop() for message handling

### Task 3: Integration
- [x] 3.1 Export messages module from state
- [x] 3.2 Connect WebSocketClient to message history
- [x] 3.3 Add message event handler support

### Task 4: Testing & Validation
- [x] 4.1 Build project successfully
- [x] 4.2 Run full test suite
- [x] 4.3 Verify 100% tests pass

---

## Dev Notes

### Source Citations & Requirements Traceability
- **Story Foundation:** Requirements from epics.md lines 5917-5955
- **Functional Requirements:** FR17 (real-time push), FR18 (message ordering), FR20 (delivery latency)

### Key Implementation Notes

**Real-Time Push Mechanism:**
- Uses tokio mpsc channels (already in place from Story 2.x)
- Per-connection sender for direct message delivery
- No polling - messages pushed immediately on receipt

**Message Ordering:**
- Timestamps serve as canonical ordering key
- ISO 8601 format ensures lexicographic = chronological
- Server generates timestamps at send time

**Client Message Handling:**
- Messages parsed immediately on receipt
- Stored in MessageHistory for display
- Event handler notified for UI updates

**Capacity Management:**
- Default 1000 message history limit
- Oldest messages evicted when full
- Prevents memory bloat for long-running sessions

### File List

**Core Implementation:**
- `profile-root/client/src/state/messages.rs` - Message history module (NEW)
- `profile-root/client/src/connection/client.rs` - Updated message handling

**Module Exports:**
- `profile-root/client/src/state/mod.rs` - Added messages module

**Tests:**
- 10 new tests in state/messages.rs
- Integration tests via WebSocketClient tests

### Completion Notes

**2025-12-27 - Story 3.3 Implementation Complete:**

This story implements real-time message push from server to client with proper message history management. Key features:

1. **Real-Time Push**: Messages delivered immediately via WebSocket
2. **Chronological Ordering**: Timestamps ensure correct message order
3. **Message History**: Thread-safe storage with capacity limits
4. **Event Handlers**: Callbacks for UI integration
5. **Error Handling**: Server errors propagated to client

**Next Steps:**
- Story 3.4: Receive & Verify Message Signature Client-Side
- Story 3.5: Display Messages Chronologically with Timestamps

---

## Testing Summary

### Unit Tests (Client Message History)
- 10 tests covering all message history operations
- Tests for: empty state, ordering, capacity, filtering, serialization

### Integration Tests
- WebSocketClient handles incoming messages
- MessageEventHandler callbacks fire correctly
- MessageHistory updated on receipt

### Performance Requirements
- Message delivery: <500ms end-to-end
- History storage: O(1) append, O(n) iteration
- Parsing: <10ms for typical messages

---

## Status: in-progress

**Next Steps:**
- Story 3.4: Receive & Verify Message Signature Client-Side
- Story 3.5: Display Messages Chronologically with Timestamps
- Add UI components for message display
