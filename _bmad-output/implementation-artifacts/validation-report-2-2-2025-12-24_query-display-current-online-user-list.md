# Story 2.2 Validation Report

**Document:** `/home/riddler/profile/_bmad-output/implementation-artifacts/2-2-query-display-current-online-user-list.md`
**Checklist:** `/home/riddler/profile/_bmad/bmm/workflows/4-implementation/create-story/checklist.md`
**Date:** 2025-12-24

---

## Summary

| Metric | Value |
|--------|-------|
| **Overall Assessment** | ⚠️ NEEDS MINOR REVISIONS |
| **Critical Issues** | 2 |
| **Enhancement Opportunities** | 5 |
| **Optimization Suggestions** | 3 |
| **Duplicate Content Issues** | 1 |

---

## 🚨 CRITICAL ISSUES (Must Fix)

### 1. **Protocol Inconsistency: lobby_update Structure Mismatch**

**Location:** Lines 110-116 vs Architecture.md Lines 407-417

**Issue:** The story's lobby_update protocol shows `joined` users WITH a `status` field:
```json
{
  "type": "lobby_update",
  "joined": [{"publicKey": "...", "status": "online"}],
  "left": ["..."]
}
```

But Architecture.md (the source of truth) shows `joined` users WITHOUT a status field:
```json
{
  "type": "lobby_update",
  "joined": [{"publicKey": "..."}]
}
```

**Evidence:**
- Story lines 110-116: Shows `joined: [{"publicKey": "...", "status": "online"}]`
- Architecture.md lines 407-411: Shows `joined: [{publicKey: "..."}]` (no status field)

**Impact:** If the developer implements the story's version with `status` in lobby_update but the server (Story 2.1) sends without status, deserialization will fail or silently ignore the field. This creates a breaking protocol mismatch.

**Recommendation:** Choose ONE consistent approach:
- **Option A:** Align with architecture - lobby_update only sends publicKey (joined users are always "online")
- **Option B:** Update architecture to include status field in lobby_update

---

### 2. **Duplicate Content: Performance Requirements Repeated**

**Location:** Lines 129-147 and Lines 139-147 (exact duplicate)

**Issue:** The Performance Requirements section is duplicated word-for-word:
- Lines 129-137: Performance Requirements (first occurrence)
- Lines 139-147: Performance Requirements (exact duplicate)

**Evidence:** Both sections contain identical code block with identical text about <200ms render, <50ms delta, <100ms propagation.

**Impact:** Wastes tokens, creates confusion about which is authoritative, looks unprofessional.

**Recommendation:** Remove duplicate (lines 139-147) and keep single authoritative section.

---

## ⚡ ENHANCEMENT OPPORTUNITIES (Should Add)

### 1. **Missing Tab Navigation Support**

**Location:** Lines 42-43, Keyboard nav section

**Issue:** The story mentions "standard Tab/Arrow keys" but doesn't explicitly describe:
- What Tab does (moves focus INTO the lobby from other elements)
- How to move focus OUT of the lobby to other components

**Evidence:**
- Story line 49: "Keyboard nav: standard Tab/Arrow keys, Enter to select"
- UX Design line 538: "Tab: Navigate between UI elements"
- Missing: Specific guidance on Tab handling for lobby component

**Recommendation:** Add clear specification:
```
Tab handling for lobby:
- Tab from previous component → moves focus to first lobby item
- Tab from last lobby item → moves focus to next component (composer)
- Shift+Tab → reverse navigation
```

---

### 2. **File Naming Convention Conflict**

**Location:** Lines 78 vs Lines 174-175, 723-728

**Issue:** Two different naming conventions appear in the story:
- Lines 78, 174-175: `lobby.rs`, `lobby_state.rs`, `lobby_item.slint`
- Lines 723-728: `user_list.rs`, `lobby_item.slint` (in different location)

**Evidence:**
- Line 78: "Lobby component: `profile-root/client/src/ui/components/user_list.rs` (NEW - this story)"
- Line 175: "`profile-root/client/src/ui/lobby.rs` - NEW: LobbyComponent implementation (renamed from user_list.rs)"
- Lines 723-728: Shows different directory structure with `components/user_list.rs`

**Impact:** Creates confusion about actual file locations and naming.

**Recommendation:** Consolidate to ONE naming convention throughout:
```
profile-root/client/src/ui/lobby.rs        # Main lobby component
profile-root/client/src/ui/lobby_state.rs   # State management
profile-root/client/ui/lobby_item.slint    # Slint component
```

---

### 3. **Missing Scroll-Selection Interaction**

**Location:** Lines 149-156 (Scroll Behavior) vs Lines 158-165 (Selection Edge Cases)

**Issue:** The scroll behavior and selection sections don't specify interaction between them:
- What happens to scroll position when user selects an item?
- If user scrolls away and selects, should selection be visible?
- Auto-scroll to selected item?

**Evidence:** Both sections exist but don't address their intersection.

**Recommendation:** Add:
```
Scroll-Selection Interaction:
- Selection does NOT change scroll position (user maintains their view)
- If selected item is scrolled out of view, NO auto-scroll (user can scroll manually)
- Selection highlight renders regardless of scroll position
```

---

### 4. **Missing Protocol Struct Definitions**

**Location:** Lines 263-283 (Client-Server Protocol section)

**Issue:** The story references `LobbyMessage`, `LobbyUser`, and `LobbyUpdateMessage` structs but doesn't:
- Confirm where they are defined
- Show the exact struct definitions
- Reference the source file (should be in `shared/protocol/`)

**Evidence:** Lines 264-283 show struct definitions but don't indicate if these are NEW (to be created) or EXISTING (in shared/protocol/).

**Recommendation:** Add explicit note:
```
// Protocol structs - Already implemented in: profile-root/shared/src/protocol/mod.rs
// No changes needed to protocol definitions for this story
```

Or if they need to be created:
```
// Protocol structs - CREATE in: profile-root/shared/src/protocol/lobby.rs
```

---

### 5. **Missing Test Coverage for Error Scenarios**

**Location:** Lines 210-229 (Testing Strategy)

**Issue:** The test strategy mentions 10+ tests but the example tests don't cover:
- Malformed JSON error handling
- Duplicate user handling
- Empty lobby with selection edge cases
- Timeout/retry scenarios

**Evidence:** Example tests (lines 402-458) cover basic happy path but not error scenarios from the Error Handling section (lines 118-127).

**Recommendation:** Add error scenario tests:
```rust
#[tokio::test]
async fn test_lobby_handles_malformed_json() {
    // Test malformed lobby message doesn't crash client
}

#[tokio::test]
async fn test_lobby_handles_duplicate_users() {
    // Test deduplication logic when same user appears twice
}
```

---

## ✨ OPTIMIZATION SUGGESTIONS (Nice to Have)

### 1. **Refine Protocol JSON Examples for Clarity**

**Location:** Lines 100-116

**Issue:** The protocol examples use placeholder ellipses (`"..."`) inconsistently:
- Some use `"3a8f2e1c..."` (showing truncation)
- Some use `"..."` (minimal placeholder)

**Recommendation:** Use consistent format throughout:
```json
// All examples should use either:
// Full: "3a8f2e1cb4d9a8f2e1cb4d9a8f2e1cb"
// OR consistent placeholder: "..."
```

---

### 2. **Consolidate Directory Structure Visualizations**

**Location:** Lines 736-756 (two different directory structures shown)

**Issue:** Two different directory structures are shown (lines 736-756 and lines 757-777) with slight differences.

**Recommendation:** Show single, authoritative directory structure.

---

### 3. **Improve Code Snippet Organization**

**Location:** Throughout "Actionable Code Snippets" section

**Issue:** Code snippets appear in multiple places:
- Lines 88-96: Lobby component states (enum)
- Lines 501-537: Lobby Component (Slint) - full example
- Lines 539-622: Lobby State Management (Rust) - full example
- Lines 625-720: WebSocket Message Handler (Rust) - full example

**Recommendation:** Consider reorganizing to show smaller, focused snippets inline with the relevant task, with comprehensive examples in an appendix.

---

## 📋 RECOMMENDATIONS SUMMARY

### 1. Must Fix (Before Development)

| Priority | Issue | Action Required |
|----------|-------|-----------------|
| **Critical** | Protocol mismatch lobby_update | Clarify if status field belongs in lobby_update or remove from story |
| **Critical** | Duplicate content | Remove duplicate Performance Requirements section (lines 139-147) |

### 2. Should Improve (Before Development)

| Priority | Issue | Action Required |
|----------|-------|-----------------|
| **High** | Tab navigation | Add explicit Tab focus handling specification |
| **High** | File naming conflict | Consolidate to single naming convention |
| **Medium** | Scroll-selection interaction | Add scroll-selection interaction specification |
| **Medium** | Protocol struct ownership | Clarify where structs are defined |
| **Medium** | Error test coverage | Add error scenario tests to test strategy |

### 3. Consider (Nice to Have)

| Priority | Issue | Action Required |
|----------|-------|-----------------|
| **Low** | JSON placeholder consistency | Standardize placeholder format |
| **Low** | Directory structure duplication | Consolidate to single visualization |
| **Low** | Code snippet organization | Consider inline snippets + appendix |

---

## 📁 OUTPUT FILES

**Validation Report:** `/home/riddler/profile/_bmad-output/implementation-artifacts/validation-report-2-2-2025-12-24_query-display-current-online-user-list.md`

---

## 🔄 NEXT STEPS

1. **Apply Critical Fixes:**
   - Resolve protocol inconsistency with architecture
   - Remove duplicate content

2. **Apply Enhancements:**
   - Add Tab navigation specification
   - Consolidate file naming
   - Clarify protocol struct locations

3. **Re-run Validation (Optional):**
   - After fixes, re-run `*validate-create-story` to confirm all issues resolved

4. **Proceed to Development:**
   - Once validated, run `*dev-story` to implement Story 2.2
