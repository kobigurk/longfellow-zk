/// Transcript management for Sumcheck protocol

use longfellow_algebra::traits::Field;
use longfellow_random::Transcript;
use crate::polynomial::UnivariatePoly;

/// Sumcheck-specific transcript
pub struct SumcheckTranscript {
    base: Transcript,
}

impl SumcheckTranscript {
    /// Create a new sumcheck transcript
    pub fn new(label: &[u8]) -> Self {
        let base = Transcript::new(b"Sumcheck-v1");
        let mut transcript = Self { base };
        transcript.append_message(b"instance", label);
        transcript
    }
    
    /// Append circuit information
    pub fn append_circuit_info(&mut self, num_layers: usize, num_copies: usize, claimed_sum: &[u8]) {
        self.base.append_message(b"num_layers", &(num_layers as u64).to_le_bytes());
        self.base.append_message(b"num_copies", &(num_copies as u64).to_le_bytes());
        self.base.append_message(b"claimed_sum", claimed_sum);
    }
    
    /// Append a polynomial
    pub fn append_polynomial<F: Field>(&mut self, round: usize, poly: &UnivariatePoly<F>) {
        let label = format!("poly_{}", round);
        self.base.append_field_elements(label.as_bytes(), &poly.coeffs);
    }
    
    /// Append wire claims
    pub fn append_wire_claims<F: Field>(&mut self, layer: usize, claims: &[F]) {
        let label = format!("wire_claims_{}", layer);
        self.base.append_field_elements(label.as_bytes(), claims);
    }
    
    /// Get challenge for binding
    pub fn challenge_binding<F: Field>(&mut self, round: usize) -> F {
        let label = format!("bind_{}", round);
        self.base.challenge_scalar(label.as_bytes())
    }
    
    /// Get multiple challenges
    pub fn challenge_bindings<F: Field>(&mut self, round: usize, count: usize) -> Vec<F> {
        let label = format!("binds_{}", round);
        self.base.challenge_scalars(label.as_bytes(), count)
    }
    
    /// Append a message
    pub fn append_message(&mut self, label: &[u8], msg: &[u8]) {
        self.base.append_message(label, msg);
    }
}