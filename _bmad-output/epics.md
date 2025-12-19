---
stepsCompleted: [1, 2, 3, 4]
inputDocuments: ["/home/riddler/profile/_bmad-output/prd.md", "/home/riddler/profile/_bmad-output/architecture.md", "/home/riddler/profile/_bmad-output/ux-design-specification.md"]
projectName: profile
userName: Riddler
date: 2025-12-19
epicStructureApproved: true
epicApprovalDate: 2025-12-19
storiesCreated: 23
totalStoriesApproved: 23
storiesCreationDate: 2025-12-19
workflowStatus: COMPLETE
workflowCompletionDate: 2025-12-19
validationStatus: PASSED
readyForDevelopment: true
---

# profile - Epic Breakdown

## Overview

This document provides the complete epic and story breakdown for **profile**, decomposing the requirements from the PRD, Architecture, and UX Design documents into implementable stories organized by user value delivery.

Profile is a privacy-forward instant messaging application where users authenticate with their own private keys and maintain complete control over their digital identity. All messages are deterministically signed with cryptographic signatures, making message authenticity verifiable by anyone while keeping identity ownership decentralized.

---

## Requirements Inventory

### Functional Requirements (47 total)

#### Key Management (FR1-FR5)
- FR1: Users can generate a new 256-bit random private key within the application
- FR2: Users can import an existing 256-bit private key by pasting it
- FR3: Users can view their public key derived from their private key
- FR4: The system derives the correct public key from any imported private key
- FR5: The system securely stores the user's private key in memory during the session

#### User Authentication & Connection (FR6-FR11)
- FR6: Users can connect to the server via WebSocket with their public key and a signature proving key ownership
- FR7: The server validates the authentication signature against the user's public key
- FR8: Upon successful authentication, the server adds the user to the active online lobby
- FR9: The server maintains an active WebSocket connection with each authenticated user
- FR10: Users receive a notification when their authentication fails (invalid signature)
- FR11: Users are disconnected from the server when their WebSocket connection closes

#### Message Operations (FR12-FR20)
- FR12: Users can compose and send a message to any online user
- FR13: Users can select a recipient from the list of online users
- FR14: The system signs each message with a deterministic signature using the user's private key before sending
- FR15: The server receives sent messages and validates the sender's signature against their public key
- FR16: The server notifies the sender if the recipient is offline (cannot receive the message)
- FR17: The server pushes received messages to the recipient in real-time via WebSocket
- FR18: The recipient's client receives the pushed message with sender's public key and signature intact
- FR19: Received messages display in chronological order in the chat interface
- FR20: Messages include a timestamp showing when they were sent

#### Cryptographic Verification (FR21-FR25)
- FR21: The recipient's client validates the message signature against the sender's public key
- FR22: Valid signatures trigger a "verified" badge (✓) displayed next to the message
- FR23: Invalid signatures are rejected and the message is not displayed
- FR24: Deterministic signatures are generated consistently: identical message + same key = identical signature every time
- FR25: Signature verification works correctly for all message content types (unicode, special characters, long text, etc.)

#### User Presence & Lobby (FR26-FR33)
- FR26: The server maintains a list of all currently online users
- FR27: Each online user entry displays their public key
- FR28: Users can query the server for the current list of online users
- FR29: The client displays the online user lobby showing all connected users
- FR30: Users can select any online user from the lobby to start messaging
- FR31: When a user connects, other users are notified that they joined the lobby
- FR32: When a user disconnects, other users are notified that they left the lobby
- FR33: The online lobby updates in real-time as users join and leave

#### Message Details & Verification Display (FR34-FR39)
- FR34: Users can drill down on any message to view full cryptographic details
- FR35: The drill-down view displays the message content
- FR36: The drill-down view displays the sender's public key
- FR37: The drill-down view displays the cryptographic signature
- FR38: The drill-down view displays the verification status (verified ✓ or invalid)
- FR39: The verified badge is prominently displayed for verified messages

#### Data Persistence (FR40-FR44)
- FR40: All message history is ephemeral and cleared when the user closes the application
- FR41: The user's private key is stored only in memory and not persisted to disk
- FR42: The online lobby state is maintained only for the current session
- FR43: The server stores no persistent user database
- FR44: The server does not persist messages between sessions

#### Offline Behavior (FR45-FR47)
- FR45: When a user attempts to send a message to an offline recipient, the server sends an offline notification
- FR46: The offline notification informs the sender that the recipient is currently unavailable
- FR47: The sender can resend the message after the recipient comes back online

---

### Non-Functional Requirements

#### Performance
- **Message Signing**: Message signing operations must complete within 100ms to feel instant to users
- **Signature Verification**: Signature verification on received messages must complete within 100ms of receipt
- **WebSocket Message Delivery**: Messages must be delivered from sender to recipient in real-time, with end-to-end latency under 500ms
- **Lobby Updates**: Changes to the online user lobby (users joining/leaving) must propagate to all connected clients within 100ms
- **Concurrent Users**: The server must support as many concurrent users as the underlying infrastructure allows, with no artificial limits imposed by the application
- **Deterministic Signature Consistency**: Signatures must be generated with 100% consistency—identical message + same key must produce identical signature every time, measurable across thousands of iterations

#### Security
- **Private Key Protection**: Private keys must never leave the client application and must be stored only in memory during the session, never persisted to disk or transmitted to the server
- **Private Key in Memory**: Private keys must be securely held in application memory and cleared from memory when the application closes
- **Signature Validation Accuracy**: All message signatures must be validated with 100% accuracy; any signature that cannot be verified against the sender's public key must be rejected and not displayed
- **Invalid Signature Handling**: Messages with invalid or unverifiable signatures must not be displayed to the user; they must be rejected with a clear indication of why (invalid signature)
- **Connection Authentication**: WebSocket connections must be authenticated using cryptographic signatures proving ownership of the private key; unauthenticated connections must be rejected
- **Message Content Encoding**: Messages must be validated as text-based UTF-8 encoded content; binary content is not supported and must be rejected

#### Scalability
- **User Growth Path**: The MVP architecture must support the addition of scalability features in Phase 2 without requiring fundamental redesign
- **Concurrent Connection Handling**: The server must handle connection/disconnection events smoothly with no performance degradation as users join and leave
- **Message Queue Management**: The system must handle message queuing and real-time delivery efficiently for all concurrent users

#### Technical Stack (from Architecture)
- **Server**: Rust language with Tokio async runtime (1.35+) for WebSocket handling
- **Client**: Rust language with Slint UI framework (1.5+) for Windows desktop
- **Shared Library**: ed25519-dalek (2.1+) for deterministic cryptographic signing
- **Communication**: WebSocket protocol via tokio-tungstenite (0.21+)
- **Serialization**: serde/serde_json (1.0+) for message serialization with canonical JSON encoding
- **Platform**: Windows desktop (cross-platform support possible post-MVP)

---

### Additional Requirements from Architecture

#### Implementation Patterns Required
1. **Rust Module Organization**: Snake_case naming, nested modules by responsibility (validation/, signing/, routing/, etc.)
2. **File Structure**: Inline unit tests with `#[cfg(test)]` blocks, integration tests in `tests/` directory
3. **Message Protocol**: Simple JSON format without wrappers, always include `type` field, snake_case field names, ISO 8601 timestamps, hex-encoded binary data
4. **Error Handling**: Predefined reason codes (signature_invalid, offline, malformed_json, auth_failed, connection_lost), simple `{type, reason, details}` format
5. **State Management**: Enum-based states (not boolean flags), pattern matching for exhaustiveness, clear state transitions

#### Architecture Decisions to Support
1. **Cryptographic Signing**: Canonical JSON encoding → deterministic ed25519 signing → binary signature with length prefix → hex display in UI
2. **Server Validation & Routing**: Strict validation sequence (authentication → format → signature → recipient lookup), fail-fast approach
3. **Client-Side Signing**: Pre-send signing with zeroize-protected private keys, immediate badge display on send
4. **WebSocket Protocol**: Simple handshake with signature proof, lobby push updates with deltas, error messages with human-readable details
5. **Error Handling**: Network disconnection preserves drafts, authentication failure triggers immediate disconnect, invalid key imports show two-level errors
6. **Testing**: Tokio integration tests, edge case coverage (unicode, special chars, long messages), 10,000-iteration determinism validation

#### Component Architecture (from UX Design)
- **9 Core Custom Components**: Lobby (user list), Chat (message display), Composer (input), VerificationBadge, DrillDown (details modal), KeyDisplay, StatusBadge, KeyboardShortcutsHelp, Notifications
- **Design System**: Dark mode theme, 8px spacing grid, monospace for cryptographic data, blue (#0066CC) for identity, green (#22c55e) for verified
- **Layout**: Two-column (Lobby + Chat), keyboard-first navigation, Enter to send, Escape to close modal
- **Drill-Down Modal**: Layer 1 (message content) → Layer 2 (signature) → Layer 3 (verification status), all expandable

---

### Requirements Coverage Map

| Requirement Category | Total | Epic Allocation |
|---|---|---|
| Key Management (FR1-5) | 5 | Epic 1 (Foundation) |
| Authentication & Connection (FR6-11) | 6 | Epic 1 (Foundation) |
| Message Operations (FR12-20) | 9 | Epic 3 (Core Messaging) |
| Cryptographic Verification (FR21-25) | 5 | Epic 3 (Core Messaging) |
| User Presence & Lobby (FR26-33) | 8 | Epic 2 (Presence) |
| Message Details & Display (FR34-39) | 6 | Epic 4 (Transparency) |
| Data Persistence (FR40-44) | 5 | Architectural Constraint (All Epics) |
| Offline Behavior (FR45-47) | 3 | Epic 3 (Core Messaging) |
| **TOTAL FRs COVERED** | **45** | **Across 4 Epics** |
| **Architectural Constraints** | **2** | **Applies to All Epics** |
| **TOTAL FRs** | **47** | **Complete Coverage** |
| Non-Functional Requirements | 6 categories | All Epics |
| Implementation Patterns | 5 patterns | Engineering foundation |
| Architecture Decisions | 6 decisions | Technical structure |
| UI Components | 9 components | Epics 3 & 4 |

**Epic Allocation Summary:**
- **Epic 1 (Foundation):** FR1-11 (11 requirements)
- **Epic 2 (Presence):** FR26-33 (8 requirements)
- **Epic 3 (Core Messaging):** FR12-25, FR45-47 (20 requirements)
- **Epic 4 (Transparency):** FR34-39 (6 requirements)
- **Architectural Constraints:** FR40-44 (5 requirements, cross-cutting)

---

## Epic List

### Epic 1: Foundation - Key Management & Authentication

**Epic Goal:** Users establish their cryptographic identity by generating or importing a private key, then prove ownership to the server through cryptographic authentication.

**User Outcome:** After this epic, users have a secure identity (private key) and can connect to the server to join the online community.

**User Archetype Success:**
- **Alex** (First-Time User): Generates a new key and feels empowered ("This is my identity. I own it.")
- **Sam** (Technical Validator): Imports an existing key and validates the cryptographic mechanism works

**Why This Epic First:** This is the foundational layer. Users cannot send or receive messages without establishing identity and proving it to the server. All future epics depend on this working correctly.

**Standalone Functionality:** ✅ Complete identity setup works independently. Users can establish identity without any messaging features.

**FRs Covered:** FR1-11 (11 requirements)
- Key generation (FR1)
- Key import with validation (FR2)
- Public key display (FR3, FR4)
- Secure key storage (FR5)
- WebSocket authentication (FR6, FR7)
- Connection management (FR8, FR9)
- Error handling & disconnection (FR10, FR11)

**Non-Functional Requirements Addressed:**
- Security: Private key protection, memory-only storage, no disk persistence
- Performance: Key generation <100ms, authentication <500ms
- Technical Stack: Rust + Tokio + ed25519-dalek

---

### Epic 2: Presence - Online Lobby & Real-Time Updates

**Epic Goal:** Maintain and display a real-time list of online users so users know who's available to message, with instant notifications when users join or leave.

**User Outcome:** After this epic, users can see who's online and receive real-time updates when people connect or disconnect.

**User Archetype Success:**
- **Alex** (First-Time User): Opens the app and sees a list of online users, feels connected
- **Sam** (Technical Validator): Verifies presence updates happen reliably and users join/leave events are broadcast correctly

**Why This Epic Second:** Once users have established identity (Epic 1), they need to know who they can message. The lobby is the gateway to messaging. Natural second step in user journey.

**Standalone Functionality:** ✅ Lobby works independently. Users can see who's online even if messaging features have bugs. This is a complete presence management system.

**Depends On:** Epic 1 (users must authenticate first to see lobby)

**FRs Covered:** FR26-33 (8 requirements)
- Server lobby maintenance (FR26)
- User listing with public keys (FR27, FR29)
- Lobby querying (FR28, FR30)
- Presence notifications (FR31, FR32)
- Real-time updates (FR33)

**Non-Functional Requirements Addressed:**
- Performance: Lobby updates propagate within 100ms
- Scalability: Support arbitrary concurrent users
- Technical Stack: Tokio broadcast for efficient multi-recipient updates

---

### Epic 3: Core Messaging - Send, Receive & Cryptographic Verification

**Epic Goal:** Enable users to send deterministically signed messages and receive cryptographically verified messages in real-time, with automatic signing and instant verification.

**User Outcome:** After this epic, users can send messages that are cryptographically proven to come from their identity, and receive verified messages with automatic signature validation. This is the primary value of Profile.

**User Archetype Success:**
- **Alex** (First-Time User): Sends first message, sees ✓ green verification badge, feels their message is proven
- **Sam** (Technical Validator): Sends identical messages twice, compares signatures, confirms deterministic signing is working

**Why This Epic Core:** This is the PRIMARY USER VALUE. Everything in Profile exists to enable this: sending and receiving cryptographically verified messages. This epic is the heart of the product.

**Standalone Functionality:** ✅ Complete messaging system. Users can send/receive/verify messages. Depends on Epics 1 & 2, but is a self-contained messaging experience.

**Depends On:** Epics 1 (authentication) & 2 (lobby/presence)

**FRs Covered:** FR12-25, FR45-47 (20 requirements)
- Message composition (FR12, FR13)
- Deterministic signing (FR14, FR24)
- Server validation (FR15)
- Message transmission (FR17, FR18)
- Chronological display (FR19, FR20)
- Client-side verification (FR21, FR22, FR23)
- Edge case handling (FR25)
- Offline notifications (FR45, FR46, FR47)

**Non-Functional Requirements Addressed:**
- Performance: Message signing <100ms, verification <100ms, delivery <500ms
- Security: Signature validation accuracy 100%, invalid signatures rejected
- Determinism: Identical signatures for identical messages (100% consistency)
- Scalability: Efficient message routing for many concurrent users

---

### Epic 4: Transparency - Drill-Down Details & Signature Inspection

**Epic Goal:** Enable users to inspect the cryptographic proof behind messages by viewing the sender's public key, full signature, and verification status.

**User Outcome:** After this epic, users can click any message to see its complete cryptographic details and validate authenticity themselves.

**User Archetype Success:**
- **Alex** (First-Time User): Clicks a message, discovers the signature, learns how verification works through exploration
- **Sam** (Technical Validator): Inspects full signatures, validates deterministic signing by comparing identical messages, confirms cryptographic foundation

**Why This Epic Last:** This layer provides transparency and education. It's not required for basic messaging (Epic 3 is complete). It adds depth for curious users and technical validation for experts.

**Standalone Functionality:** ✅ All drill-down features are self-contained. Users don't need this epic to send/receive messages, but those who want to understand the proof can.

**Depends On:** Epic 3 (messages must exist to drill down on them)

**FRs Covered:** FR34-39 (6 requirements)
- Drill-down modal (FR34)
- Message details display (FR35)
- Sender public key display (FR36)
- Full signature display (FR37)
- Verification status (FR38)
- Badge display (FR39)

**Non-Functional Requirements Addressed:**
- Security: All cryptographic data shown transparently
- User Experience: Progressive disclosure (basic info visible, detailed drill-down available)
- Accessibility: Monospace fonts, high contrast, keyboard navigation

---

## Epic Sequencing & Dependencies

```
Epic 1: Foundation (Key Mgmt & Auth)
    ↓ (Users must authenticate first)
    ↓
Epic 2: Presence (Lobby & Real-Time Updates)
    ↓ (Users must see who's available)
    ↓
Epic 3: Core Messaging (Send, Receive & Verify)
    ↓ (Users must be able to message)
    ↓
Epic 4: Transparency (Drill-Down Details)
```

**Parallel Work Within Epics:**
- Within each epic, multiple stories can be developed in parallel by different team members
- Stories within an epic may have internal dependencies but are independent of future epics

**Recommended Implementation Order:**
1. Epic 1 (Foundation) - First
2. Epic 2 (Presence) - Parallel with Epic 1 UI
3. Epic 3 (Core Messaging) - After Epics 1 & 2
4. Epic 4 (Transparency) - After Epic 3

---

## Epic 1: Foundation - Key Management & Authentication

**Epic Goal:** Establish user identity ownership through private key generation/import and prove identity to the server via cryptographic authentication.

**Why This Epic First:** Users cannot send or receive messages without first establishing their identity. This is the foundational layer that all other functionality depends on.

**User Archetype Success:** 
- **Alex** (First-Time User): Feels empowered generating a key and understanding "this is me"
- **Sam** (Technical Validator): Can import an existing key and validate the crypto works

**Key Requirements Addressed:** FR1-5 (Key Management), FR6-11 (Authentication & Connection)

---

### Story 1.1: Generate New 256-Bit Private Key

As a **new user**,
I want to **generate a new 256-bit random private key within the application**,
So that **I can establish my cryptographic identity without managing external keys**.

**Acceptance Criteria:**

**Given** a new user launches Profile for the first time
**When** the user selects "Generate New Key" from the welcome screen
**Then** the system generates a cryptographically secure 256-bit random private key
**And** the system derives the corresponding public key from the private key
**And** the system displays the public key clearly to the user (in monospace, blue color)
**And** the system securely stores the private key in memory (zeroize-protected)
**And** the user is informed "Your key has been generated. This is your identity. Keep your private key secure."

**Given** a user has successfully generated a key
**When** they proceed to the next step
**Then** their public key is remembered for the session (no re-generation needed)

**Technical Details:**
- Use ed25519-dalek for key generation (provides deterministic properties)
- Store private key as `zeroize::Zeroizing<Vec<u8>>` (auto-zeroes on drop)
- Display public key as hex string in monospace font
- Support copying public key to clipboard

**Related FRs:** FR1, FR3, FR4, FR5

---

### Story 1.2: Import Existing 256-Bit Private Key

As a **technically experienced user**,
I want to **import an existing 256-bit private key by pasting it into the application**,
So that **I can use Profile with a key I already own and trust**.

**Acceptance Criteria:**

**Given** a user chooses to import an existing key
**When** they see the "Import Private Key" screen with a paste field labeled "Enter your 256-bit private key"
**Then** the user can paste their key (as hex string) into the field

**Given** the user has entered a key
**When** the system validates the format
**Then** the system checks if it's a valid 256-bit hex string (64 characters, 0-9a-f)
**And** if valid, the system displays the derived public key
**And** if invalid, the system shows an error with two levels:
   - Simple: "That doesn't look like a valid private key. Expected a 256-bit hex string."
   - Technical (expandable): "Expected 64 hex characters (256 bits), received {actual_length}. Example: 3a8f2e1c..."

**Given** the user has successfully imported a key
**When** they confirm and proceed
**Then** the private key is stored securely in memory (zeroize-protected)
**And** they can proceed to authentication

**Given** the user enters an invalid key
**When** they see the error message
**Then** they can edit the field or start over without data loss

**Technical Details:**
- Validate format: must be exactly 64 hexadecimal characters
- Derive public key using ed25519-dalek
- Show both public and private key length in error messages (for debugging)
- Support paste from clipboard (Ctrl+V)
- No retry limits (users can attempt as many times as needed)

**Related FRs:** FR2, FR3, FR4, FR5

---

### Story 1.3: Display User's Public Key Clearly

As a **user**,
I want to **see my public key clearly and understand that it's how people verify my messages**,
So that **I can feel confident in my identity ownership and potentially share it with others**.

**Acceptance Criteria:**

**Given** a user has generated or imported a key
**When** they view the public key display
**Then** the public key is shown in full (not truncated)
**And** the key is displayed in monospace font (signals "technical, machine-readable")
**And** the key is displayed in blue color (#0066CC, indicating identity/ownership)
**And** a copy button is available next to the key
**And** the copy button works with click or keyboard (Ctrl+C when focused on key)

**Given** a user clicks the copy button
**When** they copy the public key
**Then** the key is placed in the system clipboard
**And** the button shows brief feedback ("Copied!")

**Given** throughout the application (onboarding, lobby context, message headers, drill-down)
**When** a public key is displayed
**Then** it's always shown in monospace, blue, and never truncated

**Technical Details:**
- Use `KeyDisplayComponent` (reusable across app)
- Monospace font: "Consolas", "Monaco", or platform default monospace
- Color: #0066CC (primary blue)
- Copy: use platform clipboard API
- Feedback: brief visual indication (highlight, tooltip, or text change)

**Related FRs:** FR1, FR2, FR3

---

### Story 1.4: Securely Store Private Key in Memory (Zeroize-Protected)

As a **security-conscious user**,
I want to **know that my private key is protected in memory and not persisted to disk**,
So that **my identity cannot be compromised through file access or data leaks**.

**Acceptance Criteria:**

**Given** a user's private key is loaded into memory
**When** the application is running
**Then** the private key is stored using `zeroize::Zeroizing<Vec<u8>>` (auto-zeroed on drop)
**And** the private key is never written to disk
**And** the private key is never logged or printed to console
**And** the private key is never transmitted to the server

**Given** the user closes the application
**When** the application terminates
**Then** the private key memory is automatically overwritten with zeros
**And** no traces of the key remain in memory after shutdown

**Given** the user disconnects from the server
**When** they disconnect or session ends
**Then** the private key remains in memory (for potential reconnection in same session)
**But** is cleared when application fully closes

**Technical Implementation:**
- Use `zeroize` crate dependency (Cargo.toml)
- Define private key as `zeroize::Zeroizing<Vec<u8>>` in session struct
- No println!, dbg!, or log! calls with private key
- No serialization of private key to JSON or network
- Key derivation happens only when needed (signing operations)

**Related FRs:** FR5, FR41

---

### Story 1.5: Authenticate to Server with Signature Proof

As a **user**,
I want to **authenticate to the server by proving I own my private key through a cryptographic signature**,
So that **the server can verify my identity without ever seeing my private key**.

**Acceptance Criteria:**

**Given** a user has successfully generated or imported a key
**When** they are ready to connect to the server
**Then** the client creates an authentication message: `{type: "auth", publicKey: "...", signature: "..."}`
**And** the signature is created by signing the word "auth" with their private key
**And** the client sends this authentication message via WebSocket handshake

**Given** the server receives the authentication message
**When** the server validates the signature
**Then** the server uses the provided public key to verify the signature
**And** if the signature is valid, the server adds the user to the active lobby
**And** if the signature is invalid, the server rejects the connection with error "auth_failed"

**Given** authentication succeeds
**When** the user is added to the lobby
**Then** the user's WebSocket connection is marked as authenticated
**And** other users are notified that a new user has joined
**And** the client receives the current list of online users

**Given** authentication fails
**When** the server rejects the connection
**Then** the client receives the error: `{type: "error", reason: "auth_failed", details: "Signature did not verify"}`
**And** the WebSocket connection is closed
**And** the user is shown the error and can retry

**Technical Implementation:**
- Client: Use shared crypto library `sign_message("auth", private_key)` → sends signature as hex
- Server: Use same shared library `verify_signature("auth", signature, public_key)`
- Signature format: hex-encoded binary (for transport via JSON)
- Connection only authenticated after server validates signature

**Related FRs:** FR6, FR7, FR9

---

### Story 1.6: Handle Authentication Failure & Disconnection

As a **user**,
I want to **understand why authentication failed and be able to retry**,
So that **I can recover from authentication errors and establish a valid connection**.

**Acceptance Criteria:**

**Given** a user's authentication signature is invalid
**When** the server rejects the connection
**Then** the client displays: "Authentication failed. Your signature could not be verified. Try again or check your key."
**And** the user can dismiss the error and attempt to reconnect
**And** the WebSocket connection is closed cleanly

**Given** a user is connected and authenticated
**When** the WebSocket connection drops (network issue, server shutdown, timeout)
**Then** the client detects the disconnection
**And** displays: "Connection lost. Reconnecting..." (optional for MVP, required for Phase 2)
**And** drafts are preserved in the composer (user doesn't lose their message)

**Given** a user's connection is closed by the server
**When** the server terminates the connection (invalid signature, server maintenance, etc.)
**Then** the client receives the close message
**And** the user is notified: "Connection closed. [Reason if available]"
**And** the user's message draft is preserved

**Given** the user closes the application
**When** the application terminates
**Then** the connection is closed gracefully
**And** the server is notified the user has left the lobby
**And** other connected users are notified the user is no longer online

**Technical Implementation:**
- Server: Send clear error codes (auth_failed, signature_invalid, connection_timeout)
- Client: Handle WebSocket close frames and display user-friendly messages
- Draft preservation: Keep composer text in memory (don't clear on disconnect)
- Graceful shutdown: Close WebSocket before exiting
- Error codes mapping: Defined in shared error module

**Related FRs:** FR10, FR11

---

## Epic 2: Presence - Online Lobby & Real-Time Updates

**Epic Goal:** Maintain a real-time list of online users and broadcast presence updates so users always know who's available to message.

**Why This Epic Second:** Once users have established identity, they need to know who they can message. The lobby is the gateway to the core messaging experience.

**User Archetype Success:**
- **Alex** (First-Time User): Sees a list of online users and feels connected
- **Sam** (Technical Validator): Verifies presence updates happen reliably and deterministically

**Key Requirements Addressed:** FR26-33 (User Presence & Lobby)

---

### Story 2.1: Server Maintains Active User Lobby

As a **server application**,
I want to **maintain an in-memory list of all currently authenticated users with their public keys**,
So that **I can inform clients who is available to message and route messages to online recipients**.

**Acceptance Criteria:**

**Given** a user successfully authenticates
**When** the server validates their authentication signature
**Then** the server adds their entry to the active lobby: `{publicKey: "...", activeConnection}`
**And** the lobby entry remains active as long as the WebSocket connection is open
**And** each user appears exactly once in the lobby (no duplicates)

**Given** a user already exists in the lobby
**When** they authenticate again (reconnection from same key)
**Then** the server updates their connection reference
**And** treats it as a single user (replaces previous entry, broadcasts leave/join delta)

**Given** a user's WebSocket connection closes
**When** the connection is terminated (intentional or network failure)
**Then** the server removes their entry from the lobby
**And** the lobby becomes the source of truth for who is online

**Given** the server needs to route a message
**When** a recipient is specified
**Then** the server queries the lobby: is recipient online?
**And** routes accordingly (deliver if online, offline notification if not)

**Technical Implementation:**
- Data structure: `HashMap<PublicKey, ActiveConnection>`
- Per-connection handler: manages lobby add/remove
- Atomic operations: ensure no race conditions with concurrent connections
- Broadcast mechanism: efficient delta updates (don't retransmit entire lobby each time)

**Related FRs:** FR26, FR8, FR9

---

### Story 2.2: Query & Display Current Online User List

As a **user**,
I want to **see the list of all currently online users with their public keys**,
So that **I can choose who to send a message to**.

**Acceptance Criteria:**

**Given** a user is authenticated and connected
**When** the client loads or reconnects
**Then** the server immediately sends the current lobby state: `{type: "lobby", users: [{publicKey: "..."}, ...]}`
**And** the client receives this lobby snapshot
**And** displays a list of online users in the Lobby component

**Given** the lobby is displayed
**When** the user views the list
**Then** each user is shown with:
   - Public key in monospace, blue, full (not truncated)
   - Online indicator (● green dot)
   - Selection highlight (if selected, blue background)
**And** users can be scrolled if more than fit on screen
**And** the list shows "No users online" if lobby is empty

**Given** a user is viewing the lobby
**When** they click on a user
**Then** that user is selected (highlighted in blue)
**And** the chat area activates for messaging that user
**And** the composer field receives focus

**Given** multiple users are available
**When** the user uses keyboard navigation
**Then** arrow keys move selection up/down
**And** Enter key confirms selection

**Technical Implementation:**
- Component: `LobbyComponent` (custom Slint component)
- Data: Display `{publicKey: String, isOnline: bool}`
- Selection state: tracked separately from display
- Keyboard nav: standard Tab/Arrow keys, Enter to select
- Click handler: select user and activate chat for that user

**Related FRs:** FR26, FR27, FR28, FR29, FR30

---

### Story 2.3: Broadcast User Join Notifications

As a **server**,
I want to **notify all connected users when a new user joins the lobby**,
So that **everyone sees immediately when someone becomes available to message**.

**Acceptance Criteria:**

**Given** a user successfully authenticates and is added to the lobby
**When** their entry is added
**Then** the server broadcasts to all other connected users: `{type: "lobby_update", joined: [{publicKey: "..."}, ...]}`
**And** the message includes only the newly joined users (delta, not full list)
**And** is delivered within 100ms of user join

**Given** a client receives a lobby_update with joined users
**When** the update arrives
**Then** the client adds these users to its lobby display
**And** the Lobby component re-renders to show the new users
**And** a brief notification appears: "User [key] joined" (optional visual feedback)

**Given** multiple users join in quick succession
**When** the server processes multiple connections
**Then** it either broadcasts each join separately (immediate update)
**Or** batches rapid joins into a single broadcast
**And** consistency is guaranteed (final lobby state matches server truth)

**Given** the lobby is displayed during a join event
**When** the user is viewing the list
**Then** the list updates in real-time without requiring refresh

**Technical Implementation:**
- Broadcasting: use tokio broadcast channel (efficient multi-recipient sending)
- Delta format: send only changed users (not full list)
- Delivery latency: target <100ms from join to broadcast to clients
- Batching: optional optimization if many rapid joins occur

**Related FRs:** FR31, FR33

---

### Story 2.4: Broadcast User Leave Notifications

As a **server**,
I want to **notify all connected users when a user disconnects from the lobby**,
So that **everyone knows immediately when someone is no longer available to message**.

**Acceptance Criteria:**

**Given** a user's WebSocket connection closes (intentional or network failure)
**When** the server detects the disconnection
**Then** the server removes their entry from the lobby
**And** broadcasts to all remaining users: `{type: "lobby_update", left: [{publicKey: "..."}, ...]}`
**And** the message includes only the departed users (delta, not full list)
**And** is delivered within 100ms of user disconnection

**Given** a client receives a lobby_update with left users
**When** the update arrives
**Then** the client removes these users from its lobby display
**And** the Lobby component re-renders to show available users
**And** a brief notification appears: "User [key] left" (optional visual feedback)

**Given** the departing user selected another user to message
**When** they disconnect
**Then** the recipient is notified they left (via lobby update)
**And** any pending messages to that user are handled appropriately (offline notification)

**Given** a user is currently messaging someone
**When** that person disconnects
**Then** the lobby reflects the disconnection immediately
**And** if the user tries to send another message, they receive "recipient offline" notification

**Technical Implementation:**
- Connection drop detection: WebSocket close handler
- Lobby update: broadcast to remaining connections
- Notification content: include public keys of departed users
- Delivery: within 100ms of detected disconnection
- State consistency: server is single source of truth

**Related FRs:** FR32, FR33

---

### Story 2.5: Real-Time Lobby Synchronization

As a **user**,
I want to **always see an accurate, up-to-date list of who's online**,
So that **I can confidently message anyone in the lobby without wondering if they've disconnected**.

**Acceptance Criteria:**

**Given** I am viewing the lobby
**When** users join or leave
**Then** the lobby updates in real-time (within 100ms)
**And** no manual refresh is required

**Given** I have the application open for an extended time
**When** many users join and leave
**Then** the lobby remains consistent with the server (no divergence)
**And** if I select someone and they leave, I'm notified
**And** my selection is cleared or marked unavailable

**Given** I am about to send a message
**When** I select a recipient from the lobby
**Then** that recipient is confirmed to be online
**And** if they disconnect between selection and send, I'm notified "recipient went offline"

**Given** there are latency issues or temporary network blips
**When** brief disconnections occur
**Then** the system remains resilient
**And** lobby state is eventually consistent with server
**And** I don't see ghost users or missing users

**Technical Implementation:**
- Push-based updates (not polling): server broadcasts changes
- Delta updates: send only changed users, not full list each time
- Consistency: server is single source of truth
- Client-side rendering: Slint reactively updates when lobby model changes
- Timeout handling: detect stale connections and remove (Phase 2)

**Related FRs:** FR26-33

---

## Epic 3: Core Messaging - Send, Receive & Verification

**Epic Goal:** Enable users to send deterministically signed messages and receive verified messages with instant feedback.

**Why This Epic Core:** This is the primary user value—sending and receiving cryptographically verified messages. Everything else supports this.

**User Archetype Success:**
- **Alex** (First-Time User): Sends first message, sees ✓ badge, feels empowered
- **Sam** (Technical Validator): Sends identical messages twice, confirms signatures match

**Key Requirements Addressed:** FR12-20 (Message Operations), FR21-25 (Cryptographic Verification), FR45-47 (Offline Behavior)

---

### Story 3.1: Compose & Send Message with Deterministic Signing

As a **user**,
I want to **type a message and press Enter to send it with automatic cryptographic signing**,
So that **my message is proven to come from my private key without any extra steps**.

**Acceptance Criteria:**

**Given** I have selected a recipient from the lobby
**When** the chat area is active
**Then** the message composer field receives focus automatically
**And** a placeholder text shows "Type message..."

**Given** I am typing in the composer
**When** I enter any text (including unicode, special characters, etc.)
**Then** the text is captured exactly as typed
**And** the Send button becomes enabled (was disabled when empty)

**Given** I have typed my message (e.g., "Hello, is anyone here?")
**When** I press Enter (or click Send button)
**Then** the system captures the message text
**And** immediately signs it with my private key using deterministic signing
**And** the signing completes in <100ms (feels instant)
**And** the signed message is sent to the server via WebSocket

**Given** the message is successfully sent
**When** it arrives on the server
**Then** my message appears immediately in my chat view
**And** the message displays with: [timestamp] [my_public_key] [message text] [✓ green badge]
**And** the verified badge appears automatically (I don't need to wait for verification)
**And** the composer field clears and receives focus for next message

**Given** I send the exact same message twice
**When** I compare the signatures in the drill-down view
**Then** the signatures are identical (deterministic signing proven)

**Given** I send a message with various content (unicode "你好", special chars "!@#$", long text, etc.)
**When** each message is sent and signed
**Then** all messages are signed successfully
**And** all signatures verify correctly
**And** the system handles all edge cases without errors

**Technical Implementation:**
- Composer: TextInput with Enter key handler
- Signing: Call `shared::crypto::sign_message(message, private_key)` (from shared library)
- Message object: `{type: "message", message: "...", senderPublicKey: "...", signature: "...", timestamp: "...ISO8601..."}`
- Canonical JSON: Ensure deterministic encoding before signing
- Timestamp: Generated at send time on client
- UI feedback: Message appears instantly with badge (no "checking" spinner)

**Related FRs:** FR12, FR13, FR14, FR24, FR25

---

### Story 3.2: Send Message to Server with Validation

As a **server**,
I want to **receive signed messages from clients, validate the signature, and route to recipients**,
So that **only valid, authenticated messages are delivered**.

**Acceptance Criteria:**

**Given** the server receives a message from a client
**When** it arrives via WebSocket
**Then** the server performs validation in this strict sequence:
   1. Check sender is authenticated (has active connection)
   2. Check message format is valid JSON
   3. Validate signature against sender's public key using shared crypto library
   4. Check recipient exists in lobby (if online requirement)
   5. Route accordingly (deliver if online, send offline notification if not)

**Given** the message passes all validations
**When** all checks are successful
**Then** the message is forwarded to the recipient (if online)
**And** the sender receives confirmation (implicit: message appears in their view)

**Given** any validation fails
**When** a check does not pass
**Then** the server stops processing immediately (fail-fast)
**And** returns an error to sender: `{type: "error", reason: "signature_invalid | offline | malformed_json", details: "..."}`
**And** the message is not delivered

**Given** a message with an invalid signature arrives
**When** validation fails
**Then** the error is: `{reason: "signature_invalid", details: "Signature did not verify against public key"}`
**And** the recipient never sees the invalid message

**Given** the recipient is offline
**When** the server checks the lobby
**Then** the error is: `{reason: "offline", details: "User [recipient_key] is not currently online"}`
**And** the sender is notified immediately

**Technical Implementation:**
- Validation sequence: exact order (no shortcuts, no skipping)
- Signature validation: use `shared::crypto::verify_signature(message, signature, public_key)`
- Fail-fast: stop at first error, don't continue processing
- Error codes: predefined set (signature_invalid, offline, malformed_json, auth_failed)
- Recipient check: query lobby HashMap

**Related FRs:** FR14, FR15

---

### Story 3.3: Push Message to Online Recipient in Real-Time

As a **server**,
I want to **immediately push received messages to online recipients via WebSocket**,
So that **messages arrive instantly with no polling or delays**.

**Acceptance Criteria:**

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

**Technical Implementation:**
- Push mechanism: use tokio broadcast or per-connection send
- Message ordering: use timestamps as tiebreaker
- Delivery latency: target <500ms end-to-end
- Message forwarding: forward original message as-is (don't modify)
- Client handling: WebSocket message handler receives push

**Related FRs:** FR17, FR18, FR20

---

### Story 3.4: Receive & Verify Message Signature Client-Side

As a **user receiving a message**,
I want to **automatically verify that the message signature is valid**,
So that **I can trust the verified badge means the message truly came from that public key**.

**Acceptance Criteria:**

**Given** the client receives a message via WebSocket
**When** the message includes sender's public key and signature
**Then** the client immediately verifies the signature using the shared crypto library
**And** calls `shared::crypto::verify_signature(message, signature, public_key)`
**And** verification completes in <100ms

**Given** the signature verification succeeds
**When** the verification result is true
**Then** the message is displayed in the chat
**And** a green ✓ verification badge appears next to the message
**And** the badge indicates "verified" / "cryptographically proven from this key"

**Given** the signature verification fails
**When** the verification result is false
**Then** the message is NOT displayed in the chat
**And** a warning is logged: "Invalid signature received from [public_key]"
**And** an error notification is shown to the user: "Received message with invalid signature. Message rejected."

**Given** a message passes all verification checks
**When** displayed in the chat
**Then** the message format shows: [timestamp] [sender_key] [message text] [✓ green badge]
**And** the timestamp, key, text, and badge are all visible and untruncated

**Technical Implementation:**
- Verification: use `shared::crypto::verify_signature()` from shared library
- Verification happens immediately on receipt
- Invalid messages: not displayed to user
- Valid messages: stored in message history with badge
- Badge: ✓ symbol in green (#22c55e)

**Related FRs:** FR21, FR22, FR23, FR25

---

### Story 3.5: Display Messages Chronologically with Timestamps

As a **user**,
I want to **see all messages in chronological order with timestamps**,
So that **I can follow the conversation flow and context is clear**.

**Acceptance Criteria:**

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

**Technical Implementation:**
- Message storage: in-memory list, ordered by timestamp
- Display: Slint ScrollView with auto-scroll to bottom
- Timestamp format: ISO 8601 on transport, HH:MM:SS in display
- Scroll behavior: auto-scroll to newest unless user is scrolling
- Memory management: clear on app close (ephemeral storage)

**Related FRs:** FR19, FR20

---

### Story 3.6: Handle Offline Recipient Notification

As a **user**,
I want to **be notified immediately if I try to send a message to someone who's offline**,
So that **I know the message wasn't delivered and can try again later**.

**Acceptance Criteria:**

**Given** I select a recipient from the lobby
**When** I compose and send a message
**But** that recipient has disconnected between the time I selected them and the time I sent
**Then** the server attempts to deliver the message
**And** finds the recipient is no longer online
**And** sends an offline notification back to me: `{type: "notification", event: "recipient_offline", recipient: "..."}`

**Given** I receive the offline notification
**When** it arrives at my client
**Then** a notification appears: "User [recipient_key] is offline. Message not delivered."
**And** the message I attempted to send appears in my chat with a ⚠ yellow warning badge
**And** a [Retry] button is shown
**And** the notification persists until I dismiss it or the user comes back online

**Given** the recipient comes back online
**When** they rejoin the lobby
**Then** the notification updates to: "User [recipient_key] is back online. [Try again?]"
**And** I can click to resend the message

**Given** I click [Retry]
**When** I attempt to resend
**Then** the message is sent again (with a new signature and timestamp)
**And** if the recipient is still offline, I get another notification
**And** if they're online, the message is delivered

**Given** I dismiss the offline notification
**When** I close the notification
**Then** the ⚠ badge remains on the message (history)
**But** the notification is no longer shown
**And** I can click the message to view details

**Technical Implementation:**
- Offline notification: broadcast from server when delivery fails
- Notification component: dismissible, with retry option
- Message badge: ⚠ yellow (#f59e0b) for undelivered/offline messages
- Retry mechanism: resend message (generates new signature with new timestamp)
- Notification persistence: until dismissed or recipient comes online

**Related FRs:** FR16, FR45, FR46, FR47

---

### Story 3.7: Preserve Composer Draft on Disconnection

As a **user**,
I want to **keep my message draft if the network connection drops**,
So that **I don't lose my work if there's a temporary network issue**.

**Acceptance Criteria:**

**Given** I am composing a message in the composer field
**When** I've typed text but haven't sent it yet
**Then** the text is stored in the composer field (in-memory state)

**Given** the network connection drops (WebSocket disconnects)
**When** the disconnection is detected
**Then** the message draft remains in the composer field
**And** the draft is NOT cleared or discarded
**And** I see a notification: "Connection lost. Reconnecting..."

**Given** the connection is restored (manual reconnect or auto-reconnect in Phase 2)
**When** I regain connection to the server
**Then** my draft is still in the composer
**And** I can review it, edit it, or send it
**And** nothing was lost

**Given** I intentionally close the application
**When** the app terminates
**Then** the draft is cleared (ephemeral, only in current session)

**Technical Implementation:**
- Composer state: stored in application state (not persistent)
- Disconnection detection: WebSocket close event
- Draft preservation: don't clear on disconnect
- Reconnection: preserve state across reconnect
- App close: clear all ephemeral data

**Related FRs:** FR40, FR41

---

### Story 3.8: Handle Message Composition Edge Cases

As a **user**,
I want to **send messages with any content—unicode, special characters, long text, whitespace—without errors**,
So that **the system is robust and handles real-world message variations**.

**Acceptance Criteria:**

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

**Technical Implementation:**
- Encoding: UTF-8 for all message content
- Canonical JSON: serialize message canonically before signing (ensures determinism)
- Signature: uses ed25519-dalek (handles all UTF-8 correctly)
- Edge case tests: unicode, special chars, long messages, whitespace variations
- Validation: UTF-8 check on receipt

**Related FRs:** FR24, FR25

---

## Epic 4: Transparency - Drill-Down Details & Signature Inspection

**Epic Goal:** Enable users to inspect the cryptographic proof of messages, revealing sender's public key, signature, and verification status layer-by-layer.

**Why This Epic Last:** Builds on core messaging. Provides optional depth for curious users and technical validation for Sam.

**User Archetype Success:**
- **Alex** (First-Time User): Clicks a message, discovers the signature, feels empowered understanding the proof
- **Sam** (Technical Validator): Inspects signatures, validates determinism, confirms cryptographic correctness

**Key Requirements Addressed:** FR34-39 (Message Details & Verification Display)

---

### Story 4.1: Click Message to Open Drill-Down Modal

As a **user**,
I want to **click on any message to view its full cryptographic details**,
So that **I can understand the proof behind the verification badge**.

**Acceptance Criteria:**

**Given** messages are displayed in the chat area
**When** I see a message with a ✓ or ⚠ badge
**Then** the message is clickable (cursor changes to pointer)
**And** a tooltip appears on hover: "Click to view details"

**Given** I click on a message
**When** the message is activated
**Then** a modal opens showing the full message details
**And** the modal is centered on the screen
**And** a close button (X) is visible in the top-right
**And** pressing Escape also closes the modal

**Given** the modal is open
**When** I'm viewing the details
**Then** the chat area behind it is slightly dimmed (visual indication that modal is active)
**And** I cannot interact with the chat or composer while the modal is open
**And** focus is trapped in the modal (Tab stays within the modal)

**Given** I close the modal
**When** I press Escape or click the X button
**Then** the modal closes smoothly
**And** focus returns to the message I was viewing
**And** the chat area returns to normal

**Technical Implementation:**
- Message component: add click handler
- Modal: custom Slint component `DrillDownModalComponent`
- Click propagation: prevent bubbling to other elements
- Focus management: modal gets focus on open, returns on close
- Styling: dim background, modal overlay centered

**Related FRs:** FR34

---

### Story 4.2: Display Message Details in Drill-Down Modal

As a **user**,
I want to **see the message content, sender's public key, and cryptographic signature in the drill-down**,
So that **I can verify the message comes from the claimed sender**.

**Acceptance Criteria:**

**Given** the drill-down modal is open
**When** I view the details
**Then** the modal displays in this order:

**Layer 1 (Always Visible):**
- Sender's Public Key: Full key in monospace, blue, untruncated
- Message Content: Full message text, exactly as sent
- Timestamp: HH:MM:SS showing when message was sent
- Copy buttons: next to key and message (optional, Ctrl+C works)

**Layer 2 (Expandable/Detailed):**
- Cryptographic Signature: Full hex-encoded signature in monospace
- Signature visible as: "a4f3e2c1b8d5e9f2..." (complete, not truncated)
- Copy button: for copying the signature

**Layer 3 (Verification Status):**
- Status Badge: ✓ Verified (green) or ⚠ Not Verified (red)
- Explanation: "This message came from [public_key]"

**Given** I'm viewing the signature details
**When** I want to copy the signature
**Then** I can click the Copy button
**And** the signature is placed in my clipboard
**And** brief feedback shows "Copied!"

**Given** I select text in the drill-down
**When** I press Ctrl+C
**Then** the selected text is copied (key, message, or signature)

**Technical Implementation:**
- Modal layout: vertical stack of sections
- Public key: `KeyDisplayComponent` (reusable)
- Signature: monospace, 11px or 12px font, neutral gray color
- Copy buttons: use platform clipboard API
- Verification badge: `VerificationBadgeComponent`

**Related FRs:** FR34, FR35, FR36, FR37, FR38, FR39

---

### Story 4.3: Verify Message Signature in Modal

As a **user**,
I want to **see clear verification status in the drill-down—either verified with ✓ or failed with ⚠**,
So that **I can confirm the message is authentic or identify that something went wrong**.

**Acceptance Criteria:**

**Given** the drill-down modal is open
**When** I view the verification status
**Then** I see either:
   - **✓ Verified** (green badge): "This message was cryptographically verified. It came from the owner of [public_key]."
   - **⚠ Not Verified** (red badge): "This message failed signature verification. It may have been tampered with."

**Given** a message shows as verified
**When** I see the ✓ badge
**Then** I can trust:
   - The message came from the owner of the public key shown
   - The message content has not been modified since signing
   - The signature is mathematically valid

**Given** a message shows as not verified
**When** I see the ⚠ badge
**Then** I understand:
   - The signature does not match the message and public key
   - The message may have been tampered with or corrupted
   - I should not trust the message as authentic from that sender

**Given** I drill down on any message in the chat
**When** the verification status is displayed
**Then** the status shown in the modal matches the badge shown in the chat
**And** there's no discrepancy between modal and chat view

**Technical Implementation:**
- Verification: performed when message is received (not in modal, already determined)
- Badge styling: green (#22c55e) for verified, red (#ef4444) for failed
- Symbol: ✓ for verified, ⚠ for not verified
- Explanation text: clear, non-technical language

**Related FRs:** FR38, FR39

---

### Story 4.4: Support Technical Signature Testing & Validation

As a **technical user (Sam)**,
I want to **inspect full signatures and test deterministic signing by comparing identical messages**,
So that **I can validate the cryptographic foundation is correct**.

**Acceptance Criteria:**

**Given** I send an identical message twice
**When** I drill down on both messages
**Then** the signatures are visible in full (not truncated)
**And** I can compare them side-by-side:
   - Message 1 signature: "a4f3e2c1b8d5e9f2..."
   - Message 2 signature: "a4f3e2c1b8d5e9f2..." (identical)
**And** the fact that they match confirms deterministic signing is working

**Given** I send messages with edge case content (unicode, special chars, long text, whitespace)
**When** I drill down on each message
**Then** the signature is visible for inspection
**And** I can verify that the signature correctly matches the message content
**And** the verification status shows ✓ for all valid signatures

**Given** I want to understand the signature format
**When** I see the signature in the drill-down
**Then** it's displayed as hex (e.g., "a4f3e2c1...") in monospace font
**And** the format is consistent (always hex, always monospace)
**And** documentation (future) explains the signature is ed25519, 64 bytes

**Given** I copy a signature from the drill-down
**When** I paste it elsewhere (text editor, another application)
**Then** the full hex string is pasted exactly as shown
**And** nothing is truncated or modified

**Technical Implementation:**
- Signature format: full hex display (no truncation)
- Monospace font: ensures readability and signals "machine-readable"
- Copy support: full signature text
- Accessibility: all technical details remain accessible without additional steps

**Related FRs:** FR24, FR25, FR34-39

---

## Implementation Dependency Map

The epics have sequential dependencies:

```
Epic 1: Foundation (Key Setup & Auth)
    ↓ (Users must authenticate before using lobby)
Epic 2: Presence (Lobby & Real-Time Updates)
    ↓ (Users must see who's available before messaging)
Epic 3: Core Messaging (Send/Receive & Verification)
    ↓ (After core messaging works, add transparency)
Epic 4: Transparency (Drill-Down Details)
```

**Parallel Work Within Epics:**
- Within Epic 1: Key management and authentication can be developed in parallel by different team members
- Within Epic 2: Lobby display and presence broadcasting can be developed in parallel
- Within Epic 3: Message send and receive flows can be developed in parallel
- Within Epic 4: All drill-down features are interdependent

**Testing Strategy Across Epics:**
- Epic 1: Unit tests for key generation, import validation, secure storage
- Epic 2: Integration tests for lobby state management and broadcast consistency
- Epic 3: E2E tests for deterministic signing, signature verification, edge cases
- Epic 4: UI tests for drill-down modal, detail display, signature inspection

---

## Story Estimation & Sequencing

Story estimation (story points, time estimates, velocity tracking) will be completed in the final validation step. The 23 stories are currently sequenced logically by dependency and user value delivery.

---

**Status:** Step 3 Complete ✅

**Stories Created and Approved:** 23 total
- Epic 1: 6 stories (Foundation)
- Epic 2: 5 stories (Presence)
- Epic 3: 8 stories (Core Messaging)
- Epic 4: 4 stories (Transparency)

**Next Step:** Step 4 - Final Validation (verify document completeness, check all FRs covered, ensure all stories are ready for development)
