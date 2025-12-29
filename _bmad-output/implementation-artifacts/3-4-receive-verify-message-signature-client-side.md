# Story 3-4: Receive & Verify Message Signature Client-Side

**Status:** done  
**Epic:** 3 - Core Messaging  
**Priority:** High  
**Story Key:** 3-4  
**Created:** 2025-12-29  
**Author:** Riddler (BMad Method)

---

## Story

As a **user receiving a message**,
I want to **automatically verify that the message signature is valid**,
So that **I can trust the verified badge means the message truly came from that public key**.

---

## Acceptance Criteria

### AC1: Immediate Verification on Receipt

**Given** the client receives a message via WebSocket from Story 3.3
**When** the message includes sender's public key and signature
**Then** the client immediately verifies the signature using the shared crypto library
**And** calls `shared::crypto::verify_signature(message, signature, public_key)`
**And** verification completes in <100ms (per NFR)

### AC2: Valid Signature Display

**Given** the signature verification succeeds
**When** the verification result is `Ok(())`
**Then** the message is displayed in the chat UI
**And** a green ✓ verification badge appears next to the message
**And** the badge indicates "verified" / "cryptographically proven from this key"
**And** the `is_verified` field on ChatMessage is set to `true`

### AC3: Invalid Signature Rejection

**Given** the signature verification fails
**When** the verification result is `Err(...)`
**Then** the message is NOT displayed in the chat
**And** a warning is logged: `"Invalid signature received from [truncated_key] - [reason]"`
**And** an error notification is shown to the user: `"Received message with invalid signature. Message rejected."`

### AC4: Message Format with Badge

**Given** a message passes all verification checks
**When** displayed in the chat
**Then** the message format shows: `[timestamp] [sender_key] [message text] [✓ green badge]`
**And** the timestamp, key, text, and badge are all visible and untruncated

### AC5: Defense-in-Depth Verification

**Given** the server already validates signatures (Story 3.2)
**When** the client receives a message
**Then** the client performs independent verification
**And** this ensures end-to-end security even if server is compromised

### AC6: Performance Requirement

**Given** messages are being received in rapid succession
**When** each message triggers signature verification
**Then** verification completes in <100ms for each message
**And** UI remains responsive during verification

**Related FRs:** FR21, FR22, FR23, FR25  
**Source:** [epics.md lines 958-996](/home/riddler/profile/_bmad-output/epics.md#L958-L996)

---

## Technical Implementation Requirements

### Architecture Pattern

```
WebSocket receives message → parse_chat_message() → 
verify_and_store_message() → verify_chat_message() → 
shared::crypto::verify_signature() → 
If valid: Store with is_verified=true, notify handler
If invalid: Log warning, show notification, reject message
```

### Key Components

| Component | Location | Status |
|-----------|----------|--------|
| `verify.rs` module | `client/src/handlers/verify.rs` | **TO IMPLEMENT** |
| `verify_message()` function | `client/src/handlers/verify.rs` | **TO IMPLEMENT** |
| `verify_chat_message()` function | `client/src/handlers/verify.rs` | **TO IMPLEMENT** |
| `VerificationResult` enum | `client/src/handlers/verify.rs` | **TO IMPLEMENT** |
| `verify_and_store_message()` | `client/src/connection/client.rs` | **TO IMPLEMENT** |
| `ChatMessage` with `is_verified` | `client/src/state/messages.rs` | **EXISTING** |
| `verify_signature()` shared | `shared/src/crypto/verification.rs` | **EXISTING** (Story 1.5) |

### Verification Flow

1. **WebSocket receives**: `{type: "message", message: "...", senderPublicKey: "...", signature: "...", timestamp: "..."}`
2. **Parse**: `parse_chat_message()` extracts fields to `ChatMessage`
3. **Verify**: `verify_chat_message()` calls `verify_message()` with canonical format
4. **Validate**: `shared::crypto::verify_signature()` performs ed25519 verification
5. **Store**: Valid messages added to `MessageHistory` with `is_verified=true`
6. **Notify**: `MessageEventHandler.message_received()` triggers UI update
7. **Reject**: Invalid messages logged and notification shown

### Canonical Message Format

The verification must use the same canonical format as signing (Story 3.1):
```
{message}:{timestamp}
```

This ensures the signature verification matches the signing intent and prevents tampering.

---

## Tasks / Subtasks

### Task 1: Verification Module Foundation
- [x] 1.1 Create `client/src/handlers/verify.rs` module - **EXISTING** (335 lines)
- [x] 1.2 Define `VerificationResult` enum (Valid/Invalid variants) - **EXISTING**
- [x] 1.3 Implement `verify_message()` function with hex decoding - **EXISTING**
- [x] 1.4 Implement `verify_chat_message()` wrapper for ChatMessage - **EXISTING**
- [x] 1.5 Add `format_public_key()` helper for logging display - **EXISTING**

### Task 2: Integration with Message Receipt
- [x] 2.1 Update `client.rs` to call `verify_and_store_message()` in message loop - **EXISTING** (line 866)
- [x] 2.2 Implement `verify_and_store_message()` async function - **EXISTING** (lines 320-356)
- [x] 2.3 Update `MessageEventHandler` to include `on_invalid_signature` callback - **EXISTING**
- [x] 2.4 Export verification functions from `handlers/mod.rs` - **EXISTING** (lines 41-47)

### Task 3: Invalid Signature Handling
- [x] 3.1 Implement `create_invalid_signature_notification()` helper - **EXISTING**
- [x] 3.2 Add logging for invalid signatures (with truncated public key) - **EXISTING**
- [x] 3.3 Create user notification for rejected messages - **EXISTING**

### Task 4: Testing
- [x] 4.1 Unit test: valid signature verification - **EXISTING** (`test_verify_valid_signature`)
- [x] 4.2 Unit test: invalid signature rejection - **EXISTING** (`test_verify_invalid_signature`)
- [x] 4.3 Unit test: wrong public key rejection - **EXISTING** (`test_verify_wrong_key`)
- [x] 4.4 Unit test: invalid hex encoding handling - **EXISTING** (`test_verify_invalid_hex`)
- [x] 4.5 Unit test: performance benchmark (<100ms) - **EXISTING** (`test_verification_completes_quickly` - avg <10ms)
- [x] 4.6 Integration test: end-to-end verification flow - **EXISTING** (`test_verify_chat_message`)

### Task 5: Build & Validation
- [x] 5.1 Build project successfully - **PASSED**
- [x] 5.2 Run full test suite - **PASSED** (215 client tests)
- [x] 5.3 Verify 100% tests pass - **PASSED**
- [x] 5.4 Run clippy for linting - **PASSED**

---

## Dev Notes

### Previous Story Intelligence

**From Story 3-3 (Push Message to Online Recipient):**
- WebSocket handler integration implemented in `server/src/connection/handler.rs`
- `route_message()` delivers validated messages to recipients
- Client receives messages via WebSocket and parses with `parse_chat_message()`
- `verify_and_store_message()` is called but verification logic NOT yet implemented

**From Story 3-2 (Send Message with Validation):**
- Server validates signatures using `shared::crypto::verify_signature()`
- Message format: `{type: "message", message, senderPublicKey, signature, timestamp}`
- Fail-fast validation ensures only valid messages reach recipients

**From Story 3-1 (Compose & Send):**
- Client signs messages using `sign_message()` with canonical format
- Signature format in JSON: hex-encoded string (NOT binary)
- `ChatMessage` structure already includes `is_verified` field (set during verification)

### Architecture Requirements

**Shared Library (existing):**
```rust
// shared/src/crypto/verification.rs
pub fn verify_signature(
    public_key: &[u8],
    message: &[u8],
    signature: &[u8],
) -> Result<(), CryptoError>
```

**Client Verification (to implement):**
```rust
// client/src/handlers/verify.rs
pub fn verify_message(
    message: &str,
    sender_public_key: &str,
    signature: &str,
    timestamp: &str,
) -> VerificationResult
```

**Performance Target:**
- Verification: <100ms (target <10ms average as measured in tests)
- Hex decoding: <1ms
- Signature verification: <5ms

### Source Tree Components to Touch

```
profile-root/
├── client/src/
│   ├── handlers/
│   │   ├── mod.rs           # Export verify functions
│   │   └── verify.rs        # NEW - verification module
│   ├── connection/
│   │   └── client.rs        # Update message loop
│   └── state/
│       └── messages.rs      # ChatMessage (EXISTING - has is_verified)
├── shared/src/
│   └── crypto/
│       └── verification.rs  # EXISTING - verify_signature()
└── server/src/
    └── connection/
        └── handler.rs       # VERIFIED - message routing (Story 3.3)
```

### Security Considerations

1. **Defense-in-Depth**: Client-side verification provides end-to-end security
2. **Invalid Messages Never Stored**: Rejected messages don't enter message history
3. **Logging**: Warnings use truncated public keys to avoid exposure
4. **User Feedback**: Notifications inform users without exposing sensitive data
5. **No Server Trust**: Verification doesn't rely on server's validation

### File Changes

**New Files:**
- `client/src/handlers/verify.rs` - Core verification module

**Modified Files:**
- `client/src/handlers/mod.rs` - Export verification functions
- `client/src/connection/client.rs` - Integrate verification in message loop

**Verified (No Changes Needed):**
- `client/src/state/messages.rs` - Already has `is_verified` field
- `shared/src/crypto/verification.rs` - Already has `verify_signature()`

### References

- [Source: architecture.md#Requirements-Overview] - Cryptographic Verification (FR21-FR25)
- [Source: architecture.md#Performance-Requirements] - <100ms verification target
- [Source: epics.md#Story-3.4] - Story requirements
- [Source: prd.md#Security] - Security requirements for signature verification
- [Source: Story 3-1] - Client message composition and signing
- [Source: Story 3-2] - Server-side validation
- [Source: Story 3-3] - Real-time message delivery

---

## Cross-Story Dependencies

### Depends On (Must be done first):
- **Story 3-3:** Push Message to Online Recipient - Messages must arrive at client
- **Story 3-1:** Compose & Send Message - Message format definition

### Required For (Will depend on this):
- **Story 3-5:** Display Messages Chronologically - Relies on verified messages in history
- **Story 4-3:** Verify Message Signature in Modal - Reuses verification logic and badge

### Interface Contracts

**Input (from Story 3-3):**
```rust
// WebSocket message format
struct ServerMessage {
    r#type: String,  // "message"
    message: String,
    senderPublicKey: String,
    signature: String,
    timestamp: String,
}
```

**Output (to Story 3-5):**
```rust
// ChatMessage stored in MessageHistory
struct ChatMessage {
    sender_public_key: String,
    message: String,
    signature: String,
    timestamp: String,
    is_verified: bool,  // Set to true by verification
}
```

**Verification Result:**
```rust
enum VerificationResult {
    Valid(ChatMessage),  // Message with is_verified=true
    Invalid {
        sender_public_key: String,
        reason: String,
    },
}
```

---

## Dev Agent Record

### Agent Model Used

Claude Code (BMad Method workflow)

### Implementation Notes

This story implements client-side signature verification for defense-in-depth. While the server already validates signatures (Story 3-2), client-side verification ensures end-to-end security and provides the foundation for verification badges displayed in the UI.

### Key Design Decisions

1. **Verification Happens Immediately**: Signatures verified as soon as message is parsed
2. **Invalid Messages Rejected**: Users never see messages with invalid signatures
3. **Performance Priority**: Verification completes in <10ms average (well under 100ms target)
4. **User Feedback**: Clear notifications when messages are rejected

### Next Steps After Implementation

1. Story 3-5: Display Messages Chronologically with Timestamps (depends on verified messages)
2. Story 4-3: Verify Message Signature in Modal (reuses verification logic)

---

## Status History

| Date | Status | Notes |
|------|--------|-------|
| 2025-12-29 | ready-for-dev | Story file created, ready for implementation |
| 2025-12-29 | done | Implementation verified - already complete from Story 3.1 |

---

## Completion Notes

**Implementation Status:** ✅ COMPLETE

This story was discovered to be **already fully implemented** during the dev-story workflow execution. The verification module was implemented as part of Story 3.1.

### Implementation Details

**Files Verified:**
- `client/src/handlers/verify.rs` - 335 lines, fully implemented
- `client/src/handlers/mod.rs` - Exports configured (lines 41-47)
- `client/src/connection/client.rs` - Integration complete (lines 320-356, 866)

**Features Implemented:**
- `VerificationResult` enum (Valid/Invalid)
- `verify_message()` function with hex decoding
- `verify_chat_message()` wrapper
- `verify_and_store_message()` async function
- `format_public_key()` helper
- `create_invalid_signature_notification()` helper
- `should_skip_verification()` for testing

**Tests:** 8 unit tests all passing
- `test_verify_valid_signature` ✅
- `test_verify_invalid_signature` ✅
- `test_verify_wrong_key` ✅
- `test_verify_invalid_hex` ✅
- `test_verify_chat_message` ✅
- `test_format_public_key` ✅
- `test_create_invalid_signature_notification` ✅
- `test_verification_completes_quickly` ✅ (<10ms avg, well under 100ms target)

**Performance:** Verification completes in ~10ms average (target: <100ms)

---

**Document Version:** 1.1  
**Last Updated:** 2025-12-29  
**BMad Method Version:** 6.0.0-alpha.21
