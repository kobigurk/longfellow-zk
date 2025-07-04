/// Document handling for zero-knowledge proofs

use crate::{DocumentData, DocumentType};
use longfellow_cbor::{jwt::Jwt, mdoc::Document as MdocDocument, vc::VerifiableCredential, Value};
use longfellow_core::{LongfellowError, Result};
use std::collections::HashMap;

/// Document parser and validator
pub struct DocumentParser;

impl DocumentParser {
    /// Parse a document from bytes
    pub fn parse(data: &[u8], doc_type: DocumentType) -> Result<DocumentData> {
        match doc_type {
            DocumentType::Jwt => {
                let jwt_str = std::str::from_utf8(data)
                    .map_err(|e| LongfellowError::ParseError(format!("Invalid UTF-8: {}", e)))?;
                let jwt = Jwt::from_str(jwt_str)?;
                Ok(DocumentData::Jwt(jwt))
            }
            DocumentType::Mdoc => {
                let mdoc = MdocDocument::from_bytes(data)?;
                Ok(DocumentData::Mdoc(mdoc))
            }
            DocumentType::VerifiableCredential => {
                let vc = VerifiableCredential::from_json_bytes(data)?;
                Ok(DocumentData::VerifiableCredential(vc))
            }
            DocumentType::Custom(_) => {
                Ok(DocumentData::Raw(data.to_vec()))
            }
        }
    }
    
    /// Validate document structure
    pub fn validate(doc: &DocumentData) -> Result<()> {
        match doc {
            DocumentData::Jwt(jwt) => {
                jwt.validate_time_claims()?;
                if jwt.algorithm() == "none" {
                    return Err(LongfellowError::ValidationError(
                        "Unsigned JWT not allowed".to_string()
                    ));
                }
                Ok(())
            }
            DocumentData::Mdoc(mdoc) => {
                // Check document type
                if mdoc.doc_type.is_empty() {
                    return Err(LongfellowError::ValidationError(
                        "mDOC missing document type".to_string()
                    ));
                }
                Ok(())
            }
            DocumentData::VerifiableCredential(vc) => {
                vc.validate_structure()?;
                Ok(())
            }
            DocumentData::Raw(_) => Ok(()),
        }
    }
}

/// Document claim extractor
pub struct ClaimExtractor;

impl ClaimExtractor {
    /// Extract all claims from a document
    pub fn extract_all(doc: &DocumentData) -> Result<HashMap<String, Value>> {
        match doc {
            DocumentData::Jwt(jwt) => {
                longfellow_cbor::ClaimExtractor::extract_claims(jwt)
            }
            DocumentData::Mdoc(mdoc) => {
                longfellow_cbor::ClaimExtractor::extract_claims(mdoc)
            }
            DocumentData::VerifiableCredential(vc) => {
                longfellow_cbor::ClaimExtractor::extract_claims(vc)
            }
            DocumentData::Raw(_) => {
                Ok(HashMap::new())
            }
        }
    }
    
    /// Extract specific claim
    pub fn extract_claim(doc: &DocumentData, path: &str) -> Option<Value> {
        match doc {
            DocumentData::Jwt(jwt) => {
                longfellow_cbor::ClaimExtractor::get_claim(jwt, path).cloned()
            }
            DocumentData::Mdoc(mdoc) => {
                longfellow_cbor::ClaimExtractor::get_claim(mdoc, path).cloned()
            }
            DocumentData::VerifiableCredential(vc) => {
                longfellow_cbor::ClaimExtractor::get_claim(vc, path).cloned()
            }
            DocumentData::Raw(_) => None,
        }
    }
    
    /// Extract claims for specific fields
    pub fn extract_fields(doc: &DocumentData, fields: &[String]) -> HashMap<String, Value> {
        let mut result = HashMap::new();
        
        for field in fields {
            if let Some(value) = Self::extract_claim(doc, field) {
                result.insert(field.clone(), value);
            }
        }
        
        result
    }
}

/// Document commitment generator
pub struct CommitmentGenerator;

impl CommitmentGenerator {
    /// Generate commitment to a value with randomness
    pub fn commit(value: &[u8], randomness: &[u8; 32]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(b"Longfellow-Commit-v1");
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value);
        hasher.update(randomness);
        hasher.finalize().into()
    }
    
    /// Generate commitments for multiple values
    pub fn commit_batch(values: &[Vec<u8>], randomness: &[[u8; 32]]) -> Result<Vec<[u8; 32]>> {
        if values.len() != randomness.len() {
            return Err(LongfellowError::InvalidParameter(
                "Values and randomness length mismatch".to_string()
            ));
        }
        
        Ok(values.iter()
            .zip(randomness.iter())
            .map(|(v, r)| Self::commit(v, r))
            .collect())
    }
    
    /// Verify a commitment
    pub fn verify(commitment: &[u8; 32], value: &[u8], randomness: &[u8; 32]) -> bool {
        let computed = Self::commit(value, randomness);
        computed == *commitment
    }
}

/// Document redaction for selective disclosure
pub struct DocumentRedactor;

impl DocumentRedactor {
    /// Redact fields from a document
    pub fn redact(doc: &DocumentData, fields_to_hide: &[String]) -> Result<DocumentData> {
        match doc {
            DocumentData::Jwt(_) => {
                // JWTs don't support direct redaction
                Err(LongfellowError::UnsupportedOperation(
                    "JWT redaction not supported".to_string()
                ))
            }
            DocumentData::Mdoc(mdoc) => {
                // Create a new mDOC with hidden fields removed
                let mut redacted = mdoc.clone();
                
                for field in fields_to_hide {
                    // Remove from all namespaces
                    for (_, items) in &mut redacted.issuer_signed.name_spaces {
                        items.retain(|item| !field.contains(&item.element_identifier));
                    }
                }
                
                Ok(DocumentData::Mdoc(redacted))
            }
            DocumentData::VerifiableCredential(vc) => {
                // Create a new VC with hidden fields removed
                let mut redacted = vc.clone();
                
                for field in fields_to_hide {
                    redacted.credential_subject.properties.remove(field);
                    redacted.additional.remove(field);
                }
                
                Ok(DocumentData::VerifiableCredential(redacted))
            }
            DocumentData::Raw(_) => {
                Err(LongfellowError::UnsupportedOperation(
                    "Raw document redaction not supported".to_string()
                ))
            }
        }
    }
}

/// Document signature verifier
pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Verify document signature
    pub fn verify(doc: &DocumentData, public_key: Option<&[u8]>) -> Result<bool> {
        match doc {
            DocumentData::Jwt(jwt) => {
                // Would implement JWT signature verification
                // For now, check algorithm is not "none"
                Ok(jwt.algorithm() != "none")
            }
            DocumentData::Mdoc(mdoc) => {
                // Would implement COSE signature verification
                // For now, check issuer auth exists
                Ok(!mdoc.issuer_signed.issuer_auth.signature.is_empty())
            }
            DocumentData::VerifiableCredential(vc) => {
                // Would implement proof verification
                // For now, check proof exists
                Ok(vc.proof.is_some())
            }
            DocumentData::Raw(_) => {
                Err(LongfellowError::UnsupportedOperation(
                    "Raw document signature verification not supported".to_string()
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_commitment_generation() {
        let value = b"secret value";
        let randomness = [42u8; 32];
        
        let commitment = CommitmentGenerator::commit(value, &randomness);
        
        // Verify commitment
        assert!(CommitmentGenerator::verify(&commitment, value, &randomness));
        
        // Wrong value should fail
        assert!(!CommitmentGenerator::verify(&commitment, b"wrong", &randomness));
        
        // Wrong randomness should fail
        assert!(!CommitmentGenerator::verify(&commitment, value, &[0u8; 32]));
    }
    
    #[test]
    fn test_batch_commitment() {
        let values = vec![
            b"value1".to_vec(),
            b"value2".to_vec(),
            b"value3".to_vec(),
        ];
        
        let randomness = vec![
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
        ];
        
        let commitments = CommitmentGenerator::commit_batch(&values, &randomness).unwrap();
        
        assert_eq!(commitments.len(), 3);
        
        // Verify each commitment
        for i in 0..3 {
            assert!(CommitmentGenerator::verify(
                &commitments[i],
                &values[i],
                &randomness[i]
            ));
        }
    }
}