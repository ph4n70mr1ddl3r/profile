//! UI color constants for consistent theming
//!
//! Centralized color definitions matching UX Design Specification.
//! All UI components should use these constants for consistency.

// Lobby colors (from UX Design Specification)
impl LobbyColors {
    pub const ONLINE_INDICATOR: &str = "#22c55e";
    pub const OFFLINE_INDICATOR: &str = "#6b7280";
    pub const SELECTED_BG: &str = "#0066CC";
    pub const SELECTED_TEXT: &str = "#ffffff";
    pub const KEY_COLOR: &str = "#0066CC";
    pub const DEFAULT_BG: &str = "#111827";
    pub const HOVER_BG: &str = "#374151";
    pub const SELECTED_BORDER: &str = "#0088FF";
    pub const EMPTY_TEXT: &str = "#999999";
    pub const SELECTED_DISPLAY: &str = "#0066CC";
}

struct LobbyColors;

// Composer colors (Story 2.2)
impl ComposerColors {
    pub const BACKGROUND: &str = "#1f2937";
    pub const BORDER_FOCUSED: &str = "#0066CC";
    pub const BORDER_DEFAULT: &str = "#374151";
    pub const PLACEHOLDER_TEXT: &str = "#9ca3af";
    pub const RECIPIENT_TEXT: &str = "#6b7280";
}

struct ComposerColors;

// Common UI colors
impl CommonColors {
    pub const ERROR: &str = "#ef4444";
    pub const SUCCESS: &str = "#22c55e";
    pub const WARNING: &str = "#f59e0b";
    pub const INFO: &str = "#3b82f6";
}

struct CommonColors;
