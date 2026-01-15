use hex;
use profile_shared::sign_message;
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

/// Client authentication message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientAuthMessage {
    pub r#type: String,
    #[serde(rename = "publicKey")]
    pub public_key: String,
    pub signature: String,
}

impl ClientAuthMessage {
    /// Create a new client authentication message
    pub fn new(
        public_key: Vec<u8>,
        private_key: Zeroizing<Vec<u8>>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Generate signature for "auth" message
        let signature = sign_message(&private_key, b"auth")?;

        // Encode to hex
        let public_key_hex = hex::encode(&public_key);
        let signature_hex = hex::encode(signature);

        Ok(Self {
            r#type: "auth".to_string(),
            public_key: public_key_hex,
            signature: signature_hex,
        })
    }

    /// Serialize to JSON string
    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::to_string(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use profile_shared::derive_public_key;
    use profile_shared::generate_private_key;

    #[tokio::test]
    async fn test_client_auth_message_creation() {
        // Test creating a client auth message with valid keys
        // This should work now - we're in GREEN phase

        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // This should work now that we have the implementation
        let result = ClientAuthMessage::new(public_key.clone(), private_key);

        // In GREEN phase, this should succeed
        assert!(
            result.is_ok(),
            "Client auth message creation should work: {:?}",
            result.err()
        );

        let auth_msg = result.unwrap();
        assert_eq!(auth_msg.r#type, "auth");
        assert_eq!(auth_msg.public_key, hex::encode(public_key));
        assert!(!auth_msg.signature.is_empty());

        println!("✅ Client auth message created successfully");
    }

    #[tokio::test]
    async fn test_client_auth_message_json_serialization() {
        // Test JSON serialization of client auth message

        // This test will demonstrate the expected behavior once implemented
        let result = test_auth_message_json().await;

        // This should pass once we have the minimal implementation
        assert!(
            result.is_ok(),
            "Auth message JSON serialization should work: {:?}",
            result.err()
        );
    }

    async fn test_auth_message_json() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create a test auth message - this should work once implemented

        // 1. Generate keys
        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // 2. Create auth message
        let auth_msg = ClientAuthMessage::new(public_key.clone(), private_key)?;

        // 3. Serialize to JSON
        let json = auth_msg.to_json()?;

        // Verify JSON structure
        let parsed: ClientAuthMessage = serde_json::from_str(&json)?;
        assert_eq!(parsed.r#type, "auth");
        assert_eq!(parsed.public_key, hex::encode(public_key));
        assert!(!parsed.signature.is_empty());

        println!("✅ JSON serialization works correctly");
        Ok(())
    }

    #[tokio::test]
    async fn test_signature_determinism() {
        // Test that the same inputs produce the same signature (deterministic)

        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        // Create two auth messages with same keys
        let msg1 = ClientAuthMessage::new(public_key.clone(), private_key.clone()).unwrap();
        let msg2 = ClientAuthMessage::new(public_key.clone(), private_key).unwrap();

        // Signatures should be identical (deterministic signing)
        assert_eq!(msg1.signature, msg2.signature);
        assert_eq!(msg1.public_key, msg2.public_key);

        println!("✅ Signature determinism verified");
    }

    #[tokio::test]
    async fn test_different_keys_different_signatures() {
        // Test that different key pairs produce different signatures

        let private_key1 = generate_private_key().unwrap();
        let public_key1 = derive_public_key(&private_key1).unwrap();

        let private_key2 = generate_private_key().unwrap();
        let public_key2 = derive_public_key(&private_key2).unwrap();

        // Create auth messages with different keys
        let msg1 = ClientAuthMessage::new(public_key1, private_key1).unwrap();
        let msg2 = ClientAuthMessage::new(public_key2, private_key2).unwrap();

        // Should have different public keys and signatures
        assert_ne!(msg1.public_key, msg2.public_key);
        assert_ne!(msg1.signature, msg2.signature);

        println!("✅ Different keys produce different signatures");
    }

    #[tokio::test]
    async fn test_hex_encoding_format() {
        // Test that hex encoding produces valid hex strings

        let private_key = generate_private_key().unwrap();
        let public_key = derive_public_key(&private_key).unwrap();

        let auth_msg = ClientAuthMessage::new(public_key, private_key).unwrap();

        // Verify hex encoding format (64 chars for 32-byte keys/signatures)
        assert_eq!(auth_msg.public_key.len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(auth_msg.signature.len(), 128); // 64 bytes = 128 hex chars

        // Verify hex strings contain only valid hex characters
        assert!(auth_msg.public_key.chars().all(|c| c.is_ascii_hexdigit()));
        assert!(auth_msg.signature.chars().all(|c| c.is_ascii_hexdigit()));

        println!("✅ Hex encoding format is correct");
    }
}
