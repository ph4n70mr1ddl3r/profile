---
stepsCompleted: [1, 2, 3, 4, 7, 8, 9, 10, 11]
inputDocuments: []
documentCounts:
  briefs: 0
  research: 0
  brainstorming: 0
  projectDocs: 0
workflowType: 'prd'
lastStep: 11
project_name: 'profile'
user_name: 'Riddler'
date: '2025-12-19'
workflowStatus: 'complete'
completedAt: '2025-12-19'
---

# Product Requirements Document - profile

**Author:** Riddler
**Date:** 2025-12-19

## Executive Summary

**Profile** is a simple, privacy-forward instant messaging application where users authenticate with their own private keys and maintain complete control over their digital identity.

Users can either import an existing private key or generate one directly in the application. All messages are signed with deterministic cryptographic signatures, making message authenticity verifiable by anyone while keeping identity ownership decentralized.

### What Makes This Special

Users control their identity completely—no middleman, no server managing authentication. Every message is cryptographically signed, proving it came from the owner of a specific public key. This creates a messaging experience built on verifiable identity rather than server-based trust.

The deterministic signing approach is foundational architecture designed to support future zero-knowledge proof implementations, positioning the product as a building block for more sophisticated cryptographic protocols.

## Project Classification

**Technical Type:** web_app
**Domain:** general
**Complexity Level:** low-to-medium
**Project Context:** Greenfield - new product

This is a greenfield web application focused on simple, identity-verified messaging. The core value lies in user-controlled cryptographic identity and transparent message authentication.

## Success Criteria

### Technical Success

The core success metric is **deterministic signature validation**. Every message must be signed with a deterministic signature scheme where the same message always produces the identical signature. This is non-negotiable because these signatures will later serve as nullifiers in zero-knowledge proof implementations.

**Specific Technical Requirements:**

- Any message (without content restrictions) must be successfully signed
- Deterministic signature scheme enforced: identical signatures for identical messages, 100% consistency
- Keys are 256-bit random numbers with no special key derivation or handling
- All edge cases must be tested: the signature must verify correctly regardless of message content or format
- Verified signatures display a badge in the UI indicating successful verification

### User Success

Users understand and can interact with the core signing/verification flow:

- Users can log in with either an imported private key or a generated 256-bit key
- Users send messages in a standard chat interface
- Users can drill down on any message to view cryptographic details: message content, public key, signature, and verification badge
- Users grasp that their messages are cryptographically verifiable and they control their identity through key ownership

### Business Success (POC Context)

Success for this proof of concept is completing the technical validation:

- Deterministic signing mechanism is proven reliable and consistent
- Signature verification works correctly across all edge cases
- The foundation is solid enough to support future zero-knowledge proof integration
- The concept is validated as viable to move forward with development

## Product Scope

### MVP - Proof of Concept (Technical Validation Phase)

**Core Functionality:**
- Private key authentication (import existing 256-bit key OR generate new key in-app)
- Basic chat interface for sending and receiving messages
- Deterministic signing on all messages
- Message details drill-down showing: message content, public key, signature, and verified badge
- Edge case testing to validate signing consistency

**Out of Scope for POC:**
- User profiles or identity management
- Message search or history
- Group messaging
- User discovery or contact lists
- Analytics or metrics
- Advanced UI/UX polish

### Growth Features (Post-POC)

Once technical validation is complete:
- User profiles and identity management
- Group messaging capabilities
- Message history and search
- User discovery mechanisms
- Community features

### Vision (Future)

- Zero-knowledge proof integration using message signatures as nullifiers
- Advanced privacy and authentication mechanisms
- Protocol standardization for broader adoption

## User Journeys

### Journey 1: Alex's First Time - Understanding Key Ownership

Alex discovers Profile and is drawn to the idea of truly owning their digital identity. Landing on the login screen, Alex chooses "Generate New Key" and the system instantly creates a 256-bit cryptographic key. With their public key displayed, Alex understands: "This is how people verify my messages are really from me."

Alex composes their first message: "Hello, is anyone here?" and sends it. The message appears in the chat. Curious, Alex clicks to drill down and discovers the full technical proof: their message, their public key, the deterministic signature, and a green verified badge (✓).

The breakthrough moment arrives—Alex realizes they own their identity completely. No company manages their key. No server can impersonate them. Every message they send is cryptographically verifiable. They stay in the app, confident and curious about connecting with others.

**Requirements Revealed:**
- Frictionless in-app key generation (256-bit, simple UX)
- Clear presentation of public key ownership
- Basic authentication with private key
- Simple message composition and send
- Drill-down functionality showing message, public key, signature, and verification badge
- Visual verified badge for authenticated messages

### Journey 2: Sam's Edge Case - Testing With Import and Validation

Sam is a technically savvy user with an existing 256-bit private key from another application. They want to use Profile with their existing key to validate the signing mechanism works correctly.

Sam arrives at login and selects "Import Existing Private Key," carefully pasting their key. The system accepts it and derives their public key. Sam is authenticated and ready.

Sam sends a message: "Testing deterministic signatures." They note the signature generated. Sam then sends the exact same message again from the same client—the signature is identical. Testing further, Sam tries edge cases: unicode characters, very long messages, special symbols, even empty-ish content. Each message signs and verifies correctly. The deterministic signature mechanism is proven solid.

Sam drills down on each message and watches the verified badge light up (✓). Every signature verifies correctly, regardless of message content or format. Sam has validated the core technical requirement: deterministic signatures are working reliably.

**Requirements Revealed:**
- Private key import functionality (paste existing 256-bit key)
- Automatic public key derivation from imported key
- Deterministic signature generation (identical input = identical signature)
- Edge case handling (unicode, long messages, special characters, various content)
- 100% signature verification consistency
- Drill-down detail view with verification validation for all message types

### User Journey Requirements Summary

**From these journeys, Profile requires:**

**Authentication & Key Management:**
- In-app key generation (256-bit random)
- Private key import functionality
- Public key derivation and display
- Secure key storage/handling

**Core Messaging:**
- Message composition and sending
- Message receive and display in chat
- Basic chat interface for multiple users

**Cryptographic Operations:**
- Deterministic signing on all messages
- Signature verification with 100% consistency
- Edge case handling for all message types

**User Interface & Verification:**
- Drill-down detail view (message, public key, signature)
- Verified badge display for authenticated messages
- Clear indication of signature verification status
- Accessible drill-down for non-technical users (Alex) and technical validation (Sam)

## Architecture Overview

**Profile** is a distributed system with two main components:

### Server Component (Linux, Rust)

A WebSocket server that manages real-time message delivery and user presence:

- **Technology:** Rust
- **Runtime:** Linux
- **Protocol:** WebSocket
- **Core Responsibilities:**
  - Accept client connections and authenticate via cryptographic signature
  - Maintain lobby of online users and their public keys
  - Receive signed messages from senders
  - Validate message signatures using sender's public key
  - Push validated messages to online recipients in real-time
  - Inform senders when recipients are offline

### Client Component (Windows Desktop, Rust + Slint)

A native Windows desktop application for user messaging:

- **Technology:** Rust + Slint UI framework (slint.dev)
- **Platform:** Windows (cross-platform support possible post-MVP)
- **User Interface:** Standard desktop window (no system integration)
- **Core Responsibilities:**
  - Key management (import or generate 256-bit private key)
  - WebSocket connection to server with cryptographic authentication
  - Sign messages deterministically client-side before sending
  - Display messages and validate signatures on receipt
  - Show lobby of online users
  - Drill-down message details (content, public key, signature, verification)

## API & WebSocket Specification

### Client Authentication (WebSocket Handshake)

**Connection Flow:**

1. Client initiates WebSocket connection to server
2. Client sends authentication message:
   ```
   {
     "publicKey": "...",
     "signature": "..." // signature proving ownership of private key
   }
   ```
3. Server validates signature against public key
4. Server adds client to active user lobby
5. Connection established and ready for messaging

### Message Send Flow

**Sender → Server:**

1. Client signs message deterministically with private key
2. Client sends to server:
   ```
   {
     "message": "...",
     "senderPublicKey": "...",
     "signature": "..." // deterministic signature of message
   }
   ```
3. Server validates signature against senderPublicKey

**Server → Recipient:**

4. If recipient is online (in lobby):
   - Server pushes message to recipient's WebSocket connection
5. If recipient is offline:
   - Server sends offline notification to sender
   - Message is not stored; sender must resend later

### Message Receive & Validation

**Recipient Receives:**

1. Recipient's client receives pushed message with signature
2. Client validates signature against senderPublicKey
3. Client displays message with verification badge (✓)
4. Message details available via drill-down:
   - Message content
   - Sender's public key
   - Cryptographic signature
   - Verification status badge

## User Presence & Lobby

### Online User Lobby

**Server maintains:**
- Active list of connected users
- Each user entry: `{publicKey, activeConnection}`

**Client capabilities:**
- Query server for list of online users
- Display online user lobby
- Select any online user to message
- Receive notifications when users join/leave lobby

## Data & Persistence Model

### Client-Side Data

**Ephemeral (cleared on exit):**
- Message history in current session
- User's private key (in memory only during session)
- Lobby state

**No Local Storage:**
- No persistent message history on disk
- No key storage on disk
- No client-side database

### Server-Side Data

**Minimal for POC:**
- Active WebSocket connections mapped to public keys
- Current lobby state
- No message persistence
- No user database

## Technical Requirements

### Deterministic Signing Validation

- All messages must be signed with deterministic signature scheme
- Same message + same key = identical signature every time
- Server validates sender signatures
- Client validates sender signatures on receipt
- Signatures serve as foundation for future ZK proof integration

### Real-Time Communication

- WebSocket for instant message push
- Sender → Server: synchronous send
- Server → Recipient: asynchronous push
- Low-latency delivery for online users

### Offline Handling

- No message queue or persistence
- Sender receives immediate notification: "recipient offline"
- Sender responsible for retrying later
- No automatic reconnection or message recovery

## Implementation Considerations

### Rust Ecosystem

- **Server:** Rust async runtime (tokio recommended for WebSocket handling)
- **Client:** Rust with Slint UI framework for cross-platform native windows
- **Cryptography:** Rust crypto libraries for deterministic signing (e.g., ed25519, schnorr)

### Platform Specifics

- **Windows Desktop:** Standard window, no taskbar integration, no system tray
- **Cross-Platform Future:** Slint supports Windows/Linux/macOS—architecture enables future expansion
- **No Auto-Update:** Users manually download and run new versions

### Performance & Scalability (POC Focus)

For technical validation, focus on:
- Message signing/verification performance
- WebSocket connection stability
- Signature determinism under load
- Lobby updates with multiple concurrent users

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Platform MVP - Build the solid foundation for messaging and deterministic signature validation that can be extended with additional features and eventually support zero-knowledge proof integration.

**Resource Requirements:** Small focused team (2-3 developers with Rust and cryptography experience)

**Launch Timeline:** Proof of concept validation phase focused on technical requirements

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:**
- Alex's first-time journey: Key generation → send message → view verified signature
- Sam's technical validation: Key import → deterministic signature testing → edge case validation

**Must-Have Capabilities:**

**Key Management:**
- In-app key generation (256-bit random)
- Private key import functionality
- Public key derivation and display

**Messaging Core:**
- Send message with deterministic signature
- Receive message with signature validation
- Real-time WebSocket message delivery
- Message timestamps
- Offline notification (recipient offline, try again)

**User Interface:**
- Online user lobby displaying active users (public keys only)
- Message composition and send
- Message display with verified badge (✓)
- Drill-down detail view: message content, sender public key, signature, verification status
- Simple, clean Windows desktop UI

**WebSocket Server:**
- Accept client connections with cryptographic authentication
- Validate connection signatures
- Maintain active user lobby
- Validate message signatures server-side
- Push messages to online recipients in real-time
- Notify senders when recipients offline

**Testing & Validation:**
- End-to-end automated testing suite
- Support for multiple client instances on same machine with different private keys
- Edge case validation: unicode, long messages, special characters, message variations
- Deterministic signature consistency validation

**Out of Scope for MVP:**
- User profiles or display names
- Persistent message history across sessions
- Message search or filtering
- Group messaging
- User blocking or moderation
- Connection resilience or automatic reconnection
- Message queuing for offline users
- Advanced UI/UX enhancements

### Post-MVP Features

**Phase 2 (Growth - Enhanced Experience):**
- User profiles with custom display names
- Persistent local message history
- Message search and filtering capabilities
- Connection resilience and automatic reconnection
- Enhanced UI/UX with improved visual design
- User status indicators
- Typing indicators

**Phase 3 (Expansion - Advanced Capabilities):**
- Group messaging and group chats
- User discovery and contact management
- User blocking and reporting
- Message reactions and emoji support
- Read receipts and delivery confirmation
- Zero-knowledge proof integration using message signatures as nullifiers
- Advanced cryptographic features
- Multi-platform support (macOS, Linux)
- Protocol standardization and documentation

### Risk Mitigation Strategy

**Technical Risks:**

- **Deterministic Signing Consistency:** Highest priority risk
  - Mitigation: Comprehensive E2E automated testing with edge case coverage
  - Multiple client instances enable real-world validation scenarios
  - Both client and server independently validate signatures
  
- **WebSocket Real-Time Delivery:** Medium priority risk
  - Mitigation: Rust async runtime (tokio) handles concurrent connections reliably
  - Testing with multiple simultaneous clients validates timing
  
- **Signature Validation Accuracy:** Medium priority risk
  - Mitigation: Cryptographic libraries chosen for reliability (ed25519, schnorr)
  - Automated testing validates against known test vectors

**Market/Business Risks:**
- POC/validation phase has minimal market risk
- Success metric is technical foundation, not market fit
- Focus is proving the deterministic signature model works before broader adoption

**Resource Risks:**
- Lean team (2-3 developers) is appropriate for scoped MVP
- Clear feature boundaries prevent scope creep
- Distributed system architecture is straightforward for this feature set
- Contingency: If resources constrained, message timestamps could move to Phase 2

### Development Phases Summary

**Phase 1 MVP (Current Focus):** Prove the platform foundation works
- Core messaging infrastructure
- Deterministic signing validation
- Multi-client testing capability
- Automated E2E test suite

**Phase 2 Growth (Post-Validation):** Enhance user experience
- Profiles and naming
- Message persistence
- Search and filtering
- UI/UX improvements

**Phase 3 Expansion (Long-term):** Advanced capabilities and ZK integration
- Group messaging
- ZK proof integration
- Multi-platform support
- Protocol standardization

## Functional Requirements

### Key Management

- FR1: Users can generate a new 256-bit random private key within the application
- FR2: Users can import an existing 256-bit private key by pasting it
- FR3: Users can view their public key derived from their private key
- FR4: The system derives the correct public key from any imported private key
- FR5: The system securely stores the user's private key in memory during the session

### User Authentication & Connection

- FR6: Users can connect to the server via WebSocket with their public key and a signature proving key ownership
- FR7: The server validates the authentication signature against the user's public key
- FR8: Upon successful authentication, the server adds the user to the active online lobby
- FR9: The server maintains an active WebSocket connection with each authenticated user
- FR10: Users receive a notification when their authentication fails (invalid signature)
- FR11: Users are disconnected from the server when their WebSocket connection closes

### Message Operations

- FR12: Users can compose and send a message to any online user
- FR13: Users can select a recipient from the list of online users
- FR14: The system signs each message with a deterministic signature using the user's private key before sending
- FR15: The server receives sent messages and validates the sender's signature against their public key
- FR16: The server notifies the sender if the recipient is offline (cannot receive the message)
- FR17: The server pushes received messages to the recipient in real-time via WebSocket
- FR18: The recipient's client receives the pushed message with sender's public key and signature intact
- FR19: Received messages display in chronological order in the chat interface
- FR20: Messages include a timestamp showing when they were sent

### Cryptographic Verification

- FR21: The recipient's client validates the message signature against the sender's public key
- FR22: Valid signatures trigger a "verified" badge (✓) displayed next to the message
- FR23: Invalid signatures are rejected and the message is not displayed
- FR24: Deterministic signatures are generated consistently: identical message + same key = identical signature every time
- FR25: Signature verification works correctly for all message content types (unicode, special characters, long text, etc.)

### User Presence & Lobby

- FR26: The server maintains a list of all currently online users
- FR27: Each online user entry displays their public key
- FR28: Users can query the server for the current list of online users
- FR29: The client displays the online user lobby showing all connected users
- FR30: Users can select any online user from the lobby to start messaging
- FR31: When a user connects, other users are notified that they joined the lobby
- FR32: When a user disconnects, other users are notified that they left the lobby
- FR33: The online lobby updates in real-time as users join and leave

### Message Details & Verification Display

- FR34: Users can drill down on any message to view full cryptographic details
- FR35: The drill-down view displays the message content
- FR36: The drill-down view displays the sender's public key
- FR37: The drill-down view displays the cryptographic signature
- FR38: The drill-down view displays the verification status (verified ✓ or invalid)
- FR39: The verified badge is prominently displayed for verified messages

### Data Persistence

- FR40: All message history is ephemeral and cleared when the user closes the application
- FR41: The user's private key is stored only in memory and not persisted to disk
- FR42: The online lobby state is maintained only for the current session
- FR43: The server stores no persistent user database
- FR44: The server does not persist messages between sessions

### Offline Behavior

- FR45: When a user attempts to send a message to an offline recipient, the server sends an offline notification
- FR46: The offline notification informs the sender that the recipient is currently unavailable
- FR47: The sender can resend the message after the recipient comes back online

## Non-Functional Requirements

### Performance

- **Message Signing:** Message signing operations must complete within 100ms to feel instant to users
- **Signature Verification:** Signature verification on received messages must complete within 100ms of receipt
- **WebSocket Message Delivery:** Messages must be delivered from sender to recipient in real-time, with end-to-end latency under 500ms
- **Lobby Updates:** Changes to the online user lobby (users joining/leaving) must propagate to all connected clients within 100ms
- **Concurrent Users:** The server must support as many concurrent users as the underlying infrastructure allows, with no artificial limits imposed by the application
- **Deterministic Signature Consistency:** Signatures must be generated with 100% consistency—identical message + same key must produce identical signature every time, measurable across thousands of iterations

### Security

- **Private Key Protection:** Private keys must never leave the client application and must be stored only in memory during the session, never persisted to disk or transmitted to the server
- **Private Key in Memory:** Private keys must be securely held in application memory and cleared from memory when the application closes
- **Signature Validation Accuracy:** All message signatures must be validated with 100% accuracy; any signature that cannot be verified against the sender's public key must be rejected and not displayed
- **Invalid Signature Handling:** Messages with invalid or unverifiable signatures must not be displayed to the user; they must be rejected with a clear indication of why (invalid signature)
- **Connection Authentication:** WebSocket connections must be authenticated using cryptographic signatures proving ownership of the private key; unauthenticated connections must be rejected
- **Message Content Encoding:** Messages must be validated as text-based UTF-8 encoded content; binary content is not supported and must be rejected

### Scalability

- **User Growth Path:** The MVP architecture must support the addition of scalability features in Phase 2 without requiring fundamental redesign
- **Concurrent Connection Handling:** The server must handle connection/disconnection events smoothly with no performance degradation as users join and leave
- **Message Queue Management:** The system must handle message queuing and real-time delivery efficiently for all concurrent users
