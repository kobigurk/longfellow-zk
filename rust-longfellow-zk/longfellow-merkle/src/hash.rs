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
    type Output: Clone + AsRef<[u8]> + PartialEq + Send + Sync + Serialize + for<'de> Deserialize<'de>;
    
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

/// SHA3-512 hasher implementation using Vec<u8> to avoid serde issues
#[derive(Clone)]
pub struct Sha3_512Hasher;

impl Hasher for Sha3_512Hasher {
    type Output = Vec<u8>;
    
    fn hash_leaf(data: &[u8]) -> Self::Output {
        let mut hasher = Sha3_512::new();
        hasher.update(b"leaf:");
        hasher.update(&(data.len() as u64).to_le_bytes());
        hasher.update(data);
        hasher.finalize().to_vec()
    }
    
    fn hash_pair(left: &Self::Output, right: &Self::Output) -> Self::Output {
        let mut hasher = Sha3_512::new();
        hasher.update(b"node:");
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().to_vec()
    }
    
    fn empty_hash() -> Self::Output {
        let mut hasher = Sha3_512::new();
        hasher.update(b"empty");
        hasher.finalize().to_vec()
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

/// Dynamic hash function wrapper that doesn't require dyn compatibility
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DynamicHasher {
    Sha256,
    Sha3_256,
    Sha3_512,
    Blake3,
}

impl Default for DynamicHasher {
    fn default() -> Self {
        DynamicHasher::Sha256
    }
}

impl DynamicHasher {
    pub fn new(hash_function: HashFunction) -> Self {
        match hash_function {
            HashFunction::Sha256 => DynamicHasher::Sha256,
            HashFunction::Sha3_256 => DynamicHasher::Sha3_256,
            HashFunction::Sha3_512 => DynamicHasher::Sha3_512,
            HashFunction::Blake3 => DynamicHasher::Blake3,
        }
    }
    
    pub fn hash_leaf(&self, data: &[u8]) -> Vec<u8> {
        match self {
            DynamicHasher::Sha256 => Sha256Hasher::hash_leaf(data).to_vec(),
            DynamicHasher::Sha3_256 => Sha3_256Hasher::hash_leaf(data).to_vec(),
            DynamicHasher::Sha3_512 => Sha3_512Hasher::hash_leaf(data),
            DynamicHasher::Blake3 => Blake3Hasher::hash_leaf(data).to_vec(),
        }
    }
    
    pub fn hash_pair(&self, left: &[u8], right: &[u8]) -> Vec<u8> {
        match self {
            DynamicHasher::Sha256 => {
                let mut left_arr = [0u8; 32];
                let mut right_arr = [0u8; 32];
                left_arr.copy_from_slice(&left[..32.min(left.len())]);
                right_arr.copy_from_slice(&right[..32.min(right.len())]);
                Sha256Hasher::hash_pair(&left_arr, &right_arr).to_vec()
            }
            DynamicHasher::Sha3_256 => {
                let mut left_arr = [0u8; 32];
                let mut right_arr = [0u8; 32];
                left_arr.copy_from_slice(&left[..32.min(left.len())]);
                right_arr.copy_from_slice(&right[..32.min(right.len())]);
                Sha3_256Hasher::hash_pair(&left_arr, &right_arr).to_vec()
            }
            DynamicHasher::Sha3_512 => {
                Sha3_512Hasher::hash_pair(&left.to_vec(), &right.to_vec())
            }
            DynamicHasher::Blake3 => {
                let mut left_arr = [0u8; 32];
                let mut right_arr = [0u8; 32];
                left_arr.copy_from_slice(&left[..32.min(left.len())]);
                right_arr.copy_from_slice(&right[..32.min(right.len())]);
                Blake3Hasher::hash_pair(&left_arr, &right_arr).to_vec()
            }
        }
    }
    
    pub fn empty_hash(&self) -> Vec<u8> {
        match self {
            DynamicHasher::Sha256 => Sha256Hasher::empty_hash().to_vec(),
            DynamicHasher::Sha3_256 => Sha3_256Hasher::empty_hash().to_vec(),
            DynamicHasher::Sha3_512 => Sha3_512Hasher::empty_hash(),
            DynamicHasher::Blake3 => Blake3Hasher::empty_hash().to_vec(),
        }
    }
}

/// Create a hasher based on hash function enum
pub fn create_hasher(hash_function: HashFunction) -> DynamicHasher {
    DynamicHasher::new(hash_function)
}