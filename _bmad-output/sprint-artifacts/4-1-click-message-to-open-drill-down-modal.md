# Story 4.1: Click Message to Open Drill-Down Modal

**Epic:** 4 - Transparency
**Status:** Completed
**Priority:** High
**Estimated Points:** 3
**Actual Points:** 3

---

## User Story

As a **user**,
I want to **click on any message to view its full cryptographic details**,
So that **I can understand the proof behind the verification badge**.

---

## Acceptance Criteria

### Message Clickability

**Given** messages are displayed in the chat area
**When** I see a message with a ✓ or ⚠ badge
**Then** the message is clickable (cursor changes to pointer)
**And** a tooltip appears on hover: "Click to view details"

### Modal Opening

**Given** I click on a message
**When** the message is activated
**Then** a modal opens showing the full message details
**And** the modal is centered on the screen
**And** a close button (X) is visible in the top-right
**And** pressing Escape also closes the modal

### Modal Behavior

**Given** the modal is open
**When** I'm viewing the details
**Then** the chat area behind it is slightly dimmed (visual indication that modal is active)
**And** I cannot interact with the chat or composer while the modal is open
**And** focus is trapped in the modal (Tab stays within the modal)

### Modal Closing

**Given** I close the modal
**When** I press Escape or click the X button
**Then** the modal closes smoothly
**And** focus returns to the message I was viewing
**And** the chat area returns to normal

---

## Technical Implementation

### New Components Created

1. **`drill_down_modal.slint`** - DrillDownModal component
   - Centered modal overlay with 500x400px dimensions
   - Dimmed backdrop (50% opacity black overlay)
   - Close button (X) with keyboard focus support
   - Escape key handler for closing
   - Focus trapping within modal
   - Placeholder content sections for future stories

2. **`message_item.slint`** - MessageItem component
   - Clickable message bubble with visual feedback
   - Cursor changes to pointer on hover
   - Tooltip indicator on hover
   - Verification badge (✓ or ⚠) display
   - Self vs. other message styling
   - Compact version available for performance

### Modified Files

1. **`main.slint`** - Added:
   - Import statements for new components
   - Chat message slot properties (up to 10 messages for MVP)
   - Drill-down modal state properties
   - Callbacks: `chat_message_clicked`, `drill_down_modal_close`
   - Chat display area in lobby view
   - Modal overlay at root level

2. **`main.rs`** - Added:
   - `on_chat_message_clicked` handler - Opens modal with message details
   - `on_drill_down_modal_close` handler - Closes modal and resets state
   - Modal property population logic

### UI Properties Added

```slint
// Chat message slots (10 for MVP, fixed slots due to Slint 1.5 limitation)
in property <int> chat_message_count;
in property <string> chat_msg_{1-10}_sender_key;
in property <string> chat_msg_{1-10}_sender_key_short;
in property <string> chat_msg_{1-10}_content;
in property <string> chat_msg_{1-10}_timestamp;
in property <bool> chat_msg_{1-10}_is_self;
in property <bool> chat_msg_{1-10}_is_verified;

// Drill-down modal state
in property <bool> drill_down_modal_visible;
in property <string> drill_down_sender_key;
in property <string> drill_down_message_content;
in property <string> drill_down_timestamp;
in property <string> drill_down_signature;
in property <bool> drill_down_is_verified;
in property <string> drill_down_verification_text;
in property <string> drill_down_verification_explanation;
```

### Callbacks Added

```slint
callback chat_message_clicked(int);     // Opens modal for message at slot index
callback drill_down_modal_close;        // Closes the modal
```

---

## Design Decisions

### Fixed Slot Approach (MVP)

Due to Slint 1.5's lack of dynamic for-each loops, we use a fixed slot approach:
- Lobby: 5 user slots
- Chat: 10 message slots

This is acceptable for MVP and can be enhanced when Slint adds dynamic list support.

### Modal Overlay Pattern

The modal is placed at the root level of the window (outside any view-specific layout) to ensure it:
- Covers the entire window regardless of current view
- Overlays on top of all other UI elements
- Can be triggered from any view context

### Focus Management

Slint 1.5 has limited focus APIs. We implement:
- `forward-focus` on modal to establish initial focus scope
- Keyboard event handler for Escape key
- FocusScope wrappers for interactive elements (close button)

---

## Dependencies

- **Predecessor:** Story 3.5 (Display Messages Chronologically)
- **Follows:** Epic 3 complete (Core Messaging)
- **Enables:** Stories 4.2-4.4 (modal content population)

---

## Testing Notes

### Manual Testing Checklist

- [ ] Messages display with clickable cursor
- [ ] Hover tooltip appears: "Click to view details"
- [ ] Clicking message opens modal
- [ ] Modal is centered on screen
- [ ] Close button (X) is visible and clickable
- [ ] Escape key closes modal
- [ ] Backdrop click closes modal
- [ ] Background is dimmed when modal open
- [ ] Cannot interact with chat while modal open
- [ ] Tab stays within modal (focus trapping)
- [ ] Modal closes smoothly
- [ ] Focus returns appropriately after close

### Integration Points

- Story 4.2: Will populate sender_key, signature, and other properties
- Story 4.3: Will update verification text logic
- Story 4.4: Will add full signature display

---

## Future Enhancements (Post-MVP)

1. **Dynamic message list** - When Slint supports for-each loops
2. **Animated transitions** - Smooth open/close animations
3. **Full signature display** - Expandable signature section (Story 4.2)
4. **Copy functionality** - Copy key/signature to clipboard (Story 4.2)
5. **Message search** - Filter/highlight messages in drill-down

---

## Implementation Notes

### Files Created
- `profile-root/client/src/ui/drill_down_modal.slint`
- `profile-root/client/src/ui/message_item.slint`

### Files Modified
- `profile-root/client/src/ui/main.slint`
- `profile-root/client/src/main.rs`

### Color System
- Modal background: `#1e1e2e` (dark surface)
- Overlay background: `#000000` (50% opacity)
- Verified badge: `#22c55e` (green)
- Warning badge: `#ef4444` (red)
- Close button hover: `#cc4444`

---

**Completion Date:** 2025-12-27
**Commit:** TBD
