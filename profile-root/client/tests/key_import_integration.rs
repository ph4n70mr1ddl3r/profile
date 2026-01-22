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
    let hex_input = hex::encode(original_key.as_slice());

    // Launch multiple concurrent import operations
    let mut tasks = vec![];
    for _ in 0..10 {
        let hex_clone = hex_input.clone();
        let task = tokio::spawn(async move {
            let decoded_key =
                Zeroizing::new(hex::decode(&hex_clone).expect("Hex decode should succeed"));
            let private_key = profile_shared::PrivateKey::new(decoded_key.to_vec());
            profile_shared::derive_public_key(&private_key).expect("Derivation should succeed")
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
            public_keys[0].as_bytes(),
            public_keys[i].as_bytes(),
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
    let lowercase_hex = hex::encode(original_key.as_slice());

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
        "Case-insensitive hex should decode to same bytes"
    );

    // Derive public keys - both should be identical
    let private_lower = profile_shared::PrivateKey::new(decoded_lower.to_vec());
    let private_upper = profile_shared::PrivateKey::new(decoded_upper.to_vec());
    let public_lower =
        profile_shared::derive_public_key(&private_lower).expect("Derivation should succeed");
    let public_upper =
        profile_shared::derive_public_key(&private_upper).expect("Derivation should succeed");

    assert_eq!(
        public_lower.as_bytes(),
        public_upper.as_bytes(),
        "Case insensitive import should produce same public key"
    );
}

/// Test that imported keys can be used with Arc<Mutex> (async pattern)
#[tokio::test]
async fn integration_test_import_with_async_state() {
    let original_key =
        profile_shared::generate_private_key().expect("Key generation should succeed");
    let hex_input = hex::encode(original_key.as_slice());

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
        let private_key = profile_shared::PrivateKey::new(guard.to_vec());
        profile_shared::derive_public_key(&private_key).expect("Derivation should succeed")
    };

    assert_eq!(
        public_key.as_bytes().len(),
        32,
        "Public key should be 32 bytes"
    );
}
