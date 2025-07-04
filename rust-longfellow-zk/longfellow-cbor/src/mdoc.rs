/// ISO 18013-5 mDOC (mobile Driving License) CBOR parsing

use crate::{Value, CoseSign1, ClaimExtractor};
use longfellow_core::{LongfellowError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// mDOC document type
pub const MDOC_DOCTYPE: &str = "org.iso.18013.5.1.mDL";

/// mDOC namespace for driver license
pub const MDL_NAMESPACE: &str = "org.iso.18013.5.1";

/// mDOC device response structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceResponse {
    /// Version string
    pub version: String,
    /// Documents
    pub documents: Vec<Document>,
    /// Status code
    pub status: u64,
}

/// mDOC document
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Document {
    /// Document type (e.g., "org.iso.18013.5.1.mDL")
    pub doc_type: String,
    /// Issuer signed data
    pub issuer_signed: IssuerSigned,
    /// Device signed data (optional)
    pub device_signed: Option<DeviceSigned>,
}

/// Issuer signed portion of mDOC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssuerSigned {
    /// Name spaces with signed items
    pub name_spaces: HashMap<String, Vec<IssuerSignedItem>>,
    /// Issuer authentication (COSE_Sign1)
    pub issuer_auth: CoseSign1,
}

/// Individual signed item
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IssuerSignedItem {
    /// Digest ID
    pub digest_id: u64,
    /// Random salt
    pub random: Vec<u8>,
    /// Element identifier
    pub element_identifier: String,
    /// Element value
    pub element_value: Value,
}

/// Device signed portion
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceSigned {
    /// Device authentication
    pub device_auth: DeviceAuth,
}

/// Device authentication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceAuth {
    /// Signature or MAC
    pub device_signature: Option<CoseSign1>,
    pub device_mac: Option<Value>,
}

/// Mobile security object (MSO) 
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MobileSecurityObject {
    /// Version
    pub version: String,
    /// Digest algorithm
    pub digest_algorithm: String,
    /// Value digests by namespace
    pub value_digests: HashMap<String, HashMap<u64, Vec<u8>>>,
    /// Device key info
    pub device_key_info: DeviceKeyInfo,
    /// Document type
    pub doc_type: String,
    /// Validity info
    pub validity_info: ValidityInfo,
}

/// Device key information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceKeyInfo {
    /// Device key
    pub device_key: Value,
}

/// Validity information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidityInfo {
    /// Signed timestamp
    pub signed: String,
    /// Valid from timestamp
    pub valid_from: String,
    /// Valid until timestamp
    pub valid_until: String,
}

impl Document {
    /// Parse from CBOR bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let value = Value::from_bytes(data)?;
        Self::from_value(&value)
    }
    
    /// Parse from CBOR value
    pub fn from_value(value: &Value) -> Result<Self> {
        let map = value.as_map()
            .ok_or_else(|| LongfellowError::ParseError("Document must be a map".to_string()))?;
        
        let doc_type = map.get("docType")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LongfellowError::ParseError("Missing docType".to_string()))?
            .to_string();
        
        let issuer_signed = map.get("issuerSigned")
            .ok_or_else(|| LongfellowError::ParseError("Missing issuerSigned".to_string()))?;
        
        let issuer_signed = IssuerSigned::from_value(issuer_signed)?;
        
        let device_signed = map.get("deviceSigned")
            .map(|v| DeviceSigned::from_value(v))
            .transpose()?;
        
        Ok(Self {
            doc_type,
            issuer_signed,
            device_signed,
        })
    }
    
    /// Get all namespaces
    pub fn namespaces(&self) -> Vec<&str> {
        self.issuer_signed.name_spaces.keys()
            .map(|k| k.as_str())
            .collect()
    }
    
    /// Get items for a namespace
    pub fn get_namespace_items(&self, namespace: &str) -> Option<&[IssuerSignedItem]> {
        self.issuer_signed.name_spaces.get(namespace)
            .map(|v| v.as_slice())
    }
}

impl IssuerSigned {
    /// Parse from CBOR value
    pub fn from_value(value: &Value) -> Result<Self> {
        let map = value.as_map()
            .ok_or_else(|| LongfellowError::ParseError("IssuerSigned must be a map".to_string()))?;
        
        let name_spaces_value = map.get("nameSpaces")
            .ok_or_else(|| LongfellowError::ParseError("Missing nameSpaces".to_string()))?;
        
        let name_spaces = Self::parse_name_spaces(name_spaces_value)?;
        
        let issuer_auth_bytes = map.get("issuerAuth")
            .and_then(|v| v.as_bytes())
            .ok_or_else(|| LongfellowError::ParseError("Missing issuerAuth".to_string()))?;
        
        let issuer_auth = CoseSign1::from_bytes(issuer_auth_bytes)?;
        
        Ok(Self {
            name_spaces,
            issuer_auth,
        })
    }
    
    /// Parse namespaces
    fn parse_name_spaces(value: &Value) -> Result<HashMap<String, Vec<IssuerSignedItem>>> {
        let map = value.as_map()
            .ok_or_else(|| LongfellowError::ParseError("nameSpaces must be a map".to_string()))?;
        
        let mut result = HashMap::new();
        
        for (namespace, items_value) in map {
            let items_array = items_value.as_array()
                .ok_or_else(|| LongfellowError::ParseError("Namespace items must be array".to_string()))?;
            
            let mut items = Vec::new();
            for item_value in items_array {
                items.push(IssuerSignedItem::from_value(item_value)?);
            }
            
            result.insert(namespace.clone(), items);
        }
        
        Ok(result)
    }
}

impl IssuerSignedItem {
    /// Parse from CBOR value
    pub fn from_value(value: &Value) -> Result<Self> {
        let map = value.as_map()
            .ok_or_else(|| LongfellowError::ParseError("IssuerSignedItem must be a map".to_string()))?;
        
        let digest_id = map.get("digestID")
            .and_then(|v| v.as_i64())
            .ok_or_else(|| LongfellowError::ParseError("Missing digestID".to_string()))? as u64;
        
        let random = map.get("random")
            .and_then(|v| v.as_bytes())
            .ok_or_else(|| LongfellowError::ParseError("Missing random".to_string()))?
            .to_vec();
        
        let element_identifier = map.get("elementIdentifier")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LongfellowError::ParseError("Missing elementIdentifier".to_string()))?
            .to_string();
        
        let element_value = map.get("elementValue")
            .ok_or_else(|| LongfellowError::ParseError("Missing elementValue".to_string()))?
            .clone();
        
        Ok(Self {
            digest_id,
            random,
            element_identifier,
            element_value,
        })
    }
}

impl DeviceSigned {
    /// Parse from CBOR value
    pub fn from_value(value: &Value) -> Result<Self> {
        let map = value.as_map()
            .ok_or_else(|| LongfellowError::ParseError("DeviceSigned must be a map".to_string()))?;
        
        let device_auth_value = map.get("deviceAuth")
            .ok_or_else(|| LongfellowError::ParseError("Missing deviceAuth".to_string()))?;
        
        let device_auth = DeviceAuth::from_value(device_auth_value)?;
        
        Ok(Self { device_auth })
    }
}

impl DeviceAuth {
    /// Parse from CBOR value
    pub fn from_value(value: &Value) -> Result<Self> {
        let map = value.as_map()
            .ok_or_else(|| LongfellowError::ParseError("DeviceAuth must be a map".to_string()))?;
        
        let device_signature = map.get("deviceSignature")
            .and_then(|v| v.as_bytes())
            .map(|b| CoseSign1::from_bytes(b))
            .transpose()?;
        
        let device_mac = map.get("deviceMac").cloned();
        
        Ok(Self {
            device_signature,
            device_mac,
        })
    }
}

impl ClaimExtractor for Document {
    fn extract_claims(&self) -> Result<HashMap<String, Value>> {
        let mut claims = HashMap::new();
        
        for (namespace, items) in &self.issuer_signed.name_spaces {
            for item in items {
                let key = format!("{}.{}", namespace, item.element_identifier);
                claims.insert(key, item.element_value.clone());
            }
        }
        
        Ok(claims)
    }
    
    fn get_claim(&self, path: &str) -> Option<&Value> {
        let parts: Vec<&str> = path.split('.').collect();
        if parts.len() < 2 {
            return None;
        }
        
        let namespace = parts[0];
        let element_id = parts[1..].join(".");
        
        self.issuer_signed.name_spaces.get(namespace)
            .and_then(|items| {
                items.iter()
                    .find(|item| item.element_identifier == element_id)
                    .map(|item| &item.element_value)
            })
    }
}

/// Common mDOC element identifiers
pub mod elements {
    pub const FAMILY_NAME: &str = "family_name";
    pub const GIVEN_NAME: &str = "given_name";
    pub const BIRTH_DATE: &str = "birth_date";
    pub const ISSUE_DATE: &str = "issue_date";
    pub const EXPIRY_DATE: &str = "expiry_date";
    pub const ISSUING_COUNTRY: &str = "issuing_country";
    pub const ISSUING_AUTHORITY: &str = "issuing_authority";
    pub const DOCUMENT_NUMBER: &str = "document_number";
    pub const PORTRAIT: &str = "portrait";
    pub const DRIVING_PRIVILEGES: &str = "driving_privileges";
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mdoc_claim_extraction() {
        // This would test actual mDOC parsing
        // For now, test the structure
        let mut name_spaces = HashMap::new();
        let items = vec![
            IssuerSignedItem {
                digest_id: 0,
                random: vec![1, 2, 3, 4],
                element_identifier: "given_name".to_string(),
                element_value: Value::Text("John".to_string()),
            },
            IssuerSignedItem {
                digest_id: 1,
                random: vec![5, 6, 7, 8],
                element_identifier: "family_name".to_string(),
                element_value: Value::Text("Doe".to_string()),
            },
        ];
        
        name_spaces.insert(MDL_NAMESPACE.to_string(), items);
        
        let doc = Document {
            doc_type: MDOC_DOCTYPE.to_string(),
            issuer_signed: IssuerSigned {
                name_spaces,
                issuer_auth: CoseSign1 {
                    protected: vec![],
                    unprotected: HashMap::new(),
                    payload: vec![],
                    signature: vec![],
                },
            },
            device_signed: None,
        };
        
        let claims = doc.extract_claims().unwrap();
        assert_eq!(claims.len(), 2);
        
        let given_name = doc.get_claim(&format!("{}.given_name", MDL_NAMESPACE));
        assert_eq!(given_name.and_then(|v| v.as_str()), Some("John"));
    }
}