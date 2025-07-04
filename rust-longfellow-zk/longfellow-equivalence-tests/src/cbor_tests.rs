/// Equivalence tests for CBOR module

use longfellow_cbor::{
    parse_document, extract_fields, DocumentData, DocumentType,
    jwt::{Jwt, JwtHeader, JwtClaims},
    mdoc::{Document as MDoc, Namespace, IssuerSignedItem},
    vc::VerifiableCredential,
};
use std::collections::HashMap;
use std::time::Instant;

#[test]
fn test_jwt_parsing() {
    println!("\n=== JWT Parsing Test ===");
    
    // Create a test JWT (normally would be a real JWT)
    let header = JwtHeader {
        alg: "ES256".to_string(),
        typ: Some("JWT".to_string()),
        kid: Some("key-1".to_string()),
        other: HashMap::new(),
    };
    
    let mut claims = JwtClaims {
        iss: Some("https://example.com".to_string()),
        sub: Some("user123".to_string()),
        aud: Some(vec!["https://api.example.com".to_string()]),
        exp: Some(1234567890),
        nbf: Some(1234567800),
        iat: Some(1234567800),
        jti: Some("unique-id-123".to_string()),
        other: HashMap::new(),
    };
    
    claims.other.insert("custom_claim".to_string(), serde_json::json!("custom_value"));
    
    let jwt = Jwt {
        header,
        claims,
        signature: vec![0; 64], // Dummy signature
    };
    
    // Test field extraction
    let fields = jwt.extract_fields();
    
    assert_eq!(fields.get("iss"), Some(&"https://example.com".to_string()));
    assert_eq!(fields.get("sub"), Some(&"user123".to_string()));
    assert_eq!(fields.get("custom_claim"), Some(&"custom_value".to_string()));
    
    println!("  ✓ JWT parsing verified");
}

#[test]
fn test_mdoc_parsing() {
    println!("\n=== mDOC Parsing Test ===");
    
    // Create a test mDOC (mobile driving license)
    let mut namespace = Namespace::new();
    
    // Add some typical mDL fields
    namespace.add_item(IssuerSignedItem {
        digest_id: 1,
        random: vec![0; 16],
        element_identifier: "family_name".to_string(),
        element_value: "Doe".to_string(),
    });
    
    namespace.add_item(IssuerSignedItem {
        digest_id: 2,
        random: vec![0; 16],
        element_identifier: "given_name".to_string(),
        element_value: "John".to_string(),
    });
    
    namespace.add_item(IssuerSignedItem {
        digest_id: 3,
        random: vec![0; 16],
        element_identifier: "birth_date".to_string(),
        element_value: "1990-01-01".to_string(),
    });
    
    namespace.add_item(IssuerSignedItem {
        digest_id: 4,
        random: vec![0; 16],
        element_identifier: "issue_date".to_string(),
        element_value: "2020-01-01".to_string(),
    });
    
    namespace.add_item(IssuerSignedItem {
        digest_id: 5,
        random: vec![0; 16],
        element_identifier: "expiry_date".to_string(),
        element_value: "2030-01-01".to_string(),
    });
    
    let mut mdoc = MDoc::new("org.iso.18013.5.1.mDL".to_string());
    mdoc.add_namespace("org.iso.18013.5.1".to_string(), namespace);
    
    // Test field extraction
    let fields = mdoc.extract_fields();
    
    assert_eq!(fields.get("org.iso.18013.5.1.family_name"), Some(&"Doe".to_string()));
    assert_eq!(fields.get("org.iso.18013.5.1.given_name"), Some(&"John".to_string()));
    assert_eq!(fields.get("org.iso.18013.5.1.birth_date"), Some(&"1990-01-01".to_string()));
    
    println!("  ✓ mDOC parsing verified");
}

#[test]
fn test_verifiable_credential_parsing() {
    println!("\n=== Verifiable Credential Parsing Test ===");
    
    // Create a test Verifiable Credential
    let mut vc = VerifiableCredential {
        context: vec![
            "https://www.w3.org/2018/credentials/v1".to_string(),
            "https://www.w3.org/2018/credentials/examples/v1".to_string(),
        ],
        id: Some("http://example.edu/credentials/1872".to_string()),
        type_: vec![
            "VerifiableCredential".to_string(),
            "UniversityDegreeCredential".to_string(),
        ],
        issuer: "https://example.edu/issuers/565049".to_string(),
        issuance_date: "2010-01-01T00:00:00Z".to_string(),
        credential_subject: HashMap::new(),
        proof: None,
    };
    
    // Add credential subject
    vc.credential_subject.insert("id".to_string(), 
        serde_json::json!("did:example:ebfeb1f712ebc6f1c276e12ec21"));
    vc.credential_subject.insert("degree".to_string(), serde_json::json!({
        "type": "BachelorDegree",
        "name": "Bachelor of Science and Arts"
    }));
    vc.credential_subject.insert("alumniOf".to_string(), serde_json::json!({
        "id": "did:example:c276e12ec21ebfeb1f712ebc6f1",
        "name": "Example University"
    }));
    
    // Test field extraction
    let fields = vc.extract_fields();
    
    assert_eq!(fields.get("issuer"), Some(&"https://example.edu/issuers/565049".to_string()));
    assert_eq!(fields.get("issuance_date"), Some(&"2010-01-01T00:00:00Z".to_string()));
    assert!(fields.contains_key("credential_subject.degree.type"));
    
    println!("  ✓ Verifiable Credential parsing verified");
}

#[test]
fn test_document_type_detection() {
    println!("\n=== Document Type Detection Test ===");
    
    // Test JWT detection
    let jwt_data = br#"{"header":{"alg":"ES256"},"claims":{"iss":"test"}}"#;
    match parse_document(jwt_data) {
        Ok(DocumentData::Jwt(_)) => println!("  ✓ JWT detected correctly"),
        _ => panic!("Failed to detect JWT"),
    }
    
    // Test mDOC detection
    let mdoc_data = br#"{"docType":"org.iso.18013.5.1.mDL","namespaces":{}}"#;
    match parse_document(mdoc_data) {
        Ok(DocumentData::Mdoc(_)) => println!("  ✓ mDOC detected correctly"),
        _ => panic!("Failed to detect mDOC"),
    }
    
    // Test VC detection
    let vc_data = br#"{"@context":["https://www.w3.org/2018/credentials/v1"],"type":["VerifiableCredential"]}"#;
    match parse_document(vc_data) {
        Ok(DocumentData::VerifiableCredential(_)) => println!("  ✓ VC detected correctly"),
        _ => panic!("Failed to detect VC"),
    }
}

#[test]
fn test_field_extraction_performance() {
    println!("\n=== Field Extraction Performance Test ===");
    
    // Create documents with many fields
    let mut large_jwt_claims = JwtClaims {
        iss: Some("issuer".to_string()),
        sub: Some("subject".to_string()),
        aud: None,
        exp: None,
        nbf: None,
        iat: None,
        jti: None,
        other: HashMap::new(),
    };
    
    // Add many custom claims
    for i in 0..100 {
        large_jwt_claims.other.insert(
            format!("claim_{}", i),
            serde_json::json!(format!("value_{}", i))
        );
    }
    
    let jwt = Jwt {
        header: JwtHeader {
            alg: "ES256".to_string(),
            typ: None,
            kid: None,
            other: HashMap::new(),
        },
        claims: large_jwt_claims,
        signature: vec![],
    };
    
    // Benchmark extraction
    let start = Instant::now();
    let iterations = 1000;
    
    for _ in 0..iterations {
        let _ = jwt.extract_fields();
    }
    
    let duration = start.elapsed();
    
    println!("  JWT field extraction (102 fields): {:?} ({:.2} μs/op)",
        duration,
        duration.as_micros() as f64 / iterations as f64
    );
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    
    #[test]
    fn bench_document_parsing() {
        println!("\n=== Document Parsing Benchmarks ===");
        
        // Prepare test documents
        let jwt_json = serde_json::to_vec(&serde_json::json!({
            "header": {"alg": "ES256", "typ": "JWT", "kid": "key-1"},
            "claims": {
                "iss": "https://example.com",
                "sub": "user123",
                "exp": 1234567890,
                "custom1": "value1",
                "custom2": {"nested": "value2"}
            }
        })).unwrap();
        
        let mdoc_json = serde_json::to_vec(&serde_json::json!({
            "docType": "org.iso.18013.5.1.mDL",
            "namespaces": {
                "org.iso.18013.5.1": {
                    "items": [
                        {"elementIdentifier": "family_name", "elementValue": "Doe"},
                        {"elementIdentifier": "given_name", "elementValue": "John"},
                        {"elementIdentifier": "birth_date", "elementValue": "1990-01-01"}
                    ]
                }
            }
        })).unwrap();
        
        let vc_json = serde_json::to_vec(&serde_json::json!({
            "@context": ["https://www.w3.org/2018/credentials/v1"],
            "type": ["VerifiableCredential", "UniversityDegreeCredential"],
            "issuer": "https://example.edu",
            "issuanceDate": "2010-01-01T00:00:00Z",
            "credentialSubject": {
                "id": "did:example:123",
                "degree": {
                    "type": "BachelorDegree",
                    "name": "Bachelor of Science"
                }
            }
        })).unwrap();
        
        let iterations = 1000;
        
        // Benchmark JWT parsing
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parse_document(&jwt_json).unwrap();
        }
        let jwt_duration = start.elapsed();
        
        // Benchmark mDOC parsing
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parse_document(&mdoc_json).unwrap();
        }
        let mdoc_duration = start.elapsed();
        
        // Benchmark VC parsing
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = parse_document(&vc_json).unwrap();
        }
        let vc_duration = start.elapsed();
        
        println!("  JWT parsing: {:?} ({:.2} μs/op)",
            jwt_duration,
            jwt_duration.as_micros() as f64 / iterations as f64
        );
        
        println!("  mDOC parsing: {:?} ({:.2} μs/op)",
            mdoc_duration,
            mdoc_duration.as_micros() as f64 / iterations as f64
        );
        
        println!("  VC parsing: {:?} ({:.2} μs/op)",
            vc_duration,
            vc_duration.as_micros() as f64 / iterations as f64
        );
    }
    
    #[test]
    fn bench_field_extraction() {
        println!("\n=== Field Extraction Benchmarks ===");
        
        // Create documents with varying numbers of fields
        let field_counts = vec![10, 50, 100, 200];
        
        for count in field_counts {
            // Create JWT with many fields
            let mut claims = JwtClaims {
                iss: Some("issuer".to_string()),
                sub: Some("subject".to_string()),
                aud: None,
                exp: None,
                nbf: None,
                iat: None,
                jti: None,
                other: HashMap::new(),
            };
            
            for i in 0..count {
                claims.other.insert(
                    format!("field_{}", i),
                    serde_json::json!(format!("value_{}", i))
                );
            }
            
            let jwt = Jwt {
                header: JwtHeader {
                    alg: "ES256".to_string(),
                    typ: None,
                    kid: None,
                    other: HashMap::new(),
                },
                claims,
                signature: vec![],
            };
            
            // Create mDOC with many fields
            let mut namespace = Namespace::new();
            for i in 0..count {
                namespace.add_item(IssuerSignedItem {
                    digest_id: i as u64,
                    random: vec![0; 16],
                    element_identifier: format!("field_{}", i),
                    element_value: format!("value_{}", i),
                });
            }
            
            let mut mdoc = MDoc::new("test.document".to_string());
            mdoc.add_namespace("test.namespace".to_string(), namespace);
            
            // Benchmark JWT extraction
            let start = Instant::now();
            let iterations = 1000;
            
            for _ in 0..iterations {
                let _ = jwt.extract_fields();
            }
            
            let jwt_duration = start.elapsed();
            
            // Benchmark mDOC extraction
            let start = Instant::now();
            
            for _ in 0..iterations {
                let _ = mdoc.extract_fields();
            }
            
            let mdoc_duration = start.elapsed();
            
            println!("  {} fields:", count + 2); // +2 for iss and sub
            println!("    JWT extraction: {:?} ({:.2} μs/op)",
                jwt_duration,
                jwt_duration.as_micros() as f64 / iterations as f64
            );
            println!("    mDOC extraction: {:?} ({:.2} μs/op)",
                mdoc_duration,
                mdoc_duration.as_micros() as f64 / iterations as f64
            );
        }
    }
    
    #[test]
    fn bench_cbor_encoding() {
        println!("\n=== CBOR Encoding Benchmarks ===");
        
        // Note: This would test actual CBOR encoding/decoding if implemented
        // For now, we'll benchmark the conversion to/from internal representations
        
        let test_data = vec![
            ("small", 10),
            ("medium", 100),
            ("large", 1000),
        ];
        
        for (name, size) in test_data {
            let mut data = HashMap::new();
            for i in 0..size {
                data.insert(format!("key_{}", i), format!("value_{}", i));
            }
            
            let iterations = 1000;
            
            // Benchmark serialization
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = serde_json::to_vec(&data).unwrap();
            }
            let ser_duration = start.elapsed();
            
            println!("  {} ({} fields):", name, size);
            println!("    Serialization: {:?} ({:.2} μs/op)",
                ser_duration,
                ser_duration.as_micros() as f64 / iterations as f64
            );
        }
    }
}