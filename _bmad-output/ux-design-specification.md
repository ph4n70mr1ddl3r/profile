---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]
inputDocuments: ["/home/riddler/profile/_bmad-output/prd.md"]
workflowType: 'ux-design'
lastStep: 12
project_name: 'profile'
user_name: 'Riddler'
date: 'Fri Dec 19 2025'
---

# UX Design Specification profile

**Author:** Riddler
**Date:** Fri Dec 19 2025

---

## Executive Summary

### Project Vision

Profile is a privacy-forward instant messaging platform built on cryptographic identity verification. Users authenticate with their own private keys and maintain complete control over their digital identity. Every message is deterministically signed and cryptographically verifiable, creating a messaging experience built on verifiable proof rather than server-based trust.

The core value proposition: **You own your identity completely. Every message you send is cryptographically verifiable proof that it came from you.**

### Target Users

**Alex (First-Time User):** Non-technical users drawn to identity ownership
- Values simplicity and clear user flows
- Needs the technical complexity abstracted but feels the empowerment
- Primary goal: Send their first verified message and understand they own their identity

**Sam (Technical Validator):** Technically savvy cryptography enthusiast
- Values cryptographic correctness and edge-case validation
- Wants access to technical details and signature verification
- Primary goal: Validate deterministic signing consistency across message variations

**Future ZK Developers:** Post-MVP, developers building zero-knowledge proof protocols
- Relies on Profile's signature foundation
- Values API stability and cryptographic correctness

### Key Design Challenges

1. **Making Cryptography Accessible**: Private keys, signatures, and verification badges are powerful concepts but can intimidate non-technical users. The UX must present these naturally and make users feel empowered, not confused.

2. **Balancing Simplicity with Technical Depth**: Alex needs straightforward send/receive; Sam needs cryptographic details. The design must support both user archetypes without feeling compromised for either.

3. **Building Trust Through Verification**: The connection between "I own this private key" and "This proves the message is from me" must be immediate and intuitive. Visual design plays a critical role here.

4. **Embrace Intentional Minimalism**: With no persistent history, no profiles, and no discovery features, the interface must feel intentionally focused rather than incomplete or unfinished.

### Design Opportunities

1. **Identity as the Hero**: Rather than hiding cryptographic details, celebrate them. Make key ownership the visual and conceptual centerpiece. This is what makes Profile different.

2. **Progressive Disclosure of Verification**: Users learn through interaction. Alex sends a message, then drills down to discover the cryptographic proof layer by layer. This builds understanding and confidence.

3. **Visual Verification Language**: A strong visual system where the verified badge (✓) becomes a symbol of trust, authenticity, and identity ownership. Consistent, recognizable, meaningful.

4. **Minimal, Intentional Interface**: The constraints (no history, no profiles, ephemeral data) aren't limitations—they're features. The pure focus on messaging with verified identity creates a unique, focused experience.

---

## Core User Experience

### Defining the Experience

Profile's core experience centers on one fundamental interaction: **sending and receiving messages with cryptographic certainty**. This is what users will do most frequently, and it must be effortless.

The magic happens at the moment of send. When a user composes a message and hits send, the system automatically creates a deterministic cryptographic signature—seamlessly, invisibly, without requiring any user intervention. The user thinks about their message; the platform handles the proof.

This automatic signing is not a technical detail hidden in the background—it's the foundation of trust. Every message that arrives is already verified. Every message sent is already signed. The user experience should make this feel natural and inevitable, not magical or complicated.

### Platform Strategy

**Windows Desktop as Primary Platform**

Profile launches as a Windows desktop application (Rust + Slint). This platform choice enables:

- **Keyboard & Mouse Mastery**: We design for power users who appreciate keyboard shortcuts, right-click context menus, and efficient mouse interactions. Desktop users expect and appreciate efficiency.
- **Persistent Window**: The application maintains consistent window state throughout the session, enabling users to reference lobby status, ongoing conversations, and verification details without context switching.
- **Always-Connected Assumption**: For this POC phase, we assume users maintain active WebSocket connections. This simplifies the UX—no offline states, no sync complexity, no reconnection flows.
- **Native Desktop Conventions**: Leverage Windows UI patterns users already know—taskbar integration, window management, keyboard focus states.

Future Phase 2+ can expand to mobile and web, but the desktop foundation established here will create a strong, efficient experience for technical users and power users.

### Effortless Interactions

**1. Sending a Message (The Core Loop)**

From composition to delivery should feel like a single fluid action:
- Type message in composition field
- Press Enter (or click Send button)
- Message appears in chat with verification status
- Signature creation happens invisibly, automatically
- User sees the result, not the mechanism

This is the primary interaction. Every friction point here costs us. Every unnecessary click or confirmation delays the core value.

**2. Discovering Verification (The Aha Moment)**

The drill-down interaction is where technical understanding deepens. When users click on a message to view details, they discover:
- The message content they sent
- Their public key (visual confirmation of identity)
- The cryptographic signature
- The verification badge (✓)

This drill-down should feel like opening a locked door to see what's inside—but the door opens smoothly and the contents make intuitive sense.

**3. Selecting a Recipient (Lobby as Gateway)**

Choosing who to message should require minimal cognitive load:
- Lobby displays online users with their public keys
- Users select a recipient (click or keyboard navigation)
- Composition field becomes active, ready for message
- Context is clear: "I'm messaging this person with this key"

**4. Key Setup (The Welcome Experience)**

Initial key generation or import happens once, but it's critical:
- Generate new key: One button, instant, displays the public key
- Import existing key: Clear paste field, validation feedback, public key confirmation
- Both paths end with the user understanding: "This is my identity. This is how people verify my messages."

### Critical Success Moments

**Moment 1: First Message Send**
When Alex sends their first message and sees it appear in the chat with a verification badge, they should feel: *"My message is proven to be from me. I own this."*

**Moment 2: First Drill-Down Discovery**
When Alex clicks a message to view its cryptographic details, they should feel: *"I can see the proof. I understand how this works."*

**Moment 3: Technical Validation (Sam's Workflow)**
When Sam tests deterministic signatures and confirms identical messages produce identical signatures, they should feel: *"The foundation is solid. This actually works."*

### Experience Principles

These principles guide every UX decision for Profile:

1. **Signing is Automatic, Never Explicit**
   - Users should never need to click "sign message" or confirm signature creation
   - The signature happens at send, transparently, as part of the natural message flow
   - Complexity is hidden; simplicity is visible

2. **Identity is Always Visible**
   - Public keys are displayed prominently (in lobby, in messages, in drill-down)
   - Users should always understand whose message they're reading and from which key they're sending
   - Visual connection between "my key" → "this message" → "the signature" should be immediate

3. **Verification Through Interaction, Not Configuration**
   - Users don't set up verification; they discover it
   - Drill-down reveals verification details when users are curious
   - The verified badge (✓) appears naturally; no complex UI needed

4. **Desktop Efficiency First**
   - Keyboard shortcuts enable power users to work faster
   - Right-click context menus reduce menu hunting
   - Window state is persistent and predictable
   - Design for the user who values speed and directness

5. **One Thing, Done Right**
   - Focus ruthlessly on sending/receiving messages
   - Resist feature creep (profiles, history, search, groups in MVP)
   - Every UI element serves the core message flow
   - Minimalism is a feature, not a limitation

---

## Desired Emotional Response

### Primary Emotional Goals

Profile must evoke two interconnected emotional states:

**1. Trust Over Skepticism**

Users should feel *certain* that their identity is verified and their messages are authentic. Not "this probably works" but "this provably works." The cryptographic proof should feel tangible, real, and trustworthy—not like a theoretical concept or marketing claim.

When users send a message, they should feel confident that anyone receiving it can verify it came from them. When users receive a message, they should trust that the verification badge (✓) means something concrete: cryptographic proof of sender identity.

This trust isn't faith-based (like trusting a company won't abuse data). It's proof-based. The UX must make this distinction clear and visceral.

**2. Confidence Over Confusion**

Users should never feel confused about how their identity works or how verification happens. The technical complexity must be abstracted without disappearing.

For Alex (first-time user): Confidence that their key setup was successful, that their message was signed, that the verification badge means their message is proven.

For Sam (technical validator): Confidence that they can inspect the cryptographic details, understand the signature mechanism, and validate the deterministic signing behavior.

Confusion is the enemy. Every interaction should clarify, not obscure.

### Emotional Journey Mapping

**Phase 1: Discovery & Onboarding**
- *Feeling*: Curious, cautiously optimistic
- *Goal*: Users understand "this is about me owning my identity" without being overwhelmed
- *Design Role*: Key generation/import should feel simple and empowering, not technical or intimidating

**Phase 2: First Message Send**
- *Feeling*: Anticipatory → Accomplished
- *Goal*: Users send their first message and immediately see it's verified
- *Design Role*: The send action should be effortless; the verification badge should appear naturally, making the accomplishment tangible

**Phase 3: Drill-Down Discovery**
- *Feeling*: Curious → Satisfied → Confident
- *Goal*: Users explore the cryptographic details and understand how verification works
- *Design Role*: Drill-down reveals layers of proof (message → key → signature → badge) in a way that builds understanding, not overwhelm

**Phase 4: Recurring Use**
- *Feeling*: Efficient, secure, in-control
- *Goal*: Users send messages naturally, with underlying confidence that their identity is proven
- *Design Role*: Verification becomes background magic—efficient, reliable, trustworthy

**Phase 5: Edge Case Testing (Sam's Path)**
- *Feeling*: Analytical → Validated → Impressed
- *Goal*: Sam tests deterministic signatures, validates consistency, confirms the foundation works
- *Design Role*: Access to detailed cryptographic information enables confidence in technical correctness

### Micro-Emotions: The Two Pillars

**Trust vs. Skepticism**

*Trust Design Strategies:*
- **Visible Identity**: Public keys displayed prominently and consistently. Users always know whose message they're reading.
- **Verified Badge as Promise**: The ✓ badge becomes a visual symbol of cryptographic proof, not just a checkmark. It means "this is mathematically proven."
- **Transparent Signing**: Show that signing happens automatically. Users should never wonder "did my message get signed?"
- **Proof Access**: Drill-down reveals the actual signature. Sam can inspect it; Alex can see it exists. No hidden magic.

*Skepticism Risks to Avoid:*
- Verification that only works sometimes (inconsistent behavior erodes trust instantly)
- Unclear identity context ("whose key is this?" creates doubt)
- Hidden cryptographic operations (if signing is invisible, users might doubt it happened)
- Complex drill-down views (technical details that confuse rather than clarify)

**Confidence vs. Confusion**

*Confidence Design Strategies:*
- **Clear Mental Models**: Users should understand the core flow: Key ownership → Message signature → Verification proof. Each step connects obviously to the next.
- **Progressive Disclosure**: Don't dump all cryptographic details upfront. Key setup → Send message → Drill-down to explore. This builds confidence through understanding.
- **Consistent Language**: "Key," "signature," "verify," "badge" used consistently. No jargon variation that creates confusion.
- **Feedback Clarity**: When a message sends, when verification confirms, when a key imports—feedback should be unambiguous.
- **Error Clarity**: If something fails (invalid key, offline recipient), the error message should explain why and what to do next.

*Confusion Risks to Avoid:*
- Unexplained UI elements ("what does this button do?")
- Inconsistent terminology (switching between "key," "identity," "credential")
- Hidden operations (signing that happens silently feels like magic, not confidence)
- Overwhelming drill-down views (showing every cryptographic detail at once)
- Ambiguous feedback (messages that might-or-might-not have signed successfully)

### Design Implications

**For Trust:**
- Every verified badge must represent actual cryptographic proof
- Public keys visible in three places: lobby, message headers, drill-down details
- Deterministic signatures guaranteed (100% consistency in testing)
- Drill-down reveals the complete proof chain
- No approximate trust ("probably verified")—only verified or not verified

**For Confidence:**
- Onboarding explains the key ownership model before asking users to set up
- Key generation is one button; import has clear paste field and validation
- First message send shows immediate feedback (message appears + badge visible)
- Drill-down information presented in layers (message → key → signature → status)
- Error messages are specific and actionable
- Desktop UI uses familiar Windows conventions (no experimental UI patterns)

### Emotional Design Principles

1. **Proof Precedes Feeling**
   - Trust isn't emotional manipulation—it comes from actual cryptographic proof
   - The UX's job is to make that proof visible, accessible, and comprehensible
   - Users should feel trust because they *can verify*, not because they're asked to

2. **Clarity Over Cleverness**
   - Choose direct UI over elegant-but-confusing UI
   - Use familiar terms and patterns rather than novel interactions
   - If there's a choice between surprising and clear, choose clear
   - Minimize cognitive load so users have mental bandwidth for understanding verification

3. **Identity Ownership is the Center**
   - All emotional weight should connect back to "I own this identity"
   - Every design choice should support this core feeling
   - Visual design, interaction patterns, and feedback all reinforce identity ownership

4. **Technical Details on Demand**
   - Don't hide cryptographic details (Sam needs them)
   - But don't force them upfront (Alex doesn't need them initially)
   - Drill-down provides access without requiring it
   - Progressive disclosure builds confidence through understanding

5. **Consistency Builds Trust**
   - Deterministic signatures must work consistently (no random failures)
   - UI must behave predictably (no surprising UI patterns)
   - Identity context must be consistent (same key always represents the same person)
   - Verification must be reliable (verified badge means mathematically verified, always)

---

## UX Pattern Analysis & Inspiration

### Inspiring Products Analysis

Profile's design draws inspiration from two categories of products that have mastered their respective domains:

**Messaging Apps (Discord, Slack, Signal)**

These products excel at making real-time communication feel natural and effortless. From a UX perspective:

- **User Lobbies**: Discord's server/channel structure and Slack's workspace design show how to organize users and conversations. For Profile, the lobby of online users serves the same discovery function—simple, scannable list showing who's available.
- **Presence Indicators**: Status badges (online/away/offline) build immediate understanding of conversation readiness. Profile's verification badge (✓) serves a similar psychological role but represents cryptographic proof instead of presence.
- **Rapid Message Sending**: Discord and Slack make send feel instant (Enter to send). Profile inherits this pattern—keystroke efficiency matters for technical users.
- **Visual Differentiation**: Message timestamps, sender names, and visual hierarchy make conversations scannable. Profile needs similar clarity to distinguish sent/received messages and show verification status.
- **Notification and Feedback**: Messaging apps provide clear feedback when actions succeed (message sent, user joined lobby). Profile must do the same for signing operations and verification.

**Blockchain & Crypto Tools (Metamask, hardware wallets, Etherscan)**

These products handle complex cryptographic concepts and make them accessible to both technical and non-technical users:

- **Key Visualization**: Metamask shows public addresses prominently and offers one-click copy. Profile's public key display follows this pattern—visible, copyable, not hidden away.
- **Progressive Disclosure of Technical Details**: Etherscan shows transaction basics upfront but allows drilling down to full hex data for technical users. Profile's drill-down mirrors this—basic verification visible immediately; signature/key details available on demand.
- **Verification Indicators**: Crypto tools use checkmarks (✓), badges, and color coding to indicate verification status. Profile's green verification badge (✓) uses the same visual language users already understand from blockchain experiences.
- **Import/Export Patterns**: Crypto wallets make key import straightforward (paste field, clear validation). Profile's "Import Existing Private Key" follows this familiar pattern.
- **Deterministic Consistency**: Blockchain users expect deterministic behavior (same input = same output always). Profile's deterministic signatures tap into this expectation—users familiar with crypto understand this guarantee.
- **Transparency Over Hidden Complexity**: Blockchain tools expose cryptographic details rather than hiding them. Users appreciate transparency. Profile embraces this: signatures are visible, not hidden.

### Transferable UX Patterns

**From Messaging Apps:**

1. **Online User Lobby** - Display available recipients in a scannable list with clear presence indicators
   - *Application to Profile*: Lobby shows online users by public key, making recipient selection instant and clear
   
2. **Keystroke Efficiency** - Primary actions (send message) triggered by Enter key, not just mouse clicks
   - *Application to Profile*: Power users should send messages by typing and pressing Enter (similar to Discord/Slack)
   
3. **Automatic Scrolling to Latest** - Chat automatically scrolls to newest messages as they arrive
   - *Application to Profile*: Keep conversation centered on recent messages; don't require manual scrolling
   
4. **Timestamp Consistency** - Every message displays when it was sent, making conversation flow clear
   - *Application to Profile*: Message timestamps show message creation time, supporting Sam's deterministic signature testing
   
5. **Clear Sender Context** - Always show who sent each message (sender's public key visible in header)
   - *Application to Profile*: Never leave doubt about message origin; sender key always visible

**From Blockchain Tools:**

1. **Key Display Pattern** - Show public key prominently, make it copyable, don't bury it
   - *Application to Profile*: Public key shown in three places (onboarding, lobby context, drill-down). One-click copy for technical users.
   
2. **Progressive Disclosure** - Basic info always visible; technical details available via drill-down
   - *Application to Profile*: Message shows verification badge; click to see message content → key → signature → verification status
   
3. **Verification Badge Language** - ✓ checkmark or badge universally understood as "verified/authentic"
   - *Application to Profile*: Use consistent ✓ badge for all verified messages; color (green) adds additional confirmation
   
4. **Import Workflow** - Paste field for key import, clear validation feedback, success confirmation
   - *Application to Profile*: "Import Existing Key" has clear paste field, validates 256-bit format, shows derived public key
   
5. **Deterministic Expectation** - Users familiar with crypto expect consistent, deterministic behavior
   - *Application to Profile*: Tap into this expectation—users test "does the same message produce the same signature?" and it always does
   
6. **Transparency Value** - Show cryptographic details openly rather than hiding them behind "advanced" menus
   - *Application to Profile*: Signatures visible in drill-down for all users; no "show advanced options" gatekeeping

### Anti-Patterns to Avoid

1. **Hidden Verification Mechanism** - Don't hide signatures or key information behind complex menus. Blockchain tools that bury cryptographic details create confusion and skepticism.
   - *Application*: Keep signatures and keys accessible; never make verification feel like a black box
   
2. **Inconsistent Signing Behavior** - Users lose trust immediately if the same message produces different signatures or verification sometimes works and sometimes doesn't.
   - *Application*: Deterministic signing must be 100% consistent; any inconsistency breaks the trust model
   
3. **Unclear Identity Context** - Messaging apps that don't clearly show who sent a message cause confusion. Never let users wonder whose message they're reading.
   - *Application*: Sender public key always visible; never display a message without showing whose key signed it
   
4. **Forcing Technical Details Upfront** - Don't overwhelm new users (Alex) with all cryptographic details immediately. Blockchain wallets that dump full transaction hex on new users create cognitive overload.
   - *Application*: New users see the simple flow first; technical details available via drill-down when ready
   
5. **Ambiguous Feedback** - "Your message may have been sent" or "verification might work" creates doubt. Users need certainty.
   - *Application*: Feedback must be unambiguous: message sent ✓, signature verified ✓, or specific error explaining what failed
   
6. **Keyboard Unfriendly** - Blockchain tools that require clicking through menus slow down power users. Messaging apps that don't support keyboard shortcuts feel clunky.
   - *Application*: Design for keyboard efficiency from the start. Keyboard shortcuts should be primary; mouse clicks are secondary

### Design Inspiration Strategy

**What to Adopt (Full Implementation):**

- **Online user lobby** from messaging apps - simple, scannable list of available recipients
- **Keystroke-first message sending** - Enter to send is the primary action; mouse click is secondary
- **Progressive disclosure pattern** from blockchain tools - basic info visible, drill-down for details
- **Verification badge language** - ✓ checkmark universally understood as verified/authentic
- **Timestamp on every message** - supports both casual conversation and Sam's technical validation
- **Transparent key display** - public keys visible throughout, copyable, not hidden

**What to Adapt (Modified Implementation):**

- **User presence indicators** → adapted as **verification status indicators**
  - Messaging apps show "online/away/offline"; Profile shows "verified/unverified" on messages
  - Same visual pattern (badge, color, icon) but meaning is cryptographic proof instead of presence

- **Import workflow from crypto tools** → adapted for **Profile's key import**
  - Keeps the familiar pattern (paste field, validation, confirmation)
  - Simplified: only 256-bit key import (no complex options like hardware wallet integration)

- **Message notification/feedback** → adapted as **signature confirmation feedback**
  - Messaging apps show "message sent ✓"; Profile shows "message signed & verified ✓"
  - Same feedback pattern, extended meaning

**What to Avoid (Anti-Patterns Not Adopted):**

- Hiding cryptographic details behind "advanced" options
- Inconsistent verification behavior (deterministic signatures mean this never happens)
- Unclear sender identity context
- Forcing technical details on non-technical users (solved via progressive disclosure)
- Ambiguous success/failure feedback
- Keyboard-unfriendly navigation

### Design Foundation from Inspiration

This UX pattern analysis establishes that Profile can **feel natural to messaging app users** (familiar lobbies, keystroke efficiency, clear presence) while **being transparent to blockchain tool users** (visible keys, inspection access, verifiable proofs).

The inspiration strategy bridges two worlds: the simplicity and intuitiveness of messaging apps, combined with the transparency and verifiability of blockchain tools. Users don't need to be experts in either to use Profile effectively.

---

## Design System Foundation

### Design System Choice: Native Slint Design System

**Profile adopts a native Slint-first design system** rather than adapting web-based design systems. This maximizes Slint's unique capabilities:

- **Compiled Native UI**: Slint compiles directly to machine code, enabling pixel-perfect, performance-optimized rendering without browser overhead
- **Direct Hardware Access**: Leverage Windows native capabilities (keyboard, clipboard, window management) without web sandbox limitations
- **Desktop-First Paradigm**: Build for desktop constraints and affordances from the ground up, not as an afterthought
- **No CSS Translation Layer**: Write design logic directly for Slint's layout engine, avoiding impedance mismatch between web and native
- **Deterministic Rendering**: Native rendering ensures consistent behavior across users, aligning with Profile's deterministic signature philosophy

### Rationale for Selection

**Why Native Slint Instead of Adapted Web Systems:**

1. **Performance & Responsiveness**: Native Slint UI responds instantly to user input (keyboard, mouse, window focus). No JavaScript event loop delays. Critical for the core message-send flow where every millisecond matters.

2. **Windows Integration**: Slint accesses Windows native capabilities directly:
   - Taskbar integration (pinning, progress indicators, notifications)
   - Native keyboard shortcuts (Ctrl+C for copy, Alt+Tab focus handling)
   - Clipboard access (paste private keys directly, copy public keys)
   - Window state persistence (window position, size remembered)
   - These features are cumbersome in web frameworks but native in Slint

3. **Simplicity & Minimalism**: Slint's declarative design language is built for minimal, focused UIs. No CSS complexity, no framework bloat. Profile's intentional minimalism aligns perfectly with Slint's philosophy.

4. **Deterministic Design**: Slint's compiled nature means the UI behaves identically across all systems. No browser rendering variations, no platform-specific quirks. This mirrors Profile's deterministic cryptographic requirements.

5. **Memory Efficiency**: Slint applications are lightweight and start fast. Important for users who may run Profile alongside other applications or on resource-constrained systems.

6. **Developer Experience**: Slint's Rust integration is seamless. Signing logic, cryptographic operations, and UI can communicate without serialization overhead or FFI complexity.

### Implementation Approach

**Slint Design System Structure:**

```
Profile Design System (Slint-Native)
├── Color Palette
│   ├── Semantic colors (primary, danger, success, etc.)
│   ├── Neutral grays for hierarchy
│   └── Status indicators (verified ✓, offline, error)
├── Typography
│   ├── Header fonts (monospace for keys/addresses)
│   ├── Body fonts (readable at small sizes)
│   └── Code fonts (signatures, technical details)
├── Components
│   ├── Lobby (online user list, scrollable)
│   ├── Chat (message display, auto-scroll, timestamps)
│   ├── Composer (text input, send button/Enter key)
│   ├── Key Display (public key, copyable)
│   ├── Drill-Down Modal (verification details)
│   ├── Status Badge (verified ✓, offline indicator)
│   └── Keyboard Shortcuts (help overlay, Ctrl+? trigger)
├── Layout System
│   ├── Two-column (lobby + chat)
│   ├── Responsive resizing (user-draggable divider)
│   └── Persistent window state (remember user's layout)
├── Interaction Patterns
│   ├── Keyboard-first (Tab navigation, Enter to send)
│   ├── Mouse support (click to select, right-click menus)
│   ├── Focus states (clear visual indication of active element)
│   └── Feedback (message sent ✓, signature verified ✓)
└── Design Tokens
    ├── Spacing (margins, padding, gaps)
    ├── Typography scales
    ├── Emphasis levels (color, size, weight)
    └── Timing (animation duration, transition curves)
```

### Component-Level Design Decisions

**Lobby (Online User List)**
- Single-column scrollable list
- Each user shows: public key (monospace), online indicator
- Keyboard: Arrow keys to navigate, Enter to select
- Mouse: Click to select, shows selection highlight
- No complex filtering (minimalism first)

**Chat Area**
- Chronological message display (newest at bottom)
- Auto-scroll to latest message as they arrive
- Message format: `[timestamp] [sender key] [message content] [verification badge]`
- Timestamps: HH:MM:SS for precision (supports Sam's deterministic testing)
- Verification badge: ✓ (green) for verified, ⚠ (yellow) for unverified

**Message Composer**
- Single-line text input with clear focus state
- Send button (labeled "Send") or press Enter
- Send button disabled if no text or no recipient selected
- Feedback: Message appears immediately in chat with verification badge

**Key Display (Onboarding & Drill-Down)**
- Public key shown in monospace font (Visual confirmation it's machine-readable)
- One-click copy button (leverage Windows clipboard directly)
- Not truncated (full key always visible to avoid confusion)

**Drill-Down Modal**
- Triggered by clicking any message
- Shows layers of information progressively:
  - Layer 1 (Always Visible): Message content, timestamp, sender key
  - Layer 2 (Expandable): Full cryptographic signature (hex format)
  - Layer 3 (Status): Verification badge (✓ verified / ⚠ not verified)
- Monospace fonts for all cryptographic data (keys, signatures)
- Consistent with blockchain tools' transparency expectation

**Keyboard Shortcuts**
- Enter: Send message (from composer)
- Tab: Navigate between UI elements
- Ctrl+C: Copy selected (public key in drill-down, message content)
- Escape: Close drill-down modal
- Ctrl+?: Show keyboard shortcuts help overlay
- Ctrl+L: Focus on lobby (switch from chat to user selection)

### Customization Strategy

**What Not to Customize** (Stick to Slint Defaults):
- Layout engine (let Slint handle sizing/positioning)
- Font rendering (use platform default fonts)
- Window chrome (use Windows native window controls)
- Input handling (use Slint's built-in text input)

**What to Customize** (Design System Adds Value):
- Color palette (aligned with identity ownership theme)
- Semantic spacing (consistent margins/padding throughout)
- Emphasis patterns (size, color, weight for hierarchy)
- Keyboard shortcuts (map to Profile's specific actions)
- Status indicators (verified badge design, offline indicators)
- Focus states (clear visual feedback for keyboard navigation)

**Design System as Documentation**:
Rather than a separate design tool, the design system lives in Slint:
- Define colors as named constants (e.g., `verified-green: #22c55e`)
- Define component templates (lobby-item, message-bubble, status-badge)
- Document spacing via shared constants (small: 8px, medium: 16px, large: 24px)
- Export component library so future screens reuse proven patterns

### Visual Design Principles for Slint

1. **Density First** - Windows desktop users expect information-dense interfaces. Don't waste space.
2. **Monospace Identity** - Keys, signatures, and cryptographic data always monospace (signals "technical, verifiable")
3. **Color Restraint** - Limited palette (primary, neutral, success, warning). Overuse of color dilutes meaning.
4. **Focus Clarity** - Keyboard users need obvious focus indicators. Every interactive element has clear focus state.
5. **Native Feel** - Use Windows conventions (standard window controls, native keyboard shortcuts, taskbar integration)

---

## Defining Core Experience

### 2.1 The Defining Experience

**Profile's core interaction: "Send a message and automatically prove it's from you"**

This single interaction encapsulates everything Profile stands for. When a user types a message and presses Enter (or clicks Send), they're not just sending text—they're cryptographically proving the message came from their private key. The proof is automatic, invisible to the user, but tangible in the verified badge.

This is what makes Profile different from every other messaging app: **the message and its proof are inseparable**.

Users don't think about signing. They don't think about cryptography. They think: "I'm sending a message that proves it's from me." That's the defining experience.

### 2.2 User Mental Model

**How Users Think About This Interaction:**

**Mental Model 1: Identity Ownership (Alex's View)**
- "I own my private key"
- "I send a message"
- "The message has my signature—that proves it's from me"
- "Anyone can verify my message came from me"

This mental model is simple and empowering. Alex doesn't need to understand elliptic curves or hash functions. She just needs to know: her key = her identity, her message = proven from her key.

**Mental Model 2: Deterministic Validation (Sam's View)**
- "I have a 256-bit private key"
- "I send a message"
- "The system creates a deterministic signature (same message = same signature always)"
- "I can verify the signature against my public key"
- "I can test edge cases and confirm the signing mechanism is correct"

Sam's mental model is technical. He's validating that the cryptographic mechanism is working correctly and consistently. He wants to understand the proof, inspect the signature, and confirm determinism.

**Both Mental Models Succeed With Profile Because:**
- The core interaction is the same for both (send message → automatic signing)
- Alex gets the simple experience she needs (send, see verification badge)
- Sam gets the technical depth he needs (drill-down to inspect signature)
- Neither feels like the experience was compromised for the other

### 2.3 Success Criteria for Core Experience

When users send their first message, **success looks like:**

1. **Effortless Send**
   - User types message in composer
   - User presses Enter (or clicks Send)
   - Feeling: "That was simple, just like any other messaging app"

2. **Instant Feedback**
   - Message appears in chat immediately
   - Verified badge (✓) appears automatically
   - Feeling: "It worked. My message is proven."

3. **Understanding Through Interaction**
   - User clicks message to drill down
   - Sees message content, their public key, the signature, verification status
   - Feeling: "I can see how this works. The proof is real."

4. **Recurring Confidence**
   - User sends more messages
   - Every message verified consistently
   - Feeling: "This is reliable. I trust it."

5. **Technical Validation (Sam)**
   - Send same message twice, signatures match
   - Test unicode, special chars, long messages—signatures verify
   - Feeling: "The deterministic signing is solid. This is production-ready."

**Success Metric**: Users feel confident in three things:
- Their identity ownership (key = identity)
- Message authenticity (signature = proof)
- System reliability (determinism = consistency)

### 2.4 Novel vs. Established Patterns

**The Interaction Uses Established Patterns:**

✓ **Keystroke-driven send** - Established from Slack, Discord, iMessage (press Enter to send)  
✓ **Verification badge** - Established from blockchain tools, email clients (checkmark = verified)  
✓ **Message composition** - Established from all messaging apps (text input + send button)  
✓ **Lobby/recipient selection** - Established from messaging apps (list of users to message)  

**But Profile's Unique Twist:**

The verification isn't about server trust or email authentication. It's about **cryptographic proof embedded in the message itself**. This is novel—users don't expect verification to mean "the sender provably owns this private key" rather than "the server says this is from Alex."

This novelty is powerful but requires design support:
- The verified badge must visually signal "cryptographic proof" not just "server trust"
- The drill-down must make the proof tangible (show actual signature, not just a checkmark)
- The public key visibility must reinforce "this message is proven from this specific key"

**Teaching the Novel Concept:**

Rather than forcing explanation upfront, we use progressive disclosure:
1. **Send first message** - User sees it verified, feels success
2. **Click to drill down** - User sees the signature exists, curiosity satisfied
3. **Import test key** (Sam) - User validates the mechanism works
4. **Recurring use** - User builds confidence through repeated experience

This progression lets Alex learn naturally while giving Sam what he needs for validation.

### 2.5 Experience Mechanics: Step-by-Step Flow

**THE CORE INTERACTION: "Send Message" - Broken Into Mechanics**

**Phase 1: Initiation (User Starts Sending)**

```
Precondition: User has selected a recipient from lobby, composer field has focus

Action: User types a message (e.g., "Hello, is anyone here?")

System Response: 
- Composer accepts text input
- Send button becomes enabled (was disabled when field empty)
- No cryptographic operations yet—just text input
```

**Phase 2: Interaction (User Triggers Send)**

```
Action: User presses Enter (primary) or clicks Send button (secondary)

System Response (Cryptographic Operations Begin):
1. Capture message text from composer
2. Sign message deterministically with user's private key (instant, invisible)
3. Construct message object: { message, senderPublicKey, signature, timestamp }
4. Send to server via WebSocket
```

**Phase 3: Feedback (User Sees Success)**

```
Server Processes:
1. Receives signed message from sender
2. Validates signature against senderPublicKey
3. Pushes message to recipient (if online) with signature intact
4. Sends back confirmation to sender

Client Displays (Sender's View):
- Message appears in chat immediately
- Format: "[HH:MM:SS] [sender key] [message text] [✓]"
- Verified badge (✓) in green
- Composer clears for next message

Client Displays (Recipient's View):
- Message arrives and appears in chat
- Format: "[HH:MM:SS] [sender key] [message text] [✓]"
- Verified badge automatically displays
- Message is already verified by client
```

**Phase 4: Completion & Discovery (User Understands)**

```
Completion Signal: Message appears with verification badge

Immediate Feeling: "It worked. The message is verified."

Optional Discovery Path:
- User clicks message
- Drill-down modal opens
- Shows: Message content → Sender's public key → Cryptographic signature → Verification status
- User can copy key, inspect signature, understand the proof chain
- Close modal (Escape key)
- Return to chat

Recurring Pattern: Each message sent follows same flow
- User types
- Presses Enter
- Message appears with ✓
- System is predictable and reliable
```

**CRITICAL DESIGN DETAILS IN THIS FLOW:**

1. **Signing is Invisible**: User never clicks "sign" or confirms signature creation. Signing is automatic at send, as natural as pressing Enter.

2. **Feedback is Immediate**: Message appears in chat with verification badge instantly. No waiting, no "checking signature" spinner. Deterministic signing happens too fast to see.

3. **Verification is Visible**: Badge appears automatically. User doesn't need to do anything to see verification. It's not hidden in a menu or drill-down—it's there by default.

4. **Proof is Accessible**: If user wants to see how verification works, drill-down is one click away. But it's not forced. Alex just sees the badge and feels confident. Sam can drill down to inspect details.

5. **Repetition Builds Trust**: Every message follows the same pattern. No surprises, no inconsistencies. User's confidence grows through repeated successful sends.

### 2.6 Why This Defining Experience Works

This core interaction achieves Profile's emotional and functional goals:

**Trust ✓**
- Every message is provably from the sender's key
- Signature is visible on drill-down (not hidden)
- Verification is consistent (deterministic signing)
- User can inspect the proof chain

**Confidence ✓**
- Send flow is familiar (like any messaging app)
- Feedback is clear (message appears with badge)
- No confusion about signing (it just happens)
- Visual connection between key → message → signature is clear

**Identity Ownership ✓**
- Sender's public key visible in message header
- User owns their private key completely
- Each message proves identity through signature
- Drilling down reinforces: "This signature came from my key"

**Technical Validation ✓**
- Sam can test determinism (send same message twice, same signature)
- Signature is inspectable (full hex in drill-down)
- System behavior is predictable
- Edge cases are handled (unicode, special chars, all work)

**Desktop Efficiency ✓**
- Keystroke-driven (Enter to send)
- Keyboard-first navigation
- No unnecessary steps or confirmations
- Power users can work at typing speed

This defining experience is the foundation. If we nail "send message with automatic signing," everything else—the lobby, the drill-down, the key setup—follows naturally and feels coherent.

---

## Visual Design Foundation

### Color System

**Theme Direction: Dark Mode First with Trust-Forward Colors**

Profile uses a dark color scheme optimized for desktop work environments. Dark mode reduces eye strain during long messaging sessions, aligns with technical/developer aesthetics, and creates a professional, secure feeling appropriate for identity-verified communication.

**Core Color Palette:**

```
Primary Brand Colors:
- Primary Blue: #0066CC (trust, security, cryptographic confidence)
- Verified Green: #22c55e (success, verification, signature valid)
- Error Red: #ef4444 (warnings, invalid signatures, offline status)
- Neutral Gray: #1f2937, #374151, #6b7280 (background, text, borders)

Background Colors:
- Surface Dark: #111827 (main application background)
- Surface Lighter: #1f2937 (panels, elevated surfaces)
- Surface Lighter Still: #374151 (hover states, subtle elevation)

Text Colors:
- Text Primary: #f3f4f6 (body text, high contrast)
- Text Secondary: #d1d5db (secondary info, metadata)
- Text Tertiary: #9ca3af (disabled, inactive states)

Semantic Colors:
- Success: #22c55e (verified, online, valid)
- Warning: #f59e0b (caution, attention needed)
- Error: #ef4444 (invalid, offline, critical)
- Info: #0066CC (informational, primary actions)
```

**Semantic Color Mapping:**

| Element | Color | Usage |
|---------|-------|-------|
| Verified Badge | #22c55e | Message verification ✓ indicator |
| Public Key Display | #0066CC | Identity ownership context |
| Signature (Drill-Down) | #6b7280| Technical data, monospace text |
| Online Indicator | #22c55e | User online in lobby |
| Offline Indicator | #6b7280 | User offline in lobby |
| Focus State | #0066CC | Keyboard focus highlight |
| Error Message | #ef4444 | Invalid key, offline notification |
| Hover State | #374151 | Interactive element hover |

**Accessibility Considerations:**

- All text meets WCAG AA contrast ratios (minimum 4.5:1 for body text)
- Verified badge uses both color AND symbol (✓) so meaning isn't lost for colorblind users
- Focus states are high-contrast (blue highlight on dark background)
- No color conveys information alone (always paired with icons/text)

### Typography System

**Font Stack Strategy:**

```
UI Elements (Headers, Labels, Buttons):
- Primary: "Segoe UI", "-apple-system", "BlinkMacSystemFont", sans-serif
- Fallback: System default sans-serif for each platform
- Rationale: Windows native font, clean, readable at all sizes

Body Text (Messages, Descriptions):
- Primary: "Segoe UI", sans-serif
- Size: 14px (standard), 12px (secondary), 16px (emphasis)
- Line Height: 1.5 (readable for longer content)

Cryptographic Data (Keys, Signatures):
- Family: "Consolas", "Monaco", "monospace"
- Size: 12px or 11px (smaller for technical data)
- Letter Spacing: slightly increased for clarity
- Rationale: Monospace signals "machine-readable, technical, important"
```

**Type Scale & Hierarchy:**

```
Component Sizes:
- H1 (Headers): 20px, bold, primary text color
- H2 (Subheaders): 16px, semibold, primary text color  
- H3 (Section Labels): 14px, semibold, secondary text color
- Body (Message Text): 14px, regular, primary text color
- Caption (Timestamps, Keys): 12px, regular, secondary text color
- Monospace (Signatures): 11px, regular, monospace font, neutral gray

Line Heights:
- Headers: 1.2 (compact)
- Body: 1.5 (readable)
- Monospace: 1.6 (more space for hex legibility)

Font Weights:
- Regular (400): Body text, normal emphasis
- Semibold (600): Labels, interactive elements
- Bold (700): Headers, critical information
```

**Typography Design Principles:**

1. **Clarity Over Beauty**: Prioritize readability over aesthetic flourishes
2. **Hierarchy Through Size & Weight**: Use sizing and weight, not color, for primary hierarchy
3. **Monospace for Technical Data**: Any key, signature, or cryptographic data uses monospace
4. **Consistency**: Same text role always uses same size/weight across UI
5. **Accessibility**: Minimum 14px for body text, sufficient line height for readability

### Spacing & Layout Foundation

**Spacing Grid System:**

Profile uses an **8px base unit** for consistent spacing throughout the application.

```
Spacing Scale:
- 4px: micro spacing (within components)
- 8px: base unit (padding, margins, gaps)
- 12px: small spacing (component padding)
- 16px: standard spacing (between major sections)
- 24px: large spacing (significant separation)
- 32px: XL spacing (major section breaks)

Application Rules:
- All margins and padding are multiples of 8px
- All gaps between elements are multiples of 8px
- Component padding: typically 12px-16px
- Content margins: 16px-24px
```

**Layout Structure:**

```
Main Application Layout (Two-Column):

┌─────────────────────────────────────────┐
│ Profile - [user key] (title bar)        │
├────────────────┬────────────────────────┤
│                │                        │
│     LOBBY      │       CHAT AREA        │
│  (Users List)  │   (Messages Display)   │
│                │                        │
│ • User 1 (✓)   │ [12:34:56] [key] msg ✓│
│ • User 2 (✓)   │ [12:35:01] [key] msg ✓│
│ • User 3 (○)   │                        │
│                │ [Composer Below]       │
│                │ Type message...        │
└────────────────┴────────────────────────┘

Lobby Width: 250px (fixed or draggable divider)
Chat Width: Remaining space (flexible)
Resizable: User can drag divider to adjust ratio
Persistent: Window size and layout remembered
```

**Component Spacing Rules:**

```
Lobby Component:
- Header padding: 16px
- User item padding: 8px 12px
- User item gap: 8px
- Scroll area margin: 0 (edge to edge)

Chat Component:
- Message padding: 8px 12px
- Message line height: 1.5
- Message gap: 4px (between consecutive messages)
- Timestamp/key/content all inline
- Verification badge: inline, right-aligned

Composer Component:
- Container padding: 12px 16px
- Text input padding: 8px 12px
- Send button padding: 8px 16px
- Input/button gap: 8px
- Container border-top: 1px, neutral gray

Drill-Down Modal:
- Modal padding: 24px
- Section padding: 16px
- Content margins: 12px 0
- Close button: top-right, 16px from edge
```

**Layout Principles:**

1. **Density First**: Windows desktop users expect information-dense UI; no wasted space
2. **Two-Column Focus**: Lobby on left (stable), chat on right (primary focus)
3. **Keyboard Navigation**: Tab order: Lobby → Chat → Composer (logical flow)
4. **Persistent State**: Window size, divider position, scroll state remembered
5. **Responsive Resizing**: Components expand/contract smoothly with window resizing
6. **Drag Divider**: User can adjust lobby/chat ratio by dragging center divider

### Visual Design Principles for Profile

**1. Trust Through Transparency**
- Cryptographic data always visible (keys, signatures)
- No hidden complexity behind menus
- Verification badge means mathematical certainty, not faith

**2. Monospace = Technical**
- Any cryptographic data: monospace font
- Signals "this is machine-readable, important, verifiable"
- Consistency creates expectation

**3. Dark Mode Professionalism**
- Dark theme reduces eye strain (important for all-day use)
- Aligns with developer/technical aesthetic
- Creates secure, serious tone appropriate for identity management

**4. Color Restraint**
- Limited palette (5-6 colors max)
- Each color has semantic meaning
- Overuse dilutes meaning; restraint increases impact

**5. Focus Clarity for Keyboard Users**
- Every interactive element has clear focus indicator
- Focus state uses high-contrast color (#0066CC on dark background)
- Tab order is logical and predictable

**6. Density Without Clutter**
- Efficient use of space
- Clear visual hierarchy (size, weight, spacing)
- Nothing extraneous, everything serves function

**7. Windows Native Integration**
- System fonts (Segoe UI on Windows)
- Standard window controls and chrome
- Keyboard shortcuts follow Windows conventions
- Clipboard integration seamless

### Implementation Notes for Slint

**Color Constants (Slint):**
```
colors := {
  primary-blue: #0066CC,
  verified-green: #22c55e,
  error-red: #ef4444,
  warning-orange: #f59e0b,
  surface-dark: #111827,
  surface-light: #1f2937,
  surface-lighter: #374151,
  text-primary: #f3f4f6,
  text-secondary: #d1d5db,
  text-tertiary: #9ca3af,
}
```

**Spacing Constants (Slint):**
```
spacing := {
  xs: 4px,
  sm: 8px,
  md: 12px,
  lg: 16px,
  xl: 24px,
  xxl: 32px,
}
```

**Font Constants (Slint):**
```
fonts := {
  sans: "Segoe UI",
  mono: "Consolas",
  size-small: 12px,
  size-normal: 14px,
  size-large: 16px,
  size-header: 20px,
}
```

This visual foundation ensures Profile feels cohesive, professional, and trustworthy—aligning perfectly with its core mission of cryptographic identity verification.

---

## Design Direction Exploration

### Design Directions Explored

I've created **6 comprehensive design direction mockups**, each exploring a different visual and interaction approach while maintaining our core design principles, visual foundation, and emotional goals.

**The Six Directions:**

1. **Compact Dense** - Maximum information density, monospace throughout, built for technical power users
2. **Breathing Room** - Generous spacing, message cards, larger fonts, visually accessible for new users
3. **Minimalist Focus** - Extreme minimalism, essentials only, content-first philosophy
4. **Verification Center** - Verification is the hero, large verified badges, stacked layout emphasizes proof
5. **Chat-First Traditional** - Like Slack/Discord, familiar chat pattern, inline badges, comfortable
6. **Key-Prominent Identity** - Keys are the centerpiece, sender key leads each message, identity ownership visual

**Access the Interactive Mockups:**

Open `ux-design-directions.html` in your browser to explore all six directions with:
- Full-screen mockups showing lobby, chat area, and composer
- Interactive hover states
- Side-by-side comparison capability
- Complete UI context and component examples
- Design evaluation framework for each direction

### Evaluation Criteria

Each design direction should be evaluated against these criteria:

**✓ Layout Intuitiveness**
- Does the information hierarchy match user priorities?
- Is the flow natural and logical?
- Can users find what they need without thinking?

**✓ Interaction Style**
- Does the interaction pattern support "send message with automatic signing"?
- Are keyboard shortcuts obvious?
- Does the core experience feel effortless?

**✓ Visual Weight & Density**
- Is the density appropriate (not overwhelming for Alex, not wasteful for Sam)?
- Does the spacing feel intentional?
- Is the information hierarchy clear?

**✓ Trust & Confidence**
- Do visual choices support cryptographic trust?
- Is verification prominent enough?
- Does the design communicate "provably from this key"?

**✓ Component Support**
- Do components effectively support both user archetypes?
- Can Alex understand without drilling down?
- Can Sam validate and inspect technical details?

**✓ Desktop Efficiency**
- Does the design support keyboard-first power users?
- Are shortcuts accessible?
- Is navigation quick and logical?

### Design Direction Selection Process

**Step 1: Explore All Six Directions**
Review each mockup using the evaluation criteria. Notice which directions feel most aligned with Profile's personality.

**Step 2: Identify Resonance**
Note which direction(s) feel right for your users. Look for:
- The one that feels most intuitive for the core message-send flow
- The one that makes verification feel most tangible and trustworthy
- The one that balances Alex's simplicity with Sam's technical depth

**Step 3: Consider Combinations**
You don't need to choose just one. You might:
- Use Direction 5 (Chat-First Traditional) as the base but add elements of Direction 4 (Verification Center) to make the badge more prominent
- Prefer Direction 1 (Compact Dense) but with slightly more spacing from Direction 2 (Breathing Room)
- Love Direction 6 (Key-Prominent) for the identity emphasis but want Direction 5's familiar chat layout

### Recommended Direction: Chat-First Traditional (Direction 5)

Based on Profile's requirements and emotional goals, I recommend **Direction 5: Chat-First Traditional** as the foundational approach, with the following rationale:

**Why Direction 5:**

1. **Familiar Pattern Works Well** - Users coming from Discord, Slack, Signal, or any messaging app will immediately understand the layout. This reduces cognitive load for Alex while not alienating Sam.

2. **Balanced Information Hierarchy** - The layout shows:
   - Sender's key (inline, visible, monospace)
   - Verified badge (prominent, color-coded, meaningful)
   - Message content (prominent, readable)
   - Timestamp (context, not distraction)
   
   This hierarchy serves both archetypes: Alex sees "verified" and feels confident; Sam can drill down to inspect the signature.

3. **Desktop Efficiency** - The two-column layout (lobby + chat) aligns perfectly with our keyboard-first approach. Tab order is logical: Lobby → Chat → Composer.

4. **Scales to Complexity** - As a foundation, Direction 5 allows room for enhancement:
   - Add drill-down modal (already planned for signature inspection)
   - Keyboard shortcuts overlay (Ctrl+?)
   - Advanced search/filtering (Phase 2)
   - Group messaging (Phase 3)

5. **Trust Through Familiarity** - Users already trust messaging apps. By using familiar patterns, we transfer that trust, then layer on cryptographic proof.

**Optional Enhancement from Direction 4:**

Consider making the **verified badge slightly more prominent** by:
- Using a slightly larger badge size
- Adding subtle background color (green tint) to message row
- Showing badge immediately without needing to look closely

This maintains Direction 5's familiarity while borrowing Direction 4's emphasis on verification as the key differentiator.

### Implementation Approach

**Foundation: Direction 5 with Direction 4 Enhancement**

The chosen direction will guide:

1. **Component Design** - Each UI component (message bubble, lobby item, composer) designed consistently
2. **Layout Grid** - 8px spacing system applied throughout
3. **Color Application** - Our established palette (blue for identity, green for verified, grays for hierarchy)
4. **Typography** - Monospace for keys/signatures, sans-serif for content
5. **Keyboard Navigation** - Tab order: Lobby → Chat messages → Composer
6. **Interaction States** - Hover, focus, active states for all interactive elements
7. **Accessibility** - High contrast focus indicators, color + symbol for verification

**Design Direction Specifications:**

- **Lobby Width**: 250px (or draggable, with 220-280px range)
- **Message Format**: `[timestamp] [sender_key] [badge] [message_text]`
- **Verified Badge**: Green (#22c55e), ✓ symbol, inline right-aligned
- **Unverified**: Subtle warning color, ⚠ symbol
- **Key Display**: Monospace, blue color (#0066CC), abbreviated in message header, full key in drill-down
- **Composer**: Text input + Send button, Enter key also sends
- **Drill-Down**: Modal showing message content → sender key → full signature → verification status

This direction provides the best balance of familiarity, clarity, and alignment with Profile's cryptographic mission.

---

## User Journey Flows

### Journey 1: Alex's First Experience - "Understanding Identity Ownership"

**Journey Goal:** Alex discovers Profile, generates a key, sends their first message, understands they own their identity through cryptography.

**User Archetype:** Alex - Non-technical user drawn to identity ownership, values simplicity, needs empowerment without technical overwhelm.

**Success Criteria:**
- Alex completes key generation without confusion
- Alex sends first message and sees verification badge
- Alex feels confident they "own" their identity
- Alex understands the basic concept: key = identity, signature = proof

**Journey Flow:**

```
START: Alex discovers Profile app
  ↓
[ENTRY POINT] Launch Application
  ↓
CHECK: Does user have a key?
  ├─ NO → GO TO: Key Generation Flow
  └─ YES → GO TO: Lobby Flow

[KEY GENERATION FLOW]
  ↓
SCREEN: "Welcome to Profile"
MESSAGE: "You control your identity with a private key. Generate one now or import an existing key."
CHOICE: [Generate New Key] or [Import Existing Key]
  ├─ Generate New Key:
  │   ↓
  │   ACTION: System generates 256-bit random key
  │   DISPLAY: "Your public key: [full key in monospace]"
  │   EXPLANATION: "This is how people verify messages from you. Keep your private key secret."
  │   ACTION: Offer [Copy] button for public key
  │   BUTTON: [Continue to Lobby]
  │   ↓
  │   FEELING: "I have a key now. It's mine. This is me."
  │
  └─ Import Existing Key:
      ↓
      SCREEN: Paste field with label "Enter your 256-bit private key"
      INPUT: User pastes key
      VALIDATION: Check format, show error if invalid
      IF VALID:
        ↓
        DISPLAY: "Private key loaded. Your public key: [derived key]"
        BUTTON: [Continue to Lobby]
        ↓
        FEELING: "I imported my key. This is still me."
      IF INVALID:
        ↓
        ERROR: "That doesn't look like a 256-bit key. Try again or generate a new one."
        RETRY: User can edit or start over

[LOBBY FLOW]
  ↓
SCREEN: Lobby with online users list
PROMPT: "Select someone to message, or wait for someone to message you"
DISPLAY: List of online users with public keys and online indicators (✓ green)
USER ACTION: Alex clicks on user "7B4D9C2A"
  ↓
CHAT ACTIVATED
  ↓
COMPOSER READY
TEXT FIELD: "Type message..."
FOCUS: Composer field automatically focused
USER TYPES: "Hello, is anyone here?"
  ↓
[CORE INTERACTION: SEND MESSAGE]
USER PRESSES: Enter (or clicks Send button)
  ↓
SYSTEM OPERATIONS (Invisible):
  1. Capture message text
  2. Sign message with Alex's private key (deterministic)
  3. Create signed message object
  4. Send to server via WebSocket
  ↓
INSTANT FEEDBACK:
MESSAGE APPEARS: "[12:34:56] 3A8F2E1C Hello, is anyone here? ✓"
FEELING: "It worked! The message is there. That checkmark means something good."
BADGE: Green ✓ badge visible immediately
  ↓
[OPTIONAL: DISCOVERY DRILL-DOWN]
USER CLICKS: On their message
  ↓
MODAL OPENS: Message Details
SHOWS:
  - Message content: "Hello, is anyone here?"
  - Sender key: "3A8F2E1C" (monospace, blue)
  - Signature: [Full cryptographic signature visible] (monospace, gray)
  - Status: "✓ Verified" (green, prominent)
  ↓
EXPLANATION: "This message came from my key. The signature proves it. No one else could have sent this."
  ↓
FEELING: "I understand now. My message has proof. I really do own this identity."
  ↓
USER CLOSES MODAL: [Escape] or [Close button]
  ↓
BACK TO CHAT
  ↓
USER SENDS MORE MESSAGES
Each message follows same flow: type → press Enter → appears with ✓
Repeated success builds confidence
  ↓
RECURRING USE
Alex uses Profile regularly, always feels confident in their messages
Their identity is proven, visible, owned

END: Alex has successfully completed their first journey
SUCCESS MOMENT: ✓ Identity understood, ✓ Message sent, ✓ Verification discovered
```

**Critical Design Details for Alex's Journey:**

1. **Simplicity at Every Step** - Alex never sees technical jargon, only clear guidance
2. **Immediate Feedback** - Message appears instantly, verification badge visible by default
3. **Passive Learning** - Alex learns about cryptography by doing, not by reading docs
4. **One Path to Value** - Fewest steps possible from app launch to first verified message
5. **Optional Depth** - Drill-down is available when curious, but not required

---

### Journey 2: Sam's Technical Validation - "Validating Deterministic Signatures"

**Journey Goal:** Sam imports an existing private key, sends multiple messages (including variations), validates that deterministic signing is working correctly by comparing signatures.

**User Archetype:** Sam - Technically savvy cryptography enthusiast, values correctness and validation, wants access to signature details.

**Success Criteria:**
- Sam imports key successfully
- Sam sends identical message twice, confirms signatures match
- Sam tests edge cases (unicode, special characters, long text)
- Sam validates that signing mechanism is reliable and deterministic
- Sam trusts the foundation is solid for future ZK integration

**Journey Flow:**

```
START: Sam launches Profile, ready to validate the signing mechanism
  ↓
[KEY IMPORT FLOW]
  ↓
SCREEN: Key entry interface
PASTE FIELD: "Enter your 256-bit private key"
SAM PASTES: His existing private key
  ↓
VALIDATION: Format check passes
DISPLAY: "Private key loaded. Public key: [key_hash]"
BUTTON: [Proceed]
  ↓
FEELING: "Key loaded. Now I can start testing."

[LOBBY → RECIPIENT SELECTION]
  ↓
Sam sees online users
ACTION: Selects another test user (or their own account from different session)
PURPOSE: Set up the message send flow
  ↓

[TEST SEQUENCE: DETERMINISTIC SIGNING VALIDATION]

TEST 1: IDENTICAL MESSAGE CREATES IDENTICAL SIGNATURE
  ↓
  MESSAGE 1: "Testing deterministic signatures"
  PRESS ENTER
    ↓
    System signs with private key
    SYSTEM SENDS: {message, publicKey, signature}
    ↓
    MESSAGE APPEARS: "[12:34:56] 3A8F2E1C Testing deterministic signatures ✓"
    ↓
    SAM CLICKS: On message to view details
    ↓
    MODAL SHOWS:
      - Message: "Testing deterministic signatures"
      - Public Key: [full_key_monospace]
      - Signature: "a4f3e2c1b8d5e9f2..." [full hex]
      - Status: "✓ Verified"
    ↓
    SAM NOTES: The signature value
    SAM CLOSES MODAL

  MESSAGE 2: "Testing deterministic signatures" (EXACT SAME TEXT)
  PRESS ENTER
    ↓
    System signs with private key
    MESSAGE APPEARS: "[12:35:01] 3A8F2E1C Testing deterministic signatures ✓"
    ↓
    SAM CLICKS: On message to view details
    ↓
    MODAL SHOWS:
      - Message: "Testing deterministic signatures"
      - Public Key: [full_key_monospace]
      - Signature: "a4f3e2c1b8d5e9f2..." [SAME AS MESSAGE 1]
      - Status: "✓ Verified"
    ↓
    VALIDATION: ✓ Signatures match! Deterministic signing confirmed
    FEELING: "Perfect. The signing is deterministic."

TEST 2: EDGE CASE - UNICODE CHARACTERS
  ↓
  MESSAGE: "Unicode test: 你好 🔐 ñ"
  PRESS ENTER
    ↓
    MESSAGE APPEARS: "[12:35:08] 3A8F2E1C Unicode test: 你好 🔐 ñ ✓"
    ↓
    SAM CLICKS: View details
    ↓
    MODAL: Signature shown, verified badge present
    ↓
    VALIDATION: ✓ Unicode handled correctly

TEST 3: EDGE CASE - SPECIAL CHARACTERS
  ↓
  MESSAGE: "Special: !@#$%^&*()[]{}|;:',.<>?"
  PRESS ENTER
    ↓
    MESSAGE APPEARS: "[12:35:12] 3A8F2E1C Special: !@#$%^&*()[]{}|;:',.<>? ✓"
    ↓
    SAM CLICKS: View details
    ↓
    MODAL: Signature shown, verified badge present
    ↓
    VALIDATION: ✓ Special characters handled

TEST 4: EDGE CASE - VERY LONG MESSAGE
  ↓
  MESSAGE: [Long text message, 500+ characters]
  PRESS ENTER
    ↓
    MESSAGE APPEARS: "[12:35:16] 3A8F2E1C [long message] ✓"
    ↓
    SAM CLICKS: View details
    ↓
    MODAL: Full message visible, signature shown, verified
    ↓
    VALIDATION: ✓ Long messages handled correctly

TEST 5: EDGE CASE - WHITESPACE
  ↓
  MESSAGE: "   spaces and tabs	test   "
  PRESS ENTER
    ↓
    MESSAGE APPEARS: "[12:35:20] 3A8F2E1C    spaces and tabs	test    ✓"
    ↓
    SAM CLICKS: View details
    ↓
    MODAL: Exact whitespace preserved, signature shown
    ↓
    VALIDATION: ✓ Whitespace preserved and handled

[ACCUMULATED VALIDATION RESULTS]
  ↓
SAM'S CONCLUSION:
✓ Deterministic signing works consistently
✓ Same message always produces same signature
✓ Edge cases handled correctly (unicode, special chars, length, whitespace)
✓ Signatures verify correctly every time
✓ Foundation is solid for cryptographic correctness
✓ Ready for zero-knowledge proof integration
  ↓
FEELING: "This is production-ready. The signing mechanism is reliable."

END: Sam has successfully validated the cryptographic foundation
SUCCESS MOMENT: ✓ Determinism confirmed, ✓ Edge cases validated, ✓ Trust in foundation established
```

**Critical Design Details for Sam's Journey:**

1. **Full Signature Visibility** - Sam can access complete hex signatures, not truncated versions
2. **Message Drill-Down Always Available** - Every message has accessible cryptographic details
3. **Consistent Behavior** - Same message always produces identical results
4. **Edge Case Support** - All message types (unicode, special chars, long text) handle correctly
5. **Technical Details Uncluttered** - Crypto information presented clearly without marketing language

---

### Journey Patterns Identified

**Pattern 1: Key Setup (Foundational)**
- Entry point for new users
- Two paths: Generate or Import
- Clear feedback on success
- Establishes trust in identity ownership
- Used by: Both Alex and Sam

**Pattern 2: Message Send Loop (Core Interaction)**
- User types message → Presses Enter/Click Send → Message appears with badge
- Invisible signing operation (feels instant to user)
- Immediate feedback (message in chat)
- Repeatable and consistent
- Used by: Both Alex and Sam repeatedly

**Pattern 3: Drill-Down Discovery (Optional Depth)**
- Click message → Modal opens → Shows layered details
- Available for all messages, optional not mandatory
- Progressive disclosure: message → key → signature → status
- Alex uses casually, Sam uses for validation
- Supports both archetypes

**Pattern 4: Recipient Selection (Navigation)**
- View lobby of online users
- Select user to message
- Conversation context shifts to selected user
- Clear visual indication of current recipient
- Used by: Both archetypes when switching conversations

**Pattern 5: Error Handling (Edge Cases)**
- Invalid key import: clear error, retry available
- Offline recipient: notification sent to sender, user can retry later
- Invalid signature: rejected, not displayed
- All errors specific and actionable
- Maintains user confidence even in failure states

---

### Flow Optimization Principles

**Principle 1: Minimize Steps to Value**
- From app launch to first verified message: as few steps as possible
- From "I want to verify this message" to seeing signature: one click
- From "I want to test determinism" to comparing signatures: obvious flow

**Principle 2: Clarity Over Features**
- Every screen has one primary action
- No unnecessary navigation
- Confirm successful completion before moving forward
- Clear error messages that explain what went wrong

**Principle 3: Keyboard Efficiency**
- Primary action always has keyboard shortcut
- Tab order is logical (Lobby → Chat → Composer)
- Enter key sends messages (no Ctrl+Enter required)
- Escape closes modals
- Ctrl+? shows keyboard help

**Principle 4: Feedback Immediacy**
- Message appears instantly (signing is fast)
- Verification badge appears by default (no "checking" spinner)
- Focus shifts appropriately as user progresses
- Timestamp shows moment of action
- Green checkmark universally means success

**Principle 5: Discovery Encouragement**
- Drill-down available but optional (doesn't gate functionality)
- Messages are clickable (inviting exploration)
- Monospace fonts signal "click me, there's technical info here"
- Progressive disclosure rewards curiosity

**Principle 6: Confidence Through Consistency**
- Same action always produces same result (deterministic signing)
- UI behaves predictably in all contexts
- Error recovery is always possible
- No random failures or surprising behaviors
- Repeated success builds trust

**Principle 7: Power User Support**
- Keyboard shortcuts for everything
- Access to full technical details (for Sam)
- No "advanced mode" toggle (everything accessible)
- Sam can work at typing speed
- Performance is snappy (no delays)

---

### Flow Adaptability for Future Phases

These journeys are designed to scale:

**Phase 2 (Growth) Additions:**
- Add user profiles → Add "view profile" drill-down
- Add message persistence → Add "view history" in same message list
- Add typing indicators → Show "User typing..." above input
- Add read receipts → Add status after message (seen, read)

**Phase 3 (Expansion) Additions:**
- Add group messaging → Same core flow, but recipient is group name
- Add contact management → Lobby evolves to show favorites/blocked
- Add reactions → Add "(+emoji)" after message verification badge
- Add ZK proofs → Enhance drill-down to show zero-knowledge proof status

The core journey flows remain stable; UI evolves around them.

---

## Component Strategy

### Step 11: Component Architecture & Implementation Roadmap

Component strategy defines how Profile's UI is built, which components are essential for MVP, and which can be deferred to future phases. This strategy balances technical feasibility, developer experience, and user value delivery.

### Component Inventory

Profile requires **9 core custom components** plus **Slint standard components** (Button, TextEdit, ScrollView, etc.). 

#### Core Custom Components

**1. LobbyComponent - Online User List**

**Purpose:** Display available recipients for messaging. Users select a recipient to activate the chat area.

**Key Responsibility:**
- Display list of online users with their public keys
- Show online/offline status indicators
- Support keyboard navigation (arrow keys, Enter to select)
- Show selection highlight
- Auto-scroll if list is long
- Support right-click context menu (copy key, ban user [Phase 2])

**States & Variants:**
| State | Description | Visual |
|-------|-------------|--------|
| Default | Idle, no user selected | Gray background, neutral text |
| Hover | Mouse over user | Slightly elevated background (#374151) |
| Selected | User is current recipient | Blue highlight (#0066CC), bold text |
| Online | User indicator | Green dot (●) #22c55e |
| Offline | User indicator | Gray dot (○) #6b7280|

**Accessibility:**
- Full keyboard navigation (arrow keys, Enter)
- Screen reader compatible (name read as "user key, online status")
- Focus indicator visible for keyboard users

**Composition:**
```
LobbyComponent
├── Header ("Online Users")
├── ScrollView
│   └── [User Items]
│       ├── Status Indicator (● or ○)
│       ├── User Key (monospace, blue when selected)
│       └── Hover State
└── Empty State ("No users online")
```

**Implementation Complexity:** Low (standard list component)

---

**2. ChatAreaComponent - Message Display**

**Purpose:** Display sent/received messages chronologically with verification status. Core to Profile's identity-forward experience.

**Key Responsibility:**
- Display messages in chronological order (oldest to newest, newest at bottom)
- Auto-scroll to latest message as new messages arrive
- Show sender's public key for each message
- Display verification badge (✓ green or ⚠ yellow)
- Show timestamp (HH:MM:SS)
- Support message clicking (opens drill-down modal)
- Support text selection (for copy)
- Highlight your own messages differently (subtle background)

**States & Variants:**
| Component | State | Visual |
|-----------|-------|--------|
| Message Bubble (Sent) | Default | Slight background tint, right-aligned position |
| Message Bubble (Received) | Default | Left-aligned, no tint |
| Verification Badge | Verified | ✓ green (#22c55e) |
| Verification Badge | Unverified | ⚠ yellow (#f59e0b) |
| Message | Hover | Subtle highlight, cursor pointer |
| Message | Clicked | Drill-down modal opens |

**Accessibility:**
- Each message is a keyboard-navigable item (Tab through messages)
- Message content read by screen reader with verification status
- High contrast verification badges
- Timestamp provides temporal context for accessibility

**Composition:**
```
ChatAreaComponent
├── ScrollView (auto-scroll to newest)
│   └── [Message Items]
│       ├── Timestamp [HH:MM:SS] (secondary text)
│       ├── Sender Key (monospace, blue)
│       ├── Message Content (body text, selectable)
│       └── Verification Badge (✓ or ⚠ with color)
└── Auto-scroll Trigger (when new message arrives)
```

**Interaction:**
- Click on message → Opens drill-down modal
- Select text → Copy with Ctrl+C
- Page Down / End keys → Scroll to bottom
- Arrow Up → Scroll up to see older messages

**Implementation Complexity:** Medium (auto-scroll, click handling)

---

**3. MessageComposerComponent - Text Input & Send**

**Purpose:** Capture user message input and send. The critical gateway for the core interaction.

**Key Responsibility:**
- Single-line text input (no multiline for MVP)
- Send button (or keyboard trigger)
- Enter key sends message
- Disable send button if no text or no recipient
- Clear input after send
- Show character count (optional, Phase 2)
- Support paste from clipboard (for keys, etc.)
- Manage input focus state

**States & Variants:**
| State | Description | Visual |
|-------|-------------|--------|
| Empty | No text entered | Gray placeholder "Type message..." |
| Has Text | User typing | Text visible, send button enabled (blue) |
| No Recipient | Chat area inactive | Send button disabled (gray) |
| Focused | User is typing | Blue focus border on input |
| Sent | Message just sent | Brief flash, then clear for next |

**Accessibility:**
- Label "Message input" for screen readers
- Send button labeled "Send message" (keyboard accessible)
- Enter key works for keyboard users
- Clear focus indicator

**Composition:**
```
MessageComposerComponent
├── Text Input
│   ├── Placeholder: "Type message..."
│   ├── Single line (no multiline)
│   ├── Auto-focus when recipient selected
│   └── On Enter → Trigger send
├── Send Button
│   ├── Label: "Send"
│   ├── Enabled if: text.length > 0 AND recipient selected
│   ├── On Click → Trigger send
│   └── Keyboard: no specific shortcut (Enter from input triggers)
└── Border-top: 1px separator from chat area
```

**Interaction:**
- Type → Text appears
- Press Enter → Message sent, input cleared, focus stays in input
- Click Send → Message sent, input cleared
- Ctrl+A → Select all text in input
- Ctrl+C → Copy selected text
- Ctrl+V → Paste from clipboard

**Implementation Complexity:** Low (standard text input)

---

**4. VerificationBadgeComponent - Verified Status Indicator**

**Purpose:** Communicate cryptographic verification status visually. The symbol of trust in Profile.

**Key Responsibility:**
- Display ✓ symbol in green for verified messages
- Display ⚠ symbol in yellow for unverified messages
- Communicate trust status instantly (symbol + color)
- Be clickable to open drill-down (or support drill-down trigger on message)
- Support hover state (tooltip "Verified" or "Unverified")

**States & Variants:**
| State | Symbol | Color | Meaning |
|-------|--------|-------|---------|
| Verified | ✓ | #22c55e (green) | Message cryptographically proven from sender's key |
| Unverified | ⚠ | #f59e0b (yellow) | Signature validation failed or pending |
| Pending | ⏳ | #6b7280 (gray) | Verification in progress (rare, signing is fast) |

**Accessibility:**
- Always paired with text label "Verified" or "Unverified" (not color-only)
- Screen reader: "message verified" or "message unverified"
- High contrast: green and yellow on dark background meet WCAG AA

**Composition:**
```
VerificationBadgeComponent (inline in message)
├── Symbol (✓, ⚠, or ⏳)
├── Color (semantic, matching state)
├── Optional Tooltip on Hover: "Verified" / "Unverified"
└── Clickable: triggering drill-down detail modal
```

**Implementation Complexity:** Very Low (simple icon/text component)

---

**5. DrillDownModalComponent - Signature & Key Details**

**Purpose:** Reveal cryptographic proof layer-by-layer. Where Sam validates signatures, where Alex learns.

**Key Responsibility:**
- Display message content
- Display sender's full public key (monospace)
- Display full cryptographic signature (hex, monospace)
- Display verification status (verified ✓ or failed ⚠)
- Support copying key/signature to clipboard
- Support modal actions (close, copy)
- Make details readable and uncluttered

**States & Variants:**
| Layer | Visibility | Content |
|-------|------------|---------|
| Layer 1 (Always) | Expanded | Message content, timestamp, sender info |
| Layer 2 (Expandable) | Collapsed initially | Full signature in hex format |
| Layer 3 (Status) | Always | Verification result (verified ✓ or failed ⚠) |

**Accessibility:**
- Modal is properly focused (focus trap, not lost)
- All text is readable in monospace
- Copy buttons have accessible labels
- Escape key closes modal
- Focus returns to message after close

**Composition:**
```
DrillDownModalComponent
├── Header
│   ├── Title: "Message Details"
│   └── Close Button (X) or [Escape] key
├── Body
│   ├── Message Content
│   │   ├── Text: Full message body
│   │   └── Copy Button (Ctrl+C also works)
│   │
│   ├── Sender Information
│   │   ├── Label: "Sender Public Key"
│   │   ├── Key: Full monospace key (not abbreviated)
│   │   └── Copy Button
│   │
│   ├── Signature Details (Expandable)
│   │   ├── Label: "Cryptographic Signature"
│   │   ├── Signature: Full hex string (monospace)
│   │   └── Copy Button
│   │
│   └── Verification Status
│       ├── Badge: ✓ Verified (green) or ⚠ Failed (red)
│       └── Explanation: "This message came from [sender key]"
│
└── Footer
    └── Close Button
```

**Interaction:**
- Click on message → Modal opens
- Click Copy buttons → Text copied to clipboard
- Ctrl+C while focused on text → Copies
- Escape → Close modal
- Click X button → Close modal

**Implementation Complexity:** Medium (modal management, copy functionality)

---

**6. KeyDisplayComponent - Public Key Presentation**

**Purpose:** Show public keys prominently and accessibly. Used in onboarding, lobby, and drill-down.

**Key Responsibility:**
- Display full public key in monospace font
- Make it copyable (copy button or Ctrl+C)
- Show it untruncated (full key always visible)
- Color code for context (blue for identity, gray for signature context)
- Support different contexts (onboarding, lobby item, drill-down detail)

**States & Variants:**
| Context | Display | Color | Copyable |
|---------|---------|-------|----------|
| Onboarding (yours) | Full key | Blue (#0066CC) | Yes |
| Lobby (recipient) | Abbreviated or full | Blue | Yes |
| Message Header | Abbreviated (first 8 chars) | Blue | Yes (expands) |
| Drill-Down | Full key | Blue | Yes |

**Accessibility:**
- Monospace font signals "technical data"
- Not all content is size, readable text
- Copy button has label "Copy key"
- Ctrl+C works when focused

**Composition:**
```
KeyDisplayComponent
├── Key Text (monospace, full or abbreviated)
├── Copy Button (or right-click context menu)
└── Tooltip on Hover: "Click to copy" or "Your key" / "Recipient key"
```

**Implementation Complexity:** Low (text display + copy logic)

---

**7. StatusBadgeComponent - Online/Offline & Verification Indicators**

**Purpose:** Communicate user availability and message verification status with consistent visual language.

**Key Responsibility:**
- Display online/offline status (● green or ○ gray)
- Display verification status (✓ green or ⚠ yellow)
- Fit into lobby items, message headers, and other contexts
- Be instantly recognizable

**States & Variants:**
| Badge Type | State | Symbol | Color |
|------------|-------|--------|-------|
| Online Status | Online | ● | #22c55e (green) |
| Online Status | Offline | ○ | #6b7280 (gray) |
| Verification | Verified | ✓ | #22c55e (green) |
| Verification | Unverified | ⚠ | #f59e0b (yellow) |

**Accessibility:**
- Always paired with text ("Online", "Offline", "Verified", "Unverified")
- Not color-only (symbol + text)
- Screen reader: "user online" / "user offline" / "message verified" / "message unverified"

**Composition:**
```
StatusBadgeComponent
├── Symbol (●, ○, ✓, or ⚠)
├── Color (semantic)
└── Optional Text Label
```

**Implementation Complexity:** Very Low (simple icon component)

---

**8. KeyboardShortcutsHelpComponent - Shortcuts Overlay**

**Purpose:** Provide users quick access to keyboard shortcuts without forcing learning.

**Key Responsibility:**
- Display all available keyboard shortcuts
- Triggered by Ctrl+? or ? key
- Show shortcuts organized by category (Message, Navigation, Modal)
- Support dismissal (Escape or close button)
- Be uncluttered and scannable

**States & Variants:**
| State | Description |
|-------|-------------|
| Hidden | Not visible (default) |
| Visible | Overlay open, shortcuts displayed |
| Hovered | Individual shortcuts can be hovered |

**Accessibility:**
- Modal overlay (focus trap)
- Escape closes
- All shortcuts shown in accessible format
- Clear, readable text

**Composition:**
```
KeyboardShortcutsHelpComponent
├── Header: "Keyboard Shortcuts"
├── Close Button (X) or [Escape]
├── Shortcut Groups
│   ├── Message Actions
│   │   ├── Enter: Send message
│   │   └── Ctrl+C: Copy selection
│   ├── Navigation
│   │   ├── Tab: Move focus to next element
│   │   ├── Shift+Tab: Move focus to previous
│   │   ├── Ctrl+L: Focus on lobby
│   │   └── Escape: Close drill-down modal
│   └── Application
│       ├── Ctrl+?: Show this help
│       └── Alt+Tab: Switch to another window
└── Footer
    └── "Press Escape to close"
```

**Interaction:**
- Ctrl+? or ? → Opens shortcuts help
- Escape → Closes help
- Click X → Closes help
- Any shortcut can be clicked to execute (optional, Phase 2)

**Implementation Complexity:** Low (static modal with list)

---

**9. NotificationComponentSystem - Error & Status Messages**

**Purpose:** Communicate system status, errors, and important information without blocking workflow.

**Key Responsibility:**
- Show offline notifications (when recipient goes offline)
- Show invalid key errors (during key import)
- Show connection errors (WebSocket disconnected)
- Show success confirmations (key generated, message sent)
- Auto-dismiss success messages after 3-5 seconds
- Keep error messages until user acknowledges
- Position notifications non-intrusively

**States & Variants:**
| Type | Color | Duration | Example |
|------|-------|----------|---------|
| Success | #22c55e (green) | 3-5 seconds (auto-dismiss) | "Key generated successfully" |
| Error | #ef4444 (red) | Persistent (manual dismiss) | "Invalid key format. Please try again." |
| Warning | #f59e0b (yellow) | 5-10 seconds | "Recipient offline. Message will not send." |
| Info | #0066CC (blue) | 5-10 seconds | "Connecting to server..." |

**Accessibility:**
- All notifications have text (not color-only)
- Screen reader reads notification
- Persistent on screen long enough to read
- Clear dismiss mechanism

**Composition:**
```
NotificationComponentSystem
├── Notification Queue (stack of 1-3)
└── [Individual Notifications]
    ├── Icon (✓ success, ✗ error, ! warning, ℹ info)
    ├── Message Text
    ├── Optional Action Button ("Retry", "Dismiss")
    └── Auto-dismiss or Manual Close

Position: Bottom-right or top-right (standard desktop pattern)
```

**Interaction:**
- Appears automatically based on system event
- Auto-dismisses (success) or waits for user (error)
- Click X or Dismiss button to close
- Multiple notifications can stack

**Implementation Complexity:** Medium (notification queue, auto-dismiss logic)

---

### Component Dependencies & Data Flow

```
Profile Application Structure:

┌─────────────────────────────────────────────────────────┐
│                  Application State                       │
│  - Current User Key & Public Key                         │
│  - Selected Recipient (online user)                      │
│  - Message History (for selected conversation)           │
│  - Online Users List                                     │
│  - System Notifications                                  │
└─────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┼──────────────────┐
        ▼                  ▼                   ▼
   ┌─────────────┐  ┌────────────────┐  ┌──────────────┐
   │   LOBBY     │  │    CHAT AREA   │  │  MODAL/UI    │
   │  Component  │  │   Component    │  │  Components  │
   └─────────────┘  └────────────────┘  └──────────────┘
   - Users List     - Messages List      - DrillDown
   - Selection      - Auto-scroll        - Help
   - Status Badge   - Composer           - Notifications
                    - Verification Badge

Data Dependencies:
- Lobby ← Users List from Server
- Chat ← Message History from Server
- Composer → Sends message to Server
- DrillDown ← Gets message details from Chat
- Notifications ← System events (error, success)
```

### Implementation Roadmap: Phased Delivery

**MVP (Phase 1) - Minimum Viable Product**

Goal: Launch with core message-send experience working end-to-end.

Priority 1 (Week 1-2):
- ✅ MessageComposerComponent (core interaction)
- ✅ ChatAreaComponent (displays messages)
- ✅ LobbyComponent (recipient selection)
- ✅ VerificationBadgeComponent (trust signal)
- ✅ KeyDisplayComponent (identity visibility)
- ✅ NotificationComponentSystem (errors, feedback)

Priority 2 (Week 2-3):
- ✅ DrillDownModalComponent (cryptographic depth)
- ✅ StatusBadgeComponent (online/offline, verified/unverified)
- ✅ KeyboardShortcutsHelpComponent (power user support)

**Phase 1 Completeness:**
- Users can generate or import keys
- Users can select recipients from online lobby
- Users can send messages
- Messages appear with verification status
- Users can drill down to see signatures
- All keyboard shortcuts work

**Phase 2 (Growth) - User Experience Polish**

Priority 3:
- Message persistence (store message history locally)
- Typing indicators ("User is typing...")
- Read receipts ("User read your message")
- User profiles / About section
- Message search (Phase 2.5)

**Phase 3 (Expansion) - Feature Addition**

Priority 4:
- Group messaging (create rooms, add multiple recipients)
- Reactions to messages (emoji responses)
- File sharing / Link previews
- ZK proof integration (advanced verification)

---

### Component Design Specifications

#### Specification: MessageComposerComponent

**Specification ID:** COMP-MSG-COMPOSER-001

**Version:** 1.0

**Definition:** Text input field that accepts user message and triggers send action.

**Primary Flow:**
```
User Types → Input captures text → Enter/Click Send → System validates:
  - Text not empty
  - Recipient selected
  - If valid: Send message → Clear input → Focus input
  - If invalid: Show error notification
```

**Component States:**

| State | Conditions | Visual | Behavior |
|-------|-----------|--------|----------|
| Empty | `text.length == 0` | Placeholder "Type message..." visible, send button disabled (gray) | Accept input, prevent send |
| Has Text | `text.length > 0 && recipient != null` | Input text visible, send button enabled (blue) | Send on Enter or click |
| No Recipient | `recipient == null` | Input grayed out, send button disabled | Prevent send, show tooltip "Select a recipient" |
| Focused | Input has focus | Blue border around input, cursor visible | Accept keyboard input |
| Sent | After send action | Input clears, send button disables, focus remains | Ready for next message |

**Properties:**
```
- text: String (current input text)
- maxLength: 5000 characters
- recipient: User | null
- isSending: Boolean (true while sending)
- onSend(message: String): void
- onTextChanged(text: String): void
```

**Keyboard Interactions:**
```
Enter: Send message (if valid)
Ctrl+A: Select all text
Ctrl+C: Copy selected text
Ctrl+V: Paste from clipboard
Ctrl+X: Cut selected text
Backspace: Delete character before cursor
Delete: Delete character after cursor
Home: Move cursor to start
End: Move cursor to end
```

**Accessibility:**
```
- Role: "text input"
- Label: "Message composition field"
- Placeholder: "Type message..."
- Required: true if recipient selected
- Aria-expanded: false (no dropdown)
- Focus indicator: Blue border (2px)
```

**Error Handling:**
```
- Empty text on send: Silently prevent send (button disabled)
- No recipient selected: Show tooltip "Select a recipient first"
- Message too long: Truncate at 5000 chars, show warning "Message too long"
- Send fails (offline): Show error notification "Failed to send. Try again later."
```

---

#### Specification: ChatAreaComponent

**Specification ID:** COMP-CHAT-AREA-001

**Version:** 1.0

**Definition:** Scrollable message list displaying messages in chronological order with verification status, sender keys, and timestamps.

**Primary Flow:**
```
Server sends message → Chat displays message with:
  - Timestamp
  - Sender key (monospace)
  - Message content
  - Verification badge
Auto-scroll to newest message
```

**Component States:**

| State | Conditions | Visual | Behavior |
|-------|-----------|--------|----------|
| Empty | No messages | "No messages yet. Send the first one!" | Awaiting input |
| Populated | Messages exist | Messages displayed chronologically | Display and scroll |
| New Message | Message arrives | Auto-scroll to newest | Smooth scroll animation |
| Hover | Mouse over message | Slight highlight (#374151 bg) | Invite click for drill-down |
| Selected (Own) | Message from user | Subtle background tint | Visual differentiation |

**Message Item Format:**
```
[HH:MM:SS] [SENDER_KEY] [MESSAGE_TEXT] [BADGE]

Example:
[12:34:56] 3A8F2E1C Hello, is anyone here? ✓
```

**Properties:**
```
- messages: Message[] (array of message objects)
- selectedRecipient: User | null
- currentUserKey: String
- onMessageClick(message: Message): void
- onScroll(position: Number): void
```

**Message Object:**
```
{
  id: String (unique identifier)
  timestamp: DateTime
  senderKey: String (public key, monospace)
  messageContent: String
  signature: String (hex format)
  isVerified: Boolean
  isOwn: Boolean (true if current user sent)
}
```

**Scroll Behavior:**
```
- Initial Load: Scroll to bottom (show newest messages)
- New Message Arrives: Auto-scroll to bottom with smooth animation
- User Scrolls Up: Stop auto-scrolling (let user review history)
- User Scrolls to Bottom: Resume auto-scrolling for new messages
- Page Down / End Key: Jump to bottom
- Page Up / Home Key: Jump to top
```

**Interaction:**
```
- Click on message → Open drill-down modal
- Right-click on message → Context menu: Copy content, Copy key, Copy signature
- Ctrl+A → Select all messages (optional, Phase 2)
- Ctrl+C → Copy selected message content
- Mouse wheel or arrow keys → Scroll
```

**Accessibility:**
```
- Role: "feed" (message feed)
- Each message: Role "article", name "[timestamp] [sender] message content"
- Keyboard navigation: Tab through messages
- Screen reader: Reads message content, sender key, verification status
- Focus indicator: Blue border on focused message
```

**Error Handling:**
```
- Message display fails: Show placeholder "[Message failed to load]"
- Signature verification fails: Show ⚠ badge instead of ✓
- Scroll performance issue: Virtualize list for 1000+ messages (Phase 2)
```

---

### Component Integration Checklist

Before implementation of each component, verify:

- [ ] **State Management**: How is component state managed? (Local, Redux, Context?)
- [ ] **Data Binding**: How does data flow to/from component?
- [ ] **Keyboard Navigation**: Tab order, Enter, Escape behaviors defined?
- [ ] **Accessibility**: WCAG AA compliance, screen reader tested?
- [ ] **Error States**: All failure scenarios handled gracefully?
- [ ] **Performance**: Component renders efficiently even with large data (100+ messages)?
- [ ] **Testing**: Unit tests, integration tests, visual regression tests planned?
- [ ] **Documentation**: Component usage documented for developers?

---

### Slint Integration Notes

**Component Framework Choice: Slint** (as established in Design System)

**Slint Component Patterns for Profile:**

```slint
// Define shared colors, spacing, fonts
export struct Colors {
  primary-blue: color;
  verified-green: color;
  error-red: color;
  surface-light: color;
  text-primary: color;
  text-secondary: color;
}

// Define component template
export component MessageBubble inherits Rectangle {
  in property <{
    timestamp: string,
    senderKey: string,
    content: string,
    isVerified: bool,
    isOwn: bool,
  }> data;

  callback clicked <-> message-clicked;

  // Layout and styling here
}
```

**Key Slint Advantages for Profile:**
- Direct Rust integration (cryptographic operations embedded)
- Native Windows UI (taskbar, clipboard, keyboard shortcuts)
- Compiled performance (deterministic rendering matches deterministic signing)
- Minimal abstraction layers (reduces bugs)

---

### Success Metrics for Component Strategy

**Usability:**
- Users can send first message within 2 minutes (Alex & Sam)
- Drill-down can be accessed within 1 click
- All keyboard shortcuts discoverable and intuitive
- No crashes or hangs during normal use

**Technical:**
- Component reusability: 80%+ code shared across similar components
- Testing coverage: 90%+ of component logic covered
- Performance: Sub-200ms render time for message lists with 500+ messages
- Accessibility: WCAG AA compliance achieved

**User Experience:**
- Verification badge always visible and meaningful
- Message send feels instant (signing < 100ms)
- Drill-down reveals information clearly without overwhelming
- Keyboard-first users can work at typing speed

---

## UX Patterns & Consistency Framework

### Step 12: Interaction Patterns, Visual Consistency, and Design Language

A strong UX pattern library ensures Profile feels cohesive and predictable. When users learn one interaction pattern, they can predict how similar patterns will work elsewhere in the application.

### Pattern Categories & Definitions

#### 1. Selection Patterns

**Pattern: Single Selection with Highlight**

Used in: Lobby (selecting recipient), Message list (selecting message for drill-down)

**Interaction:**
```
User Action: Click on item
Visual Feedback: Item highlighted with blue background, text bold
Focus: Focus border visible around selected item
Navigation: Arrow keys move selection up/down
Confirm: Enter key or click confirms selection
Clear: Escape key or click elsewhere deselects
```

**Consistency Rules:**
- All single-selection contexts use same visual highlight (blue #0066CC)
- Tab order follows visual order (top to bottom)
- Arrow keys work for navigation in all lists
- Enter confirms selection consistently

**Exception:** None for MVP

---

#### 2. Disclosure / Expansion Patterns

**Pattern: Click to Expand / Drill-Down Modal**

Used in: Message details (click message → drill-down modal)

**Interaction:**
```
User Action: Click on message
Visual Feedback: Modal appears with smooth fade-in
Content Revealed: Layered information (message → key → signature → status)
Navigation: Tab through expandable sections
Close: Escape key, click X button, or click outside modal
```

**Consistency Rules:**
- All expandable content uses drill-down modal pattern
- Modal always centered on screen
- Escape always closes (no exceptions)
- Content scrolls if longer than viewport
- Focus trapped in modal while open

**Accessibility:** Modal dialog ARIA attributes applied

---

#### 3. Input Patterns

**Pattern: Text Input with Validation**

Used in: Message composer, Key import field

**Interaction:**
```
User Action: Click in field, type text
Visual Feedback: Blue focus border, text appears
Validation: Real-time validation (if applicable)
Error: Error message shown below field (red text)
Submit: Enter key or button click
Clear: Ctrl+A → Delete or Backspace
```

**Consistency Rules:**
- All text inputs have same focus border (blue, 2px)
- Placeholder text same color (#9ca3af) across all inputs
- Validation messages same position and color (red, below field)
- Enter key submits in all single-line inputs
- Ctrl+A selects all text in input

**Error State Example:**
```
Input: [Invalid key paste]
↓
Error Message (Red): "That doesn't look like a 256-bit key. Check format and try again."
↓
User Corrects: [Valid key paste]
↓
Error Clears: Message disappears
↓
Success Indication: "Private key loaded. Public key: [derived_key]"
```

---

#### 4. Confirmation & Feedback Patterns

**Pattern: Immediate Feedback with Progress**

Used in: Message send, Key generation, Signature verification

**Interaction:**
```
User Action: Trigger action (press Enter to send)
Visual Feedback (Immediate): Action acknowledges (button disables briefly)
Processing: Invisible operation (signing < 100ms)
Result: Success feedback appears (message in chat with ✓ badge)
Or: Error feedback appears (red notification message)
```

**Consistency Rules:**
- All success has green color (#22c55e) + ✓ symbol
- All errors have red color (#ef4444) + ✗ or ⚠ symbol
- All warnings have yellow color (#f59e0b) + ⚠ symbol
- Feedback appears in consistent location (notifications in bottom-right or inline)
- Success auto-dismisses after 3-5 seconds
- Errors persist until user acts

**Example Feedback States:**
```
Success: "✓ Message sent" (green, auto-dismiss 4s)
Error: "✗ Failed to send. Connection lost." (red, manual dismiss)
Warning: "⚠ Recipient offline. Message queued." (yellow, auto-dismiss 6s)
Info: "ℹ Connecting to server..." (blue, auto-dismiss when complete)
```

---

#### 5. Keyboard Navigation Patterns

**Pattern: Consistent Tab Order & Shortcut Mapping**

Used in: Entire application

**Primary Tab Order:**
```
1. Lobby (user list)
2. Chat Area (message list)
3. Message Composer (text input)
4. Send Button (optional, Enter key primary)

Within Lobby:
- Arrow Up/Down: Navigate users
- Enter: Select user

Within Chat:
- Arrow Up/Down: Navigate messages
- Enter: Open drill-down of selected message

Within Composer:
- Tab: Move to send button (or next UI)
- Shift+Tab: Return to chat
- Enter: Send message

Global Shortcuts:
- Ctrl+L: Focus lobby
- Ctrl+?: Show keyboard help
- Escape: Close modal or deselect
- Ctrl+C: Copy (context-dependent)
- Ctrl+V: Paste (in input fields)
```

**Consistency Rules:**
- Tab always moves forward; Shift+Tab always moves backward
- Arrow keys always move within lists (never page/scroll globally)
- Enter always confirms selection or submits
- Escape always closes or cancels
- Ctrl+ shortcuts use standard conventions (L=Lobby, C=Copy, V=Paste)

**Focus Indicator Design:**
- Always a 2px blue border (#0066CC) around focused element
- High contrast for visibility on dark background
- Visible on all keyboard-navigable elements

---

#### 6. Icon & Badge Patterns

**Pattern: Consistent Use of Symbols**

Used in: Status indicators, verification badges, online/offline status

**Symbol Meanings (Do Not Vary):**
```
✓ (Checkmark): Success, Verified, Confirmed
✗ or ⚠: Error, Warning, Unverified
● (Filled Circle): Online, Active, Connected
○ (Empty Circle): Offline, Inactive, Disconnected
⏳ (Hourglass): Pending, Processing, Loading
ℹ: Information, Details Available
→: Next, Forward, Open
← : Back, Previous
[]: Copy, Clipboard
...or … : More options, Loading
```

**Consistency Rules:**
- Each symbol has single meaning across entire app
- Symbols always paired with color (✓ = green, ✗ = red, etc.)
- Symbols always paired with text label (for accessibility)
- Never use symbols alone to convey meaning
- Monospace font used for cryptographic data symbols (keys, signatures)

---

#### 7. Color Usage Patterns

**Pattern: Semantic Color Mapping**

**Color Never Used for:**
- Information alone (always paired with symbol or text)
- Non-semantic decoration (no "just because it looks nice" colors)
- Hover states in message list (use subtle background tint #374151 instead)

**Semantic Color Rules:**
```
🟦 Blue (#0066CC): Identity, Trust, Primary Actions, Focus
- Sender key text
- Focus border on inputs
- Primary button states
- "You" message indicator

🟩 Green (#22c55e): Success, Verified, Online, Positive
- ✓ Verification badge
- ● Online indicator
- Success notification
- Valid state

🟥 Red (#ef4444): Error, Invalid, Offline, Critical
- ✗ Error indicators
- ○ Offline indicator
- Failed signature
- Critical alerts

🟨 Yellow (#f59e0b): Warning, Caution, Attention
- ⚠ Unverified message
- Warning notifications
- Caution prompts

⬜ Gray (#6b7280, #9ca3af, #d1d5db): Neutral, Secondary, Disabled
- Placeholder text
- Secondary information
- Disabled buttons
- Inactive states
```

**Color Application Examples:**
```
Message from you: 
  - Text: Blue sender key → Indicates it's your identity
  - Badge: Green ✓ → Verified

Message from other:
  - Text: Blue sender key → Shows whose identity sent it
  - Badge: Green ✓ → Verified

Offline recipient:
  - Indicator: Gray ○ → Offline
  - Message: Yellow ⚠ → Warning state

Failed signature:
  - Badge: Red ✗ → Error
  - Notification: Red text → "Signature verification failed"
```

---

#### 8. Error Recovery Patterns

**Pattern: Clear, Actionable Error Messages**

Used in: All failure scenarios

**Error Message Structure:**
```
[Error Type Indicator] + [What Happened] + [Why It Happened] + [What to Do]

Example 1: Invalid Key Import
"✗ That doesn't look like a 256-bit key. Please check the format and try again."

Example 2: Failed Message Send
"✗ Failed to send message. Connection lost to server. [Retry] button available."

Example 3: Signature Verification Failed
"✗ Message signature verification failed. This message may be corrupted or from an untrusted source."
```

**Consistency Rules:**
- All errors start with symbol (✗ or ⚠)
- All errors use red color (#ef4444)
- All errors explain what went wrong (not just "Error")
- All errors suggest next action (retry, check, contact support)
- All errors are specific (not generic "Error occurred")
- All errors persist on screen until user acknowledges

---

### Visual Consistency Checklist

**Color Consistency:**
- [ ] All green used only for success/verified/online (never other meaning)
- [ ] All red used only for error/invalid/offline (never other meaning)
- [ ] All blue used only for identity/trust/focus (never other meaning)
- [ ] No colors used without paired text/symbol
- [ ] Dark mode palette applied throughout (no light mode)

**Typography Consistency:**
- [ ] Body text always 14px Segoe UI (unless specifically 16px header or 12px caption)
- [ ] Monospace (Consolas) used only for: keys, signatures, code
- [ ] Headers always bold, secondary always semibold, body always regular
- [ ] Line heights consistent: headers 1.2, body 1.5, monospace 1.6

**Spacing Consistency:**
- [ ] All spacing multiples of 8px (4px, 8px, 12px, 16px, 24px, 32px)
- [ ] Component padding always 12px or 16px (not arbitrary)
- [ ] Message gaps always 4px or 8px (tight but readable)
- [ ] Sidebar width always 250px (or user-adjustable range 220-280px)

**Interaction Consistency:**
- [ ] Tab order logical and predictable everywhere
- [ ] Escape always closes (no exceptions)
- [ ] Enter always confirms (in forms and lists)
- [ ] Ctrl+C always copies (where applicable)
- [ ] Focus border always blue, always 2px, always visible

**Accessibility Consistency:**
- [ ] All interactive elements keyboard accessible
- [ ] All focus states visible and high-contrast
- [ ] All colors paired with symbols/text (colorblind-friendly)
- [ ] All modals properly focused (trap focus)
- [ ] All notifications readable for 5+ seconds minimum

---

### Pattern Documentation for Developers

**When implementing, developers should:**

1. **Reference this pattern guide** before creating new UI
2. **Check for existing patterns** that match their use case
3. **Follow the pattern exactly** (don't create variations)
4. **Document any new patterns** if existing patterns don't fit
5. **Test patterns with keyboard navigation** (ensure consistency)
6. **Test with screen reader** (ensure accessibility)

**Example Developer Guidance:**
```
"I'm building a new notification component for Phase 2."

Step 1: Check UX Patterns guide
→ Found: "Confirmation & Feedback Patterns"

Step 2: Apply pattern
- Use green (#22c55e) for success, red (#ef4444) for error
- Pair color with symbol (✓ or ✗)
- Auto-dismiss after 3-5 seconds (success) or manual dismiss (error)
- Place in bottom-right corner (consistent with existing notifications)

Step 3: Test
- Tab through notification (should be accessible)
- Test with screen reader (should read "notification type, message")
- Test auto-dismiss timing (should feel right)

Step 4: Verify
- Matches existing notifications visually
- Follows same interaction pattern
- No new variations introduced
```

---

### Pattern Exceptions & Variations

**No variations allowed for MVP**. Every component must follow the patterns exactly.

**Phase 2+ Exceptions** (only if explicitly designed):
- Alternative notification position (top center instead of bottom-right)
- Additional keyboard shortcuts (Ctrl+N for "new conversation")
- Dark mode toggle (Light mode as Phase 2+ feature)

**Principles:**
- Every exception must be justified (user research-backed)
- Every exception must be documented
- Every exception must apply globally (not per-component)
- Exceptions should be rare; consistency should be the default

---

### Consistency Across User Journeys

**Alex's Journey - Pattern Expectations:**
1. Key Generation: Simple input → Success feedback → Proceed
2. Message Send: Type → Press Enter → Message appears → See badge
3. Drill-Down: Click message → Modal opens → See details → Escape to close

**Sam's Journey - Pattern Expectations:**
1. Key Import: Paste → Validation feedback → Proceed
2. Message Send: Type → Press Enter → Message appears → See badge
3. Drill-Down: Click message → Modal opens → Inspect signature → Escape to close
4. Signature Testing: Send identical messages → Compare signatures → Validate determinism

**Both journeys use the same patterns**, ensuring consistency and predictability.

---

### Measuring Pattern Consistency

**During User Testing:**
- [ ] Users predict correctly how UI will behave
- [ ] Users can discover patterns without instruction
- [ ] Users feel confident using unfamiliar parts based on known patterns
- [ ] No "surprise" UI interactions

**During Development:**
- [ ] All components follow color scheme exactly
- [ ] All inputs have same focus state
- [ ] All modals follow same close behavior
- [ ] All errors follow same message structure
- [ ] All success feedback follows same visual treatment

**During Code Review:**
- Pattern violations caught early
- Consistency documented in PR
- Deviations justified or corrected
- Testing verifies pattern compliance
