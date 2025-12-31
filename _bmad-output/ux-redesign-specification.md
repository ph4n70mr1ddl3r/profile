---
# UX Redesign Specification: Profile Secure Messaging

**Author:** UX Design Specialist  
**Date:** 2025-12-31  
**Version:** 1.0  
**Status:** Draft - Ready for Implementation

---

## Executive Summary

This document provides a comprehensive redesign specification for the Profile Secure Messaging application. Based on the extensive UI/UX review conducted on 2025-12-31, this specification addresses critical issues (hardcoded slots, fixed window size) and important improvements (accessibility, responsiveness, user flows).

### Key Findings from Review

| Category | Current Score | Target Score |
|----------|---------------|--------------|
| Visual Design | 9/10 | 10/10 |
| Component Quality | 8/10 | 9/10 |
| User Flow | 7/10 | 9/10 |
| Accessibility | 6/10 | 9/10 |
| Responsiveness | 4/10 | 9/10 |
| **OVERALL** | **7/10** | **9/10** |

### Scope of Redesign

**In Scope:**
- Dynamic message list (remove 10-slot limit)
- Dynamic lobby list (remove 10-user limit)
- Window resize handling
- Keyboard navigation improvements
- Empty states
- Connection status indicator
- Keyboard shortcuts help modal
- User menu (disconnect/change key)

**Out of Scope:**
- Theme system (dark only for MVP)
- Mobile/responsive layouts (Phase 2)
- Animation polish (Phase 2)

---

## 1. Architecture Changes

### 1.1 Data Model Updates

#### Current Problem
```slint
// main.slint - Hardcoded slots (PROBLEMATIC)
in property <string> chat_msg_1_sender_key: "";
in property <string> chat_msg_2_sender_key: "";
// ... 10 more similar properties
```

#### Proposed Solution
```slint
// New data structures in Rust (shared types)
pub struct MessageData {
    pub sender_key: String,
    pub sender_key_short: String,
    pub content: String,
    pub timestamp: String,
    pub signature: String,
    pub is_self: bool,
    pub is_verified: bool,
}

pub struct LobbyUserData {
    pub public_key: String,
    pub is_online: bool,
    pub is_selected: bool,
}

// Slint component using arrays
export component AppWindow {
    in property <[MessageData]> chat_messages: [];
    in property <[LobbyUserData]> lobby_users: [];
}
```

### 1.2 Slint 1.5 Dynamic Lists

**Note:** Slint 1.5 has limited `for-each` support. Two approaches:

#### Approach A: Use `for` with Fixed Container (Recommended)
```slint
// Dynamic message list with for-each
VerticalLayout {
    spacing: 8px;
    
    for message[index] in root.chat_messages: {
        MessageItem {
            sender_key: message.sender_key;
            sender_key_short: message.sender_key_short;
            message_content: message.content;
            timestamp: message.timestamp;
            is_self: message.is_self;
            is_verified: message.is_verified;
            
            clicked => {
                root.on_message_clicked(index);
            }
        }
    }
}
```

#### Approach B: Container with Overflow (Fallback)
If `for` is unstable, use scrollable container:
```slint
ScrollView {
    width: 100%;
    height: 100%;
    
    VerticalLayout {
        spacing: 8px;
        
        // Same MessageItem components, dynamically populated
    }
}
```

---

## 2. Component Redesign Specifications

### 2.1 AppWindow (Main Layout)

#### Current State (Issues)
- Fixed 800x600px size
- 10 hardcoded message slots
- 10 hardcoded lobby slots
- No window controls

#### Redesigned Specification

```slint
export component AppWindow inherits Window {
    title: "Profile - Cryptographic Messaging";
    min-width: 640px;
    min-height: 480px;
    width: 1024px;  // New default
    height: 768px;  // New default
    
    // Resize corner indicator
    resize-corner-size: 12px;
    
    // Application state
    in property <bool> connected: false;
    in property <string> connection_status: "Disconnected";
    in property <string> status_message: "";
    in property <bool> status_is_error: false;
    
    // View state
    in-out property <string> current_view: "welcome";
    
    // Dynamic data (replaces 10 hardcoded slots)
    in property <[MessageData]> chat_messages: [];
    in property <[LobbyUserData]> lobby_users: [];
    
    // Selection state
    in property <int> selected_lobby_index: -1;
    in property <int> selected_message_index: -1;
    
    // Callback definitions
    callback generate_key_pressed;
    callback import_key_pressed;
    callback lobby_user_clicked(int);
    callback message_clicked(int);
    callback send_message(string);
    callback close_requested;
    callback disconnect_pressed;
    callback change_key_pressed;
}
```

### 2.2 New Component: ConnectionStatus

**Purpose:** Display connection state and allow disconnect/change key

```slint
export component ConnectionStatus {
    in property <bool> connected: false;
    in property <string> status_text: "Disconnected";
    in property <string> public_key_short: "";
    
    callback disconnect_pressed;
    callback change_key_pressed;
    callback click;
    
    Rectangle {
        background: connected ? #22c55e : #6b7280;
        height: 32px;
        
        HorizontalLayout {
            padding: 8px;
            spacing: 12px;
            
            // Status indicator
            Rectangle {
                width: 8px;
                height: 8px;
                border-radius: 4px;
                background: connected ? #ffffff : #cccccc;
            }
            
            // Status text
            Text {
                text: connected ? "Connected" : "Offline";
                color: #ffffff;
                font-size: 12px;
                vertical-alignment: center;
            }
            
            // Public key (abbreviated)
            Text {
                text: connected ? "• " + public_key_short : "";
                color: rgba(255,255,255,0.7);
                font-size: 11px;
                font-family: "monospace";
                vertical-alignment: center;
            }
            
            Rectangle { horizontal-stretch: 1; }
            
            // User menu button (three dots)
            if (connected) {
                menu_button := FocusScope {
                    width: 24px;
                    height: 24px;
                    
                    Text {
                        text: "⋮";
                        color: #ffffff;
                        font-size: 16px;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                    
                    TouchArea {
                        width: parent.width;
                        height: parent.height;
                        clicked => {
                            // Show dropdown menu
                        }
                    }
                }
            }
        }
        
        TouchArea {
            width: parent.width;
            height: parent.height;
            clicked => { root.click(); }
        }
    }
}
```

### 2.3 New Component: KeyboardShortcutsHelp

**Purpose:** Modal displaying all keyboard shortcuts

```slint
export component KeyboardShortcutsHelp {
    in property <bool> is_visible: false;
    
    callback close_requested;
    
    Rectangle {
        visible: root.is_visible;
        
        // Backdrop
        Rectangle {
            width: parent.width;
            height: parent.height;
            background: #000000;
            opacity: 0.5;
            
            TouchArea {
                clicked => { root.close_requested(); }
            }
        }
        
        // Modal
        Rectangle {
            x: (parent.width - 400px) / 2;
            y: (parent.height - 360px) / 2;
            width: 400px;
            height: 360px;
            background: #1e1e2e;
            border-radius: 12px;
            
            VerticalLayout {
                padding: 20px;
                spacing: 16px;
                
                // Header
                HorizontalLayout {
                    Text {
                        text: "Keyboard Shortcuts";
                        font-size: 18px;
                        font-weight: 700;
                        color: #ffffff;
                    }
                    
                    Rectangle { horizontal-stretch: 1; }
                    
                    // Close button
                    close_btn := FocusScope {
                        width: 24px;
                        height: 24px;
                        
                        Text {
                            text: "✕";
                            color: #ef4444;
                            font-size: 16px;
                            horizontal-alignment: center;
                            vertical-alignment: center;
                        }
                        
                        TouchArea {
                            clicked => { root.close_requested(); }
                        }
                    }
                }
                
                // Shortcuts grid
                HorizontalLayout {
                    spacing: 24px;
                    
                    // Left column
                    VerticalLayout {
                        spacing: 8px;
                        
                        shortcut_item("Enter", "Send message");
                        shortcut_item("Escape", "Close modal");
                        shortcut_item("Ctrl+C", "Copy selected");
                    }
                    
                    // Right column
                    VerticalLayout {
                        spacing: 8px;
                        
                        shortcut_item("↑/↓", "Navigate lobby");
                        shortcut_item("Tab", "Next element");
                        shortcut_item("?", "Show this help");
                    }
                }
                
                // Footer
                Text {
                    text: "Press ? anywhere to show this help";
                    font-size: 12px;
                    color: #888888;
                    horizontal-alignment: center;
                }
            }
        }
    }
    
    // Helper for shortcut items
    shortcut_item(key_desc: string, action: string) {
        HorizontalLayout {
            spacing: 8px;
            
            Rectangle {
                width: 80px;
                background: #333333;
                border-radius: 4px;
                height: 24px;
                
                Text {
                    text: key_desc;
                    color: #ffffff;
                    font-size: 11px;
                    horizontal-alignment: center;
                    vertical-alignment: center;
                }
            }
            
            Text {
                text: action;
                color: #cccccc;
                font-size: 12px;
                vertical-alignment: center;
            }
        }
    }
}
```

### 2.4 Updated: MessageComposer

**Changes:** Add placeholder, improve states

```slint
export component MessageComposer {
    in property <string> recipient: "";
    in property <bool> recipient_selected: true;
    in property <bool> focused: false;
    in-out property <string> message_text: "";
    callback send_message(string);
    callback enter_pressed();
    
    property <bool> internal_can_send: 
        root.message_text != "" && root.recipient_selected;
    
    VerticalLayout {
        spacing: 8px;
        
        // Recipient indicator or warning
        if (root.recipient != "" && root.recipient_selected) {
            Text {
                text: "Messaging: " + recipient;
                font-size: 12px;
                color: #6b7280;
            }
        } else if (!root.recipient_selected) {
            Text {
                text: "Select a user to start messaging";
                font-size: 12px;
                color: #ef4444;
            }
        }
        
        // Composer box
        Rectangle {
            background: #1f2937;
            border-radius: 8px;
            border-width: 2px;
            border-color: root.focused ? #0066CC : 
                         (!root.recipient_selected ? #ef4444 : #374151);
            height: 80px;
            
            HorizontalLayout {
                padding: 12px;
                spacing: 8px;
                
                // Text input with placeholder
                TextInput {
                    text: message_text;
                    placeholder-text: root.recipient_selected 
                        ? "Type a message..." 
                        : "Select a user first";
                    enabled: root.recipient_selected;
                    horizontal-alignment: left;
                    vertical-alignment: center;
                    color: #ffffff;
                }
                
                // Send button
                send_btn := Rectangle {
                    background: root.internal_can_send ? #0066CC : #6b7280;
                    border-radius: 4px;
                    height: 30px;
                    min-width: 80px;
                    
                    Text {
                        text: "Send";
                        color: #ffffff;
                        horizontal-alignment: center;
                        vertical-alignment: center;
                    }
                    
                    TouchArea {
                        enabled: root.internal_can_send;
                        clicked => {
                            if (root.internal_can_send) {
                                root.send_message(root.message_text);
                            }
                        }
                    }
                }
            }
        }
        
        // Character count (optional, for long messages)
        if (root.message_text.length > 280) {
            Text {
                text: root.message_text.length + " characters";
                font-size: 10px;
                color: #f59e0b;
                horizontal-alignment: right;
            }
        }
    }
}
```

### 2.5 Updated: LobbyItem with Keyboard Navigation

**Changes:** Add keyboard navigation support

```slint
export component LobbyItem {
    in property <string> public_key;
    in property <bool> is_online: true;
    in property <bool> is_selected: false;
    in property <bool> has_focus: false;  // NEW
    
    callback clicked;
    callback focus_received;  // NEW
    
    forward-focus: touch_scope;
    
    Rectangle {
        background: root.is_selected ? #0066CC : #111827;
        height: 36px;
        border-width: root.has_focus ? 2px : 0px;  // NEW
        border-color: #0088FF;  // NEW
        
        // Online indicator
        Rectangle {
            x: 8px;
            y: 14px;
            width: 8px;
            height: 8px;
            border-radius: 4px;
            background: root.is_online ? #22c55e : #6b7280;
        }
        
        // Public key
        Text {
            x: 24px;
            y: 10px;
            text: public_key;
            font-family: "Consolas, Monaco, monospace";
            font-size: 12px;
            color: root.is_selected ? #ffffff : #0066CC;
        }
        
        // TouchArea
        touch_scope := TouchArea {
            width: parent.width;
            height: parent.height;
            
            clicked => { root.clicked(); }
            
            // Keyboard navigation (NEW)
            key-pressed(event) => {
                if (event.text == "\n" || event.text == " ") {
                    root.clicked();
                    return accept;
                }
                return reject;
            }
        }
    }
}
```

### 2.6 New Component: EmptyLobbyState

**Purpose:** Show when no users are in lobby

```slint
export component EmptyLobbyState {
    Rectangle {
        width: 100%;
        height: 100%;
        background: #111827;
        
        VerticalLayout {
            padding: 32px;
            spacing: 16px;
            alignment: center;
            
            Text {
                text: "👋";
                font-size: 48px;
                horizontal-alignment: center;
            }
            
            Text {
                text: "No users online";
                font-size: 16px;
                color: #ffffff;
                horizontal-alignment: center;
            }
            
            Text {
                text: "Share your public key to start chatting";
                font-size: 12px;
                color: #888888;
                horizontal-alignment: center;
            }
        }
    }
}
```

---

## 3. User Flow Improvements

### 3.1 Updated User Flow Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                     PROFILE USER FLOW                           │
└─────────────────────────────────────────────────────────────────┘

    ┌──────────────┐
    │  WELCOME     │
    │   SCREEN     │
    └──────┬───────┘
           │
           ├── Generate Key ───► Key Display ──► Lobby (→ Chat)
           │
           └── Import Key ─────► Key Display ──► Lobby (→ Chat)
                                   │
                                   │ (user clicks user menu)
                                   ▼
                           ┌──────────────┐
                           │   LOBBY      │──── Empty State ───►
                           │              │      (no users)
                           └──────┬───────┘
                                  │
                          User Selects User
                                  │
                                  ▼
                           ┌──────────────┐
                           │    CHAT      │──── Message Drill-Down
                           │              │
                           └──────┬───────┘
                                  │
                          User Clicks Menu
                                  │
                                  ▼
                           ┌──────────────┐
                           │  KEYBOARD    │──── ? Shortcuts Help
                           │  SHORTCUTS   │
                           └──────────────┘
```

### 3.2 New States Added

| State | Component | Trigger |
|-------|-----------|---------|
| Empty Lobby | EmptyLobbyState | lobby_users.length == 0 |
| No Recipient Selected | MessageComposer | selected_lobby_index == -1 |
| Disconnected | ConnectionStatus | connected == false |
| Help Modal | KeyboardShortcutsHelp | User presses ? |

---

## 4. Accessibility Improvements

### 4.1 Keyboard Navigation Matrix

| Key | Action | Context |
|-----|--------|---------|
| Enter | Select/Activate | Lobby item, message, button |
| Escape | Close modal | Drill-down, help modal |
| ↑/↓ | Navigate lobby | Lobby view |
| Tab | Next element | Global |
| Shift+Tab | Previous element | Global |
| Ctrl+C | Copy | Key display, message, signature |
| ? | Show help | Global |

### 4.2 Focus Order Specification

```
Welcome Screen:
  [Generate Key] → [Import Key] → [Status Message]

Lobby:
  [Connection Status] → [User 1] → [User 2] → ... → [User N]

Chat:
  [Back to Lobby] → [Message 1] → ... → [Message N] → [Composer]

Drill-Down Modal:
  [Close X] → [Copy Key] → [Copy Message] → [Copy Signature]

Help Modal:
  [Close X] → [Shortcuts List]
```

### 4.3 Screen Reader Labels

All interactive elements should have:
- `accessible-label`: Short description
- `accessible-description`: Detailed description
- `accessible-value`: Current value (if applicable)

---

## 5. Responsive Design

### 5.1 Window Size Recommendations

| Size | Layout | Notes |
|------|--------|-------|
| 640x480 | Minimum | Core functionality works |
| 800x600 | Recommended | Comfortable use |
| 1024x768 | Default | Optimal for development |
| Fullscreen | Optional | Power user preference |

### 5.2 Layout Adapting Rules

```slint
// Pseudocode for responsive behavior
if (window.width < 800) {
    // Single column layout
    lobby.width = 200;
    chat.width = parent.width - 200;
} else {
    // Side-by-side layout
    lobby.width = 280;
    chat.width = parent.width - 280;
}

if (window.height < 500) {
    // Compressed composer
    composer.height = 60;
} else {
    // Full composer
    composer.height = 80;
}
```

---

## 6. Implementation Priority

### Phase 1: Critical (Week 1)

| # | Component | Change | Files Modified |
|---|-----------|--------|----------------|
| 1 | AppWindow | Enable resize | main.slint, main.rs |
| 2 | AppWindow | Dynamic message list | main.slint, main.rs |
| 3 | AppWindow | Dynamic lobby list | main.slint, main.rs |

### Phase 2: Important (Week 2)

| # | Component | Change | Files Created/Modified |
|---|-----------|--------|------------------------|
| 4 | ConnectionStatus | New component | connection_status.slint |
| 5 | MessageComposer | Placeholder, states | composer.slint |
| 6 | EmptyLobbyState | New component | empty_lobby_state.slint |

### Phase 3: Polish (Week 3)

| # | Component | Change | Files Created/Modified |
|---|-----------|--------|------------------------|
| 7 | KeyboardShortcutsHelp | New component | keyboard_shortcuts_help.slint |
| 8 | LobbyItem | Keyboard nav | lobby_item.slint |
| 9 | All components | A11y labels | Multiple |

---

## 7. Testing Requirements

### 7.1 UI Tests

| Test | Description | Expected Result |
|------|-------------|-----------------|
| Window resize | Drag window corner | UI adapts, no cutoffs |
| Dynamic list | Add 15+ messages | All messages visible, scrollable |
| Keyboard nav | Tab through all elements | Focus visible, logical order |
| Empty state | Clear lobby | EmptyLobbyState shown |
| Help modal | Press ? | Modal opens, closes on Escape |

### 7.2 Accessibility Tests

| Test | Tool | Target |
|------|------|--------|
| Keyboard only | Manual | All features work |
| Screen reader | NVDA/JAWS | Labels read correctly |
| Color contrast | axe DevTools | All text meets WCAG AA |
| Focus visible | Manual | All focus states visible |

---

## 8. Dependencies

### 8.1 Slint Features Required

| Feature | Min Version | Used For |
|---------|-------------|----------|
| `for` with arrays | 1.5+ | Dynamic lists |
| `ScrollView` | 1.0+ | Overflow handling |
| `FocusScope` | 1.0+ | Keyboard navigation |
| `accessible-*` | 1.5+ | Screen reader support |

### 8.2 New Dependencies

None required - all features use existing Slint 1.5 capabilities.

---

## 9. Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| `for` with arrays unstable | High | Use ScrollView fallback |
| Performance with many messages | Medium | Implement message pagination (max 100 visible) |
| Keyboard nav regressions | Medium | Add integration tests for key sequences |

---

## 10. Success Criteria

### Quantitative Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Max messages displayable | 10 | Unlimited (with scroll) |
| Max lobby users displayable | 10 | Unlimited (with scroll) |
| Window resize | ❌ No | ✅ Yes |
| Keyboard shortcuts documented | 2 | 10+ |

### Qualitative Metrics

- [ ] All features accessible via keyboard
- [ ] Empty states provide guidance
- [ ] Help available via ? shortcut
- [ ] Connection status always visible

---

## 11. Appendix

### A. File Structure After Redesign

```
profile-root/client/src/ui/
├── welcome_screen.slint           # Updated (keyboard nav)
├── key_display.slint              # Keep (already good)
├── import_key_screen.slint        # Update (inline validation)
├── lobby_item.slint               # Updated (keyboard nav)
├── message_item.slint             # Keep (already good)
├── message_item_compact.slint     # Keep (already good)
├── composer.slint                 # Updated (placeholder, states)
├── drill_down_modal.slint         # Keep (already good)
├── connection_status.slint        # NEW
├── empty_lobby_state.slint        # NEW
└── keyboard_shortcuts_help.slint  # NEW
```

### B. Color Palette (Reference)

| Color | Hex | Usage |
|-------|-----|-------|
| Dark Surface | #111827 | Backgrounds |
| Dark Modal | #1e1e2e | Modals |
| Identity Blue | #0066CC | Keys, self, selection |
| Success Green | #22c55e | Verified, online |
| Error Red | #ef4444 | Errors, invalid |
| Warning Amber | #f59e0b | Long messages |
| Neutral Gray | #6b7280 | Secondary text |
| Text White | #ffffff | Primary text |

---

*Document Version 1.0 - 2025-12-31*
*Ready for implementation*
