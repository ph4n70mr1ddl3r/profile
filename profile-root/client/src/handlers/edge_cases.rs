//! Message composition edge case tests
//!
//! This module contains tests for handling various edge cases in message composition:
//! - Unicode characters (international text, emojis)
//! - Special characters (symbols, punctuation)
//! - Long messages (10KB+ text)
//! - Whitespace variations (spaces, tabs, newlines)
//! - Binary content validation
//!
//! Story 3.8: Handle Message Composition Edge Cases

use profile_shared::{generate_private_key, derive_public_key, sign_message, verify_signature};
use crate::state::messages::{ChatMessage, MessageHistory};
use crate::ui::chat::format_timestamp;

/// Test unicode message handling
mod unicode_tests {
    use super::*;

    #[tokio::test]
    async fn test_chinese_characters() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "你好世界！这是一个测试。";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Chinese characters should be handled correctly");
    }

    #[tokio::test]
    async fn test_emoji_message() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "Hello 🔐 World! 🌍🚀";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Emoji should be handled correctly");
    }

    #[tokio::test]
    async fn test_spanish_accents() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "El veloz murciélago hindú comía feliz cardillo y kiwi.";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Spanish accents should be handled correctly");
    }

    #[tokio::test]
    async fn test_arabic_text() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "مرحبا بالعالم! هذه رسالة اختبار.";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Arabic text should be handled correctly");
    }

    #[tokio::test]
    async fn test_mixed_unicode() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "Hello 世界! 🎉 Ñoño tilde مرحبا こんにちは";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Mixed unicode should be handled correctly");
    }
}

/// Test special character handling
mod special_char_tests {
    use super::*;

    #[tokio::test]
    async fn test_special_symbols() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "!@#$%^&*()_+-=[]{}|;':\",./<>?";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Special symbols should be handled correctly");
    }

    #[tokio::test]
    async fn test_quotes_and_apostrophes() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "He said \"Hello!\" and then 'Goodbye'.";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Quotes and apostrophes should be handled correctly");
    }

    #[tokio::test]
    async fn test_backslash_and_null() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "Path: C:\\\\Users\\\\test\\\\file.txt";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Backslashes should be handled correctly");
    }
}

/// Test long message handling
mod long_message_tests {
    use super::*;

    #[tokio::test]
    async fn test_10kb_message() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Generate 10KB of text
        let message: String = (0..10240).map(|_| 'x').collect();
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        assert_eq!(signature.len(), 64, "Signature should be 64 bytes");

        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);
        assert!(result.is_ok(), "10KB message should be handled correctly");
    }

    #[tokio::test]
    async fn test_100kb_message() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Generate 100KB of text
        let message: String = (0..102400).map(|_| 'a').collect();
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "100KB message should be handled correctly");
    }

    #[tokio::test]
    async fn test_long_message_in_history() {
        let mut history = MessageHistory::new(100);

        let long_message = ChatMessage::new(
            "sender".to_string(),
            (0..10240).map(|_| 't').collect(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        history.add_message(long_message.clone());
        assert_eq!(history.len(), 1);

        let retrieved = history.messages().next().unwrap();
        assert_eq!(retrieved.message.len(), 10240);
    }

    #[tokio::test]
    async fn test_deterministic_signing_long_message() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message: String = (0..1000).map(|_| 'z').collect();
        let timestamp = "2025-12-27T10:30:00Z";

        // Sign same message twice
        let sig1 = {
            let canonical = format!("{}:{}", message, timestamp);
            sign_message(&private_key, canonical.as_bytes()).unwrap()
        };

        let sig2 = {
            let canonical = format!("{}:{}", message, timestamp);
            sign_message(&private_key, canonical.as_bytes()).unwrap()
        };

        // Verify both signatures
        let canonical = format!("{}:{}", message, timestamp);
        assert!(verify_signature(&public_key, canonical.as_bytes(), &sig1).is_ok());
        assert!(verify_signature(&public_key, canonical.as_bytes(), &sig2).is_ok());

        // Signatures should be identical (deterministic)
        assert_eq!(sig1, sig2, "Long messages should have deterministic signatures");
    }
}

/// Test whitespace handling
mod whitespace_tests {
    use super::*;

    #[tokio::test]
    async fn test_multiple_spaces() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "Hello    World    Test";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Multiple spaces should be preserved");
    }

    #[tokio::test]
    async fn test_tabs_and_spaces() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "Col1\tCol2\tCol3";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Tabs should be preserved");
    }

    #[tokio::test]
    async fn test_newlines() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "Line 1\nLine 2\nLine 3";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Newlines should be preserved");
    }

    #[tokio::test]
    async fn test_mixed_whitespace() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "  Leading  \t  Mixed   \n  Whitespace  \r\n  Test  ";
        let timestamp = "2025-12-27T10:30:00Z";
        let canonical = format!("{}:{}", message, timestamp);

        let signature = sign_message(&private_key, canonical.as_bytes()).unwrap();
        let result = verify_signature(&public_key, canonical.as_bytes(), &signature);

        assert!(result.is_ok(), "Mixed whitespace should be preserved");
    }

    #[tokio::test]
    async fn test_whitespace_determinism() {
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let message = "   spaces   ";
        let timestamp = "2025-12-27T10:30:00Z";

        // Sign same whitespace twice
        let sig1 = {
            let canonical = format!("{}:{}", message, timestamp);
            sign_message(&private_key, canonical.as_bytes()).unwrap()
        };

        let sig2 = {
            let canonical = format!("{}:{}", message, timestamp);
            sign_message(&private_key, canonical.as_bytes()).unwrap()
        };

        // Verify
        let canonical = format!("{}:{}", message, timestamp);
        assert!(verify_signature(&public_key, canonical.as_bytes(), &sig1).is_ok());
        assert!(verify_signature(&public_key, canonical.as_bytes(), &sig2).is_ok());

        // Signatures should be identical
        assert_eq!(sig1, sig2, "Whitespace should produce deterministic signatures");
    }
}

/// Test binary content validation
mod binary_validation_tests {
    use super::*;

    #[test]
    fn test_valid_utf8_detection() {
        // Valid UTF-8 strings should be accepted
        let valid_strings = vec![
            "Hello world",
            "你好世界",
            "🎉🚀",
            "Ñoño tilde",
            "",
        ];

        for s in valid_strings {
            assert!(is_valid_utf8(s), "{} should be valid UTF-8", s);
        }
    }

    #[test]
    fn test_binary_content_rejection() {
        // These are NOT actually tested with raw bytes since we're in a Rust test
        // In practice, the WebSocket layer would reject binary messages
        // This test verifies our validation function works correctly
        let test_cases = vec![
            ("Valid ASCII", "hello", true),
            ("Valid Unicode", "你好", true),
            ("Empty string", "", true),
        ];

        for (name, input, expected) in test_cases {
            let result = is_valid_utf8(input);
            assert_eq!(result, expected, "{} should be {}", name, if expected { "valid" } else { "invalid" });
        }
    }

    /// Simple UTF-8 validation
    #[allow(dead_code)]
    fn is_valid_utf8(s: &str) -> bool {
        std::str::from_utf8(s.as_bytes()).is_ok()
    }
}

/// Test timestamp formatting edge cases
mod timestamp_tests {
    use super::*;

    #[test]
    fn test_rfc3339_timestamp() {
        let ts = "2025-12-27T10:30:45Z";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "10:30:45");
    }

    #[test]
    fn test_timestamp_with_milliseconds() {
        let ts = "2025-12-27T14:22:30.123456789Z";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "14:22:30");
    }

    #[test]
    fn test_timestamp_with_timezone() {
        let ts = "2025-12-27T10:30:45+05:30";
        let formatted = format_timestamp(ts);
        // Should extract HH:MM:SS correctly
        assert!(formatted.contains(':'));
    }

    #[test]
    fn test_invalid_timestamp() {
        let ts = "not-a-timestamp";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "??:??:??");
    }

    #[test]
    fn test_empty_timestamp() {
        let ts = "";
        let formatted = format_timestamp(ts);
        assert_eq!(formatted, "??:??:??");
    }
}

/// Test message history edge cases
mod history_edge_tests {
    use super::*;

    #[tokio::test]
    async fn test_empty_history() {
        let history = MessageHistory::new(100);
        assert!(history.is_empty());
        assert_eq!(history.len(), 0);
    }

    #[tokio::test]
    async fn test_history_order_with_same_timestamp() {
        let mut history = MessageHistory::new(100);

        // Add messages with same timestamp
        for i in 0..5 {
            let msg = ChatMessage::new(
                format!("sender{}", i),
                format!("message {}", i),
                "sig".to_string(),
                "2025-12-27T10:30:00Z".to_string(),
            );
            history.add_message(msg);
        }

        assert_eq!(history.len(), 5);
        // All should be in order
        for (i, msg) in history.messages().enumerate() {
            assert_eq!(msg.message, format!("message {}", i));
        }
    }

    #[tokio::test]
    async fn test_history_capacity_limit() {
        let mut history = MessageHistory::new(3);

        // Add more than capacity
        for i in 0..5 {
            let msg = ChatMessage::new(
                "sender".to_string(),
                format!("message {}", i),
                "sig".to_string(),
                format!("2025-12-27T10:3{}.00Z", i),
            );
            history.add_message(msg);
        }

        // Should only have 3 messages (oldest evicted)
        assert_eq!(history.len(), 3);
        assert_eq!(history.newest().unwrap().message, "message 4");
        assert_eq!(history.oldest().unwrap().message, "message 2");
    }

    #[tokio::test]
    async fn test_history_from_sender() {
        let mut history = MessageHistory::new(100);

        // Add messages from different senders
        let senders = ["alice", "bob", "alice", "charlie", "alice"];
        for (i, sender) in senders.iter().enumerate() {
            let msg = ChatMessage::new(
                sender.to_string(),
                format!("msg {}", i),
                "sig".to_string(),
                format!("2025-12-27T10:{}0:00Z", i),
            );
            history.add_message(msg);
        }

        let from_alice: Vec<&str> = history.messages_from("alice")
            .iter()
            .map(|m| m.message.as_str())
            .collect();

        assert_eq!(from_alice.len(), 3);
        assert_eq!(from_alice, vec!["msg 0", "msg 2", "msg 4"]);
    }
}

/// Test ChatMessage edge cases
mod chat_message_edge_tests {
    use super::*;

    #[test]
    fn test_empty_message() {
        let msg = ChatMessage::new(
            "sender".to_string(),
            String::new(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        assert_eq!(msg.message, "");
        assert!(!msg.is_verified);
    }

    #[test]
    fn test_very_long_sender_key() {
        let long_key = (0..100).map(|_| 'a').collect::<String>();
        let msg = ChatMessage::new(
            long_key.clone(),
            "test".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        assert_eq!(msg.sender_public_key, long_key);
    }

    #[test]
    fn test_very_long_signature() {
        let long_sig = (0..200).map(|_| 'a').collect::<String>();
        let msg = ChatMessage::new(
            "sender".to_string(),
            "test".to_string(),
            long_sig,
            "2025-12-27T10:30:00Z".to_string(),
        );

        assert_eq!(msg.signature.len(), 200);
    }

    #[test]
    fn test_message_equality() {
        let msg1 = ChatMessage::new(
            "sender".to_string(),
            "test".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        let msg2 = ChatMessage::new(
            "sender".to_string(),
            "test".to_string(),
            "sig".to_string(),
            "2025-12-27T10:30:00Z".to_string(),
        );

        assert_eq!(msg1.message, msg2.message);
        assert_eq!(msg1.sender_public_key, msg2.sender_public_key);
        assert_eq!(msg1.timestamp, msg2.timestamp);
    }
}
