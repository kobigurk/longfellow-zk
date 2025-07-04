/// JWT (JSON Web Token) parsing and validation

use crate::{Value, ClaimExtractor, base64url};
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// JWT structure
#[derive(Clone, Debug)]
pub struct Jwt {
    /// Header (decoded)
    pub header: JwtHeader,
    /// Payload (decoded)
    pub payload: JwtPayload,
    /// Signature bytes
    pub signature: Vec<u8>,
    /// Raw header (base64url encoded)
    pub raw_header: String,
    /// Raw payload (base64url encoded)
    pub raw_payload: String,
}

/// JWT header
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtHeader {
    /// Algorithm
    pub alg: String,
    /// Type (usually "JWT")
    pub typ: Option<String>,
    /// Key ID
    pub kid: Option<String>,
    /// Additional headers
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

/// JWT payload (claims)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtPayload {
    /// Issuer
    pub iss: Option<String>,
    /// Subject
    pub sub: Option<String>,
    /// Audience
    pub aud: Option<Value>,
    /// Expiration time
    pub exp: Option<i64>,
    /// Not before time
    pub nbf: Option<i64>,
    /// Issued at time
    pub iat: Option<i64>,
    /// JWT ID
    pub jti: Option<String>,
    /// Additional claims
    #[serde(flatten)]
    pub additional: HashMap<String, Value>,
}

impl Jwt {
    /// Parse JWT from string
    pub fn from_str(jwt: &str) -> Result<Self> {
        let parts: Vec<&str> = jwt.split('.').collect();
        if parts.len() != 3 {
            return Err(LongfellowError::ParseError(
                format!("JWT must have 3 parts, got {}", parts.len())
            ));
        }
        
        let raw_header = parts[0].to_string();
        let raw_payload = parts[1].to_string();
        
        // Decode header
        let header_bytes = base64url::decode(parts[0])?;
        let header_str = std::str::from_utf8(&header_bytes)
            .map_err(|e| LongfellowError::ParseError(format!("Invalid UTF-8 in header: {}", e)))?;
        let header: JwtHeader = serde_json::from_str(header_str)
            .map_err(|e| LongfellowError::ParseError(format!("Invalid JSON in header: {}", e)))?;
        
        // Decode payload
        let payload_bytes = base64url::decode(parts[1])?;
        let payload_str = std::str::from_utf8(&payload_bytes)
            .map_err(|e| LongfellowError::ParseError(format!("Invalid UTF-8 in payload: {}", e)))?;
        let payload: JwtPayload = serde_json::from_str(payload_str)
            .map_err(|e| LongfellowError::ParseError(format!("Invalid JSON in payload: {}", e)))?;
        
        // Decode signature
        let signature = base64url::decode(parts[2])?;
        
        Ok(Self {
            header,
            payload,
            signature,
            raw_header,
            raw_payload,
        })
    }
    
    /// Get signing input (header.payload)
    pub fn signing_input(&self) -> String {
        format!("{}.{}", self.raw_header, self.raw_payload)
    }
    
    /// Check if JWT is expired
    pub fn is_expired(&self) -> bool {
        if let Some(exp) = self.payload.exp {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            exp < now
        } else {
            false
        }
    }
    
    /// Check if JWT is not yet valid
    pub fn is_not_yet_valid(&self) -> bool {
        if let Some(nbf) = self.payload.nbf {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            nbf > now
        } else {
            false
        }
    }
    
    /// Validate time-based claims
    pub fn validate_time_claims(&self) -> Result<()> {
        if self.is_expired() {
            return Err(LongfellowError::ValidationError("JWT is expired".to_string()));
        }
        
        if self.is_not_yet_valid() {
            return Err(LongfellowError::ValidationError("JWT is not yet valid".to_string()));
        }
        
        Ok(())
    }
    
    /// Get algorithm
    pub fn algorithm(&self) -> &str {
        &self.header.alg
    }
}

impl ClaimExtractor for Jwt {
    fn extract_claims(&self) -> Result<HashMap<String, Value>> {
        let mut claims = HashMap::new();
        
        // Standard claims
        if let Some(ref iss) = self.payload.iss {
            claims.insert("iss".to_string(), Value::Text(iss.clone()));
        }
        if let Some(ref sub) = self.payload.sub {
            claims.insert("sub".to_string(), Value::Text(sub.clone()));
        }
        if let Some(ref aud) = self.payload.aud {
            claims.insert("aud".to_string(), aud.clone());
        }
        if let Some(exp) = self.payload.exp {
            claims.insert("exp".to_string(), Value::Integer(exp));
        }
        if let Some(nbf) = self.payload.nbf {
            claims.insert("nbf".to_string(), Value::Integer(nbf));
        }
        if let Some(iat) = self.payload.iat {
            claims.insert("iat".to_string(), Value::Integer(iat));
        }
        if let Some(ref jti) = self.payload.jti {
            claims.insert("jti".to_string(), Value::Text(jti.clone()));
        }
        
        // Additional claims
        for (key, value) in &self.payload.additional {
            claims.insert(key.clone(), value.clone());
        }
        
        Ok(claims)
    }
    
    fn get_claim(&self, path: &str) -> Option<&Value> {
        match path {
            "iss" => self.payload.iss.as_ref().map(|s| &Value::Text(s.clone())),
            "sub" => self.payload.sub.as_ref().map(|s| &Value::Text(s.clone())),
            "aud" => self.payload.aud.as_ref(),
            "exp" => self.payload.exp.map(|i| &Value::Integer(i)),
            "nbf" => self.payload.nbf.map(|i| &Value::Integer(i)),
            "iat" => self.payload.iat.map(|i| &Value::Integer(i)),
            "jti" => self.payload.jti.as_ref().map(|s| &Value::Text(s.clone())),
            _ => self.payload.additional.get(path),
        }
    }
}

/// Supported JWT algorithms
#[derive(Clone, Debug, PartialEq)]
pub enum JwtAlgorithm {
    /// HMAC with SHA-256
    HS256,
    /// HMAC with SHA-384
    HS384,
    /// HMAC with SHA-512
    HS512,
    /// RSA with SHA-256
    RS256,
    /// RSA with SHA-384
    RS384,
    /// RSA with SHA-512
    RS512,
    /// ECDSA with P-256 and SHA-256
    ES256,
    /// ECDSA with P-384 and SHA-384
    ES384,
    /// ECDSA with P-521 and SHA-512
    ES512,
    /// No signature
    None,
}

impl JwtAlgorithm {
    /// Parse from string
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "HS256" => Ok(Self::HS256),
            "HS384" => Ok(Self::HS384),
            "HS512" => Ok(Self::HS512),
            "RS256" => Ok(Self::RS256),
            "RS384" => Ok(Self::RS384),
            "RS512" => Ok(Self::RS512),
            "ES256" => Ok(Self::ES256),
            "ES384" => Ok(Self::ES384),
            "ES512" => Ok(Self::ES512),
            "none" => Ok(Self::None),
            _ => Err(LongfellowError::InvalidParameter(
                format!("Unknown JWT algorithm: {}", s)
            )),
        }
    }
    
    /// Convert to string
    pub fn as_str(&self) -> &str {
        match self {
            Self::HS256 => "HS256",
            Self::HS384 => "HS384",
            Self::HS512 => "HS512",
            Self::RS256 => "RS256",
            Self::RS384 => "RS384",
            Self::RS512 => "RS512",
            Self::ES256 => "ES256",
            Self::ES384 => "ES384",
            Self::ES512 => "ES512",
            Self::None => "none",
        }
    }
}

/// Build a JWT
pub struct JwtBuilder {
    header: JwtHeader,
    payload: JwtPayload,
}

impl JwtBuilder {
    /// Create a new JWT builder
    pub fn new(alg: JwtAlgorithm) -> Self {
        Self {
            header: JwtHeader {
                alg: alg.as_str().to_string(),
                typ: Some("JWT".to_string()),
                kid: None,
                additional: HashMap::new(),
            },
            payload: JwtPayload {
                iss: None,
                sub: None,
                aud: None,
                exp: None,
                nbf: None,
                iat: None,
                jti: None,
                additional: HashMap::new(),
            },
        }
    }
    
    /// Set key ID
    pub fn kid(mut self, kid: String) -> Self {
        self.header.kid = Some(kid);
        self
    }
    
    /// Set issuer
    pub fn issuer(mut self, iss: String) -> Self {
        self.payload.iss = Some(iss);
        self
    }
    
    /// Set subject
    pub fn subject(mut self, sub: String) -> Self {
        self.payload.sub = Some(sub);
        self
    }
    
    /// Set audience
    pub fn audience(mut self, aud: String) -> Self {
        self.payload.aud = Some(Value::Text(aud));
        self
    }
    
    /// Set expiration (seconds from now)
    pub fn expires_in(mut self, seconds: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.payload.exp = Some(now + seconds as i64);
        self
    }
    
    /// Add custom claim
    pub fn claim(mut self, key: String, value: Value) -> Self {
        self.payload.additional.insert(key, value);
        self
    }
    
    /// Build unsigned JWT
    pub fn build_unsigned(self) -> Result<String> {
        let header_json = serde_json::to_string(&self.header)
            .map_err(|e| LongfellowError::SerializationError(format!("Header serialization failed: {}", e)))?;
        let payload_json = serde_json::to_string(&self.payload)
            .map_err(|e| LongfellowError::SerializationError(format!("Payload serialization failed: {}", e)))?;
        
        let header_b64 = base64url::encode(header_json.as_bytes());
        let payload_b64 = base64url::encode(payload_json.as_bytes());
        
        Ok(format!("{}.{}", header_b64, payload_b64))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_jwt_parsing() {
        // Example JWT (without valid signature)
        let jwt_str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        
        let jwt = Jwt::from_str(jwt_str).unwrap();
        
        assert_eq!(jwt.header.alg, "HS256");
        assert_eq!(jwt.header.typ, Some("JWT".to_string()));
        assert_eq!(jwt.payload.sub, Some("1234567890".to_string()));
        assert_eq!(jwt.payload.iat, Some(1516239022));
        
        let claims = jwt.extract_claims().unwrap();
        assert_eq!(claims.get("sub").and_then(|v| v.as_str()), Some("1234567890"));
        assert_eq!(claims.get("name").and_then(|v| v.as_str()), Some("John Doe"));
    }
    
    #[test]
    fn test_jwt_builder() {
        let jwt = JwtBuilder::new(JwtAlgorithm::HS256)
            .issuer("test-issuer".to_string())
            .subject("user123".to_string())
            .audience("test-app".to_string())
            .expires_in(3600)
            .claim("role".to_string(), Value::Text("admin".to_string()))
            .build_unsigned()
            .unwrap();
        
        assert!(jwt.contains('.'));
        let parts: Vec<&str> = jwt.split('.').collect();
        assert_eq!(parts.len(), 2); // No signature part
    }
}