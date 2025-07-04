/// CBOR parsing and validation for cryptographic documents
/// 
/// This module implements CBOR (Concise Binary Object Representation) parsing
/// for various cryptographic document formats including JWT, mDOC, and W3C VCs.

use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod mdoc;
pub mod jwt;
pub mod vc;

/// CBOR value type
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl Value {
    /// Parse CBOR from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        ciborium::de::from_reader(data)
            .map_err(|e| LongfellowError::ParseError(format!("CBOR decode error: {}", e)))
    }
    
    /// Encode to CBOR bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        ciborium::ser::into_writer(self, &mut buf)
            .map_err(|e| LongfellowError::SerializationError(format!("CBOR encode error: {}", e)))?;
        Ok(buf)
    }
    
    /// Get as string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Text(s) => Some(s),
            _ => None,
        }
    }
    
    /// Get as bytes
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::Bytes(b) => Some(b),
            _ => None,
        }
    }
    
    /// Get as integer
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Integer(i) => Some(*i),
            _ => None,
        }
    }
    
    /// Get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    
    /// Get as array
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(a) => Some(a),
            _ => None,
        }
    }
    
    /// Get as map
    pub fn as_map(&self) -> Option<&HashMap<String, Value>> {
        match self {
            Value::Map(m) => Some(m),
            _ => None,
        }
    }
}

/// COSE (CBOR Object Signing and Encryption) header
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoseHeader {
    /// Algorithm identifier
    pub alg: i32,
    /// Key identifier
    pub kid: Option<Vec<u8>>,
    /// Content type
    pub content_type: Option<String>,
    /// Additional protected headers
    pub protected: HashMap<String, Value>,
    /// Unprotected headers
    pub unprotected: HashMap<String, Value>,
}

/// COSE signature structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoseSignature {
    /// Protected headers (base64url encoded)
    pub protected: Vec<u8>,
    /// Unprotected headers
    pub unprotected: HashMap<String, Value>,
    /// Signature bytes
    pub signature: Vec<u8>,
}

/// COSE_Sign1 structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CoseSign1 {
    /// Protected headers
    pub protected: Vec<u8>,
    /// Unprotected headers
    pub unprotected: HashMap<String, Value>,
    /// Payload
    pub payload: Vec<u8>,
    /// Signature
    pub signature: Vec<u8>,
}

impl CoseSign1 {
    /// Parse from CBOR bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let value = Value::from_bytes(data)?;
        let array = value.as_array()
            .ok_or_else(|| LongfellowError::ParseError("COSE_Sign1 must be array".to_string()))?;
        
        if array.len() != 4 {
            return Err(LongfellowError::ParseError(
                format!("COSE_Sign1 must have 4 elements, got {}", array.len())
            ));
        }
        
        let protected = array[0].as_bytes()
            .ok_or_else(|| LongfellowError::ParseError("Protected must be bytes".to_string()))?
            .to_vec();
            
        let unprotected = array[1].as_map()
            .ok_or_else(|| LongfellowError::ParseError("Unprotected must be map".to_string()))?
            .clone();
            
        let payload = array[2].as_bytes()
            .ok_or_else(|| LongfellowError::ParseError("Payload must be bytes".to_string()))?
            .to_vec();
            
        let signature = array[3].as_bytes()
            .ok_or_else(|| LongfellowError::ParseError("Signature must be bytes".to_string()))?
            .to_vec();
        
        Ok(Self {
            protected,
            unprotected,
            payload,
            signature,
        })
    }
    
    /// Get protected headers
    pub fn protected_headers(&self) -> Result<HashMap<String, Value>> {
        if self.protected.is_empty() {
            return Ok(HashMap::new());
        }
        
        let value = Value::from_bytes(&self.protected)?;
        value.as_map()
            .ok_or_else(|| LongfellowError::ParseError("Protected headers must be map".to_string()))
            .map(|m| m.clone())
    }
    
    /// Create signature input for verification
    pub fn signature_input(&self, external_aad: &[u8]) -> Vec<u8> {
        let mut sig_structure = Vec::new();
        
        // Signature1 context
        sig_structure.extend_from_slice(b"Signature1");
        
        // Protected headers
        sig_structure.extend_from_slice(&self.protected);
        
        // External AAD
        sig_structure.extend_from_slice(external_aad);
        
        // Payload
        sig_structure.extend_from_slice(&self.payload);
        
        sig_structure
    }
}

/// Base64URL encoding/decoding utilities
pub mod base64url {
    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
    use longfellow_core::{LongfellowError, Result};
    
    /// Encode to base64url
    pub fn encode(data: &[u8]) -> String {
        URL_SAFE_NO_PAD.encode(data)
    }
    
    /// Decode from base64url
    pub fn decode(s: &str) -> Result<Vec<u8>> {
        URL_SAFE_NO_PAD.decode(s)
            .map_err(|e| LongfellowError::ParseError(format!("Base64 decode error: {}", e)))
    }
}

/// Extract claims from CBOR structure
pub trait ClaimExtractor {
    /// Extract claims as key-value pairs
    fn extract_claims(&self) -> Result<HashMap<String, Value>>;
    
    /// Get specific claim by path (e.g., "address.street")
    fn get_claim(&self, path: &str) -> Option<Value>;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cbor_value_parsing() {
        let data = vec![
            0x84, // Array of 4 elements
            0x43, 0x01, 0x02, 0x03, // Bytes
            0xa0, // Empty map
            0x44, 0x04, 0x05, 0x06, 0x07, // Bytes
            0x48, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, // Bytes
        ];
        
        let value = Value::from_bytes(&data).unwrap();
        assert!(value.as_array().is_some());
        
        let array = value.as_array().unwrap();
        assert_eq!(array.len(), 4);
        assert!(array[0].as_bytes().is_some());
        assert!(array[1].as_map().is_some());
    }
    
    #[test]
    fn test_base64url() {
        let data = b"hello world";
        let encoded = base64url::encode(data);
        let decoded = base64url::decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
}