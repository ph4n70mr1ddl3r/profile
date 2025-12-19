//! Key import handler - validates and imports user-provided private keys

use crate::state::SharedKeyState;
use profile_shared::{derive_public_key, PrivateKey};
use zeroize::Zeroizing;

/// Handle the "Import Key" button press
/// 
/// Validates user input through 10 steps, then imports the key into session state.
/// Returns the derived public key as hex for UI display.
/// 
/// # Validation Steps
/// 1. Trim whitespace (users paste with trailing newlines)
/// 2. Check for empty input (better error message)
/// 3. Check length BEFORE decode (64 hex chars = 32 bytes)
/// 4. Check content (only 0-9, a-f, A-F allowed)
/// 5. Decode hex to bytes
/// 6. Validate decoded length (defense in depth)
/// 7. Check for degenerate keys (all zeros)
/// 8. Wrap in zeroize-protected container
/// 9. Verify key derivation works (calls derive_public_key)
/// 10. Verify public key doesn't equal private key (sanity check)
pub async fn handle_import_key(
    key_state: &SharedKeyState,
    user_input: String,
) -> Result<String, String> {
    // Step 1: Trim whitespace
    let trimmed = user_input.trim();
    
    // Step 2a: Check for empty input (better error message)
    if trimmed.is_empty() {
        return Err("No private key entered. Please paste your 64-character hexadecimal key.".into());
    }
    
    // Step 2b: Check length BEFORE attempting decode
    if trimmed.len() != 64 {
        return Err(format!(
            "Expected 64 hexadecimal characters (256 bits), received {}. Example: 3a8f2e1c9b4d6f7a...",
            trimmed.len()
        ));
    }
    
    // Step 3: Check content (only 0-9, a-f, A-F)
    if !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(
            "Invalid characters found. Only hexadecimal characters (0-9, a-f) are allowed.".into()
        );
    }
    
    // Step 4: Decode hex to bytes
    let key_bytes = hex::decode(trimmed)
        .map_err(|e| format!("Failed to decode hexadecimal: {}", e))?;
    
    // Step 5: Validate decoded length (defense in depth)
    if key_bytes.len() != 32 {
        return Err(format!(
            "Invalid key length after decoding: {} bytes (expected 32 bytes)",
            key_bytes.len()
        ));
    }
    
    // Step 6: Check for degenerate keys (all zeros)
    if key_bytes.iter().all(|&b| b == 0) {
        return Err("All-zero keys are not cryptographically valid. Please use a different key.".into());
    }
    
    // Wrap in zeroize-protected container (security step before validation)
    let private_key: PrivateKey = Zeroizing::new(key_bytes);
    
    // Step 7: Verify key derivation works (validates key is usable)
    let public_key = derive_public_key(&private_key)
        .map_err(|e| format!("Cannot derive public key from this private key: {}", e))?;
    
    // Defense-in-depth: Verify public key doesn't equal private key (same check as Story 1.1)
    if public_key.as_slice() == private_key.as_slice() {
        return Err(
            "Invalid key: Public key cannot equal private key. This indicates a cryptographic error.".into()
        );
    }
    
    // Convert to hex for display
    let public_key_hex = hex::encode(&public_key);
    
    // Defense-in-depth: Validate hex encoding (paranoid checks that should never fail)
    // These checks defend against hypothetical bugs in derive_public_key() or hex::encode()
    // In practice: derive_public_key() always returns 32 bytes, hex::encode() always produces valid hex
    // Kept for consistency with Story 1.1 (session.rs) and defense-in-depth security principle
    if public_key_hex.len() != 64 {
        return Err(format!(
            "Internal validation error: Public key hex length is {} (expected 64). This indicates a bug in key derivation.",
            public_key_hex.len()
        ));
    }
    
    if !public_key_hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Internal validation error: Public key contains non-hex characters. This indicates a bug in hex encoding.".into());
    }
    
    if public_key_hex.chars().all(|c| c == '0') {
        return Err("Internal validation error: Public key is all zeros (should have been caught earlier). This indicates a bug.".into());
    }
    
    // Lock the state and store the imported key
    let mut state = key_state.lock().await;
    state.set_generated_key(private_key, public_key);
    
    Ok(public_key_hex)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::create_shared_key_state;
    use profile_shared::generate_private_key;

    #[tokio::test]
    async fn test_import_valid_key_success() {
        let key_state = create_shared_key_state();
        
        // Generate a valid key to use for testing
        let private_key = generate_private_key().unwrap();
        let private_key_hex = hex::encode(&*private_key);
        
        let result = handle_import_key(&key_state, private_key_hex).await;
        
        assert!(result.is_ok(), "Valid key should import successfully");
        let public_key_hex = result.unwrap();
        assert_eq!(public_key_hex.len(), 64, "Public key should be 64 hex chars");
        
        // Verify state was updated
        let state = key_state.lock().await;
        assert!(state.is_key_set(), "Key should be stored in state");
    }

    #[tokio::test]
    async fn test_import_rejects_short_key() {
        let key_state = create_shared_key_state();
        let short_key = "abc123";  // Way too short
        
        let result = handle_import_key(&key_state, short_key.to_string()).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("64 hexadecimal characters"), "Error should mention expected length");
        assert!(err.contains("received 6"), "Error should show actual length");
    }

    #[tokio::test]
    async fn test_import_rejects_long_key() {
        let key_state = create_shared_key_state();
        let long_key = "a".repeat(100);  // Too long
        
        let result = handle_import_key(&key_state, long_key).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("64 hexadecimal characters"));
        assert!(err.contains("received 100"));
    }

    #[tokio::test]
    async fn test_import_rejects_invalid_characters() {
        let key_state = create_shared_key_state();
        // 64 chars but with invalid characters
        let invalid_key = "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz";
        
        let result = handle_import_key(&key_state, invalid_key.to_string()).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("hexadecimal"), "Error should mention hexadecimal requirement");
    }

    #[tokio::test]
    async fn test_import_rejects_all_zero_key() {
        let key_state = create_shared_key_state();
        let zero_key = "0".repeat(64);
        
        let result = handle_import_key(&key_state, zero_key).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("All-zero"), "Error should mention all-zero keys are invalid");
    }

    #[tokio::test]
    async fn test_import_handles_whitespace() {
        let key_state = create_shared_key_state();
        
        // Generate valid key and add whitespace
        let private_key = generate_private_key().unwrap();
        let private_key_hex = hex::encode(&*private_key);
        let with_whitespace = format!("  {}  \n", private_key_hex);
        
        let result = handle_import_key(&key_state, with_whitespace).await;
        
        assert!(result.is_ok(), "Whitespace should be trimmed automatically");
    }

    #[tokio::test]
    async fn test_import_accepts_uppercase_hex() {
        let key_state = create_shared_key_state();
        
        // Generate valid key and uppercase it
        let private_key = generate_private_key().unwrap();
        let private_key_hex = hex::encode(&*private_key).to_uppercase();
        
        let result = handle_import_key(&key_state, private_key_hex).await;
        
        assert!(result.is_ok(), "Uppercase hex should be accepted");
    }

    #[tokio::test]
    async fn test_import_deterministic_public_key() {
        let key_state1 = create_shared_key_state();
        let key_state2 = create_shared_key_state();
        
        // Import same key twice
        let private_key = generate_private_key().unwrap();
        let private_key_hex = hex::encode(&*private_key);
        
        let result1 = handle_import_key(&key_state1, private_key_hex.clone()).await;
        let result2 = handle_import_key(&key_state2, private_key_hex).await;
        
        assert_eq!(
            result1.unwrap(),
            result2.unwrap(),
            "Importing same key should produce same public key (deterministic)"
        );
    }

    #[tokio::test]
    async fn test_import_does_not_leak_key_in_error() {
        let key_state = create_shared_key_state();
        let invalid_key = "invalid_but_64_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
        
        let result = handle_import_key(&key_state, invalid_key.to_string()).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        // Ensure error message doesn't contain the user's input
        assert!(
            !err.contains(invalid_key),
            "Error message should not leak user's key input for security"
        );
    }

    #[tokio::test]
    async fn test_import_rejects_empty_input() {
        let key_state = create_shared_key_state();
        let empty = "    \n\n   ";  // Whitespace only
        
        let result = handle_import_key(&key_state, empty.to_string()).await;
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("No private key entered"),
            "Should have specific empty input message, got: {}",
            err
        );
    }
}
