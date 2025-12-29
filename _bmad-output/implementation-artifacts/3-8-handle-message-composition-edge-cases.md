# Story 3.8: Handle Message Composition Edge Cases

Status: ready-for-dev

## Story

As a **user**,
I want to **send messages containing unicode, special characters, very long text, and whitespace variations**,
So that **the system handles all content types correctly and signatures remain deterministic and verifiable**.

## Acceptance Criteria

### Unicode Content Handling

**Given** I compose a message containing Chinese characters
**When** I send the message
**Then** the message is signed and delivered correctly
**And** the recipient sees the Chinese characters exactly as typed

**Given** I compose a message containing emojis
**When** I send the message
**Then** the emoji are preserved through signing and delivery
**And** the signature verifies correctly

**Given** I compose a message with mixed unicode (multiple languages, symbols)
**When** I send the message
**Then** all unicode content is preserved
**And** the signature is deterministic (same content = same signature)

**Unicode Coverage Required:**
- Chinese (简体中文)
- Japanese (日本語)
- Korean (한국어)
- Arabic (العربية)
- Hebrew (עברית)
- Spanish accents (Ñoño tilde)
- Emoji (🔐🚀🌍🎉)
- Mixed content ("Hello 世界! 🎉")

### Special Characters Handling

**Given** I compose a message with special symbols
**When** I send the message
**Then** special characters are preserved exactly as typed
**And** the signature is deterministic

**Special Characters Coverage:**
- Punctuation: `!@#$%^&*()_+-=[]{}|;':",./<>?`
- Quotes and apostrophes: `"Hello!"` and `'Goodbye'`
- Backslashes: `C:\\Users\\test\\file.txt`
- Null characters (rejected before signing)

### Long Message Handling

**Given** I compose a very long message (10KB+)
**When** I send the message
**Then** the message is signed and delivered successfully
**And** the signature is deterministic (same as sending same message twice)

**Given** I compose a 100KB message
**When** I send the message
**Then** the message is handled without errors
**And** performance remains acceptable (<100ms signing)

**Given** I send multiple long messages
**When** they are stored in message history
**Then** all messages are preserved
**And** history capacity limits are respected (oldest evicted when full)

### Whitespace Handling

**Given** I compose a message with leading/trailing spaces
**When** I send the message
**Then** the whitespace is preserved exactly
**And** the signature reflects the exact content

**Given** I compose a message with tabs and mixed whitespace
**When** I send the message
**Then** all whitespace variations are preserved
**And** the signature is deterministic

**Given** I compose a message with newlines
**When** I send the message
**Then** newlines are preserved in the displayed message
**And** the signature is deterministic

### Empty Message Handling

**Given** I attempt to send an empty message
**When** I press Enter or click Send
**Then** the message is not sent
**And** the Send button remains disabled
**And** the composer gives visual feedback (placeholder remains)

**Given** I have typed content and then deleted it all
**When** the composer is empty
**Then** the Send button is disabled
**And** the draft is preserved (empty string)

### Binary Content Validation

**Given** I attempt to send non-UTF-8 content
**When** the system validates the message
**Then** the content is rejected with an error
**And** the error message explains: "Binary content is not allowed. Messages must be UTF-8 text."

### Deterministic Signature Consistency

**Given** I send the exact same message content twice
**When** I compare the signatures in the drill-down view
**Then** the signatures are byte-for-byte identical
**And** this is true for all content types (unicode, special chars, long messages)

**Given** I test 10,000 iterations of the same message
**When** I compare all signatures
**Then** all 10,000 signatures are identical
**And** this validates 100% deterministic consistency

### Timestamp Edge Cases

**Given** I receive a message with various timestamp formats
**When** the timestamp is displayed
**Then** it's formatted as HH:MM:SS for readability
**And** timestamps with milliseconds are handled correctly
**And** invalid timestamps show "??:??:??"

---

## Developer Context Section - CRITICAL IMPLEMENTATION GUIDE

**CRITICAL MISSION:** This story ensures the message composition system handles all edge cases robustly. The deterministic signing mechanism must produce identical signatures for identical content, regardless of content type. This is the final validation of Epic 3's core messaging functionality.

### Technical Specifications

**Core Technology Stack:**
- **Language:** Rust
- **Unicode Handling:** Native Rust String/str types (UTF-8 by default)
- **Cryptography:** ed25519-dalek 2.2.0 (deterministic signing)
- **Serialization:** serde_json for canonical JSON encoding
- **Length Handling:** No artificial limits (server accepts what WebSocket transports)

**Dependencies from Previous Stories:**
- ✅ Story 3.1: Message composer (basic composition and signing)
- ✅ Story 3.2: Server validation (message format validation)
- ✅ Story 3.3: Message history (long message storage)
- ✅ Story 3.4: Client verification (signature verification)
- ✅ Story 3.5: Chat display (timestamp formatting)
- ✅ Story 3.6: Offline notifications (error handling)
- ✅ Story 3.7: Draft preservation (empty message handling)

**Existing Implementation Reference:**
- `profile-root/client/src/handlers/edge_cases.rs` - Existing test module (552 lines)
- `profile-root/client/src/connection/message.rs` - ClientMessage with unicode/long message tests
- `profile-root/client/src/ui/chat.rs` - Timestamp formatting

### Architecture & Implementation Guide

**Client Structure:**
- **Edge case module:** `profile-root/client/src/handlers/edge_cases.rs` - Enhanced validation
- **Message composer:** `profile-root/client/src/connection/message.rs` - Signing with edge cases
- **Validation utilities:** `profile-root/client/src/utils/validation.rs` - New for content validation

**Edge Case Validation Flow:**
```
User types message → validate_content() → 
if invalid: show error, disable send →
if valid: ClientMessage::new() → sign_message() →
serialized JSON → WebSocket send →
server validates → recipient receives
```

**Content Validation Function:**
```rust
pub fn validate_message_content(content: &str) -> Result<(), ValidationError> {
    // Check for valid UTF-8
    if !content.is_char_boundary(content.len()) {
        return Err(ValidationError::InvalidUtf8);
    }
    
    // Check for null bytes (binary indicators)
    if content.bytes().any(|b| b == 0) {
        return Err(ValidationError::BinaryContentRejected);
    }
    
    // Optional: Check max length (configurable)
    let max_length = get_max_message_length();
    if content.len() > max_length {
        return Err(ValidationError::MessageTooLong);
    }
    
    Ok(())
}
```

### Implementation Details

**1. Enhanced Content Validation (validation.rs:15-80)**
- UTF-8 validation using Rust's native string types
- Null byte detection for binary content
- Configurable max length (default 1MB)
- Returns specific error codes for different failure modes

**2. Unicode Handling (message.rs:203-232)**
- Already implemented: Unicode messages work correctly
- Additional test coverage for edge languages

**3. Long Message Handling (message.rs:235-256)**
- Already implemented: 10KB+ messages work
- Performance validation: <100ms signing time
- Memory consideration: History capacity limits

**4. Whitespace Preservation (edge_cases.rs:231-320)**
- Already implemented in edge_cases.rs
- Tests verify multiple spaces, tabs, newlines
- Deterministic signature verification

**5. Empty Message Handling (composer.rs)**
- Send button disabled when empty
- Draft preserved as empty string
- Visual feedback for empty state

### Cross-Story Dependency Map

**Dependencies:**
- **Depends On:** Stories 3.1-3.7 (all messaging functionality)
- **Required For:** Epic 4 (drill-down verification with edge cases)

**Interface Contracts:**
- Content validation happens before signing
- Invalid content produces clear error messages
- All valid content types produce deterministic signatures
- Empty messages are rejected at UI level

---

## Implementation Analysis

### Features Required

| Feature | Location | Status | Notes |
|---------|----------|--------|-------|
| UTF-8 content validation | handlers/validation.rs | NEW | Reject binary content |
| Unicode message signing | connection/message.rs | EXISTING | Tests in edge_cases.rs |
| Long message handling | connection/message.rs | EXISTING | 10KB+ tested |
| Whitespace preservation | handlers/edge_cases.rs | EXISTING | Tests verify determinism |
| Empty message handling | state/composer.rs | EXISTING | UI disables send |
| Timestamp edge cases | ui/chat.rs | EXISTING | Tests in edge_cases.rs |
| Binary content rejection | handlers/validation.rs | NEW | Null byte detection |
| 10,000 iteration test | handlers/edge_cases.rs | EXISTING | Tests verify determinism |

### Test Coverage Summary

**Existing Tests (edge_cases.rs: 552 lines):**
- Unicode tests: Chinese, Emoji, Spanish, Arabic, Mixed (5 tests)
- Special character tests: Symbols, Quotes, Backslash (3 tests)
- Long message tests: 10KB, 100KB, History, Determinism (4 tests)
- Whitespace tests: Spaces, Tabs, Newlines, Mixed, Determinism (5 tests)
- Binary validation tests: UTF-8, Content rejection (2 tests)
- Timestamp tests: RFC3339, Milliseconds, Timezone, Invalid, Empty (5 tests)
- History edge tests: Empty, Same timestamp, Capacity, From sender (4 tests)
- ChatMessage edge tests: Empty, Long key, Long signature, Equality (4 tests)

**Total Existing Tests: 32 edge case tests**

**New Tests Required:**
- Content validation function (5 tests)
- Max length enforcement (2 tests)
- Performance benchmarks (2 tests)

---

## Tasks / Subtasks

### Task 1: Content Validation Module
- [ ] 1.1 Create validation.rs with content validation function
- [ ] 1.2 Implement UTF-8 validation
- [ ] 1.3 Implement binary content detection (null bytes)
- [ ] 1.4 Implement max length validation (configurable)
- [ ] 1.5 Add content validation tests (5 tests)

### Task 2: Integration with Composer
- [ ] 2.1 Call validate_content() before ClientMessage::new()
- [ ] 2.2 Handle validation errors with user feedback
- [ ] 2.3 Update composer state to reflect validation status
- [ ] 2.4 Add Send button disable logic based on validation

### Task 3: Performance Validation
- [ ] 3.1 Measure signing time for 100KB messages
- [ ] 3.2 Verify <100ms performance target
- [ ] 3.3 Add performance benchmark tests
- [ ] 3.4 Document performance characteristics

### Task 4: Documentation & Final Tests
- [ ] 4.1 Verify all 32 existing edge case tests pass
- [ ] 4.2 Add documentation for edge case handling
- [ ] 4.3 Create integration test for full edge case flow
- [ ] 4.4 Update story documentation with findings

### Task 5: Validation & Build
- [ ] 5.1 Build project successfully
- [ ] 5.2 Run full test suite
- [ ] 5.3 Verify 100% tests pass
- [ ] 5.4 Run clippy for linting

---

## Dev Notes

### Source Citations & Requirements Traceability

**Story Foundation:**
- Requirements from epics.md lines 5814-5864 (message composition)
- Architecture.md Decision 1: Cryptographic Signing (canonical JSON)
- Architecture.md Decision 6: Testing & Validation (edge cases)

**Functional Requirements:**
- FR12: Compose and send messages
- FR13: Select recipient
- FR14: Automatic cryptographic signing
- FR24: Deterministic signatures
- FR25: Handle edge cases (unicode, special chars, long messages, empty)

**Performance Requirements:**
- Signing <100ms (tested with 100KB messages)
- Verification <100ms
- No memory issues with long messages

**Edge Case Coverage Requirements:**
From Architecture.md Decision 6 (Testing & Validation):
- Empty message: ""
- Unicode: "你好 🔐 ñ"
- Special characters: "!@#$%^&*()"
- Long message: 10KB+ text
- Whitespace variations: "   spaces\ttabs   "
- Line breaks: "line1\nline2\r\nline3"
- Binary content: [rejected with error]

### Key Implementation Notes

**Unicode Handling:**
- Rust's String type is UTF-8 by default
- No additional encoding handling required
- All unicode content is preserved through serialization
- Signatures work correctly with unicode input

**Deterministic Signing:**
- Canonical format: `{message}:{timestamp}`
- Same message + same timestamp = identical signature
- Verified with 10,000 iteration tests
- All edge cases tested for determinism

**Whitespace Preservation:**
- All whitespace is preserved in message content
- JSON serialization handles whitespace correctly
- Signatures reflect exact content (including spaces)

**Binary Content Rejection:**
- Null byte (0x00) detection
- WebSocket layer handles binary frames separately
- User sees clear error: "Binary content is not allowed"

**Empty Message Handling:**
- UI disables Send button when empty
- Draft preserved as empty string
- No attempt to sign empty messages

### File List

**Core Implementation:**
- `profile-root/client/src/utils/validation.rs` - Content validation module (NEW)
- `profile-root/client/src/connection/message.rs` - Updated with validation call
- `profile-root/client/src/state/composer.rs` - Updated with validation state

**Module Exports:**
- `profile-root/client/src/utils/mod.rs` - Export validation module

**Existing (Reference):**
- `profile-root/client/src/handlers/edge_cases.rs` - 32 edge case tests
- `profile-root/client/src/connection/message.rs` - ClientMessage implementation
- `profile-root/client/src/ui/chat.rs` - Timestamp formatting

**Tests:**
- `profile-root/client/src/utils/validation.rs` - 5 validation tests
- `profile-root/client/src/handlers/edge_cases.rs` - 32 existing tests

### Completion Notes

**2025-12-29 - Story 3.8 Implementation:**

This story completes Epic 3 by ensuring all message composition edge cases are properly handled. Key validation points:

1. **Unicode Support**: Full UTF-8 support for international text and emoji
2. **Special Characters**: All punctuation and symbols preserved
3. **Long Messages**: 10KB-100KB messages handled efficiently
4. **Whitespace**: All variations preserved through signing
5. **Empty Messages**: Handled gracefully at UI level
6. **Binary Content**: Rejected with clear error messages
7. **Determinism**: 100% consistent signatures verified

**Existing Implementation:**
- 32 edge case tests already exist in edge_cases.rs
- Unicode and long message tests in message.rs pass
- Whitespace preservation tested and working

**Required Work:**
- Create content validation module
- Integrate validation into composer flow
- Add performance benchmarks
- Final validation of all edge cases

---

## Testing Summary

### Unit Tests (Content Validation)
- Valid UTF-8 detection
- Binary content rejection
- Max length enforcement
- Null byte detection
- Empty string handling

### Unit Tests (Edge Cases - Existing)
- 32 tests in handlers/edge_cases.rs
- Unicode (5 tests)
- Special characters (3 tests)
- Long messages (4 tests)
- Whitespace (5 tests)
- Binary validation (2 tests)
- Timestamps (5 tests)
- History edges (4 tests)
- ChatMessage edges (4 tests)

### Performance Tests
- 100KB message signing time
- 10,000 iteration determinism test
- Memory usage for long messages

### Integration Tests
- Full edge case flow from compose to verify
- Server validation of edge case messages
- Client verification of edge case signatures

### Performance Requirements
- Message signing: <100ms (including 100KB messages)
- Content validation: <1ms
- Verification: <100ms
- Memory: No issues with 100KB messages

---

## Status: ready-for-dev

**Story Context:**
- Epic 3: Core Messaging - FINAL story
- Previous: Story 3.7 (Preserve Composer Draft on Disconnection)
- Next: Epic 4: Transparency - Drill-Down Details

**Implementation Approach:**
1. Review existing edge case tests in handlers/edge_cases.rs
2. Create content validation module in utils/validation.rs
3. Integrate validation into composer flow
4. Add performance benchmarks
5. Run full test suite to validate
6. Document findings

**Key Success Criteria:**
- All 32 existing edge case tests pass
- New content validation tests pass
- Performance targets met (<100ms signing)
- Binary content properly rejected
- Epic 3 messaging complete and robust
