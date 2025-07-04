/// Statement definitions for zero-knowledge proofs

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A statement to be proven in zero-knowledge
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Statement {
    /// Document type
    pub document_type: DocumentType,
    
    /// Predicates to prove
    pub predicates: Vec<Predicate>,
    
    /// Fields to reveal
    pub revealed_fields: Vec<String>,
    
    /// Fields to keep private
    pub private_fields: Vec<String>,
    
    /// Additional context
    pub context: HashMap<String, String>,
}

impl Statement {
    /// Create a new statement
    pub fn new(document_type: DocumentType) -> Self {
        Self {
            document_type,
            predicates: Vec::new(),
            revealed_fields: Vec::new(),
            private_fields: Vec::new(),
            context: HashMap::new(),
        }
    }
    
    /// Add a predicate
    pub fn add_predicate(mut self, predicate: Predicate) -> Self {
        self.predicates.push(predicate);
        self
    }
    
    /// Add a revealed field
    pub fn reveal_field(mut self, field: String) -> Self {
        self.revealed_fields.push(field);
        self
    }
    
    /// Add a private field
    pub fn keep_private(mut self, field: String) -> Self {
        self.private_fields.push(field);
        self
    }
    
    /// Add context
    pub fn with_context(mut self, key: String, value: String) -> Self {
        self.context.insert(key, value);
        self
    }
    
    /// Validate the statement
    pub fn validate(&self) -> Result<(), String> {
        // Check for duplicate fields
        for field in &self.revealed_fields {
            if self.private_fields.contains(field) {
                return Err(format!("Field {} cannot be both revealed and private", field));
            }
        }
        
        // Validate predicates
        for predicate in &self.predicates {
            predicate.validate()?;
        }
        
        Ok(())
    }
}

/// Document types supported
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentType {
    /// JSON Web Token
    Jwt,
    /// ISO mDOC
    Mdoc,
    /// W3C Verifiable Credential
    VerifiableCredential,
    /// Custom document type
    Custom(&'static str),
}

/// Predicates that can be proven
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Predicate {
    /// Field equals a specific value
    FieldEquals {
        field: String,
        value: String,
    },
    
    /// Field exists in the document
    FieldExists {
        field: String,
    },
    
    /// Field is greater than a value
    FieldGreaterThan {
        field: String,
        value: i64,
    },
    
    /// Age is over a certain number of years
    AgeOver {
        years: u32,
    },
    
    /// Document has valid signature
    ValidSignature,
    
    /// Document is from a specific issuer
    ValidIssuer {
        issuer: String,
    },
    
    /// Document is not expired
    NotExpired,
    
    /// Custom predicate
    Custom {
        id: String,
        params: HashMap<String, String>,
    },
}

impl Predicate {
    /// Validate the predicate
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Self::FieldEquals { field, .. } | Self::FieldExists { field } => {
                if field.is_empty() {
                    return Err("Field name cannot be empty".to_string());
                }
            }
            Self::FieldGreaterThan { field, value } => {
                if field.is_empty() {
                    return Err("Field name cannot be empty".to_string());
                }
                if *value < 0 {
                    return Err("Comparison value must be non-negative".to_string());
                }
            }
            Self::AgeOver { years } => {
                if *years == 0 || *years > 150 {
                    return Err("Age must be between 1 and 150 years".to_string());
                }
            }
            Self::ValidIssuer { issuer } => {
                if issuer.is_empty() {
                    return Err("Issuer cannot be empty".to_string());
                }
            }
            Self::Custom { id, .. } => {
                if id.is_empty() {
                    return Err("Custom predicate ID cannot be empty".to_string());
                }
            }
            _ => {}
        }
        Ok(())
    }
    
    /// Get fields referenced by this predicate
    pub fn referenced_fields(&self) -> Vec<&str> {
        match self {
            Self::FieldEquals { field, .. } |
            Self::FieldExists { field } |
            Self::FieldGreaterThan { field, .. } => vec![field.as_str()],
            Self::AgeOver { .. } => vec!["birthDate", "birth_date", "dateOfBirth"],
            Self::ValidSignature => vec!["signature"],
            Self::ValidIssuer { .. } => vec!["issuer", "iss"],
            Self::NotExpired => vec!["exp", "expirationDate", "validUntil"],
            Self::Custom { .. } => vec![],
        }
    }
}

/// Common statement templates
pub mod templates {
    use super::*;
    
    /// Create an age verification statement
    pub fn age_verification(min_age: u32, reveal_name: bool) -> Statement {
        let mut stmt = Statement::new(DocumentType::Mdoc)
            .add_predicate(Predicate::ValidSignature)
            .add_predicate(Predicate::NotExpired)
            .add_predicate(Predicate::AgeOver { years: min_age });
        
        if reveal_name {
            stmt = stmt.reveal_field("given_name".to_string())
                      .reveal_field("family_name".to_string());
        }
        
        stmt.keep_private("birth_date".to_string())
    }
    
    /// Create a credential verification statement
    pub fn credential_verification(credential_type: &str, issuer: &str) -> Statement {
        Statement::new(DocumentType::VerifiableCredential)
            .add_predicate(Predicate::ValidSignature)
            .add_predicate(Predicate::NotExpired)
            .add_predicate(Predicate::ValidIssuer { issuer: issuer.to_string() })
            .add_predicate(Predicate::FieldEquals {
                field: "type".to_string(),
                value: credential_type.to_string(),
            })
            .reveal_field("type".to_string())
            .reveal_field("issuanceDate".to_string())
    }
    
    /// Create a selective disclosure statement
    pub fn selective_disclosure(fields_to_reveal: Vec<String>) -> Statement {
        let mut stmt = Statement::new(DocumentType::Jwt)
            .add_predicate(Predicate::ValidSignature);
        
        for field in fields_to_reveal {
            stmt = stmt.reveal_field(field);
        }
        
        stmt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_statement_creation() {
        let stmt = Statement::new(DocumentType::Jwt)
            .add_predicate(Predicate::ValidSignature)
            .add_predicate(Predicate::FieldExists { field: "sub".to_string() })
            .reveal_field("iss".to_string())
            .keep_private("email".to_string());
        
        assert_eq!(stmt.predicates.len(), 2);
        assert_eq!(stmt.revealed_fields.len(), 1);
        assert_eq!(stmt.private_fields.len(), 1);
        
        assert!(stmt.validate().is_ok());
    }
    
    #[test]
    fn test_invalid_statement() {
        let stmt = Statement::new(DocumentType::Mdoc)
            .reveal_field("name".to_string())
            .keep_private("name".to_string());
        
        assert!(stmt.validate().is_err());
    }
    
    #[test]
    fn test_predicate_validation() {
        assert!(Predicate::FieldEquals {
            field: "".to_string(),
            value: "test".to_string(),
        }.validate().is_err());
        
        assert!(Predicate::AgeOver { years: 0 }.validate().is_err());
        assert!(Predicate::AgeOver { years: 200 }.validate().is_err());
        assert!(Predicate::AgeOver { years: 18 }.validate().is_ok());
    }
    
    #[test]
    fn test_templates() {
        let age_stmt = templates::age_verification(21, true);
        assert!(age_stmt.predicates.iter().any(|p| matches!(p, Predicate::AgeOver { years: 21 })));
        assert!(age_stmt.revealed_fields.contains(&"given_name".to_string()));
        assert!(age_stmt.private_fields.contains(&"birth_date".to_string()));
    }
}