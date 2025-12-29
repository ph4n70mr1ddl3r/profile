# Story 3-6: Handle Offline Recipient Notification

**Status:** ready-for-dev  
**Epic:** 3 - Core Messaging  
**Priority:** High  
**Story Key:** 3-6  
**Created:** 2025-12-29  
**Author:** Riddler (BMad Method)

---

## Story

As a **user**,
I want to **be notified immediately if I try to send a message to someone who's offline**,
So that **I know the message wasn't delivered and can try again later**.

---

## Acceptance Criteria

### AC1: Server Detects Offline Recipient and Sends Notification

**Given** I select a recipient from the lobby
**When** I compose and send a message
**But** that recipient has disconnected between the time I selected them and the time I sent
**Then** the server attempts to deliver the message
**And** finds the recipient is no longer online
**And** sends an offline notification back to me: `{type: "notification", event: "recipient_offline", recipient: "...", originalMessage: "..."}`
**And** the server does NOT queue or persist the message

### AC2: Client Displays Offline Notification

**Given** I receive the offline notification
**When** it arrives at my client
**Then** a notification appears: "User [truncated_key] is offline. Message not delivered."
**And** the message I attempted to send appears in my chat with a ⚠ yellow warning badge (#f59e0b)
**And** a [Retry] button is shown in the notification
**And** the notification persists until I dismiss it or the user comes back online

### AC3: Notification Updates When Recipient Returns Online

**Given** the recipient comes back online
**When** they rejoin the lobby
**Then** the notification updates to: "User [truncated_key] is back online. [Try again?]"
**And** I can click to resend the message

### AC4: Retry Mechanism

**Given** I click [Retry]
**When** I attempt to resend
**Then** the message is sent again (with a new signature and new timestamp)
**And** if the recipient is still offline, I get another notification
**And** if they're online, the message is delivered successfully

### AC5: Dismiss Notification

**Given** I dismiss the offline notification
**When** I close the notification
**Then** the ⚠ badge remains on the message (history)
**But** the notification is no longer shown
**And** I can click the message to view details (Story 4-1)

### AC6: Notification Format

**Given** the server detects an offline recipient
**When** it sends the notification
**Then** the format is:
```json
{
  "type": "notification",
  "event": "recipient_offline",
  "recipient": "recipient-public-key",
  "originalMessage": "msg-12345"
}
```

**Related FRs:** FR16, FR45, FR46, FR47  
**Source:** [epics.md lines 1041-1087](/home/riddler/profile/_bmad-output/epics.md#L1041-L1087)

---

## Technical Implementation Requirements

### Architecture Pattern

```
Server detects offline recipient → 
send_offline_notification() → 
WebSocket push notification → 
Client displays notification with retry button →
User clicks retry → resend with new signature/timestamp →
Route message again
```

### Key Components

| Component | Location | Status |
|-----------|----------|--------|
| Server offline detection | `server/src/message/mod.rs:route_message()` | **EXISTING** |
| Server error response | `server/src/connection/handler.rs` | **EXISTING** |
| Client notification handling | `client/src/connection/client.rs` | **TO IMPLEMENT** |
| Notification display component | `client/src/ui/notification.rs` | **TO CREATE** |
| Offline message badge | `client/src/ui/chat.rs` | **TO IMPLEMENT** |
| Retry mechanism | `client/src/handlers/send.rs` | **TO IMPLEMENT** |
| Undelivered message state | `client/src/state/messages.rs` | **TO IMPLEMENT** |

### Notification Structure

```rust
// Client-side notification
pub struct OfflineNotification {
    pub recipient_key: String,
    pub recipient_key_short: String,  // First 8 + "..." + last 8
    pub original_message: String,
    pub is_retry_available: bool,     // true if recipient came back online
    pub is_dismissed: bool,
}

// Message with undelivered state
pub struct ChatMessage {
    pub sender_public_key: String,
    pub message: String,
    pub signature: String,
    pub timestamp: String,
    pub is_verified: bool,
    pub is_undelivered: bool,         // NEW: true if message failed to deliver
}
```

### Server Notification Flow

1. **Recipient lookup fails:** `lobby.get_connection(recipient_key)` returns `None`
2. **Generate notification:** Create `{type: "notification", event: "recipient_offline", recipient: "...", originalMessage: "..."}`
3. **Send to sender:** Use `ActiveConnection.sender` to push notification
4. **DO NOT store message:** Messages are ephemeral (FR40-FR44)

### Client Notification Flow

1. **Parse notification:** `parse_notification()` extracts event type and data
2. **Create notification:** Instantiate `OfflineNotification` with recipient info
3. **Display UI:** Show notification in chat area with ⚠ badge
4. **Track message:** Mark message as `is_undelivered=true`
5. **Handle events:**
   - User clicks retry → resend message (new signature/timestamp)
   - User dismisses notification → set `is_dismissed=true`, hide UI
   - Recipient comes online → update `is_retry_available=true`

---

## Tasks / Subtasks

### Task 1: Server-Side Notification Infrastructure
- [ ] 1.1 Enhance `route_message()` to detect offline recipient and send notification (AC1)
- [ ] 1.2 Create `create_offline_notification()` function in server protocol module (AC6)
- [ ] 1.3 Add `recipient_offline` event handling in WebSocket handler (AC1)
- [ ] 1.4 Update error response to include original message reference (AC6)

### Task 2: Client-Side Notification Handling
- [ ] 2.1 Create `client/src/ui/notification.rs` module (AC2)
- [ ] 2.2 Implement `OfflineNotification` struct with display fields (AC2)
- [ ] 2.3 Add `parse_notification()` function in `client.rs` (AC2)
- [ ] 2.4 Create `handle_notification()` function for offline events (AC2)

### Task 3: UI Components for Offline State
- [ ] 3.1 Add `is_undelivered` field to `ChatMessage` (AC2)
- [ ] 3.2 Update `DisplayMessage` to include `is_undelivered` badge (AC2)
- [ ] 3.3 Implement ⚠ yellow badge in `format_verification_status()` (AC2)
- [ ] 3.4 Create notification banner component with dismiss button (AC5)
- [ ] 3.5 Implement notification persistence state (AC5)

### Task 4: Retry Mechanism
- [ ] 4.1 Create `resend_message()` function that generates new signature (AC4)
- [ ] 4.2 Add retry button to notification UI (AC4)
- [ ] 4.3 Wire retry button to `resend_message()` (AC4)
- [ ] 4.4 Clear undelivered state on successful retry delivery (AC4)

### Task 5: Recipient Online Detection Update
- [ ] 5.1 Connect lobby join events to notification updates (AC3)
- [ ] 5.2 Update notification text when recipient comes back online (AC3)
- [ ] 5.3 Enable retry button when `is_retry_available=true` (AC3)

### Task 6: Testing
- [ ] 6.1 Unit test: Server sends offline notification for offline recipient (AC1)
- [ ] 6.2 Unit test: Client parses notification correctly (AC2)
- [ ] 6.3 Unit test: ⚠ badge displayed for undelivered messages (AC2)
- [ ] 6.4 Unit test: Retry generates new signature (AC4)
- [ ] 6.5 Unit test: Notification updates on recipient return (AC3)
- [ ] 6.6 Unit test: Dismiss notification hides UI (AC5)
- [ ] 6.7 Integration test: End-to-end offline notification flow
- [ ] 6.8 Integration test: Retry success when recipient comes back online

### Task 7: Build & Validation
- [ ] 7.1 Build project successfully
- [ ] 7.2 Run full test suite
- [ ] 7.3 Verify 100% tests pass
- [ ] 7.4 Run clippy for linting

---

## Dev Notes

### Previous Story Intelligence

**From Story 3-5 (Display Messages Chronologically):**
- `DisplayMessage` struct already has `verification_badge()` method
- `ChatView` manages message display and scroll state
- `format_timestamp()` handles RFC3339 parsing
- Client has `MessageHistory` with `ChatMessage` storage
- Notification UI infrastructure exists (`create_invalid_signature_notification()`)

**From Story 3-4 (Receive & Verify Message Signature):**
- `verify_and_store_message()` handles message storage with verification status
- `ChatMessage.is_verified` field controls verification badge display
- Invalid signature notification system already implemented
- `MessageEventHandler` callbacks for UI notifications

**From Story 3-3 (Push Message to Online Recipient):**
- `route_message()` in `server/src/message/mod.rs:206-249` handles recipient lookup
- Server already sends error responses via `Message::Error`
- Client receives messages via WebSocket and parses with `parse_chat_message()`
- `ActiveConnection.sender` used for pushing messages to clients

**From Story 3-2 (Send Message with Validation):**
- Server validation checks recipient existence in lobby
- Error response format defined: `{type: "error", reason: "offline", details: "..."}`
- Fail-fast principle: stop at first validation error
- Message format includes all necessary fields for notification correlation

### Architecture Requirements

**Server Architecture (existing patterns):**
```rust
// server/src/message/mod.rs - route_message() pattern
pub fn route_message(
    message: &SignedMessage,
    lobby: &LobbyManager,
    sender: &PublicKey,
) -> Result<(), RoutingError> {
    match lobby.get_connection(&message.recipient) {
        Some(recipient) => {
            // Push to online recipient
            push_message(recipient, message);
            Ok(())
        }
        None => {
            // Send offline notification to sender
            send_offline_notification(sender, &message.recipient);
            Err(RoutingError::RecipientOffline)
        }
    }
}
```

**Client Architecture (to implement):**
```rust
// client/src/ui/notification.rs
pub struct OfflineNotification {
    pub recipient_key: String,
    pub recipient_key_short: String,
    pub original_message: String,
    pub is_retry_available: bool,
    pub is_dismissed: bool,
}

// Retry mechanism
pub async fn resend_message(
    client: &Client,
    original_message: &ChatMessage,
) -> Result<(), ClientError> {
    // Generate new signature with new timestamp
    let new_timestamp = Utc::now().to_rfc3339();
    let canonical = format!("{}:{}", original_message.message, new_timestamp);
    let new_signature = sign_message(&canonical, &client.private_key);

    // Send as new message
    client.send_message(
        &original_message.message,
        &new_timestamp,
        &new_signature,
        &original_message.recipient_key,
    ).await
}
```

**Offline Notification Protocol (from architecture.md Decision 5):**
```json
{
  "type": "notification",
  "event": "recipient_offline",
  "recipient": "recipient-public-key",
  "originalMessage": "msg-12345",
  "retryButton": true
}
```

### Source Tree Components to Touch

```
profile-root/
├── server/src/
│   ├── message/
│   │   └── mod.rs                    # MODIFY - Add offline notification in route_message()
│   ├── connection/
│   │   └── handler.rs                # MODIFY - Send notification via WebSocket
│   └── protocol/                     # CREATE - Notification types (if needed)
│       └── mod.rs
├── client/src/
│   ├── connection/
│   │   └── client.rs                 # MODIFY - Add parse_notification(), handle_notification()
│   ├── handlers/
│   │   └── mod.rs                    # MODIFY - Export notification handlers
│   ├── state/
│   │   └── messages.rs               # MODIFY - Add is_undelivered to ChatMessage
│   └── ui/
│       ├── mod.rs                    # MODIFY - Export notification module
│       ├── chat.rs                   # MODIFY - Add undelivered badge
│       └── notification.rs           # CREATE - OfflineNotification component
└── shared/src/
    └── protocol/
        └── mod.rs                    # MODIFY - Add Notification enum variants
```

### Performance Requirements

- **Notification delivery:** <100ms from server detection to client display
- **Retry signing:** <100ms (same as initial message signing)
- **UI updates:** <50ms for notification banner rendering
- **Memory:** Minimal overhead per notification (~100 bytes)

### Security Considerations

1. **No Message Persistence:** Offline messages are NOT stored (ephemeral architecture)
2. **New Signatures on Retry:** Each retry generates fresh signature with new timestamp
3. **Notification Only:** Server sends notification, does not attempt delivery
4. **Key Display:** Use truncated keys in notifications (privacy)
5. **Audit Trail:** ⚠ badge remains on message history even after notification dismissed

### File Changes

**New Files:**
- `client/src/ui/notification.rs` - OfflineNotification component and UI

**Modified Files:**
- `server/src/message/mod.rs` - Add offline notification logic in route_message()
- `server/src/connection/handler.rs` - Connect notification to WebSocket push
- `client/src/connection/client.rs` - Add notification parsing and handling
- `client/src/state/messages.rs` - Add `is_undelivered` field to ChatMessage
- `client/src/ui/chat.rs` - Add undelivered badge display
- `client/src/ui/mod.rs` - Export notification module
- `client/src/handlers/mod.rs` - Export notification handlers

**Verified (No Changes Needed):**
- `server/src/lobby/manager.rs` - Already has get_connection() for recipient lookup
- `shared/src/crypto/verification.rs` - Already has verify_signature()
- `shared/src/protocol/mod.rs` - May need Notification variants (check existing)

### References

- [Source: epics.md#Story-3.6] - Story requirements and acceptance criteria
- [Source: architecture.md#Decision-5] - Offline Recipient Notification protocol
- [Source: architecture.md#Pattern-5] - Error Messages and Notification Format
- [Source: Story 3-2] - Server validation and routing infrastructure
- [Source: Story 3-3] - Real-time message push mechanism
- [Source: Story 3-4] - Client verification and notification patterns
- [Source: Story 3-5] - Chat display and message history

---

## Cross-Story Dependencies

### Depends On (Must be done first):
- **Story 3-2:** Send Message to Server with Validation - Server routing logic
- **Story 3-3:** Push Message to Online Recipient - WebSocket notification mechanism
- **Story 3-4:** Receive & Verify Message Signature - Notification handling patterns
- **Story 3-5:** Display Messages Chronologically - ChatMessage and DisplayMessage

### Required For (Will depend on this):
- **Story 3-7:** Preserve Composer Draft on Disconnection - Disconnection notification handling
- **Story 4-1:** Click Message to Open Drill-Down Modal - Undelivered message click handling
- **Story 4-2:** Display Message Details in Modal - Show delivery status in drill-down

### Interface Contracts

**Server Output (to Client):**
```rust
// Notification format sent via WebSocket
struct ServerNotification {
    r#type: String,           // "notification"
    event: String,            // "recipient_offline"
    recipient: String,        // Recipient public key
    originalMessage: String,  // Message ID for correlation
}
```

**Client Input (from Server):**
```rust
// Client parses and handles notification
fn handle_notification(notification: ServerNotification) {
    match notification.event {
        "recipient_offline" => show_offline_notification(notification.recipient),
        _ => log::warn!("Unknown notification event: {}", notification.event),
    }
}
```

**Retry Flow:**
```rust
// Client resend with new signature
async fn retry_send(
    client: &Client,
    original: &ChatMessage,
) -> Result<(), ClientError> {
    let new_timestamp = Utc::now().to_rfc3339();
    let canonical = format!("{}:{}", original.message, new_timestamp);
    let new_signature = client.sign_message(&canonical).await?;

    client.send_message(
        &original.message,
        &new_timestamp,
        &new_signature,
        &original.recipient_key,
    ).await
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
- Story 3-5: Display Messages Chronologically with Timestamps
- Architecture Decision Document: `/home/riddler/profile/_bmad-output/architecture.md`

### Implementation Notes

This story implements the offline recipient notification system. Key implementation points:

1. **Server-Side:** Enhance `route_message()` to detect offline recipients and send `notification` type messages via WebSocket
2. **Client-Side:** Parse notifications, display with ⚠ badge, implement retry mechanism
3. **State Management:** Track `is_undelivered` on messages, notification dismissal state
4. **Retry:** Generate new signature with new timestamp on each retry attempt
5. **Update on Return:** Lobby join events trigger notification updates

### Key Design Decisions

1. **Sticky Notifications:** Notifications persist until dismissed or recipient returns
2. **Badge Persistence:** ⚠ badge remains on message history even after notification dismissed
3. **New Signatures:** Each retry generates fresh signature (required for cryptographic integrity)
4. **No Message Queue:** Messages are ephemeral - sender must retry manually
5. **Truncated Keys:** Display uses first 8 + "..." + last 8 for readability

---

## Status History

| Date | Status | Notes |
|------|--------|-------|
| 2025-12-29 | ready-for-dev | Story file created with comprehensive context from previous stories |

---

**Document Version:** 1.0  
**Last Updated:** 2025-12-29  
**BMad Method Version:** 6.0.0-alpha.21
