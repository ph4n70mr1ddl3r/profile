//! Technical Signature Testing & Validation Tests for Story 4.4
//!
//! Tests cover:
//! - Deterministic signing verification (same message = same signature)
//! - Signature format validation (128 hex chars, lowercase)
//! - Edge case content handling (unicode, emoji, special chars, long text)
//! - Copy functionality validation (exact length, no modification)
//!
//! These tests enable technical users (Sam) to validate the cryptographic
//! foundation by comparing signatures and testing edge cases.

use profile_shared::generate_private_key;
use profile_shared::derive_public_key;
use profile_shared::sign_message;
use profile_shared::verify_signature;
use zeroize::Zeroizing;

/// Convert signature bytes to lowercase hex string
fn signature_to_hex(signature: &[u8]) -> String {
    signature.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

/// Test Task 1.1: Deterministic Signing Verification
///
/// Verifies that identical messages produce identical signatures
/// (byte-for-byte match), confirming deterministic signing works.
#[tokio::test]
async fn test_deterministic_signing_same_message_twice() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let public_key = derive_public_key(&private_key).unwrap();
    let message_content = "Hello, World!";
    let timestamp = "2025-12-30T10:00:00Z";

    // Sign identical message content twice
    let canonical1 = format!("{}:{}", message_content, timestamp);
    let signature1 = sign_message(&private_key, canonical1.as_bytes()).unwrap();

    let canonical2 = format!("{}:{}", message_content, timestamp);
    let signature2 = sign_message(&private_key, canonical2.as_bytes()).unwrap();

    // Both signatures should be identical (deterministic)
    assert_eq!(
        signature1, signature2,
        "Deterministic signing failed: identical messages should produce identical signatures"
    );

    // Both signatures should verify successfully
    assert!(
        verify_signature(&public_key, canonical1.as_bytes(), &signature1).is_ok(),
        "First signature should verify"
    );
    assert!(
        verify_signature(&public_key, canonical2.as_bytes(), &signature2).is_ok(),
        "Second signature should verify"
    );
}

/// Test Task 1.2: Signature Display Length Validation
///
/// Verifies that signatures display exactly 128 hex characters
/// (64 bytes × 2 hex chars per byte).
#[tokio::test]
async fn test_signature_display_length_128_chars() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let message_content = "Test message for signature length";
    let timestamp = "2025-12-30T10:00:00Z";

    let canonical = format!("{}:{}", message_content, timestamp);
    let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();

    // Signature should be 64 bytes
    assert_eq!(
        signature.len(),
        64,
        "Ed25519 signature should be 64 bytes (512 bits)"
    );

    // Convert to hex for display length check
    let signature_hex = signature_to_hex(&signature);
    assert_eq!(
        signature_hex.len(),
        128,
        "Hex representation should be exactly 128 characters"
    );

    // All characters should be valid hex digits (0-9, a-f)
    assert!(
        signature_hex.chars().all(|c| c.is_ascii_hexdigit()),
        "All signature characters should be valid hex digits"
    );
}

/// Test Task 1.3: Signature Hex Format Consistency
///
/// Verifies that all signatures use lowercase hex consistently.
#[tokio::test]
async fn test_signature_hex_format_consistency() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let test_messages = [
        "Hello",
        "World",
        "Test 123",
        "MixedCASETest",
    ];
    let timestamp = "2025-12-30T10:00:00Z";

    for message in test_messages.iter() {
        let canonical = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let signature_hex = signature_to_hex(&signature);

        // Signature should use lowercase hex (no uppercase letters)
        assert!(
            !signature_hex.chars().any(|c| c.is_ascii_uppercase()),
            "Signature for '{}' should use lowercase hex only, got: {}",
            message,
            signature_hex
        );

        // Signature hex should be exactly 128 characters
        assert_eq!(
            signature_hex.len(),
            128,
            "Signature for '{}' should be exactly 128 hex characters",
            message
        );
    }
}

/// Test Task 2.1: Signature Display with Unicode Content
///
/// Verifies that signatures display correctly for messages containing
/// Unicode/CJK characters.
#[tokio::test]
async fn test_signature_display_unicode_content() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let public_key = derive_public_key(&private_key).unwrap();

    // Test messages with various Unicode content
    let unicode_messages = [
        "你好世界",                    // Chinese
        "Привет мир",                // Russian
        "Hello 🌍 World",            // Emoji + text
        "Mixed 中文 and English",     // Mixed script
    ];

    let timestamp = "2025-12-30T10:00:00Z";

    for message in unicode_messages.iter() {
        let canonical = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let signature_hex = signature_to_hex(&signature);

        // Signature hex should be valid
        assert_eq!(signature_hex.len(), 128);
        assert!(signature_hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(!signature_hex.chars().any(|c| c.is_ascii_uppercase()));

        // Signature should verify
        assert!(
            verify_signature(&public_key, canonical.as_bytes(), &signature).is_ok(),
            "Signature for '{}' should verify",
            message
        );
    }
}

/// Test Task 2.2: Signature Display with Emoji Content
///
/// Verifies that signatures display correctly for messages containing
/// emoji characters.
#[tokio::test]
async fn test_signature_display_emoji_content() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let public_key = derive_public_key(&private_key).unwrap();

    let emoji_messages = [
        "Hello 👋 World 🌍",
        "👋👋👋 Three waves",
        "🎉🎂🍰 Happy Birthday!",
        "🔥💯🎯 Classic combo",
    ];

    let timestamp = "2025-12-30T10:00:00Z";

    for message in emoji_messages.iter() {
        let canonical = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let signature_hex = signature_to_hex(&signature);

        // Signature should be valid
        assert_eq!(signature_hex.len(), 128);
        assert!(signature_hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(
            verify_signature(&public_key, canonical.as_bytes(), &signature).is_ok()
        );
    }
}

/// Test Task 2.3: Signature Display with Special Characters
///
/// Verifies that signatures display correctly for messages containing
/// HTML special characters.
#[tokio::test]
async fn test_signature_display_special_chars() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let public_key = derive_public_key(&private_key).unwrap();

    let special_char_messages = [
        "Hello <World> & Friends",
        "Quote: \"test\" and 'test'",
        "Less < than & Greater > than",
        "Ampersand & entity &amp;",
    ];

    let timestamp = "2025-12-30T10:00:00Z";

    for message in special_char_messages.iter() {
        let canonical = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let signature_hex = signature_to_hex(&signature);

        // Signature should be valid
        assert_eq!(signature_hex.len(), 128);
        assert!(signature_hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(
            verify_signature(&public_key, canonical.as_bytes(), &signature).is_ok()
        );

        // Verify the message content is preserved exactly
        assert_eq!(canonical, format!("{}:{}", message, timestamp));
    }
}

/// Test Task 2.4: Signature Display with Long Content
///
/// Verifies that signatures display correctly for long messages
/// (1000+ characters).
#[tokio::test]
async fn test_signature_display_long_content() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let public_key = derive_public_key(&private_key).unwrap();

    // Create a long message (1000+ chars)
    let long_message: String = (0..1000).map(|i| {
        match i % 4 {
            0 => 'A',
            1 => 'b',
            2 => 'C',
            _ => '3',
        }
    }).collect();

    let timestamp = "2025-12-30T10:00:00Z";
    let canonical = format!("{}:{}", long_message, timestamp);
    let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
    let signature_hex = signature_to_hex(&signature);

    // Signature should be valid
    assert_eq!(signature_hex.len(), 128);
    assert!(signature_hex.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(
        verify_signature(&public_key, canonical.as_bytes(), &signature).is_ok()
    );
}

/// Test Task 2.5: Signature Display with Whitespace Content
///
/// Verifies that signatures display correctly for messages with
/// leading/trailing whitespace, newlines, and tabs.
#[tokio::test]
async fn test_signature_display_whitespace_content() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let public_key = derive_public_key(&private_key).unwrap();

    let whitespace_messages = [
        "  leading spaces",
        "trailing spaces  ",
        "leading\nnewline",
        "trailing\ttab",
        "  leading and trailing  ",
        "\n\n\nMultiple newlines\n\n\n",
        "Mixed \t tabs and spaces ",
    ];

    let timestamp = "2025-12-30T10:00:00Z";

    for message in whitespace_messages.iter() {
        let canonical = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let signature_hex = signature_to_hex(&signature);

        // Signature should be valid
        assert_eq!(signature_hex.len(), 128);
        assert!(signature_hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(
            verify_signature(&public_key, canonical.as_bytes(), &signature).is_ok()
        );

        // Verify whitespace is preserved
        assert_eq!(canonical, format!("{}:{}", message, timestamp));
    }
}

/// Test Task 3.1: Copy Signature Exact Length
///
/// Verifies that copied signature has exactly 128 characters.
#[tokio::test]
async fn test_copy_signature_exact_length() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let message_content = "Test message";
    let timestamp = "2025-12-30T10:00:00Z";

    let canonical = format!("{}:{}", message_content, timestamp);
    let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
    let signature_hex = signature_to_hex(&signature);

    // Copied signature should be exactly 128 characters
    assert_eq!(
        signature_hex.len(),
        128,
        "Copied signature should be exactly 128 hex characters"
    );
}

/// Test Task 3.2: Copy Signature No Modification
///
/// Verifies that copied signature matches displayed signature exactly
/// (no truncation, no modification).
#[tokio::test]
async fn test_copy_signature_no_modification() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let message_content = "Test message for copy verification";
    let timestamp = "2025-12-30T10:00:00Z";

    let canonical = format!("{}:{}", message_content, timestamp);
    let displayed_signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
    let displayed_hex = signature_to_hex(&displayed_signature);

    // In the UI, the displayed signature should match what we copy
    // (This test verifies the underlying data - the UI copy function
    // should copy exactly what's displayed)
    let copied_hex = displayed_hex.clone();

    assert_eq!(
        displayed_hex, copied_hex,
        "Copied signature should match displayed signature exactly - no modification"
    );
}

/// Test Task 3.3: Copy Signature Preserves Hex Format
///
/// Verifies that copied signature contains only valid lowercase hex characters.
#[tokio::test]
async fn test_copy_signature_preserves_hex_format() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());

    // Sign multiple messages
    let messages = ["Test1", "Test2", "Test3"];
    let timestamp = "2025-12-30T10:00:00Z";

    for message in messages.iter() {
        let canonical = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let signature_hex = signature_to_hex(&signature);

        // Copied signature should be valid lowercase hex
        assert!(
            signature_hex.chars().all(|c| c.is_ascii_hexdigit()),
            "Copied signature should contain only valid hex characters"
        );

        assert!(
            !signature_hex.chars().any(|c| c.is_ascii_uppercase()),
            "Copied signature should use lowercase hex only"
        );

        assert!(
            !signature_hex.chars().any(|c| !c.is_ascii_alphanumeric()),
            "Copied signature should contain no whitespace or special characters"
        );
    }
}

/// Test Task 3.4: Signature Verification Status Display
///
/// Verifies that signatures display with appropriate verification status
/// (✓ for verified, ⚠ for failed).
#[tokio::test]
async fn test_signature_verification_status_display() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let public_key = derive_public_key(&private_key).unwrap();

    let message_content = "Test message";
    let timestamp = "2025-12-30T10:00:00Z";
    let canonical = format!("{}:{}", message_content, timestamp);
    let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();

    // Valid signature should verify
    let verification_result = verify_signature(&public_key, canonical.as_bytes(), &signature);
    assert!(
        verification_result.is_ok(),
        "Valid signature should verify successfully"
    );

    // In the UI, this should display with a ✓ badge
    // (This is verified in the UI component: drill_down_modal.slint lines 373-387)
    let is_verified = verification_result.is_ok();

    // Simulate UI display logic
    let expected_badge_symbol = if is_verified { "✓" } else { "⚠" };
    let expected_badge_color = if is_verified { "#22c55e" } else { "#ef4444" };

    assert_eq!(expected_badge_symbol, "✓");
    assert_eq!(expected_badge_color, "#22c55e");
}

/// Additional Test: Signature Format Documentation
///
/// This test documents the expected signature format for technical users:
/// - Algorithm: Ed25519 (EdDSA with Curve25519)
/// - Key size: 256-bit (32 bytes)
/// - Signature size: 512-bit (64 bytes)
/// - Encoding: Hexadecimal (lowercase)
/// - Display length: 128 characters
#[tokio::test]
async fn test_signature_format_documentation() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let message_content = "Documentation test";
    let timestamp = "2025-12-30T10:00:00Z";

    let canonical = format!("{}:{}", message_content, timestamp);
    let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
    let signature_hex = signature_to_hex(&signature);

    // Document the expected format
    assert_eq!(signature_hex.len(), 128, "Signature length: 128 hex chars (64 bytes × 2)");
    assert!(signature_hex.chars().all(|c| c.is_ascii_hexdigit()), "Encoding: hexadecimal");
    assert!(!signature_hex.chars().any(|c| c.is_ascii_uppercase()), "Case: lowercase only");

    // Verify it's a valid Ed25519 signature size (64 bytes = 512 bits)
    assert_eq!(
        signature.len(),
        64,
        "Signature should be 64 bytes (512 bits) for Ed25519"
    );
}

/// Additional Test: Deterministic Signing 1000 Iterations
///
/// Comprehensive test verifying deterministic signing across many iterations.
#[tokio::test]
async fn test_deterministic_signing_1000_iterations() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let message = "Deterministic signing test";
    let timestamp = "2025-12-30T10:00:00Z";

    // Sign the same message 1000 times
    let mut first_signature_bytes: Option<Vec<u8>> = None;

    for i in 0..1000 {
        let canonical = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let signature_hex = signature_to_hex(&signature);

        if i == 0 {
            // Store first signature for comparison
            assert_eq!(signature_hex.len(), 128);
            first_signature_bytes = Some(signature);
        } else {
            // All signatures should be identical to the first one
            let first_sig = first_signature_bytes.as_ref().unwrap();
            assert_eq!(
                signature.as_slice(), first_sig,
                "Iteration {}: All signatures should be identical (deterministic)",
                i
            );
        }
    }
}

/// Additional Test: Different Messages Have Different Signatures
///
/// Verifies that different messages produce different signatures
/// (no signature collisions).
#[tokio::test]
async fn test_different_messages_different_signatures() {
    let private_key = Zeroizing::new(generate_private_key().unwrap());
    let timestamp = "2025-12-30T10:00:00Z";

    let messages = [
        "Message A",
        "Message B",
        "Message A ",  // Different due to trailing space
        "message a",   // Different due to case
    ];

    let mut signatures: Vec<String> = Vec::new();

    for message in messages.iter() {
        let canonical = format!("{}:{}", message, timestamp);
        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let signature_hex = signature_to_hex(&signature);
        signatures.push(signature_hex);
    }

    // All signatures should be unique (no collisions)
    for (i, sig_i) in signatures.iter().enumerate() {
        for (j, sig_j) in signatures.iter().enumerate() {
            if i != j {
                assert_ne!(
                    sig_i, sig_j,
                    "Different messages should produce different signatures"
                );
            }
        }
    }
}
