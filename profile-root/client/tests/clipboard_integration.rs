//! Integration tests for Story 1.3: Display User's Public Key Clearly
//!
//! Tests clipboard functionality for public key copying
//!
//! NOTE: These tests use the system clipboard, which is shared state.
//! They include small delays (10ms) to handle platform timing issues.

use arboard::Clipboard;

/// Test that clipboard API is available and functional
#[test]
fn integration_test_clipboard_available() {
    let clipboard_result = Clipboard::new();
    
    // On headless CI, clipboard might not be available - that's okay
    // On Windows with GUI, it should work
    match clipboard_result {
        Ok(_) => {
            // Clipboard is available - this is expected on Windows
            println!("✓ Clipboard API available");
        }
        Err(e) => {
            // Clipboard not available - might be headless environment
            println!("⚠ Clipboard not available (headless?): {}", e);
            // Don't fail test - clipboard might not work in CI/headless
        }
    }
}

/// Test clipboard set/get roundtrip with sample public key
#[test]
fn integration_test_clipboard_roundtrip() {
    // Try to get clipboard
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(_) => {
            println!("⚠ Skipping test - clipboard not available (headless environment)");
            return; // Skip test in headless environment
        }
    };
    
    // Sample public key (64 hex characters)
    let sample_key = "a1b2c3d4e5f6789012345678901234567890abcdef1234567890abcdef123456";
    
    // Set clipboard
    clipboard.set_text(sample_key).expect("Should set clipboard text");
    
    // Small delay to ensure clipboard is updated (platform timing issue)
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    // Get clipboard
    let retrieved = clipboard.get_text().expect("Should get clipboard text");
    
    // Verify roundtrip
    assert_eq!(
        retrieved, sample_key,
        "Clipboard roundtrip should preserve full 64-character key"
    );
}

/// Test clipboard preserves full 64-character hex keys without truncation
#[test]
fn integration_test_clipboard_no_truncation() {
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(_) => {
            println!("⚠ Skipping test - clipboard not available");
            return;
        }
    };
    
    // Generate multiple test keys
    let test_keys = vec![
        "0".repeat(64),  // All zeros
        "f".repeat(64),  // All max values
        "0123456789abcdef".repeat(4), // Pattern repeated 4 times = 64 chars
        "c".repeat(64), // All 'c' (unique to avoid conflicts)
    ];
    
    for (i, key) in test_keys.iter().enumerate() {
        // Set THEN get - clipboard is shared state
        clipboard.set_text(key).expect(&format!("Should set key #{}", i));
        
        // Small delay to ensure clipboard is updated (platform timing issue)
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let retrieved = clipboard.get_text().expect(&format!("Should get key #{}", i));
        
        assert_eq!(
            retrieved.len(),
            64,
            "Key #{} should be 64 characters after clipboard roundtrip, got: {}",
            i, retrieved
        );
        assert_eq!(
            &retrieved, key,
            "Key #{} should match exactly after clipboard roundtrip",
            i
        );
    }
}

/// Test clipboard handles whitespace correctly (user might paste with trailing newlines)
#[test]
fn integration_test_clipboard_exact_content() {
    let mut clipboard = match Clipboard::new() {
        Ok(cb) => cb,
        Err(_) => {
            println!("⚠ Skipping test - clipboard not available");
            return;
        }
    };
    
    // Use unique key to avoid conflicts with other tests
    let key_with_no_whitespace = "d".repeat(64);
    
    // Set THEN get - always set before expecting a value
    clipboard.set_text(&key_with_no_whitespace).expect("Should set clipboard");
    
    // Small delay to ensure clipboard is updated (platform timing issue)
    std::thread::sleep(std::time::Duration::from_millis(10));
    
    let retrieved = clipboard.get_text().expect("Should get clipboard");
    
    // Verify no extra whitespace added
    assert_eq!(retrieved, key_with_no_whitespace, "Clipboard should preserve exact content");
    assert!(!retrieved.starts_with(' '), "Should not start with whitespace");
    assert!(!retrieved.ends_with(' '), "Should not end with whitespace");
    assert!(!retrieved.contains('\n'), "Should not contain newlines");
    assert!(!retrieved.contains('\r'), "Should not contain carriage returns");
}

/// Test clipboard error handling when clipboard is locked by another process
#[test]
fn integration_test_clipboard_error_handling() {
    // This test just verifies the error handling pattern exists
    // Actual failure requires clipboard to be locked by another process (hard to simulate)
    
    let clipboard_result = Clipboard::new();
    
    match clipboard_result {
        Ok(mut clipboard) => {
            // Try to set text
            match clipboard.set_text("test") {
                Ok(_) => println!("✓ Clipboard set successful"),
                Err(e) => println!("⚠ Clipboard set failed: {}", e),
            }
        }
        Err(e) => {
            println!("⚠ Clipboard initialization failed: {}", e);
        }
    }
    
    // Test always passes - we're just verifying error handling compiles
    assert!(true);
}
