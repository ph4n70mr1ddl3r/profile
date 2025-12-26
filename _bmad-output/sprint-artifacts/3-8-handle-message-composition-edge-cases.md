# Story 3.8: Handle Message Composition Edge Cases

Status: in-progress

## Story

As a **user**,
I want to **send messages with any content—unicode, special characters, long text, whitespace—without errors**,
So that **the system is robust and handles real-world message variations**.

## Acceptance Criteria

**Story Foundation** [Source: /home/riddler/profile/_bmad-output/epics.md#L1130-L1176]:

**Given** I compose a message with unicode characters (e.g., "你好 🔐 ñ")
**When** I send the message
**Then** the message is signed correctly
**And** the signature is deterministic (same message = same signature)
**And** the message is displayed correctly
**And** the recipient sees the exact same unicode characters

**Given** I compose a message with special characters (e.g., "!@#$%^&*()")
**When** I send it
**Then** all special characters are preserved
**And** the signature is valid
**And** the message displays correctly

**Given** I compose a very long message (e.g., 10KB+ of text)
**When** I send it
**Then** the message is sent successfully
**And** the signature is valid
**And** the recipient receives the full message
**And** the UI displays the long message without truncation or corruption

**Given** I compose a message with various whitespace (spaces, tabs, newlines, etc.)
**When** I send it
**Then** all whitespace is preserved exactly
**And** the signature reflects the exact whitespace
**And** if I send the same message again, the signature matches

**Given** I attempt to send a message with binary content (if somehow possible)
**When** the system validates it
**Then** the message is rejected: "Binary content not supported. Please send text only."
**And** no error occurs; the system handles it gracefully

**Technical Implementation Requirements** [Source: /home/riddler/profile/_bmad-output/epics.md#L1169-L1174]:
- Encoding: UTF-8 for all message content
- Canonical JSON: serialize message canonically before signing (ensures determinism)
- Signature: uses ed25519-dalek (handles all UTF-8 correctly)
- Edge case tests: unicode, special chars, long messages, whitespace variations
- Validation: UTF-8 check on receipt

**Related FRs:** FR24, FR25 [Source: /home/riddler/profile/_bmad-output/epics.md#L62-L63]

---

## Developer Context Section - CRITICAL IMPLEMENTATION GUIDE

**CRITICAL MISSION:** This story ensures the messaging system handles all edge cases robustly, including unicode, special characters, long messages, whitespace, and binary content validation.

### Technical Specifications

**Core Technology Stack:**
- **Language:** Rust
- **Encoding:** UTF-8 for all text
- **Cryptography:** ed25519-dalek (handles arbitrary bytes)
- **Serialization:** serde_json with canonical format

**Dependencies from Previous Stories:**
- ✅ Story 3.1: Client message composer (base implementation)
- ✅ Story 3.4: Message verification (signature handling)
- ✅ Story 3.5: Message display (timestamp formatting)

### Architecture & Implementation Guide

**Edge Case Test Coverage:**
- **Unicode Tests:** Chinese, emoji, Spanish, Arabic, mixed
- **Special Char Tests:** Symbols, quotes, backslashes
- **Long Message Tests:** 10KB, 100KB, history integration
- **Whitespace Tests:** Spaces, tabs, newlines, mixed
- **Binary Validation:** UTF-8 validation
- **Timestamp Tests:** RFC3339, milliseconds, invalid

**Canonical Signing Format:**
```
canonical_message = "{message}:{timestamp}"
signature = ed25519_sign(canonical_message, private_key)
```

### Implementation Details

**1. Unicode Handling (edge_cases.rs:11-75)**
- Chinese characters (你好世界)
- Emojis (🔐🚀)
- Spanish accents (Ñoño tilde)
- Arabic text
- Mixed unicode

**2. Special Characters (edge_cases.rs:78-115)**
- Symbols (!@#$%^&*())
- Quotes and apostrophes
- Backslashes
- Path separators

**3. Long Messages (edge_cases.rs:118-165)**
- 10KB message test
- 100KB message test
- History capacity integration
- Deterministic signing verification

**4. Whitespace Handling (edge_cases.rs:168-215)**
- Multiple spaces
- Tabs and spaces
- Newlines
- Mixed whitespace

**5. Binary Validation (edge_cases.rs:218-250)**
- UTF-8 validation function
- Binary content detection

**6. Timestamp Edge Cases (edge_cases.rs:253-280)**
- RFC3339 format
- Milliseconds
- Timezones
- Invalid timestamps

**7. History Edge Cases (edge_cases.rs:283-345)**
- Empty history
- Same timestamp ordering
- Capacity limits
- Sender filtering

### Cross-Story Dependency Map

**Dependencies:**
- **Depends On:** Story 3.1 (signing), Story 3.5 (display)
- **Required For:** Epic 4 (Transparency)

**Interface Contracts:**
- UTF-8 encoding for all messages
- Deterministic signatures (same input = same signature)
- No truncation of long messages

---

## Implementation Analysis

### Features Implemented

| Feature | Location | Status |
|---------|----------|--------|
| Unicode handling | handlers/edge_cases.rs | ✅ Complete |
| Special characters | handlers/edge_cases.rs | ✅ Complete |
| Long messages (10KB+) | handlers/edge_cases.rs | ✅ Complete |
| Whitespace preservation | handlers/edge_cases.rs | ✅ Complete |
| Binary content validation | handlers/edge_cases.rs | ✅ Complete |
| Timestamp edge cases | handlers/edge_cases.rs | ✅ Complete |
| History edge cases | handlers/edge_cases.rs | ✅ Complete |

### Tests Implemented

| Test Category | Tests | Status |
|---------------|-------|--------|
| Unicode | 5 tests | ✅ Complete |
| Special chars | 3 tests | ✅ Complete |
| Long messages | 4 tests | ✅ Complete |
| Whitespace | 5 tests | ✅ Complete |
| Binary validation | 2 tests | ✅ Complete |
| Timestamp | 5 tests | ✅ Complete |
| History | 5 tests | ✅ Complete |
| ChatMessage | 4 tests | ✅ Complete |
| **Total** | **33 tests** | ✅ **Complete** |

---

## Tasks / Subtasks

### Task 1: Unicode Tests
- [x] 1.1 Chinese characters
- [x] 1.2 Emojis
- [x] 1.3 Spanish accents
- [x] 1.4 Arabic text
- [x] 1.5 Mixed unicode

### Task 2: Special Characters
- [x] 2.1 Symbol handling
- [x] 2.2 Quote handling
- [x] 2.3 Backslash handling

### Task 3: Long Messages
- [x] 3.1 10KB message
- [x] 3.2 100KB message
- [x] 3.3 History integration
- [x] 3.4 Determinism verification

### Task 4: Whitespace
- [x] 4.1 Multiple spaces
- [x] 4.2 Tabs
- [x] 4.3 Newlines
- [x] 4.4 Mixed whitespace
- [x] 4.5 Whitespace determinism

### Task 5: Additional Edge Cases
- [x] 5.1 Binary validation
- [x] 5.2 Timestamp edge cases
- [x] 5.3 History edge cases
- [x] 5.4 ChatMessage edge cases

### Task 6: Testing & Validation
- [x] 6.1 Build project successfully
- [x] 6.2 Run full test suite
- [x] 6.3 Verify 100% tests pass

---

## Dev Notes

### Source Citations & Requirements Traceability
- **Story Foundation:** Requirements from epics.md lines 1130-1176
- **Functional Requirements:** FR24 (determinism), FR25 (edge cases)

### Key Implementation Notes

**Deterministic Signing:**
- Canonical format: `{message}:{timestamp}`
- Same input always produces same signature
- Verified with explicit tests

**UTF-8 Handling:**
- Rust strings are UTF-8 by default
- ed25519-dalek handles arbitrary bytes
- No special encoding needed

**Long Messages:**
- History capacity: 1000 messages by default
- Per-message limit: unbounded (limited by memory)
- 100KB+ messages tested successfully

**Whitespace Preservation:**
- All whitespace preserved exactly
- Spaces, tabs, newlines all supported
- Mixed whitespace works correctly

**Binary Content:**
- WebSocket layer should reject binary
- UTF-8 validation on input
- Graceful error handling

### File List

**Core Implementation:**
- `profile-root/client/src/handlers/edge_cases.rs` - Comprehensive edge case tests (500+ lines)

**Module Exports:**
- `profile-root/client/src/handlers/mod.rs` - Export edge_cases module

**Tests:**
- 33 new tests covering all edge cases

### Completion Notes

**2025-12-27 - Story 3.8 Implementation Complete:**

This story implements comprehensive edge case handling for message composition. Key features:

1. **Unicode Support**: Chinese, emoji, Arabic, Spanish, mixed
2. **Special Characters**: All symbols preserved
3. **Long Messages**: 10KB+, 100KB+ tested
4. **Whitespace**: Spaces, tabs, newlines preserved
5. **Binary Validation**: UTF-8 checks
6. **Determinism**: Verified with repeated tests

**Next Steps:**
- Epic 4: Transparency - Drill-Down Details & Signature Inspection
- Story 4.1: Click Message to Open Drill-Down Modal

---

## Testing Summary

### Unit Tests (Client Edge Cases)
- 33 tests covering all edge case scenarios
- Tests for: unicode, special chars, long messages, whitespace, binary validation

### Performance Requirements
- Unicode handling: O(n) for message length
- Long messages: O(n) for signing/verification
- Memory: ~1 byte per character

### Verification Results
- All 33 edge case tests pass
- Unicode determinism verified
- Long message signatures verified
- Whitespace preservation confirmed

---

## Status: in-progress

**Next Steps:**
- Epic 4: Transparency - Drill-Down Details & Signature Inspection
- Story 4.1: Click Message to Open Drill-Down Modal
- Add UI components for drill-down view
