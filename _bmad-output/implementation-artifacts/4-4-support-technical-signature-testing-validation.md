# Story 4-4: Support Technical Signature Testing & Validation

**Epic:** 4 - Transparency  
**Status:** ready-for-dev  
**Priority:** Medium  
**Story Key:** 4-4  
**Created:** 2025-12-30  
**Author:** Riddler (BMad Method)

---

## Story

As a **technical user (Sam)**,
I want to **inspect full signatures and test deterministic signing by comparing identical messages**,
So that **I can validate the cryptographic foundation is correct**.

---

## Story Definition from epics.md

**Context:** Technical users (Sam) need the ability to validate the cryptographic implementation by:
1. Verifying deterministic signing works correctly (same message = same signature)
2. Testing edge case content handling (unicode, special characters, long text)
3. Understanding the signature format for debugging and validation
4. Copying signatures for external comparison tools

**Target User:** Sam (Technical Validator) - a developer or security engineer who wants to verify the cryptographic implementation is correct.

---

## Acceptance Criteria

### AC1: Deterministic Signing Verification

**Given** I send an identical message twice  
**When** I drill down on both messages  
**Then** the signatures are visible in full (not truncated)  
**And** I can compare them side-by-side:
```
Message 1 signature: a4f3e2c1b8d5e9f2a7c4d8e1f3b6a9d2c8e7f1a3b5d8c7e6f5a4b3c2d1e0f9a8
Message 2 signature: a4f3e2c1b8d5e9f2a7c4d8e1f3b6a9d2c8e7f1a3b5d8c7e6f5a4b3c2d1e0f9a8
```
**And** the fact that they match confirms deterministic signing is working

**Technical Validation:**
- ✅ Both signatures display exactly 128 hex characters
- ✅ Signatures are identical (byte-for-byte match)
- ✅ Copy functionality preserves exact signature text
- ✅ User can paste both signatures into external tools for comparison

### AC2: Edge Case Content

**Given** I send messages with edge case content (unicode, special chars, long text, whitespace)  
**When** I drill down on each message  
**Then** the signature is visible for inspection  
**And** I can verify that the signature correctly matches the message content  
**And** the verification status shows ✓ for all valid signatures

**Edge Cases to Test:**
| Content Type | Example | Expected Behavior |
|--------------|---------|-------------------|
| Unicode/CJK | `"你好世界 Привет мир 🌍"` | Full display, signature verifies |
| Emoji | `"Hello 👋 World 🌍"` | Correct display, signature verifies |
| Special chars | `"$&lt;>&quot;'` | Correct encoding, signature verifies |
| Long text | 1000+ characters | Wraps properly, signature verifies |
| Whitespace | `"  leading\ntrailing\t "` | Preserved exactly, signature verifies |
| Mixed content | `"Hello\n\tWorld! 你好 🌍"` | All preserved, signature verifies |

### AC3: Signature Format Understanding

**Given** I want to understand the signature format  
**When** I see the signature in the drill-down  
**Then** it's displayed as hex (e.g., `"a4f3e2c1..."`) in monospace font  
**And** the format is consistent (always hex, always monospace)  
**And** documentation (future) explains the signature is ed25519, 64 bytes

**Format Specification:**
- **Encoding:** Hexadecimal (0-9, a-f)
- **Length:** 128 characters (64 bytes × 2 hex chars)
- **Font:** Monospace (Consolas, Monaco, or system monospace)
- **Color:** Neutral gray (#6b7280) for technical data
- **Wrapping:** Word-wrap enabled for readability

### AC4: Copy Functionality

**Given** I copy a signature from the drill-down  
**When** I paste it elsewhere (text editor, another application)  
**Then** the full hex string is pasted exactly as shown  
**And** nothing is truncated or modified

**Copy Verification:**
- ✅ Exact 128-character string copied
- ✅ No whitespace added or removed
- ✅ Lowercase hex maintained (consistent format)
- ✅ Error handling for clipboard failures
- ✅ Visual feedback ("Copied!" or "Error!")

---

## Technical Implementation Requirements

### Architecture Pattern

```
User Sends Message (same content twice)
            │
            ▼
    ┌─────────────────────┐
    │ Message Sent →      │
    │ Signature Generated │
    │ (ed25519, deterministic)
    └─────────────────────┘
            │
            ▼
    User Clicks Message (drill-down)
            │
            ▼
    ┌─────────────────────┐
    │ Modal Opens →       │
    │ Full signature      │
    │ displayed           │
    └─────────────────────┘
            │
            ▼
    ┌─────────────────────┐
    │ Copy Signature →    │
    │ Paste to compare    │
    │ (side-by-side)      │
    └─────────────────────┘
```

### Key Components

| Component | Location | Status | Purpose |
|-----------|----------|--------|---------|
| `DrillDownModalComponent` | `client/src/ui/drill_down_modal.slint` | **EXISTING** | Modal with signature display |
| Signature display section | `drill_down_modal.slint` lines 407-478 | **EXISTING** | Full hex signature in monospace |
| Copy button (signature) | `drill_down_modal.slint` lines 424-467 | **EXISTING** | Copy with feedback |
| Error states | `drill_down_modal.slint` lines 60-62 | **EXISTING** | Error feedback for copy |
| `on_chat_message_clicked` | `client/src/main.rs` | **EXISTING** | Modal population |
| Copy handlers | `client/src/main.rs` | **EXISTING** | Clipboard operations |
| Modal properties | `client/src/main.rs` | **EXISTING** | Signature binding |

### What's Already Implemented (Stories 4-1, 4-2, 4-3)

1. **Full Signature Display** - 128 hex characters in monospace font
2. **Copy Button** - With "Copied!" and "Error!" feedback states
3. **Verification Status** - ✓ badge with explanation text
4. **Modal Infrastructure** - Open/close, focus trapping, backdrop click

### New Work Required for Story 4-4

The existing implementation fully satisfies AC1-AC4 requirements. This story focuses on:
1. **Adding comprehensive tests** for deterministic signing verification
2. **Adding edge case tests** for unicode, special chars, long text, whitespace
3. **Validating copy functionality** preserves exact signature text
4. **Documenting the signature format** for technical users

---

## Tasks / Subtasks

### Task 1: Add Deterministic Signing Verification Tests

**Objective:** Verify that identical messages produce identical signatures

- [ ] 1.1 Add test: `test_deterministic_signing_same_message_twice`
  - Send identical message content twice
  - Open drill-down for both messages
  - Assert signatures are byte-for-byte identical
  - Assert both display full 128-character hex strings

- [ ] 1.2 Add test: `test_signature_display_length_128_chars`
  - Send any message
  - Open drill-down
  - Assert signature length is exactly 128 characters
  - Assert all characters are valid hex (0-9, a-f)

- [ ] 1.3 Add test: `test_signature_hex_format_consistency`
  - Send multiple messages
  - Check all signatures use lowercase hex
  - Verify no uppercase letters present
  - Verify consistent monospace font rendering

### Task 2: Add Edge Case Content Tests

**Objective:** Verify signature display works correctly with edge case message content

- [ ] 2.1 Add test: `test_signature_display_unicode_content`
  - Send message with Unicode (CJK characters)
  - Open drill-down
  - Assert signature displays correctly
  - Assert verification status is ✓

- [ ] 2.2 Add test: `test_signature_display_emoji_content`
  - Send message with emoji characters
  - Open drill-down
  - Assert signature displays correctly
  - Assert verification status is ✓

- [ ] 2.3 Add test: `test_signature_display_special_chars`
  - Send message with HTML special characters (`<>&"'`)
  - Open drill-down
  - Assert signature displays correctly
  - Assert message content preserves special chars

- [ ] 2.4 Add test: `test_signature_display_long_content`
  - Send message with 1000+ characters
  - Open drill-down
  - Assert signature displays correctly
  - Assert message content wraps properly
  - Assert verification status is ✓

- [ ] 2.5 Add test: `test_signature_display_whitespace_content`
  - Send message with leading/trailing whitespace, newlines, tabs
  - Open drill-down
  - Assert signature displays correctly
  - Assert message content preserves all whitespace

### Task 3: Validate Copy Functionality

**Objective:** Verify copy functionality preserves exact signature text

- [ ] 3.1 Add test: `test_copy_signature_exact_length`
  - Open drill-down for any message
  - Click copy signature button
  - Get clipboard contents
  - Assert length is exactly 128 characters

- [ ] 3.2 Add test: `test_copy_signature_no_modification`
  - Open drill-down for any message
  - Get displayed signature from modal
  - Click copy signature button
  - Assert clipboard matches displayed signature exactly

- [ ] 3.3 Add test: `test_copy_signature_preserves_hex_format`
  - Open drill-down for any message
  - Click copy signature button
  - Assert clipboard contains only valid hex characters
  - Assert lowercase format is preserved

- [ ] 3.4 Add test: `test_copy_signature_error_handling`
  - Mock clipboard failure scenario
  - Click copy signature button
  - Assert "Error!" is displayed on button
  - Verify no panic or crash occurs

### Task 4: Document Signature Format

**Objective:** Add documentation for technical users understanding the signature format

- [ ] 4.1 Update `drill_down_modal.slint` header comments
  - Document signature format (ed25519, 64 bytes = 128 hex chars)
  - Add example signature format
  - Document monospace requirement for readability

- [ ] 4.2 Add inline documentation for signature section
  - Add comment above signature Text element explaining format
  - Document that hex display is consistent and machine-readable

### Task 5: Build & Validation

- [ ] 5.1 Build project successfully
- [ ] 5.2 Run all new tests and verify they pass
- [ ] 5.3 Run full test suite and verify no regressions
- [ ] 5.4 Run clippy for linting

---

## Dev Notes

### Source Citations & Requirements Traceability

**Story Foundation:** Requirements from `epics.md` lines 1328-1369

**Previous Stories (Already Implemented):**

| Story | Implementation | File |
|-------|----------------|------|
| 4-1 | Modal infrastructure | `drill_down_modal.slint`, `main.rs` |
| 4-2 | Full signature display | `drill_down_modal.slint` lines 470-477 |
| 4-2 | Copy button for signature | `drill_down_modal.slint` lines 424-467 |
| 4-3 | Verification status | `drill_down_modal.slint` lines 365-405 |

### Existing Implementation Details

**Signature Display (from Story 4-2):**
```slint
// drill_down_modal.slint lines 470-477
Text {
    text: root.signature;
    font-family: "Consolas, Monaco, monospace";
    font-size: 10px;
    color: #6b7280;
    wrap: word-wrap;
    horizontal-alignment: left;
}
```

**Copy Button with States (from Story 4-2):**
```slint
// drill_down_modal.slint lines 424-467
signature_copy_button := FocusScope {
    width: 60px;
    height: 20px;

    Rectangle {
        background: signature_copy_button.has-focus ? #4444AA : 
                   (root.signature_error ? #ef4444 : 
                   (root.signature_copied ? #22c55e : #333333));
        // ... button styling
    }
    
    // TouchArea with copy_signature callback
    // Key handling for Enter/Space
}
```

**Copy Handlers (from Story 4-2):**
```rust
// main.rs - Existing copy handlers
fn on_drill_down_copy_signature(&self) {
    let signature = self.get_drill_down_signature();
    match self.copy_to_clipboard(&signature) {
        Ok(()) => {
            self.set_drill_down_signature_copied(true);
            // Timer resets after 1 second
        }
        Err(e) => {
            log::error!("Failed to copy signature: {}", e);
            self.set_drill_down_signature_error(true);
            // Timer resets after 2 seconds
        }
    }
}
```

### Technical Background: Deterministic Signing

**What is deterministic signing?**

Ed25519 signatures are deterministic when the same message signed with the same key always produces the same signature. This is a property of the Ed25519 algorithm itself (RFC 8032), not all signature schemes are deterministic (some use random nonces).

**Why this matters for validation:**

1. **Reproducibility:** Technical users can verify signatures are correct by re-signing and comparing
2. **Debugging:** If signatures don't match, there's an implementation bug
3. **Security:** Non-deterministic signatures can indicate a broken random number generator

**Existing Tests (Story 3-8):**

The codebase already has deterministic signing tests from Story 3-8:
```
test_deterministic_signing_10k_iterations
test_deterministic_signing_different_messages
```

Story 4-4 adds end-to-end tests through the UI to verify the modal displays signatures correctly for comparison.

### Signature Format Specification

| Property | Value |
|----------|-------|
| Algorithm | Ed25519 (EdDSA with Curve25519) |
| Key size | 256-bit (32 bytes) |
| Signature size | 512-bit (64 bytes) |
| Encoding | Hexadecimal (lowercase) |
| Display length | 128 characters |
| Font | Monospace (Consolas, Monaco, system monospace) |
| Color | #6b7280 (neutral gray) |
| Wrapping | Word-wrap enabled |

### File Structure

**Files to Create (for tests):**
```
profile-root/
└── client/
    └── tests/
        └── signature_testing_tests.rs    # NEW - Technical signature tests
```

**Files to Modify:**
```
profile-root/
├── client/
│   └── src/
│       └── ui/
│           └── drill_down_modal.slint    # MODIFY - Add documentation comments
└── client/
    └── tests/
        └── signature_testing_tests.rs    # MODIFY - Add tests
```

**Files to Read (reference):**
- `profile-root/client/src/main.rs` - Copy handlers implementation
- `profile-root/client/src/ui/drill_down_modal.slint` - Modal component
- `profile-root/client/src/state/messages.rs` - Message storage
- `profile-root/shared/src/crypto/signing.rs` - Ed25519 implementation

---

## Cross-Story Dependencies

### Depends On (Must be done first)

- **Story 4-1:** Click Message to Open Drill-Down Modal - Modal infrastructure exists
- **Story 4-2:** Display Message Details in Modal - Full signature display implemented
- **Story 4-3:** Verify Message Signature in Modal - Verification status implemented
- **Story 3-8:** Handle Message Composition Edge Cases - Edge case message handling

### Required For (Will depend on this)

- **Story 5-1 (future):** Export Message Details - Technical users can export signatures
- **Story 5-2 (future):** Signature Verification Tool - External verification capability

### Interface Contracts

**Input (from Story 4-2):**
```rust
// Modal properties (already bound)
struct DrillDownModal {
    signature: String,           // 128-character hex string
    signature_copied: bool,      // Copy feedback state
    signature_error: bool,       // Error feedback state
}
```

**Output (to tests):**
```rust
// Test assertions
assert_eq!(signature.len(), 128);
assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
assert!(signature == copied_signature);
```

---

## Testing Strategy

### Test Categories

1. **Deterministic Signing Tests** - Verify same message = same signature
2. **Format Validation Tests** - Verify signature format compliance
3. **Copy Functionality Tests** - Verify clipboard operations
4. **Edge Case Tests** - Verify signature display with various content

### Test File Location

New test file: `profile-root/client/tests/signature_testing_tests.rs`

### Test Implementation Pattern

```rust
use crate::common::TestClient;

#[tokio::test]
async fn test_deterministic_signing_same_message_twice() {
    let client = TestClient::new().await;
    let test_message = "Hello, World!";
    
    // Send message twice with same content
    client.send_message(test_message).await;
    client.send_message(test_message).await;
    
    // Open drill-down for both messages
    client.click_message(0).await;
    let sig1 = client.get_drill_down_signature().await;
    client.close_drill_down().await;
    
    client.click_message(1).await;
    let sig2 = client.get_drill_down_signature().await;
    
    // Assert signatures are identical
    assert_eq!(sig1, sig2, "Deterministic signing failed: signatures differ");
    assert_eq!(sig1.len(), 128, "Signature length incorrect");
}
```

### Manual Testing Checklist

- [ ] Open modal for two identical messages, compare signatures visually
- [ ] Open modal for unicode message, verify signature displays correctly
- [ ] Open modal for emoji message, verify signature displays correctly
- [ ] Copy signature, paste to text editor, verify exact match
- [ ] Copy signature with clipboard failure, verify "Error!" feedback
- [ ] Open modal for long message (1000+ chars), verify signature displays correctly
- [ ] Open modal for message with special chars, verify all preserved

---

## Performance Considerations

- **Modal open time:** <50ms (already implemented)
- **Copy operation:** <100ms including feedback display
- **Test execution:** Each test should complete in <1 second
- **Memory:** No additional allocations for signature display

---

## Security Considerations

1. **Clipboard Operations:** No sensitive data exposed beyond what user explicitly copies
2. **Error Messages:** Generic error messages, no technical details leaked to UI
3. **Signature Display:** Technical data displayed as-is, no sanitization needed (machine-readable)
4. **Copy Confirmation:** Visual feedback confirms operation success/error

---

## Accessibility Considerations

- **Monospace Font:** Ensures readability for technical users
- **Full Display:** No truncation, all technical details accessible
- **Keyboard Navigation:** Copy button accessible via keyboard (Enter/Space)
- **Focus Management:** Focus properly trapped and restored
- **Visual Feedback:** Clear success/error states for copy operations

---

## Implementation Notes

### What's Already Working (No Changes Needed)

1. ✅ Full 128-character signature display in monospace font
2. ✅ Copy button with "Copied!" feedback (1 second)
3. ✅ Error state with "Error!" feedback (2 seconds)
4. ✅ Hex format consistency (lowercase)
5. ✅ Word-wrap for long signatures
6. ✅ Verification badge display (✓/⚠)

### New Work Summary

This story adds:
1. **Tests** to verify the existing functionality works correctly for technical validation
2. **Documentation** in code explaining the signature format for future developers

No changes to production code are required - only test additions and documentation comments.

---

## Senior Developer Review Notes

**Review Focus Areas:**
1. Test coverage for deterministic signing verification
2. Edge case coverage (unicode, emoji, special chars, long text)
3. Copy functionality validation
4. Documentation completeness

**Review Checklist:**
- [ ] All ACs have corresponding tests
- [ ] Tests follow existing patterns from `modal_verification_tests.rs`
- [ ] Documentation comments added to signature display code
- [ ] Build passes with no warnings
- [ ] All tests pass including new ones

---

## Change Log

| Date | Change | Author |
|------|--------|--------|
| 2025-12-30 | Story file created | Riddler |

---

## Status History

| Date | Status | Notes |
|------|--------|-------|
| 2025-12-30 | ready-for-dev | Story file created, ready for implementation |

---

## References

- **Source:** `epics.md` lines 1328-1369 (Story 4.4 definition)
- **Source:** `epics.md` lines 1286-1324 (Story 4.3 foundation)
- **Source:** `epics.md` lines 1240-1280 (Story 4.2 foundation)
- **Source:** `epics.md` lines 1194-1235 (Story 4.1 foundation)
- **Related FRs:** FR24, FR25, FR34-39
- **RFC 8032:** Ed25519 - Edwards-Curve Digital Signature Algorithm
- **Story 3-8:** Handle Message Composition Edge Cases (deterministic signing tests)
- **Story 4-2:** Display Message Details in Drill-Down Modal (copy functionality)
