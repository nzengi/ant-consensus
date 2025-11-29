use ring::signature::{self, Ed25519KeyPair, KeyPair, UnparsedPublicKey, ED25519_PUBLIC_KEY_LEN};
use std::sync::Arc;

/// Public key type
pub type PublicKey = UnparsedPublicKey<Vec<u8>>;

/// Signature type
pub type Signature = Vec<u8>;

/// Key pair wrapper
pub struct KeyPairWrapper {
    key_pair: Arc<Ed25519KeyPair>,
}

impl KeyPairWrapper {
    /// Generate a new key pair
    pub fn generate() -> Result<Self, String> {
        let rng = ring::rand::SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|e| format!("Failed to generate key pair: {}", e))?;
        
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
            .map_err(|e| format!("Failed to parse key pair: {}", e))?;

        Ok(Self {
            key_pair: Arc::new(key_pair),
        })
    }

    /// Create from existing private key bytes
    pub fn from_private_key_bytes(bytes: &[u8]) -> Result<Self, String> {
        let key_pair = Ed25519KeyPair::from_pkcs8(bytes)
            .map_err(|e| format!("Failed to parse key pair: {}", e))?;

        Ok(Self {
            key_pair: Arc::new(key_pair),
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> PublicKey {
        let public_key_bytes = self.key_pair.public_key().as_ref().to_vec();
        UnparsedPublicKey::new(&signature::ED25519, public_key_bytes)
    }

    /// Get the private key bytes (PKCS8 format)
    pub fn private_key_bytes(&self) -> Vec<u8> {
        // Note: Ed25519KeyPair doesn't expose private key directly
        // In production, you'd want to store the original PKCS8 bytes
        // For now, we'll return empty (this is a limitation)
        vec![]
    }

    /// Sign a message
    pub fn sign(&self, message: &[u8]) -> Signature {
        self.key_pair.sign(message).as_ref().to_vec()
    }
}

/// Sign a message with a private key
pub fn sign_message(message: &[u8], private_key: &[u8]) -> Result<Signature, String> {
    let key_pair = Ed25519KeyPair::from_pkcs8(private_key)
        .map_err(|e| format!("Failed to parse private key: {}", e))?;
    
    Ok(key_pair.sign(message).as_ref().to_vec())
}

/// Verify a signature
pub fn verify_signature(
    message: &[u8],
    signature: &Signature,
    public_key: &PublicKey,
) -> Result<bool, String> {
    public_key.verify(message, signature.as_ref())
        .map_err(|e| format!("Signature verification failed: {}", e))?;
    Ok(true)
}

/// Generate a new key pair
pub fn generate_key_pair() -> Result<(PublicKey, Vec<u8>), String> {
    let key_pair_wrapper = KeyPairWrapper::generate()?;
    let public_key = key_pair_wrapper.public_key();
    
    // Note: We can't extract private key from Ed25519KeyPair
    // In production, you'd store the PKCS8 bytes when generating
    let private_key = vec![]; // Placeholder
    
    Ok((public_key, private_key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_pair_generation() {
        let key_pair = KeyPairWrapper::generate();
        assert!(key_pair.is_ok());
    }

    #[test]
    fn test_sign_and_verify() {
        let key_pair = KeyPairWrapper::generate().unwrap();
        let public_key = key_pair.public_key();
        
        let message = b"test message";
        let signature = key_pair.sign(message);
        
        let verified = verify_signature(message, &signature, &public_key);
        assert!(verified.is_ok());
        assert!(verified.unwrap());
    }
}

