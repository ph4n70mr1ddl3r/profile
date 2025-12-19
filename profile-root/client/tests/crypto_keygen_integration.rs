//! Integration tests for Story 1.1: Generate New 256-Bit Private Key
//!
//! Tests the complete end-to-end flow:
//! 1. Generate a new private key
//! 2. Derive the public key
//! 3. Store both in session state
//! 4. Verify correctness and properties

use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn integration_test_generate_and_store_key() {
    // Simulate "Generate New Key" button click
    let private_key = profile_shared::generate_private_key()
        .expect("Key generation should succeed");

    // Derive public key
    let public_key = profile_shared::derive_public_key(&private_key)
        .expect("Derivation should succeed");

    // Verify properties
    assert_eq!(private_key.len(), 32, "Private key must be 32 bytes");
    assert_eq!(public_key.len(), 32, "Public key must be 32 bytes");
    assert_ne!(&private_key[..], &public_key[..], "Keys should be different");

    // Convert to hex for display
    let public_key_hex = hex::encode(&public_key);
    assert_eq!(
        public_key_hex.len(),
        64,
        "Hex encoding of 32 bytes must be 64 characters"
    );

    // Verify keys can be used with Arc<Mutex> (async pattern)
    let key_state = Arc::new(Mutex::new(private_key));
    let _ = key_state.lock().await; // Should not panic
}

#[tokio::test]
async fn integration_test_multiple_key_generations_are_unique() {
    // Generate 100 keys and verify all are unique
    let mut keys = vec![];

    for _ in 0..100 {
        let key = profile_shared::generate_private_key()
            .expect("Key generation should always succeed");
        keys.push(key);
    }

    // Verify all keys are unique
    for i in 0..keys.len() {
        for j in (i + 1)..keys.len() {
            assert_ne!(
                &keys[i][..],
                &keys[j][..],
                "All generated keys must be unique (collision at {}:{})",
                i,
                j
            );
        }
    }
}

#[tokio::test]
async fn integration_test_derivation_is_deterministic() {
    // Generate a key once
    let private_key = profile_shared::generate_private_key()
        .expect("Key generation should succeed");

    // Derive public key 100 times
    let mut derived_keys = vec![];
    for _ in 0..100 {
        let public_key = profile_shared::derive_public_key(&private_key)
            .expect("Derivation should always succeed");
        derived_keys.push(public_key);
    }

    // All derivations must be identical
    for i in 1..derived_keys.len() {
        assert_eq!(
            &derived_keys[0][..],
            &derived_keys[i][..],
            "Derivation must be deterministic (failed at iteration {})",
            i
        );
    }
}

#[tokio::test]
async fn integration_test_performance_under_100ms() {
    use std::time::Instant;
    use std::time::Duration;

    let mut max_elapsed = Duration::from_millis(0);
    for _ in 0..10 {
        let start = Instant::now();
        let private_key = profile_shared::generate_private_key()
            .expect("Key generation should succeed");

        let _public_key = profile_shared::derive_public_key(&private_key)
            .expect("Derivation should succeed");

        let elapsed = start.elapsed();
        if elapsed > max_elapsed {
            max_elapsed = elapsed;
        }
    }

    assert!(
        max_elapsed < Duration::from_millis(100),
        "Key generation+derivation must be <100ms per key (max was {:?})",
        max_elapsed
    );
}

#[tokio::test]
async fn integration_test_async_concurrent_generation() {
    // Spawn 10 concurrent async tasks, each generating a key
    let mut tasks = vec![];

    for _ in 0..10 {
        let task = tokio::spawn(async {
            profile_shared::generate_private_key()
                .and_then(|private| {
                    profile_shared::derive_public_key(&private)?;
                    Ok(private)
                })
                .expect("Should succeed")
        });
        tasks.push(task);
    }

    // Wait for all tasks
    let mut keys = vec![];
    for task in tasks {
        let key = task.await.expect("Task should complete");
        keys.push(key);
    }

    // All keys should be unique
    for i in 0..keys.len() {
        for j in (i + 1)..keys.len() {
            assert_ne!(
                &keys[i][..],
                &keys[j][..],
                "Concurrent keys should all be unique"
            );
        }
    }

}

#[tokio::test]
async fn integration_test_hex_encoding_roundtrip() {
    let private_key = profile_shared::generate_private_key()
        .expect("Key generation should succeed");

    let public_key = profile_shared::derive_public_key(&private_key)
        .expect("Derivation should succeed");

    // Encode to hex
    let hex_string = hex::encode(&public_key);

    // Verify hex format
    assert_eq!(hex_string.len(), 64, "Hex string should be 64 characters");
    assert!(
        hex_string.chars().all(|c| c.is_ascii_hexdigit()),
        "All characters should be valid hex"
    );

    // Decode back to bytes
    let decoded_vec = hex::decode(&hex_string)
        .expect("Should decode successfully");
    
    assert_eq!(public_key, decoded_vec, "Roundtrip should match");
}
