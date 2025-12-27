# Story 4.2: Display Message Details in Drill-Down Modal

**Epic:** 4 - Transparency
**Status:** Review
**Priority:** High
**Estimated Points:** 5
**Predecessor:** Story 4.1 (Click Message to Open Drill-Down Modal)

---

## User Story

As a **user**,
I want to **see the message content, sender's public key, and cryptographic signature in the drill-down modal**,
So that **I can verify the message comes from the claimed sender and inspect the cryptographic proof**.

---

## Acceptance Criteria

### Layer 1: Core Message Information (Always Visible)

**Given** the drill-down modal is open
**When** I view the details
**Then** the sender's full public key is displayed:
- Full key in monospace font (not truncated)
- Blue color (#0066CC) indicating cryptographic identity
- Untruncated and fully visible

**Given** the modal is open
**When** I view the message content
**Then** the message is displayed:
- Exactly as sent (no modification, no truncation)
- In readable text format
- Preserving all whitespace and special characters

**Given** the modal is open
**When** I view the timestamp
**Then** the timestamp is displayed in HH:MM:SS format:
- 24-hour format for precision (e.g., "14:35:09")
- Shows when the message was sent
- Matches the timestamp in the chat view

**Given** the modal is open
**When** I want to copy the public key
**Then** a copy button is available next to the key:
- Clicking copies the full public key to clipboard
- Brief visual feedback shows "Copied!"
- Keyboard shortcut Ctrl+C also works when key is focused

**Given** the modal is open
**When** I want to copy the message content
**Then** a copy button is available next to the message:
- Clicking copies the message text to clipboard
- Brief visual feedback shows "Copied!"
- Keyboard shortcut Ctrl+C also works when message is focused

### Layer 2: Cryptographic Signature (Expandable/Detailed)

**Given** the modal is open
**When** I scroll or expand to signature section
**Then** the cryptographic signature is displayed:
- Full hex-encoded signature in monospace font
- Complete, not truncated (e.g., "a4f3e2c1b8d5e9f2a7c4d8e1f3b6a9...")
- Signature format: 128 hex characters (64 bytes × 2 chars/byte)
- Neutral gray color (#6b7280) for technical data

**Given** the signature is displayed
**When** I want to copy it
**Then** a copy button is available:
- Clicking copies the full signature to clipboard
- Brief visual feedback shows "Copied!"
- The entire signature is copied as a single string

### Layer 3: Verification Status

**Given** the modal is open
**When** I view the verification status
**Then** I see one of the following:

**If verified:**
- Green badge (✓) with text "Verified"
- Background color: #dcfce7 (light green)
- Text color: #166534 (dark green)
- Explanation text: "This message came from [full_public_key]"

**If not verified:**
- Red badge (⚠) with text "Not Verified"
- Background color: #fef2f2 (light red)
- Text color: #991b1b (dark red)
- Explanation text: "Signature verification failed for this message"

**Given** the message was sent by myself
**When** I view the verification status
**Then** the verification badge shows as verified
**And** the explanation reads: "This message came from your public key"

### Copy Functionality

**Given** I'm viewing any section in the modal
**When** I press Ctrl+C with text selected
**Then** the selected text is copied to clipboard
**And** the copy works for: public key, message content, signature

**Given** I click a copy button
**When** the copy completes
**Then** the button briefly changes to show "Copied!"
**And** after 1 second, the button returns to normal
**And** the clipboard contains the exact content

---

## Technical Implementation

### Modal Content Layout

The drill-down modal in `drill_down_modal.slint` will be updated with the following sections:

```
┌─────────────────────────────────────────────────────┐
│  Message Details                              [X]   │
├─────────────────────────────────────────────────────┤
│                                                     │
│  VERIFICATION STATUS                                │
│  ┌─────────────────────────────────────────────┐   │
│  │  ✓ Verified                                 │   │
│  │  This message came from                     │   │
│  │  a1b2c3d4e5f6g7h8i9j0...                    │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  SENDER'S PUBLIC KEY                           [📋] │
│  ┌─────────────────────────────────────────────┐   │
│  │  a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u  │   │
│  │  v1w2x3y4z5a6b7c8d9e0f1g2h3i4j5k6l7m8n9o0p  │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  MESSAGE CONTENT                               [📋] │
│  ┌─────────────────────────────────────────────┐   │
│  │  Hello! This is my message text exactly as  │   │
│  │  sent, with all characters preserved.       │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  TIMESTAMP                                        │
│  ┌─────────────────────────────────────────────┐   │
│  │  14:35:09                                   │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
│  CRYPTOGRAPHIC SIGNATURE                      [📋] │
│  ┌─────────────────────────────────────────────┐   │
│  │  a4f3e2c1b8d5e9f2a7c4d8e1f3b6a9d2c8e7f1a3b  │   │
│  │  5d8c7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1c0d  │   │
│  │  ... (full 128 hex characters)              │   │
│  └─────────────────────────────────────────────┘   │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### New UI Properties Required

```slint
// In DrillDownModal component
in property <string> sender_public_key;        // Full 88-char hex string
in property <string> message_content;          // Full message text
in property <string> timestamp_hhmmss;         // "HH:MM:SS" format
in property <string> signature_hex;            // Full 128-char hex signature
in property <bool> is_verified;                // true = verified, false = failed
in property <string> verification_message;     // "This message came from..."

// Copy button states
in property <bool> key_copy_button_copied;     // Temporary true after copy
in property <bool> message_copy_button_copied;
in property <bool> signature_copy_button_copied;
```

### Callbacks Required

```slint
callback copy_public_key;                      // Copies sender_public_key
callback copy_message_content;                 // Copies message_content
callback copy_signature;                       // Copies signature_hex
```

### Copy-to-Clipboard Implementation Details

**Platform-Specific Clipboard API:**

For Windows desktop (MVP target), use the `windows` crate clipboard API:

```rust
use std::os::windows::ffi::OsStrExt;
use std::io::Write;

fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Open clipboard
    let clipboard = open_clipboard(None)?;
    
    // Empty clipboard
    empty_clipboard(clipboard)?;
    
    // Convert to UTF-16 for Windows
    let wide: Vec<u16> = text.encode_wide().chain(std::iter::once(0)).collect();
    
    // Set clipboard data
    set_clipboard_data(clipboard, CF_UNICODETEXT, &wide)?;
    
    // Close clipboard
    close_clipboard(clipboard)?;
    
    Ok(())
}
```

**Alternative Cross-Platform Approach:**

Use the `copypasta` crate for cross-platform clipboard support:

```rust
use copypasta::ClipboardContext;
use copypasta::ClipboardProvider;

fn copy_to_clipboard(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut ctx = ClipboardContext::new()?;
    ctx.set_contents(text.to_owned())?;
    Ok(())
}
```

Add to `Cargo.toml`:
```toml
[dependencies]
copypasta = { version = "0.10", features = ["windows"] }
```

**UI Feedback Implementation:**

When copy button is clicked:
1. Call Rust handler to copy text to clipboard
2. Set temporary `copied` state property to `true`
3. Start 1000ms timer
4. When timer fires, set `copied` state back to `false`

```slint
// Copy button with feedback
Rectangle {
    width: copy_button.width;
    height: copy_button.height;
    
    Text {
        text: copied ? "Copied!" : "📋";
        color: copied ? #22c55e : #6b7280;
    }
    
    callback clicked => {
        copy_to_clipboard(text_to_copy);
        copied = true;
        reset_timer.start();
    }
}
```

### Data Flow from Message History to Modal Display

```
Message History Store (in memory)
           │
           ▼
┌─────────────────────────────┐
│ Message struct:             │
│ - sender_public_key: String │
│ - content: String           │
│ - timestamp: DateTime       │
│ - signature: Vec<u8>        │
│ - is_verified: bool         │
└─────────────────────────────┘
           │
           ▼
    User clicks message
           │
           ▼
   chat_message_clicked(index)
           │
           ▼
┌─────────────────────────────┐
│ on_chat_message_clicked:    │
│ 1. Get message at index     │
│ 2. Extract properties       │
│ 3. Format timestamp HH:MM:SS│
│ 4. Convert signature to hex │
│ 5. Set modal properties     │
│ 6. Open modal               │
└─────────────────────────────┘
           │
           ▼
    Modal properties updated
           │
           ▼
┌─────────────────────────────┐
│ DrillDownModal displays:    │
│ - sender_public_key (blue)  │
│ - message_content           │
│ - timestamp_hhmmss          │
│ - signature_hex (gray)      │
│ - verification_status       │
└─────────────────────────────┘
```

**Message Struct Reference (from shared library):**

```rust
// In profile-shared/src/protocol/mod.rs
pub struct Message {
    pub sender_public_key: PublicKey,      // [u8; 32] internally, hex string externally
    pub content: String,                   // UTF-8 message text
    pub timestamp: DateTime<Utc>,          // ISO 8601 timestamp
    pub signature: Signature,              // [u8; 64] ed25519 signature
    pub is_verified: bool,                 // Set during client verification
}

impl Message {
    pub fn to_hex(&self) -> String {
        self.signature.as_bytes().iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
    
    pub fn format_timestamp_hhmmss(&self) -> String {
        let time = self.timestamp.time();
        format!("{:02}:{:02}:{:02}", 
            time.hour(), 
            time.minute(), 
            time.second()
        )
    }
}
```

**Modal Population Logic (Rust handler):**

```rust
// In profile-client/src/main.rs
fn on_chat_message_clicked(&self, index: usize) {
    let message = &self.message_history[index];
    
    // Set modal properties
    self.set_drill_down_sender_key(&message.sender_public_key.to_hex());
    self.set_drill_down_message_content(&message.content);
    self.set_drill_down_timestamp(&message.format_timestamp_hhmmss());
    self.set_drill_down_signature_hex(&message.to_hex());
    self.set_drill_down_is_verified(message.is_verified);
    
    // Set verification message
    let verification_msg = if message.sender_public_key == self.own_public_key {
        "This message came from your public key"
    } else {
        &format!("This message came from {}", message.sender_public_key.to_hex())
    };
    self.set_drill_down_verification_message(verification_msg);
    
    // Open modal
    self.set_drill_down_modal_visible(true);
}
```

### Components Modified

1. **`drill_down_modal.slint`** - Add content sections:
   - Verification status section (top)
   - Public key section with copy button
   - Message content section with copy button
   - Timestamp section
   - Signature section with copy button

2. **`main.slint`** - Update property bindings:
   - Bind modal properties to DrillDownModal component
   - Add copy button callbacks

3. **`main.rs`** - Add handlers:
   - `on_copy_public_key` handler
   - `on_copy_message_content` handler
   - `on_copy_signature` handler
   - Update `on_chat_message_clicked` to populate all properties

4. **New component: `copy_button.slint`** (optional, for reusability):
   - Reusable copy button component
   - Props: `text_to_copy`, `copied_state`
   - Callbacks: `on_copy`

---

## Dependencies

### Predecessor Dependencies

**Story 4.1 (Completed):**
- `DrillDownModal` component shell exists with 500x400px dimensions
- `chat_message_clicked(int)` callback implemented
- `drill_down_modal_visible` property for modal state
- Modal overlay and backdrop styling in place
- Close button (X) and Escape key handling working

### Component Dependencies

| Component | Source | Purpose |
|-----------|--------|---------|
| `KeyDisplayComponent` | Story 1.3 | Reuse for public key display with consistent styling |
| `VerificationBadgeComponent` | Story 3.4 | Reuse for verification status badge |
| `MessageItem` | Story 4.1 | Source of click events to open modal |

### Data Structure Dependencies

| Data | Source | Purpose |
|------|--------|---------|
| `Message` struct | `profile-shared/src/protocol` | Contains all fields needed for modal |
| `PublicKey::to_hex()` | `profile-shared/src/crypto` | Convert key to hex string |
| `Signature::to_hex()` | `profile-shared/src/crypto` | Convert signature to hex string |
| `format_timestamp_hhmmss()` | Utility | Format timestamp for display |

### External Dependencies

**New crate dependencies to add:**

```toml
# profile-client/Cargo.toml

[dependencies]
# Existing dependencies...
copypasta = { version = "0.10", features = ["windows"], optional = true }

# Or use native Windows API (no additional dependency)
# std::os::windows::ffi::OsStrExt - already in std lib
```

**Platform note:** For MVP Windows target, using the Windows API directly avoids the copypasta dependency. For cross-platform support post-MVP, add copypasta.

---

## Design Decisions

### Three-Layer Information Architecture

The modal content is organized into three layers for progressive disclosure:

1. **Layer 1 (Immediate):** Core message info (key, content, timestamp) - always visible
2. **Layer 2 (Expanded):** Technical signature data - visible for inspection
3. **Layer 3 (Status):** Verification result - prominent at top

This mirrors the UX specification's "Layer 1 → Layer 2 → Layer 3" design and allows Sam (technical validator) to quickly scan verification status while Alex (casual user) can focus on the message content.

### Copy Button Placement

Copy buttons are placed inline with section headers (right side) rather than as separate buttons to:
- Reduce visual clutter
- Keep copy action in context of what's being copied
- Support natural left-to-right reading flow

### Hex Signature Display

The 128-character ed25519 signature is displayed in 4 lines of 32 characters each (plus continuation indicators if needed) to:
- Prevent horizontal scrolling
- Make the hex string more readable
- Fit within the 500px modal width

---

## Testing Notes

### Manual Testing Checklist

- [ ] Modal opens when message is clicked
- [ ] Public key displays in full (88 characters, not truncated)
- [ ] Public key displays in blue (#0066CC) with monospace font
- [ ] Message content displays exactly as sent (no modification)
- [ ] Timestamp displays in HH:MM:SS format (e.g., "14:35:09")
- [ ] Signature displays in full hex (128 characters)
- [ ] Signature displays in neutral gray (#6b7280) with monospace font
- [ ] Clicking public key copy button copies full key to clipboard
- [ ] Clicking message copy button copies full message to clipboard
- [ ] Clicking signature copy button copies full signature to clipboard
- [ ] Copy button shows "Copied!" feedback for 1 second
- [ ] Ctrl+C copies selected text when focus is in modal
- [ ] Verified messages show green ✓ badge with "Verified" text
- [ ] Not verified messages show red ⚠ badge with "Not Verified" text
- [ ] Verification explanation text displays correctly
- [ ] Self-sent messages show "This message came from your public key"
- [ ] Modal content wraps and scrolls appropriately for long messages
- [ ] Long signatures wrap to multiple lines without breaking

### Integration Test Cases

```rust
// profile-client/tests/drill_down_details_test.rs

#[tokio::test]
async fn test_modal_displays_full_public_key() {
    // Setup: User has a message from sender with public key
    let client = TestClient::new().await;
    let sender_key = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u...";
    
    // Action: Click on message to open drill-down
    client.click_message(0).await;
    
    // Assert: Modal displays full key
    assert_eq!(client.get_drill_down_public_key().await, sender_key);
}

#[tokio::test]
async fn test_copy_functionality() {
    let client = TestClient::new().await;
    let test_message = "Hello, World!";
    
    // Action: Click copy button for message
    client.click_message_copy_button().await;
    
    // Assert: Clipboard contains the message
    let clipboard = get_clipboard().await;
    assert_eq!(clipboard, test_message);
}

#[tokio::test]
async fn test_signature_display_format() {
    let client = TestClient::new().await;
    let expected_signature = "a4f3e2c1b8d5e9f2a7c4d8e1f3b6a9d2c8e7f1a3b5d8c7e6f5a4b3c2d1e0f9a8b7c6d5e4f3a2b1c0d";
    
    // Action: Open drill-down for message with known signature
    client.click_message(0).await;
    
    // Assert: Signature displays in full hex
    assert_eq!(client.get_drill_down_signature().await, expected_signature);
}
```

### Edge Cases to Test

| Edge Case | Expected Behavior |
|-----------|-------------------|
| Very long message (10KB) | Modal scrolls, content not truncated |
| Very long signature | Wraps to multiple lines, not truncated |
| Unicode message content (CJK, emoji) | Displays correctly, copies correctly |
| Message with only whitespace | Preserves all whitespace |
| Message with newlines | Preserves line breaks, displays with wrapping |

---

## Future Enhancements (Post-MVP)

1. **Expandable signature section** - Collapsible signature section for shorter default view
2. **Signature verification on demand** - Button to re-verify signature in modal
3. **Raw signature bytes** - Toggle between hex and base64 display
4. **Key fingerprint** - Display shortened fingerprint (first 8 chars) as identifier
5. **Export all details** - Button to export message details as JSON file
6. **Animated transitions** - Smooth open/close animations for modal sections
7. **Mobile-optimized layout** - Responsive design for smaller screens

---

## Implementation Notes

### Files Created

- `profile-root/client/src/ui/copy_button.slint` (optional reusable component)

### Files Modified

- `profile-root/client/src/ui/drill_down_modal.slint` - Add content sections
- `profile-root/client/src/ui/main.slint` - Update property bindings
- `profile-root/client/src/main.rs` - Add copy handlers, update modal population
- `profile-root/client/src/ui/message_item.slint` - Ensure click handler passes all data

### Color System

| Element | Color | Usage |
|---------|-------|-------|
| Public key text | #0066CC | Primary blue, identity |
| Verification badge (verified) | #22c55e | Green, success |
| Verification badge (not verified) | #ef4444 | Red, error |
| Signature text | #6b7280 | Neutral gray, technical |
| Timestamp text | #9ca3af | Light gray, metadata |
| Copy button | #6b7280 | Neutral, unobtrusive |
| Copy button (copied) | #22c55e | Green, feedback |

### Typography

- **Public key:** Monospace, 11px, blue (#0066CC)
- **Message content:** Default UI font, 14px, white (#ffffff)
- **Timestamp:** Monospace, 12px, light gray (#9ca3af)
- **Signature:** Monospace, 10px, neutral gray (#6b7280)
- **Verification badge:** Bold, 12px, color-coded
- **Explanation text:** Default UI font, 12px, white (#ffffff)

### Estimated Implementation Effort

| Task | Effort |
|------|--------|
| Update DrillDownModal.slint with content sections | 2 hours |
| Add copy button components and handlers | 2 hours |
| Implement clipboard copy functionality | 1 hour |
| Update data flow from message history to modal | 1 hour |
| Manual testing and bug fixes | 2 hours |
| **Total** | **8 hours (1 day)** |

---

**Dependencies Summary:**
- Predecessor: Story 4.1 (DrillDownModal shell)
- Components: KeyDisplayComponent, VerificationBadgeComponent
- Data: Message struct with all cryptographic fields
- No new external crate dependencies required for Windows

**Ready for Development:** Yes

---

## Dev Agent Record

### Implementation Summary

**Story:** 4.2 - Display Message Details in Drill-Down Modal
**Completed:** 2025-12-27
**Status:** Ready for review

### Changes Made

#### Files Modified

1. **`profile-root/client/src/ui/main.slint`**
   - Added `chat_msg_X_signature` properties for all 10 message slots
   - Added drill-down modal copy button state properties (`drill_down_key_copied`, `drill_down_message_copied`, `drill_down_signature_copied`)
   - Added drill-down modal copy callbacks (`drill_down_copy_key`, `drill_down_copy_message`, `drill_down_copy_signature`)
   - Updated DrillDownModal component binding to include copy button states and callbacks

2. **`profile-root/client/src/ui/drill_down_modal.slint`**
   - Added copy button properties (`key_copied`, `message_copied`, `signature_copied`)
   - Added copy callbacks (`copy_key`, `copy_message`, `copy_signature`)
   - Implemented copy buttons with visual feedback ("Copied!" for 1 second)
   - Updated signature display color to #6b7280 (neutral gray) per AC

3. **`profile-root/client/src/main.rs`**
   - Added `copy_to_clipboard()` helper function using arboard crate
   - Added `update_chat_messages_ui()` function to populate message slots from message history
   - Added message history state initialization
   - Updated `on_chat_message_clicked` to retrieve signature from UI slots
   - Updated verification explanation to show "your public key" for self-sent messages
   - Added copy handlers for drill-down modal (`on_drill_down_copy_key`, `on_drill_down_copy_message`, `on_drill_down_copy_signature`)
   - Updated lobby user selection to also update chat messages

4. **`profile-root/client/src/ui/chat.rs`**
   - Added `signature` field to `DisplayMessage` struct

5. **`profile-root/client/src/handlers/offline.rs`**
   - Added `signature` field to `create_undelivered_display_message` function

#### Key Features Implemented

1. **Full Public Key Display** - Sender's full public key displayed in blue (#0066CC) with monospace font
2. **Message Content Display** - Message displayed exactly as sent with word-wrap
3. **Timestamp Display** - HH:MM:SS format using `format_timestamp()` function
4. **Full Signature Display** - Signature displayed in neutral gray (#6b7280) with monospace font
5. **Copy Functionality** - Three copy buttons (key, message, signature) with "Copied!" feedback
6. **Verification Status** - Green verified badge or red not-verified badge with explanation
7. **Self-Message Detection** - Shows "This message came from your public key" for self-sent messages

### Technical Notes

- Used existing `arboard` crate for clipboard operations (already in dependencies)
- No new external dependencies required
- Copy buttons use 1-second timer for "Copied!" feedback
- Signature is retrieved from message history via UI slot properties
- Verification explanation differentiates between self-sent and other messages

### Testing

- All existing tests pass (32 tests)
- Build succeeds with no errors
- Clippy warnings are minor and pre-existing
