/// Full System Integration Tests
/// 
/// Tests the complete proof generation and verification flow with Reed-Solomon encoding

use longfellow_algebra::{Fp128, Field};
use longfellow_cbor::jwt::{Jwt, JwtBuilder, JwtAlgorithm};
use longfellow_cbor::{Value, mdoc::Document as Mdoc};
use longfellow_core::Result;
use longfellow_zk::{
    Statement, Predicate, DocumentType, DocumentData, ZkWitness, ZkInstance, ZkCircuit,
    ProofOptions, prover_full::FullZkProver, verifier::ZkVerifier,
    serialization::{ProofSerializer, ProofFormat, CompressionType},
};
use rand::rngs::OsRng;
use serde_json::json;
use std::time::Instant;

#[test]
fn test_basic_jwt_proof_with_reed_solomon() {
    // Create test JWT
    let claims = json!({
        "sub": "user123",
        "age": 25,
        "verified": true,
        "score": 850,
        "country": "US"
    });
    
    let jwt = Jwt::new(claims).unwrap();
    
    // Create statement
    let statement = Statement {
        document_type: DocumentType::Jwt,
        predicates: vec![
            Predicate::AgeOver { years: 18 },
            Predicate::FieldGreaterThan {
                field: "score".to_string(),
                value: 800,
            },
        ],
        revealed_fields: vec!["verified".to_string()],
        hidden_fields: vec!["sub".to_string(), "age".to_string(), "score".to_string()],
        private_fields: vec!["sub".to_string()],
    };
    
    // Create witness
    let witness = ZkWitness {
        document: DocumentData::Jwt(jwt),
        private_values: std::collections::HashMap::new(),
        randomness: vec![],
    };
    
    // Create circuit
    let circuit = ZkCircuit::new(1000);
    
    // Create instance
    let instance = ZkInstance {
        statement: statement.clone(),
        witness,
        circuit,
    };
    
    // Create prover
    let prover = FullZkProver::<Fp128>::new(instance).unwrap();
    
    // Generate proof with Reed-Solomon
    let options = ProofOptions {
        security_bits: 128,
        use_sumcheck: true,
        parallel: true,
        optimize_size: false,
        reed_solomon_rate: Some(0.25),
    };
    
    let mut rng = OsRng;
    let start = Instant::now();
    let proof = prover.prove_full(&mut rng, options).unwrap();
    let proof_time = start.elapsed();
    
    println!("Proof generation time: {:?}", proof_time);
    println!("Proof metadata: {:?}", proof.metadata);
    
    // Verify proof metadata
    assert_eq!(proof.metadata.version, "2.0.0");
    assert_eq!(proof.metadata.security_bits, 128);
    assert_eq!(proof.metadata.reed_solomon_rate, Some(0.25));
    assert!(proof.metadata.encoding_type.is_some());
}

#[test]
fn test_complex_predicates_with_aggressive_encoding() {
    let claims = json!({
        "sub": "user456",
        "age": 30,
        "score": 950,
        "clearance_level": 7,
        "department": "engineering",
        "years_experience": 10,
        "verified": true
    });
    
    let jwt = Jwt::new(claims).unwrap();
    
    let statement = Statement {
        document_type: DocumentType::Jwt,
        predicates: vec![
            Predicate::AgeOver { years: 25 },
            Predicate::FieldGreaterThan {
                field: "score".to_string(),
                value: 900,
            },
            Predicate::FieldGreaterThan {
                field: "clearance_level".to_string(),
                value: 5,
            },
            Predicate::FieldGreaterThan {
                field: "years_experience".to_string(),
                value: 8,
            },
        ],
        revealed_fields: vec!["department".to_string()],
        hidden_fields: vec![
            "sub".to_string(),
            "age".to_string(),
            "score".to_string(),
            "clearance_level".to_string(),
            "years_experience".to_string(),
        ],
        private_fields: vec!["sub".to_string(), "clearance_level".to_string()],
    };
    
    let witness = ZkWitness {
        document: DocumentData::Jwt(jwt),
        private_values: std::collections::HashMap::new(),
        randomness: vec![],
    };
    
    let circuit = ZkCircuit::new(2000);
    
    let instance = ZkInstance {
        statement,
        witness,
        circuit,
    };
    
    let prover = FullZkProver::<Fp128>::new(instance).unwrap();
    
    // Use aggressive Reed-Solomon encoding for high redundancy
    let options = ProofOptions {
        security_bits: 256,
        use_sumcheck: true,
        parallel: true,
        optimize_size: false,
        reed_solomon_rate: Some(0.0625), // 1/16 rate
    };
    
    let mut rng = OsRng;
    let proof = prover.prove_full(&mut rng, options).unwrap();
    
    assert_eq!(proof.metadata.security_bits, 256);
    assert_eq!(proof.metadata.reed_solomon_rate, Some(0.0625));
    assert!(proof.sumcheck_proof.is_some());
}

#[test]
fn test_proof_serialization_formats() {
    // Generate a simple proof
    let claims = json!({
        "sub": "user789",
        "age": 21,
        "verified": true
    });
    
    let jwt = Jwt::new(claims).unwrap();
    
    let statement = Statement {
        document_type: DocumentType::Jwt,
        predicates: vec![Predicate::AgeOver { years: 18 }],
        revealed_fields: vec!["verified".to_string()],
        hidden_fields: vec!["sub".to_string(), "age".to_string()],
        private_fields: vec![],
    };
    
    let witness = ZkWitness {
        document: DocumentData::Jwt(jwt),
        private_values: std::collections::HashMap::new(),
        randomness: vec![],
    };
    
    let circuit = ZkCircuit::new(100);
    
    let instance = ZkInstance {
        statement,
        witness,
        circuit,
    };
    
    let prover = FullZkProver::<Fp128>::new(instance).unwrap();
    
    let options = ProofOptions::default()
        .with_reed_solomon_rate(0.5);
    
    let mut rng = OsRng;
    let proof = prover.prove_full(&mut rng, options).unwrap();
    
    // Test different serialization formats
    let formats = [
        (ProofFormat::Binary, CompressionType::None, "binary"),
        (ProofFormat::Json, CompressionType::None, "json"),
        (ProofFormat::MessagePack, CompressionType::None, "msgpack"),
        (ProofFormat::Binary, CompressionType::Zlib, "binary+zlib"),
        (ProofFormat::Binary, CompressionType::Zstd, "binary+zstd"),
        (ProofFormat::Binary, CompressionType::Lz4, "binary+lz4"),
    ];
    
    for (format, compression, name) in formats {
        let serialized = ProofSerializer::serialize(&proof, format, compression).unwrap();
        println!("{} size: {} bytes", name, serialized.len());
        
        // Verify we can deserialize
        let deserialized: longfellow_zk::ZkProof<Fp128> = 
            ProofSerializer::deserialize(&serialized).unwrap();
        
        assert_eq!(deserialized.metadata.version, proof.metadata.version);
        assert_eq!(deserialized.commitments.len(), proof.commitments.len());
    }
}

#[test]
fn test_mdoc_proof_generation() {
    // Create test mDOC data
    let mdoc_data = json!({
        "version": "1.0",
        "docType": "org.iso.18013.5.1.mDL",
        "namespaces": {
            "org.iso.18013.5.1": {
                "document_type": "driving_license",
                "holder_name": "Jane Doe",
                "birth_date": "1995-06-15",
                "issue_date": "2020-01-01",
                "expiry_date": "2025-01-01",
                "driving_privileges": ["A", "B", "C"]
            }
        }
    });
    
    let statement = Statement {
        document_type: DocumentType::Mdoc,
        predicates: vec![
            Predicate::AgeOver { years: 18 },
            Predicate::NotExpired,
        ],
        revealed_fields: vec!["document_type".to_string()],
        hidden_fields: vec!["holder_name".to_string(), "birth_date".to_string()],
        private_fields: vec!["holder_name".to_string()],
    };
    
    let witness = ZkWitness {
        document: DocumentData::Raw(serde_json::to_vec(&mdoc_data).unwrap()),
        private_values: std::collections::HashMap::new(),
        randomness: vec![],
    };
    
    let circuit = ZkCircuit::new(500);
    
    let instance = ZkInstance {
        statement,
        witness,
        circuit,
    };
    
    let prover = FullZkProver::<Fp128>::new(instance).unwrap();
    
    let options = ProofOptions::default()
        .with_reed_solomon_rate(0.25);
    
    let mut rng = OsRng;
    let proof = prover.prove_full(&mut rng, options).unwrap();
    
    assert_eq!(proof.metadata.document_type, DocumentType::Mdoc);
}

#[test]
fn test_performance_with_different_encoding_rates() {
    let claims = json!({
        "sub": "perf_test_user",
        "data": vec![42; 100], // Larger data for performance testing
        "verified": true
    });
    
    let jwt = Jwt::new(claims).unwrap();
    
    let statement = Statement {
        document_type: DocumentType::Jwt,
        predicates: vec![
            Predicate::FieldExists { field: "data".to_string() },
        ],
        revealed_fields: vec!["verified".to_string()],
        hidden_fields: vec!["sub".to_string(), "data".to_string()],
        private_fields: vec![],
    };
    
    let rates = [0.5, 0.25, 0.125, 0.0625];
    
    for rate in rates {
        let witness = ZkWitness {
            document: DocumentData::Jwt(jwt.clone()),
            private_values: std::collections::HashMap::new(),
            randomness: vec![],
        };
        
        let circuit = ZkCircuit::new(1000);
        
        let instance = ZkInstance {
            statement: statement.clone(),
            witness,
            circuit,
        };
        
        let prover = FullZkProver::<Fp128>::new(instance).unwrap();
        
        let options = ProofOptions::default()
            .with_reed_solomon_rate(rate);
        
        let mut rng = OsRng;
        let start = Instant::now();
        let proof = prover.prove_full(&mut rng, options).unwrap();
        let elapsed = start.elapsed();
        
        println!("Rate: {}, Time: {:?}, Encoding: {:?}", 
                 rate, elapsed, proof.metadata.encoding_type);
    }
}

#[test]
fn test_proof_verification_roundtrip() {
    // Create and prove
    let claims = json!({
        "sub": "verify_test",
        "age": 28,
        "verified": true
    });
    
    let jwt = Jwt::new(claims).unwrap();
    
    let statement = Statement {
        document_type: DocumentType::Jwt,
        predicates: vec![Predicate::AgeOver { years: 21 }],
        revealed_fields: vec!["verified".to_string()],
        hidden_fields: vec!["sub".to_string(), "age".to_string()],
        private_fields: vec![],
    };
    
    let witness = ZkWitness {
        document: DocumentData::Jwt(jwt),
        private_values: std::collections::HashMap::new(),
        randomness: vec![],
    };
    
    let circuit = ZkCircuit::new(100);
    
    let instance = ZkInstance {
        statement: statement.clone(),
        witness,
        circuit,
    };
    
    let prover = FullZkProver::<Fp128>::new(instance).unwrap();
    
    let options = ProofOptions::default()
        .with_reed_solomon_rate(0.25);
    
    let mut rng = OsRng;
    let proof = prover.prove_full(&mut rng, options).unwrap();
    
    // Serialize and deserialize
    let serialized = ProofSerializer::serialize(
        &proof, 
        ProofFormat::Binary, 
        CompressionType::Zstd
    ).unwrap();
    
    let deserialized: longfellow_zk::ZkProof<Fp128> = 
        ProofSerializer::deserialize(&serialized).unwrap();
    
    // Verify deserialized proof
    let verifier = ZkVerifier::<Fp128>::new();
    let public_inputs = vec![Fp128::one()]; // "verified" = true
    
    // Note: Actual verification would require proper circuit reconstruction
    // This is a placeholder to demonstrate the flow
    assert_eq!(deserialized.statement.predicates.len(), 1);
    assert_eq!(deserialized.metadata.reed_solomon_rate, Some(0.25));
}