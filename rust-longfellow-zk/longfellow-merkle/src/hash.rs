/// Hash function abstractions for Merkle trees

use serde::{Deserialize, Serialize};
use sha2::Sha256;
use sha3::{Sha3_256, Sha3_512};
use blake3;
use sha2::Digest;

/// Supported hash functions
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HashFunction {
    Sha256,
    Sha3_256,
    Sha3_512,
    Blake3,
}

/// Trait for hash functions used in Merkle trees
pub trait Hasher: Clone + Send + Sync {
    /// Output type of the hash function
    type Output: Clone + AsRef<[u8]> + PartialEq + Send + Sync;
    
    /// Hash a leaf node
    fn hash_leaf(data: &[u8]) -> Self::Output;
    
    /// Hash two nodes together
    fn hash_pair(left: &Self::Output, right: &Self::Output) -> Self::Output;
    
    /// Get empty hash (for padding)
    fn empty_hash() -> Self::Output;
}

/// SHA-256 hasher implementation
#[derive(Clone)]
pub struct Sha256Hasher;

impl Hasher for Sha256Hasher {
    type Output = [u8; 32];
    
    fn hash_leaf(data: &[u8]) -> Self::Output {
        let mut hasher = Sha256::new();
        hasher.update(b"leaf:");
        hasher.update(data);
        hasher.finalize().into()
    }
    
    fn hash_pair(left: &Self::Output, right: &Self::Output) -> Self::Output {
        let mut hasher = Sha256::new();
        hasher.update(b"node:");
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
    
    fn empty_hash() -> Self::Output {
        let mut hasher = Sha256::new();
        hasher.update(b"empty");
        hasher.finalize().into()
    }
}

/// SHA3-256 hasher implementation
#[derive(Clone)]
pub struct Sha3_256Hasher;

impl Hasher for Sha3_256Hasher {
    type Output = [u8; 32];
    
    fn hash_leaf(data: &[u8]) -> Self::Output {
        let mut hasher = Sha3_256::new();
        hasher.update(b"leaf:");
        hasher.update(&(data.len() as u64).to_le_bytes());
        hasher.update(data);
        hasher.finalize().into()
    }
    
    fn hash_pair(left: &Self::Output, right: &Self::Output) -> Self::Output {
        let mut hasher = Sha3_256::new();
        hasher.update(b"node:");
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
    
    fn empty_hash() -> Self::Output {
        let mut hasher = Sha3_256::new();
        hasher.update(b"empty");
        hasher.finalize().into()
    }
}

/// SHA3-512 hasher implementation
#[derive(Clone)]
pub struct Sha3_512Hasher;

impl Hasher for Sha3_512Hasher {
    type Output = [u8; 64];
    
    fn hash_leaf(data: &[u8]) -> Self::Output {
        let mut hasher = Sha3_512::new();
        hasher.update(b"leaf:");
        hasher.update(&(data.len() as u64).to_le_bytes());
        hasher.update(data);
        hasher.finalize().into()
    }
    
    fn hash_pair(left: &Self::Output, right: &Self::Output) -> Self::Output {
        let mut hasher = Sha3_512::new();
        hasher.update(b"node:");
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
    
    fn empty_hash() -> Self::Output {
        let mut hasher = Sha3_512::new();
        hasher.update(b"empty");
        hasher.finalize().into()
    }
}

/// BLAKE3 hasher implementation
#[derive(Clone)]
pub struct Blake3Hasher;

impl Hasher for Blake3Hasher {
    type Output = [u8; 32];
    
    fn hash_leaf(data: &[u8]) -> Self::Output {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"leaf:");
        hasher.update(data);
        *hasher.finalize().as_bytes()
    }
    
    fn hash_pair(left: &Self::Output, right: &Self::Output) -> Self::Output {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"node:");
        hasher.update(left);
        hasher.update(right);
        *hasher.finalize().as_bytes()
    }
    
    fn empty_hash() -> Self::Output {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"empty");
        *hasher.finalize().as_bytes()
    }
}

/// Create a hasher based on hash function enum
pub fn create_hasher(hash_function: HashFunction) -> Box<dyn Hasher<Output = Vec<u8>>> {
    match hash_function {
        HashFunction::Sha256 => Box::new(VecHasher::<Sha256Hasher>::new()),
        HashFunction::Sha3_256 => Box::new(VecHasher::<Sha3_256Hasher>::new()),
        HashFunction::Sha3_512 => Box::new(VecHasher::<Sha3_512Hasher>::new()),
        HashFunction::Blake3 => Box::new(VecHasher::<Blake3Hasher>::new()),
    }
}

/// Wrapper to convert array output to Vec for dynamic dispatch
struct VecHasher<H: Hasher> {
    _phantom: std::marker::PhantomData<H>,
}

impl<H: Hasher> Clone for VecHasher<H> {
    fn clone(&self) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<H: Hasher> VecHasher<H> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<H: Hasher> Hasher for VecHasher<H>
where
    H::Output: AsRef<[u8]>,
{
    type Output = Vec<u8>;
    
    fn hash_leaf(data: &[u8]) -> Self::Output {
        H::hash_leaf(data).as_ref().to_vec()
    }
    
    fn hash_pair(left: &Self::Output, right: &Self::Output) -> Self::Output {
        let left_array = left.as_slice();
        let right_array = right.as_slice();
        
        // This is a bit inefficient but necessary for dynamic dispatch
        let left_bytes = unsafe {
            std::slice::from_raw_parts(left_array.as_ptr(), left_array.len())
        };
        let right_bytes = unsafe {
            std::slice::from_raw_parts(right_array.as_ptr(), right_array.len())
        };
        
        match std::mem::size_of::<H::Output>() {
            32 => {
                let mut left_arr = [0u8; 32];
                let mut right_arr = [0u8; 32];
                left_arr.copy_from_slice(&left_bytes[..32]);
                right_arr.copy_from_slice(&right_bytes[..32]);
                
                let result = if std::any::TypeId::of::<H>() == std::any::TypeId::of::<Sha256Hasher>() {
                    Sha256Hasher::hash_pair(&left_arr, &right_arr).to_vec()
                } else if std::any::TypeId::of::<H>() == std::any::TypeId::of::<Sha3_256Hasher>() {
                    Sha3_256Hasher::hash_pair(&left_arr, &right_arr).to_vec()
                } else if std::any::TypeId::of::<H>() == std::any::TypeId::of::<Blake3Hasher>() {
                    Blake3Hasher::hash_pair(&left_arr, &right_arr).to_vec()
                } else {
                    panic!("Unknown 32-byte hasher");
                };
                result
            }
            64 => {
                let mut left_arr = [0u8; 64];
                let mut right_arr = [0u8; 64];
                left_arr.copy_from_slice(&left_bytes[..64]);
                right_arr.copy_from_slice(&right_bytes[..64]);
                
                Sha3_512Hasher::hash_pair(&left_arr, &right_arr).to_vec()
            }
            _ => panic!("Unsupported hash output size"),
        }
    }
    
    fn empty_hash() -> Self::Output {
        H::empty_hash().as_ref().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_consistency() {
        let data = b"test data";
        
        // Test that same data produces same hash
        let hash1 = Sha3_256Hasher::hash_leaf(data);
        let hash2 = Sha3_256Hasher::hash_leaf(data);
        assert_eq!(hash1, hash2);
        
        // Test that different data produces different hash
        let hash3 = Sha3_256Hasher::hash_leaf(b"different");
        assert_ne!(hash1, hash3);
    }
    
    #[test]
    fn test_different_hashers() {
        let data = b"test";
        
        let sha256_hash = Sha256Hasher::hash_leaf(data);
        let sha3_hash = Sha3_256Hasher::hash_leaf(data);
        let blake3_hash = Blake3Hasher::hash_leaf(data);
        
        // Different hashers should produce different results
        assert_ne!(sha256_hash, sha3_hash);
        assert_ne!(sha256_hash, blake3_hash);
        assert_ne!(sha3_hash, blake3_hash);
    }
    
    #[test]
    fn test_hash_pair_order() {
        let left = Sha3_256Hasher::hash_leaf(b"left");
        let right = Sha3_256Hasher::hash_leaf(b"right");
        
        let hash_lr = Sha3_256Hasher::hash_pair(&left, &right);
        let hash_rl = Sha3_256Hasher::hash_pair(&right, &left);
        
        // Order should matter
        assert_ne!(hash_lr, hash_rl);
    }
}