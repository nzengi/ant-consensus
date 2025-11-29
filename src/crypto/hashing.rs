use ring::digest;

/// Hash a byte slice using SHA-256
pub fn hash_sha256(data: &[u8]) -> [u8; 32] {
    let hash = digest::digest(&digest::SHA256, data);
    let mut result = [0u8; 32];
    result.copy_from_slice(hash.as_ref());
    result
}

/// Hash a string using SHA-256
pub fn hash_string(s: &str) -> [u8; 32] {
    hash_sha256(s.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_sha256() {
        let data = b"test data";
        let hash1 = hash_sha256(data);
        let hash2 = hash_sha256(data);
        
        // Same input should produce same hash
        assert_eq!(hash1, hash2);
        
        // Different input should produce different hash
        let hash3 = hash_sha256(b"different data");
        assert_ne!(hash1, hash3);
    }
}

