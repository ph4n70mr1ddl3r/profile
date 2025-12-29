# Story 4-1: Click Message to Open Drill-Down Modal

**Status:** ready-for-dev  
**Epic:** 4 - Transparency  
**Priority:** High  
**Story Key:** 4-1  
**Created:** 2025-12-29  
**Author:** Riddler (BMad Method)

---

## Story

As a **user**,
I want to **click on any message to view its full cryptographic details**,
So that **I can understand the proof behind the verification badge**.

---

## Acceptance Criteria

### AC1: Message Clickability

**Given** messages are displayed in the chat area
**When** I see a message with a ✓ or ⚠ badge
**Then** the message is clickable (cursor changes to pointer)
**And** a tooltip appears on hover: "Click to view details"

### AC2: Modal Opening

**Given** I click on a message
**When** the message is activated
**Then** a modal opens showing the full message details
**And** the modal is centered on the screen
**And** a close button (X) is visible in the top-right
**And** pressing Escape also closes the modal

### AC3: Modal Overlay Behavior

**Given** the modal is open
**When** I'm viewing the details
**Then** the chat area behind it is slightly dimmed (visual indication that modal is active)
**And** I cannot interact with the chat or composer while the modal is open
**And** focus is trapped in the modal (Tab stays within the modal)

### AC4: Modal Closing

**Given** I close the modal
**When** I press Escape or click the X button
**Then** the modal closes smoothly
**And** focus returns to the message I was viewing
**And** the chat area returns to normal

### AC5: Message Data Population

**Given** the modal is open
**When** it displays message details
**Then** the modal shows:
   - Message content (from Story 3-5)
   - Sender's public key (from Story 3-5 DisplayMessage)
   - Signature (hex-encoded, from Story 3-4)
   - Verification status (from Story 3-4)

**Related FRs:** FR34  
**Source:** [epics.md lines 1194-1235](/home/riddler/profile/_bmad-output/epics.md#L1194-L1235)

---

## Technical Implementation Requirements

### Architecture Pattern

```
Message Click (chat view) →
on_chat_message_clicked() handler →
Retrieve message from MessageHistory →
Set DrillDownModal properties →
Display modal overlay (centered, dimmed background) →
Focus trapped in modal
```

### Key Components

| Component | Location | Status |
|-----------|----------|--------|
| `MessageHistory` | `client/src/state/messages.rs` | **EXISTING** (Story 3.3) |
| `DisplayMessage` | `client/src/ui/chat.rs` | **EXISTING** (Story 3.5) |
| `DrillDownModalComponent` | `client/src/ui/drill_down_modal.slint` | **TO CREATE** |
| `on_chat_message_clicked()` | `client/src/main.rs` | **TO IMPLEMENT** |
| `drill_down_modal` properties | `client/src/main.rs` | **TO ADD** |
| `handlers/drill_down.rs` | `client/src/handlers/drill_down.rs` | **TO CREATE** |

### Modal Properties Structure

```slint
// DrillDownModalComponent properties (drill_down_modal.slint)
properties:
    bool is_visible;
    string message_content;
    string sender_public_key;
    string sender_key_short;
    string signature;
    bool is_verified;
    string timestamp;
```

### Click Handler Pattern

```rust
// handlers/drill_down.rs
pub fn on_chat_message_clicked(message_id: String) {
    // Retrieve message from MessageHistory
    let message = message_history.get(&message_id)
        .expect("Message should exist");

    // Set modal properties
    drill_down_modal.set_message_content(message.content.clone());
    drill_down_modal.set_sender_public_key(message.sender_public_key.clone());
    drill_down_modal.set_signature(message.signature.clone());
    drill_down_modal.set_is_verified(message.is_verified);
    drill_down_modal.set_timestamp(message.timestamp.clone());

    // Show modal
    drill_down_modal.set_is_visible(true);
}
```

### Modal Overlay Styling

```
DrillDownModalComponent {
    // Overlay container (full screen, semi-transparent)
    background: rgba(0, 0, 0, 0.5);  // Dimmed background

    // Modal content (centered, max-width)
    modal-container {
        width: 500px;
        height: auto;
        background: colors.surface-light;
        border-radius: 8px;

        // Header with close button
        header {
            title: "Message Details";
            close-button: X;
        }

        // Content sections
        body {
            // Message, key, signature, verification status
        }
    }
}
```

### Focus Management Requirements

1. **On Modal Open:**
   - Focus moves to modal container
   - First interactive element gets focus
   - Escape key listener activated

2. **On Modal Close:**
   - Focus returns to clicked message element
   - Escape key listener removed
   - Tab trapping disabled

3. **Tab Trapping:**
   - While modal open, Tab cycles through modal elements only
   - Shift+Tab cycles in reverse

---

## Tasks / Subtasks

### Task 1: Create DrillDownModalComponent (Slint)
- [ ] 1.1 Create `drill_down_modal.slint` in `client/src/ui/`
- [ ] 1.2 Define modal overlay structure (full-screen, dimmed)
- [ ] 1.3 Define modal container (centered, rounded corners)
- [ ] 1.4 Add header with title and close (X) button
- [ ] 1.5 Define content sections: message, key, signature, verification
- [ ] 1.6 Bind properties: is_visible, message_content, sender_public_key, etc.

### Task 2: Create Drill-Down Handler Module
- [ ] 2.1 Create `handlers/drill_down.rs`
- [ ] 2.2 Implement `on_chat_message_clicked(message_id: String)` function
- [ ] 2.3 Implement `close_drill_down_modal()` function
- [ ] 2.4 Implement `copy_to_clipboard(text: String)` helper

### Task 3: Integrate with Main Application
- [ ] 3.1 Add DrillDownModalComponent to `main.slint`
- [ ] 3.2 Export handler from `handlers/mod.rs`
- [ ] 3.3 Add modal properties to main window state
- [ ] 3.4 Wire click events to `on_chat_message_clicked`
- [ ] 3.5 Wire close button to `close_drill_down_modal`
- [ ] 3.6 Add Escape key handler for modal close

### Task 4: Implement Focus Management
- [ ] 4.1 Set focus to modal on open
- [ ] 4.2 Implement Tab trapping within modal
- [ ] 4.3 Restore focus to message on close

### Task 5: Visual Polish
- [ ] 5.1 Add hover cursor change on message items
- [ ] 5.2 Add tooltip "Click to view details" on message hover
- [ ] 5.3 Add smooth transition for modal open/close
- [ ] 5.4 Style close button (X) appropriately

### Task 6: Testing
- [ ] 6.1 Unit test: on_chat_message_clicked retrieves correct message
- [ ] 6.2 Unit test: close_drill_down_modal clears properties
- [ ] 6.3 Unit test: focus management (open/close)
- [ ] 6.4 Integration test: Modal opens on message click
- [ ] 6.5 Integration test: Modal closes on Escape
- [ ] 6.6 Integration test: Modal closes on X button click
- [ ] 6.7 Integration test: Dimmed background visible
- [ ] 6.8 Integration test: Tab trapping within modal

### Task 7: Build & Validation
- [ ] 7.1 Build project successfully
- [ ] 7.2 Run full test suite
- [ ] 7.3 Verify 100% tests pass
- [ ] 7.4 Run clippy for linting

---

## Dev Notes

### Previous Story Intelligence

**From Story 3-5 (Display Messages Chronologically):**
- `DisplayMessage` struct already has all fields needed for drill-down:
  - `sender_key: String` (full key for drill-down)
  - `sender_key_short: String` (truncated for chat view)
  - `content: String` (message text)
  - `timestamp: String` (HH:MM:SS format)
  - `signature: String` (hex-encoded for drill-down)
  - `is_verified: bool` (✓ badge status)
- `MessageHistory` stores all messages with unique IDs
- `ChatView` maintains message list and newest message ID

**From Story 3-4 (Receive & Verify Message Signature):**
- `ChatMessage.is_verified` field set during signature verification
- `VerificationResult` enum with Valid/Invalid variants
- Verification completes in <10ms average
- Signature stored as hex-encoded string in `message.signature`

**From Story 3-8 (Handle Message Composition Edge Cases):**
- 32 edge case tests verify message handling
- Unicode, special characters, long messages all handled
- Empty messages properly rejected
- Deterministic signing verified (10,000 iteration tests)

### Architecture Requirements

**Core Technology Stack:**
- **Language:** Rust
- **UI Framework:** Slint 1.5+
- **Message Storage:** MessageHistory (VecDeque with unique IDs)
- **State Management:** Slint properties for modal visibility and content

**Shared Library (existing):**
```rust
// shared/src/crypto/verification.rs
pub fn verify_signature(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), CryptoError>
```

**Message Structure (existing):**
```rust
// client/src/state/messages.rs
pub struct ChatMessage {
    pub id: String,
    pub sender_public_key: String,
    pub message: String,
    pub signature: String,
    pub timestamp: String,  // ISO 8601 format
    pub is_verified: bool,
}
```

**DisplayMessage Structure (existing):**
```rust
// client/src/ui/chat.rs
pub struct DisplayMessage {
    pub id: String,
    pub sender_key: String,          // Full key
    pub sender_key_short: String,    // Truncated
    pub content: String,
    pub timestamp: String,           // HH:MM:SS
    pub signature: String,
    pub is_verified: bool,
    pub is_self: bool,
    pub original_timestamp: String,  // ISO 8601 for ordering
}
```

### Source Tree Components to Touch

```
profile-root/
├── client/src/
│   ├── ui/
│   │   ├── mod.rs                   # MODIFY - Export drill_down_modal
│   │   ├── drill_down_modal.slint   # NEW - Modal component
│   │   ├── chat.rs                  # READ - DisplayMessage structure
│   │   └── message_item.slint       # MODIFY - Add click handler
│   ├── handlers/
│   │   ├── mod.rs                   # MODIFY - Export drill_down handler
│   │   └── drill_down.rs            # NEW - Click and modal handlers
│   └── main.rs                      # MODIFY - Wire modal properties
├── shared/src/
│   └── crypto/
│       └── verification.rs          # READ - Signature format
```

### Performance Requirements

- **Modal open:** <50ms (instantaneous to user)
- **Message retrieval:** O(1) with message ID
- **Property binding:** Slint handles efficiently
- **Memory:** No additional message copies

### Security Considerations

1. **Focus Trapping:** Prevent interaction with underlying chat while modal open
2. **Data Exposure:** Modal shows same data as chat (no sensitive info exposed)
3. **Input Validation:** Message ID must exist before modal opens
4. **Escape Handler:** Modal must close on Escape (accessibility requirement)

### UX Design Requirements

From [ux-design-specification.md#DrillDownModalComponent](/home/riddler/profile/_bmad-output/ux-design-specification.md#L527-L536):

**Modal Structure:**
```
DrillDownModalComponent
├── Header
│   ├── Title: "Message Details"
│   └── Close Button (X) or [Escape] key
├── Body
│   ├── Message Content
│   ├── Sender Information (Public Key)
│   ├── Signature Details (Expandable)
│   └── Verification Status
└── Footer
    └── Close Button
```

**Colors:**
- Verified Green: #22c55e
- Error Red: #ef4444
- Surface Light: #1f2937
- Text Primary: #f3f4f6

**Typography:**
- Keys/Signatures: Monospace (Consolas, Monaco)
- Message/Body: Segoe UI, 14px
- Labels: Segoe UI, 12px, semibold

### References

- [Source: epics.md#Story-4.1] - Story requirements and acceptance criteria
- [Source: architecture.md#Requirements-Overview] - FR34 (message details display)
- [Source: ux-design-specification.md#DrillDownModalComponent] - Modal component specification
- [Source: Story 3-3] - MessageHistory implementation
- [Source: Story 3-4] - ChatMessage with is_verified field
- [Source: Story 3-5] - DisplayMessage with all required fields
- [Source: Story 4-3] - Verification status display in modal (text strings)

---

## Cross-Story Dependencies

### Depends On (Must be done first):
- **Story 3-3:** Push Message to Online Recipient - Messages stored in history
- **Story 3-4:** Receive & Verify Message Signature - Verification status set
- **Story 3-5:** Display Messages with Timestamps - DisplayMessage with all fields

### Required For (Will depend on this):
- **Story 4-2:** Display Message Details in Modal - Content display in modal
- **Story 4-3:** Verify Message Signature in Modal - Verification status text (text only, already done)
- **Story 4-4:** Support Technical Signature Testing - Copy signature for comparison

### Interface Contracts

**Input (from Story 3-5):**
```rust
// DisplayMessage from ChatView
struct DisplayMessage {
    id: String,
    sender_key: String,          // Full key
    sender_key_short: String,    // Truncated
    content: String,
    timestamp: String,           // HH:MM:SS
    signature: String,
    is_verified: bool,
    is_self: bool,
}
```

**Output (to Story 4-2):**
```rust
// Modal properties
struct DrillDownModal {
    is_visible: bool,
    message_content: String,
    sender_public_key: String,
    signature: String,
    is_verified: bool,
    timestamp: String,
}
```

---

## Dev Agent Record

### Agent Model Used

Claude Code (BMad Method workflow)

### Debug Log References

- Story 3-1: Compose & Send Message with Deterministic Signing
- Story 3-3: Push Message to Online Recipient
- Story 3-4: Receive & Verify Message Signature
- Story 3-5: Display Messages Chronologically with Timestamps
- Story 3-8: Handle Message Composition Edge Cases
- Story 4-3: Verify Message Signature in Modal (reference for modal structure)

### Implementation Notes

This story implements the drill-down modal infrastructure—click handling, modal display, and focus management. Story 4-2 will populate the modal with content. Story 4-3 already implemented verification text strings.

**Key Design Decisions:**

1. **Click Handler Location:** Centralized in `handlers/drill_down.rs` for maintainability
2. **Modal Properties:** Simple string/bool properties bound to Slint component
3. **Focus Management:** Critical for accessibility and user experience
4. **Reuse DisplayMessage:** All required data already available from Story 3-5
5. **Escape Key Handler:** Standard desktop UX pattern for modal dismissal

**Implementation Strategy:**

1. Create `DrillDownModalComponent` in Slint with required properties
2. Create handler module with click and close logic
3. Integrate with main application (wire events, bind properties)
4. Implement focus trapping for accessibility
5. Add visual polish (cursor, tooltip, transitions)
6. Comprehensive testing

**Dependencies to Leverage:**
- `MessageHistory` for message retrieval
- `DisplayMessage` struct for all content
- Slint property binding for reactive UI updates
- Story 4-3's modal structure as reference

### File List

**New Files:**
- `client/src/ui/drill_down_modal.slint` - Modal component (~150 lines)
- `client/src/handlers/drill_down.rs` - Click and modal handlers (~100 lines)

**Modified Files:**
- `client/src/ui/mod.rs` - Export drill_down_modal
- `client/src/handlers/mod.rs` - Export drill_down handler
- `client/src/main.rs` - Wire modal properties and events
- `client/src/ui/message_item.slint` - Add click handler and cursor

---

## Status History

| Date | Status | Notes |
|------|--------|-------|
| 2025-12-29 | ready-for-dev | Story file created with comprehensive context from Epic 3 and Epic 4 stories |

---

**Document Version:** 1.0  
**Last Updated:** 2025-12-29  
**BMad Method Version:** 6.0.0-alpha.21
