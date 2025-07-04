/// Random number generation and transcript handling for zero-knowledge proofs

use longfellow_algebra::traits::Field;
use rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sha3::{Digest, Sha3_256};
use std::marker::PhantomData;
use zeroize::Zeroize;

/// Transcript for Fiat-Shamir transform
pub struct Transcript {
    hasher: Sha3_256,
    counter: u64,
}

impl Transcript {
    pub fn new(label: &[u8]) -> Self {
        let mut hasher = Sha3_256::new();
        hasher.update(b"Longfellow-ZK-v1");
        hasher.update(&(label.len() as u64).to_le_bytes());
        hasher.update(label);
        
        Self { hasher, counter: 0 }
    }

    pub fn append_message(&mut self, label: &[u8], message: &[u8]) {
        self.hasher.update(&self.counter.to_le_bytes());
        self.hasher.update(&(label.len() as u64).to_le_bytes());
        self.hasher.update(label);
        self.hasher.update(&(message.len() as u64).to_le_bytes());
        self.hasher.update(message);
        self.counter += 1;
    }

    pub fn append_field_element<F: Field>(&mut self, label: &[u8], elem: &F) {
        let bytes = elem.to_bytes_le();
        self.append_message(label, &bytes);
    }

    pub fn append_field_elements<F: Field>(&mut self, label: &[u8], elems: &[F]) {
        self.append_message(label, &(elems.len() as u64).to_le_bytes());
        for elem in elems {
            let bytes = elem.to_bytes_le();
            self.hasher.update(&bytes);
        }
        self.counter += 1;
    }

    pub fn challenge_scalar<F: Field>(&mut self, label: &[u8]) -> F {
        self.append_message(b"challenge", label);
        
        let mut challenge_bytes = [0u8; 64];
        let hash = self.hasher.clone().finalize();
        challenge_bytes[..32].copy_from_slice(&hash);
        
        // For larger fields, we might need multiple hashes
        if F::MODULUS_BITS > 256 {
            let mut extended_hasher = self.hasher.clone();
            extended_hasher.update(b"extended");
            let extended_hash = extended_hasher.finalize();
            challenge_bytes[32..].copy_from_slice(&extended_hash);
        }
        
        // Reduce modulo field size
        F::from_bytes_le(&challenge_bytes[..F::MODULUS_BITS as usize / 8])
            .unwrap_or_else(|_| {
                // Fallback: use rejection sampling
                let mut rng = ChaCha20Rng::from_seed(hash.into());
                loop {
                    let mut bytes = [0u8; 32];
                    rng.fill_bytes(&mut bytes);
                    if let Ok(elem) = F::from_bytes_le(&bytes) {
                        return elem;
                    }
                }
            })
    }

    pub fn challenge_scalars<F: Field>(&mut self, label: &[u8], n: usize) -> Vec<F> {
        (0..n)
            .map(|i| {
                let mut indexed_label = label.to_vec();
                indexed_label.extend_from_slice(&i.to_le_bytes());
                self.challenge_scalar(&indexed_label)
            })
            .collect()
    }

    pub fn finalize(self) -> [u8; 32] {
        self.hasher.finalize().into()
    }
}

/// Field-specific random number generator
pub struct FieldRng<F: Field, R: RngCore + CryptoRng> {
    rng: R,
    _phantom: PhantomData<F>,
}

impl<F: Field, R: RngCore + CryptoRng> FieldRng<F, R> {
    pub fn new(rng: R) -> Self {
        Self {
            rng,
            _phantom: PhantomData,
        }
    }

    pub fn from_seed(seed: [u8; 32]) -> FieldRng<F, ChaCha20Rng> {
        FieldRng {
            rng: ChaCha20Rng::from_seed(seed),
            _phantom: PhantomData,
        }
    }

    pub fn random_field_element(&mut self) -> F {
        // Use rejection sampling for uniform distribution
        let byte_len = (F::MODULUS_BITS as usize + 7) / 8;
        let mut bytes = vec![0u8; byte_len];
        
        loop {
            self.rng.fill_bytes(&mut bytes);
            if let Ok(elem) = F::from_bytes_le(&bytes) {
                return elem;
            }
        }
    }

    pub fn random_field_elements(&mut self, n: usize) -> Vec<F> {
        (0..n).map(|_| self.random_field_element()).collect()
    }

    pub fn random_nonzero_field_element(&mut self) -> F {
        loop {
            let elem = self.random_field_element();
            if elem != F::zero() {
                return elem;
            }
        }
    }
}

/// Pseudo-random function for deterministic randomness
pub struct PseudoRandomFunction<F: Field> {
    key: [u8; 32],
    _phantom: PhantomData<F>,
}

impl<F: Field> PseudoRandomFunction<F> {
    pub fn new(key: [u8; 32]) -> Self {
        Self {
            key,
            _phantom: PhantomData,
        }
    }

    pub fn evaluate(&self, input: &[u8]) -> F {
        let mut hasher = Sha3_256::new();
        hasher.update(&self.key);
        hasher.update(&(input.len() as u64).to_le_bytes());
        hasher.update(input);
        
        let hash = hasher.finalize();
        let rng = ChaCha20Rng::from_seed(hash.into());
        
        FieldRng::<F, _>::new(rng).random_field_element()
    }

    pub fn evaluate_domain(&self, domain: &[u8], index: u64) -> F {
        let mut input = domain.to_vec();
        input.extend_from_slice(&index.to_le_bytes());
        self.evaluate(&input)
    }
}

impl<F: Field> Zeroize for PseudoRandomFunction<F> {
    fn zeroize(&mut self) {
        self.key.zeroize();
    }
}

/// Verifier's transcript for proof verification
pub struct VerifierTranscript {
    base: Transcript,
}

impl VerifierTranscript {
    pub fn new(label: &[u8]) -> Self {
        Self {
            base: Transcript::new(label),
        }
    }

    pub fn append_message(&mut self, label: &[u8], message: &[u8]) {
        self.base.append_message(label, message);
    }

    pub fn append_proof_message(&mut self, message: &[u8]) {
        self.base.append_message(b"proof", message);
    }

    pub fn challenge_scalar<F: Field>(&mut self, label: &[u8]) -> F {
        self.base.challenge_scalar(label)
    }

    pub fn challenge_scalars<F: Field>(&mut self, label: &[u8], n: usize) -> Vec<F> {
        self.base.challenge_scalars(label, n)
    }

    pub fn verify_proof_hash(&self, expected_hash: &[u8; 32]) -> bool {
        let actual_hash = self.base.hasher.clone().finalize();
        actual_hash.as_slice() == expected_hash
    }
}

/// Generate a random oracle query
pub fn random_oracle<F: Field>(domain: &[u8], input: &[u8]) -> F {
    let mut hasher = Sha3_256::new();
    hasher.update(b"Longfellow-RandomOracle");
    hasher.update(&(domain.len() as u64).to_le_bytes());
    hasher.update(domain);
    hasher.update(&(input.len() as u64).to_le_bytes());
    hasher.update(input);
    
    let hash = hasher.finalize();
    let rng = ChaCha20Rng::from_seed(hash.into());
    
    FieldRng::<F, _>::new(rng).random_field_element()
}

#[cfg(test)]
mod tests {
    use super::*;
    use longfellow_algebra::Fp128;
    use rand::rngs::OsRng;

    #[test]
    fn test_transcript_deterministic() {
        let mut t1 = Transcript::new(b"test");
        let mut t2 = Transcript::new(b"test");
        
        t1.append_message(b"msg1", b"data1");
        t2.append_message(b"msg1", b"data1");
        
        let c1: Fp128 = t1.challenge_scalar(b"challenge");
        let c2: Fp128 = t2.challenge_scalar(b"challenge");
        
        assert_eq!(c1, c2);
    }

    #[test]
    fn test_transcript_different_messages() {
        let mut t1 = Transcript::new(b"test");
        let mut t2 = Transcript::new(b"test");
        
        t1.append_message(b"msg1", b"data1");
        t2.append_message(b"msg1", b"data2");
        
        let c1: Fp128 = t1.challenge_scalar(b"challenge");
        let c2: Fp128 = t2.challenge_scalar(b"challenge");
        
        assert_ne!(c1, c2);
    }

    #[test]
    fn test_field_rng() {
        let mut rng = FieldRng::<Fp128, _>::new(OsRng);
        
        let elem1 = rng.random_field_element();
        let elem2 = rng.random_field_element();
        
        assert_ne!(elem1, elem2); // Should be different with high probability
        assert_ne!(elem1, Fp128::zero());
        assert_ne!(elem2, Fp128::zero());
    }

    #[test]
    fn test_prf_deterministic() {
        let key = [42u8; 32];
        let prf = PseudoRandomFunction::<Fp128>::new(key);
        
        let val1 = prf.evaluate(b"input");
        let val2 = prf.evaluate(b"input");
        
        assert_eq!(val1, val2);
        
        let val3 = prf.evaluate(b"different");
        assert_ne!(val1, val3);
    }
}