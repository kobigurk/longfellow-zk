/// Longfellow Interoperability Library
/// 
/// This library provides FFI bindings for C++ to interact with the Longfellow ZK system

pub mod cpp_verifier_interop;
pub mod cpp_verifier_full;
pub mod generate_test_proofs;
pub mod ligero_verifier_real;

// Re-export the main FFI functions
pub use cpp_verifier_interop::{
    longfellow_proof_from_bytes,
    longfellow_proof_from_json,
    longfellow_verify_proof,
    longfellow_proof_free,
    longfellow_error_free,
    longfellow_proof_metadata_json,
    longfellow_field_from_u64,
    longfellow_batch_verify,
};