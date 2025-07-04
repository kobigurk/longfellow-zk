/// Cryptographic utilities

use sha2::{Sha256, Digest as Sha2Digest};
use sha3::{Sha3_256, Digest as Sha3Digest};
use longfellow_core::{LongfellowError, Result};

/// Compute SHA-256 hash
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Compute SHA3-256 hash
pub fn sha3_256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Verify SHA-256 hash
pub fn verify_sha256(data: &[u8], expected: &[u8; 32]) -> bool {
    &sha256(data) == expected
}

/// Verify SHA3-256 hash
pub fn verify_sha3_256(data: &[u8], expected: &[u8; 32]) -> bool {
    &sha3_256(data) == expected
}

/// HMAC-SHA256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;
    const IPAD: u8 = 0x36;
    const OPAD: u8 = 0x5C;
    
    // Prepare key
    let mut k = vec![0u8; BLOCK_SIZE];
    if key.len() > BLOCK_SIZE {
        let hashed = sha256(key);
        k[..32].copy_from_slice(&hashed);
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    
    // Inner hash
    let mut inner = vec![0u8; BLOCK_SIZE + data.len()];
    for i in 0..BLOCK_SIZE {
        inner[i] = k[i] ^ IPAD;
    }
    inner[BLOCK_SIZE..].copy_from_slice(data);
    let inner_hash = sha256(&inner);
    
    // Outer hash
    let mut outer = vec![0u8; BLOCK_SIZE + 32];
    for i in 0..BLOCK_SIZE {
        outer[i] = k[i] ^ OPAD;
    }
    outer[BLOCK_SIZE..].copy_from_slice(&inner_hash);
    
    sha256(&outer)
}

/// Key derivation function (KDF)
pub fn kdf_sha256(secret: &[u8], salt: &[u8], info: &[u8], output_len: usize) -> Vec<u8> {
    // Simple KDF using HMAC-SHA256
    let mut output = Vec::with_capacity(output_len);
    let mut counter = 1u32;
    
    while output.len() < output_len {
        let mut input = Vec::new();
        input.extend_from_slice(salt);
        input.extend_from_slice(info);
        input.extend_from_slice(&counter.to_be_bytes());
        
        let block = hmac_sha256(secret, &input);
        let remaining = output_len - output.len();
        let to_copy = remaining.min(32);
        output.extend_from_slice(&block[..to_copy]);
        
        counter += 1;
    }
    
    output
}

/// Generate random bytes
pub fn random_bytes(len: usize) -> Vec<u8> {
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut bytes = vec![0u8; len];
    rng.fill_bytes(&mut bytes);
    bytes
}

/// Constant-time comparison
pub fn ct_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    
    diff == 0
}

/// Base64 encoding
pub fn base64_encode(data: &[u8]) -> String {
    base64::encode(data)
}

/// Base64 decoding
pub fn base64_decode(s: &str) -> Result<Vec<u8>> {
    base64::decode(s)
        .map_err(|e| LongfellowError::ParseError(format!("Base64 decode error: {}", e)))
}

/// URL-safe base64 encoding
pub fn base64url_encode(data: &[u8]) -> String {
    base64::encode_config(data, base64::URL_SAFE_NO_PAD)
}

/// URL-safe base64 decoding
pub fn base64url_decode(s: &str) -> Result<Vec<u8>> {
    base64::decode_config(s, base64::URL_SAFE_NO_PAD)
        .map_err(|e| LongfellowError::ParseError(format!("Base64URL decode error: {}", e)))
}

/// Hex encoding
pub fn hex_encode(data: &[u8]) -> String {
    hex::encode(data)
}

/// Hex decoding
pub fn hex_decode(s: &str) -> Result<Vec<u8>> {
    hex::decode(s)
        .map_err(|e| LongfellowError::ParseError(format!("Hex decode error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sha256() {
        let data = b"hello world";
        let hash = sha256(data);
        let expected = hex_decode(
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        ).unwrap();
        assert_eq!(&hash[..], &expected[..]);
    }
    
    #[test]
    fn test_sha3_256() {
        let data = b"hello world";
        let hash = sha3_256(data);
        assert_eq!(hash.len(), 32);
    }
    
    #[test]
    fn test_hmac_sha256() {
        let key = b"key";
        let data = b"The quick brown fox jumps over the lazy dog";
        let hmac = hmac_sha256(key, data);
        let expected = hex_decode(
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        ).unwrap();
        assert_eq!(&hmac[..], &expected[..]);
    }
    
    #[test]
    fn test_base64() {
        let data = b"hello world";
        let encoded = base64_encode(data);
        assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
        
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
    
    #[test]
    fn test_ct_equal() {
        let a = b"secret";
        let b = b"secret";
        let c = b"public";
        
        assert!(ct_equal(a, b));
        assert!(!ct_equal(a, c));
    }
}