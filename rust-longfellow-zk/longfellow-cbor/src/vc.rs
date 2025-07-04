/// W3C Verifiable Credentials parsing

use crate::{Value, ClaimExtractor};
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// W3C Verifiable Credential
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiableCredential {
    /// Context (usually includes "https://www.w3.org/2018/credentials/v1")
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    
    /// Credential ID
    pub id: Option<String>,
    
    /// Credential types
    #[serde(rename = "type")]
    pub types: Vec<String>,
    
    /// Issuer (can be string or object)
    pub issuer: Issuer,
    
    /// Issuance date
    #[serde(rename = "issuanceDate")]
    pub issuance_date: String,
    
    /// Expiration date (optional)
    #[serde(rename = "expirationDate")]
    pub expiration_date: Option<String>,
    
    /// Credential subject
    #[serde(rename = "credentialSubject")]
    pub credential_subject: CredentialSubject,
    
    /// Proof (optional)
    pub proof: Option<Proof>,
    
    /// Additional properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

/// Issuer information
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Issuer {
    /// Simple string issuer
    String(String),
    /// Complex issuer object
    Object {
        id: String,
        name: Option<String>,
        #[serde(flatten)]
        additional: HashMap<String, Value>,
    },
}

/// Credential subject
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialSubject {
    /// Subject ID
    pub id: Option<String>,
    
    /// Subject properties
    #[serde(flatten)]
    pub properties: HashMap<String, Value>,
}

/// Proof structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proof {
    /// Proof type (e.g., "Ed25519Signature2018")
    #[serde(rename = "type")]
    pub proof_type: String,
    
    /// Creation timestamp
    pub created: Option<String>,
    
    /// Verification method
    #[serde(rename = "verificationMethod")]
    pub verification_method: Option<String>,
    
    /// Proof purpose
    #[serde(rename = "proofPurpose")]
    pub proof_purpose: Option<String>,
    
    /// Signature value
    #[serde(rename = "signatureValue")]
    pub signature_value: Option<String>,
    
    /// JWS (for JWT proofs)
    pub jws: Option<String>,
    
    /// Additional proof properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

impl VerifiableCredential {
    /// Parse from JSON bytes
    pub fn from_json_bytes(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data)
            .map_err(|e| LongfellowError::ParseError(format!("VC JSON parse error: {}", e)))
    }
    
    /// Parse from JSON string
    pub fn from_json_str(s: &str) -> Result<Self> {
        serde_json::from_str(s)
            .map_err(|e| LongfellowError::ParseError(format!("VC JSON parse error: {}", e)))
    }
    
    /// Convert to JSON bytes
    pub fn to_json_bytes(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self)
            .map_err(|e| LongfellowError::SerializationError(format!("VC JSON serialization error: {}", e)))
    }
    
    /// Check if credential is a certain type
    pub fn has_type(&self, cred_type: &str) -> bool {
        self.types.iter().any(|t| t == cred_type)
    }
    
    /// Get issuer ID
    pub fn issuer_id(&self) -> &str {
        match &self.issuer {
            Issuer::String(s) => s,
            Issuer::Object { id, .. } => id,
        }
    }
    
    /// Validate credential structure
    pub fn validate_structure(&self) -> Result<()> {
        // Must have VerifiableCredential type
        if !self.has_type("VerifiableCredential") {
            return Err(LongfellowError::ValidationError(
                "Credential must have type VerifiableCredential".to_string()
            ));
        }
        
        // Must have valid context
        if self.context.is_empty() {
            return Err(LongfellowError::ValidationError(
                "Credential must have at least one context".to_string()
            ));
        }
        
        // Should have W3C context
        let w3c_context = "https://www.w3.org/2018/credentials/v1";
        if !self.context.iter().any(|c| c == w3c_context) {
            return Err(LongfellowError::ValidationError(
                format!("Credential should include context: {}", w3c_context)
            ));
        }
        
        Ok(())
    }
}

impl ClaimExtractor for VerifiableCredential {
    fn extract_claims(&self) -> Result<HashMap<String, Value>> {
        let mut claims = HashMap::new();
        
        // Add credential metadata
        claims.insert("id".to_string(), 
            self.id.as_ref().map(|s| Value::Text(s.clone())).unwrap_or(Value::Null));
        claims.insert("issuer".to_string(), Value::Text(self.issuer_id().to_string()));
        claims.insert("issuanceDate".to_string(), Value::Text(self.issuance_date.clone()));
        
        if let Some(ref exp) = self.expiration_date {
            claims.insert("expirationDate".to_string(), Value::Text(exp.clone()));
        }
        
        // Add credential subject claims
        if let Some(ref id) = self.credential_subject.id {
            claims.insert("credentialSubject.id".to_string(), Value::Text(id.clone()));
        }
        
        for (key, value) in &self.credential_subject.properties {
            claims.insert(format!("credentialSubject.{}", key), value.clone());
        }
        
        // Add additional properties
        for (key, value) in &self.additional {
            claims.insert(key.clone(), value.clone());
        }
        
        Ok(claims)
    }
    
    fn get_claim(&self, path: &str) -> Option<Value> {
        if path.starts_with("credentialSubject.") {
            let key = &path["credentialSubject.".len()..];
            if key == "id" {
                self.credential_subject.id.as_ref().map(|s| Value::Text(s.clone()))
            } else {
                self.credential_subject.properties.get(key).cloned()
            }
        } else {
            match path {
                "id" => self.id.as_ref().map(|s| Value::Text(s.clone())),
                "issuer" => Some(Value::Text(self.issuer_id().to_string())),
                "issuanceDate" => Some(Value::Text(self.issuance_date.clone())),
                "expirationDate" => self.expiration_date.as_ref().map(|s| Value::Text(s.clone())),
                _ => self.additional.get(path).cloned(),
            }
        }
    }
}

/// Verifiable Presentation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerifiablePresentation {
    /// Context
    #[serde(rename = "@context")]
    pub context: Vec<String>,
    
    /// Presentation ID
    pub id: Option<String>,
    
    /// Presentation types
    #[serde(rename = "type")]
    pub types: Vec<String>,
    
    /// Verifiable credentials
    #[serde(rename = "verifiableCredential")]
    pub verifiable_credential: Vec<VerifiableCredential>,
    
    /// Holder
    pub holder: Option<String>,
    
    /// Proof
    pub proof: Option<Proof>,
    
    /// Additional properties
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

impl VerifiablePresentation {
    /// Validate presentation structure
    pub fn validate_structure(&self) -> Result<()> {
        // Must have VerifiablePresentation type
        if !self.types.iter().any(|t| t == "VerifiablePresentation") {
            return Err(LongfellowError::ValidationError(
                "Presentation must have type VerifiablePresentation".to_string()
            ));
        }
        
        // Validate all credentials
        for credential in &self.verifiable_credential {
            credential.validate_structure()?;
        }
        
        Ok(())
    }
}

/// Common credential types
pub mod credential_types {
    pub const VERIFIABLE_CREDENTIAL: &str = "VerifiableCredential";
    pub const VERIFIABLE_PRESENTATION: &str = "VerifiablePresentation";
    
    // Example credential types
    pub const UNIVERSITY_DEGREE: &str = "UniversityDegreeCredential";
    pub const DRIVING_LICENSE: &str = "DrivingLicenseCredential";
    pub const VACCINE_CERTIFICATE: &str = "VaccineCertificate";
    pub const AGE_CREDENTIAL: &str = "AgeCredential";
}

/// Proof types
pub mod proof_types {
    pub const ED25519_SIGNATURE_2018: &str = "Ed25519Signature2018";
    pub const ED25519_SIGNATURE_2020: &str = "Ed25519Signature2020";
    pub const ECDSA_SECP256K1_SIGNATURE_2019: &str = "EcdsaSecp256k1Signature2019";
    pub const RSA_SIGNATURE_2018: &str = "RsaSignature2018";
    pub const JSON_WEB_SIGNATURE_2020: &str = "JsonWebSignature2020";
    pub const BBS_PLUS_SIGNATURE_2020: &str = "BbsBlsSignature2020";
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vc_parsing() {
        let vc_json = r#"{
            "@context": [
                "https://www.w3.org/2018/credentials/v1",
                "https://www.w3.org/2018/credentials/examples/v1"
            ],
            "id": "http://example.edu/credentials/3732",
            "type": ["VerifiableCredential", "UniversityDegreeCredential"],
            "issuer": "https://example.edu/issuers/14",
            "issuanceDate": "2010-01-01T19:23:24Z",
            "credentialSubject": {
                "id": "did:example:ebfeb1f712ebc6f1c276e12ec21",
                "degree": {
                    "type": "BachelorDegree",
                    "name": "Bachelor of Science and Arts"
                }
            }
        }"#;
        
        let vc = VerifiableCredential::from_json_str(vc_json).unwrap();
        
        assert_eq!(vc.id, Some("http://example.edu/credentials/3732".to_string()));
        assert!(vc.has_type("VerifiableCredential"));
        assert!(vc.has_type("UniversityDegreeCredential"));
        assert_eq!(vc.issuer_id(), "https://example.edu/issuers/14");
        
        let claims = vc.extract_claims().unwrap();
        assert!(claims.contains_key("credentialSubject.degree"));
        
        vc.validate_structure().unwrap();
    }
}