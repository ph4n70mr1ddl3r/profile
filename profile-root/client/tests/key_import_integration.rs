//! Integration tests for Story 1.2: Import Existing 256-Bit Private Key
//!
//! Tests the complete end-to-end flow at the crypto level:
//! 1. Simulate user importing a hex-encoded private key
//! 2. Validate hex decoding
//! 3. Derive the public key
//! 4. Verify correctness and properties

use std::sync::Arc;
use tokio::sync::Mutex;
use zeroize::Zeroizing;

/// Test the complete import flow: hex decode -> derive public key
#[tokio::test]
async fn integration_test_import_hex_and_derive() {
    // Simulate user pasting a valid hex key
    let original_key =
        profile_shared::generate_private_key().expect("Key generation should succeed");
    let hex_input = hex::encode(&*original_key);

    // Simulate handler validation: decode hex
    let decoded_key =
        Zeroizing::new(hex::decode(&hex_input).expect("Hex decode should succeed for valid input"));

    // Verify decoded length
    assert_eq!(decoded_key.len(), 32, "Decoded key must be 32 bytes");

    // Derive public key (same as generation path)
    let public_key =
        profile_shared::derive_public_key(&decoded_key).expect("Derivation should succeed");

    // Verify properties
    assert_eq!(public_key.len(), 32, "Public key must be 32 bytes");
    assert_ne!(
        &decoded_key[..],
        &public_key[..],
        "Keys should be different"
    );

    // Convert to hex for display
    let public_key_hex = hex::encode(&public_key);
    assert_eq!(
        public_key_hex.len(),
        64,
        "Hex encoding of 32 bytes must be 64 characters"
    );
}

/// Test that importing a key produces the same result as generating it (Story 1.1 compatibility)
#[tokio::test]
async fn integration_test_import_matches_generation() {
    // Generate a key (Story 1.1 path)
    let generated_key =
        profile_shared::generate_private_key().expect("Key generation should succeed");
    let generated_public =
        profile_shared::derive_public_key(&generated_key).expect("Derivation should succeed");

    // Import the same key (Story 1.2 path: hex encode -> decode -> derive)
    let hex_input = hex::encode(&*generated_key);
    let imported_key = Zeroizing::new(hex::decode(&hex_input).expect("Hex decode should succeed"));
    let imported_public =
        profile_shared::derive_public_key(&imported_key).expect("Derivation should succeed");

    // Both paths should produce identical public keys
    assert_eq!(
        &generated_public[..],
        &imported_public[..],
        "Import and generation should produce identical public keys for the same private key"
    );
}

/// Test error cases: invalid hex strings should fail gracefully
#[tokio::test]
async fn integration_test_import_validation_errors() {
    // Too short
    let short_hex = "abc123";
    assert!(
        hex::decode(short_hex).is_err() || hex::decode(short_hex).unwrap().len() != 32,
        "Short hex should fail validation"
    );

    // Too long
    let long_hex = "a".repeat(100);
    let decoded = hex::decode(&long_hex).expect("Should decode");
    assert_ne!(decoded.len(), 32, "Long hex should not be 32 bytes");

    // Invalid characters
    let invalid_hex = "xyz123invalid";
    assert!(
        hex::decode(invalid_hex).is_err(),
        "Invalid hex characters should fail"
    );

    // All zeros (degenerate key - would pass hex decode but fail crypto validation)
    let zero_hex = "0".repeat(64);
    let zero_key = hex::decode(&zero_hex).expect("Should decode zeros");
    assert_eq!(zero_key.len(), 32, "Should be correct length");
    assert!(
        zero_key.iter().all(|&b| b == 0),
        "Should be all zeros (handler must reject this)"
    );
}

/// Test import performance: hex decode + derive should be fast
#[tokio::test]
async fn integration_test_import_performance() {
    use std::time::Duration;
    use std::time::Instant;

    let original_key =
        profile_shared::generate_private_key().expect("Key generation should succeed");
    let hex_input = hex::encode(&*original_key);

    let mut max_elapsed = Duration::from_millis(0);
    for _ in 0..10 {
        let start = Instant::now();

        // Simulate import: decode + derive
        let decoded_key =
            Zeroizing::new(hex::decode(&hex_input).expect("Hex decode should succeed"));
        let _public_key =
            profile_shared::derive_public_key(&decoded_key).expect("Derivation should succeed");

        let elapsed = start.elapsed();
        if elapsed > max_elapsed {
            max_elapsed = elapsed;
        }
    }

    // Should be fast (<100ms) since there's no blocking I/O
    assert!(
        max_elapsed < Duration::from_millis(100),
        "Import (decode+derive) must be <100ms (max was {:?})",
        max_elapsed
    );
}

/// Test concurrent import operations are safe
#[tokio::test]
async fn integration_test_concurrent_import_safe() {
    let original_key =
        profile_shared::generate_private_key().expect("Key generation should succeed");
    let hex_input = hex::encode(&*original_key);

    // Launch multiple concurrent import operations
    let mut tasks = vec![];
    for _ in 0..10 {
        let hex_clone = hex_input.clone();
        let task = tokio::spawn(async move {
            let decoded_key =
                Zeroizing::new(hex::decode(&hex_clone).expect("Hex decode should succeed"));
            profile_shared::derive_public_key(&decoded_key).expect("Derivation should succeed")
        });
        tasks.push(task);
    }

    // All should complete successfully
    let mut public_keys = vec![];
    for task in tasks {
        let public_key = task.await.expect("Task should complete");
        public_keys.push(public_key);
    }

    // All should produce the same public key (deterministic)
    for i in 1..public_keys.len() {
        assert_eq!(
            &public_keys[0][..],
            &public_keys[i][..],
            "All concurrent imports should produce the same result"
        );
    }
}

/// Test hex encoding is case-insensitive for import
#[tokio::test]
async fn integration_test_case_insensitive_import() {
    let original_key =
        profile_shared::generate_private_key().expect("Key generation should succeed");

    // Encode in lowercase
    let lowercase_hex = hex::encode(&*original_key);

    // Convert to uppercase
    let uppercase_hex = lowercase_hex.to_uppercase();

    // Both should decode to the same key
    let decoded_lower =
        Zeroizing::new(hex::decode(&lowercase_hex).expect("Lowercase should decode"));
    let decoded_upper =
        Zeroizing::new(hex::decode(&uppercase_hex).expect("Uppercase should decode"));

    assert_eq!(
        &decoded_lower[..],
        &decoded_upper[..],
        "Case should not matter for hex decoding"
    );

    // Both should derive the same public key
    let public_lower =
        profile_shared::derive_public_key(&decoded_lower).expect("Derivation should succeed");
    let public_upper =
        profile_shared::derive_public_key(&decoded_upper).expect("Derivation should succeed");

    assert_eq!(
        &public_lower[..],
        &public_upper[..],
        "Case insensitive import should produce same public key"
    );
}

/// Test that imported keys can be used with Arc<Mutex> (async pattern)
#[tokio::test]
async fn integration_test_import_with_async_state() {
    let original_key =
        profile_shared::generate_private_key().expect("Key generation should succeed");
    let hex_input = hex::encode(&*original_key);

    // Decode
    let decoded_key = Zeroizing::new(hex::decode(&hex_input).expect("Hex decode should succeed"));

    // Store in async-safe state (Arc<Mutex> pattern used in client)
    let key_state = Arc::new(Mutex::new(decoded_key));

    // Verify we can lock and access it
    {
        let guard = key_state.lock().await;
        assert_eq!(guard.len(), 32, "Key should be 32 bytes");
    }

    // Derive public key from locked state
    let public_key = {
        let guard = key_state.lock().await;
        profile_shared::derive_public_key(&*guard).expect("Derivation should succeed")
    };

    assert_eq!(public_key.len(), 32, "Public key should be 32 bytes");
}
