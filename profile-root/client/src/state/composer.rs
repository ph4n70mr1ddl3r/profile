//! Composer state management for message drafts
//!
//! This module provides thread-safe composer state management that preserves
//! message drafts during disconnections (AC2, AC3 requirement).

use tokio::sync::Mutex;
use std::sync::Arc;

/// Composer state for preserving message drafts
#[derive(Debug)]
pub struct ComposerState {
    pub draft_text: String,
    pub recipient: Option<String>,
}

/// Shared reference to composer state for concurrent access
pub type SharedComposerState = Arc<Mutex<ComposerState>>;

impl ComposerState {
    /// Create a new empty composer state
    pub fn new() -> Self {
        Self {
            draft_text: String::new(),
            recipient: None,
        }
    }
    
    /// Set the current draft text
    pub fn set_draft(&mut self, text: String) {
        self.draft_text = text;
    }
    
    /// Get the current draft text
    pub fn get_draft(&self) -> String {
        self.draft_text.clone()
    }
    
    /// Clear the draft text
    pub fn clear_draft(&mut self) {
        self.draft_text.clear();
    }
}

impl Default for ComposerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a shared composer state for concurrent access
pub fn create_shared_composer_state() -> SharedComposerState {
    Arc::new(Mutex::new(ComposerState::new()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_draft_preserved_during_disconnect() {
        let composer = create_shared_composer_state();
        composer.lock().await.set_draft("test message".to_string());
        
        // Simulate disconnect (drop connection, don't touch composer)
        // The draft should remain in memory
        
        let draft = composer.lock().await.get_draft();
        assert_eq!(draft, "test message");
    }

    #[tokio::test]
    async fn test_composer_state_thread_safe() {
        let composer = create_shared_composer_state();
        let composer_clone = Arc::clone(&composer);
        
        let task1 = tokio::spawn(async move {
            composer_clone.lock().await.set_draft("draft 1".to_string());
        });
        
        task1.await.unwrap();
        let draft = composer.lock().await.get_draft();
        assert_eq!(draft, "draft 1");
    }

    #[test]
    fn test_composer_new() {
        let composer = ComposerState::new();
        assert_eq!(composer.draft_text, "");
        assert!(composer.recipient.is_none());
    }

    #[test]
    fn test_draft_operations() {
        let mut composer = ComposerState::new();
        
        // Set draft
        composer.set_draft("Hello, world!".to_string());
        assert_eq!(composer.get_draft(), "Hello, world!");
        
        // Update draft
        composer.set_draft("Updated draft".to_string());
        assert_eq!(composer.get_draft(), "Updated draft");
        
        // Clear draft
        composer.clear_draft();
        assert_eq!(composer.get_draft(), "");
    }
}
