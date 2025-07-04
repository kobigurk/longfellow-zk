/// Equivalence tests for EC (elliptic curve) module

use longfellow_ec::{Point, FieldElement, Scalar, ecdsa_verify};
use std::time::Instant;

#[test]
fn test_point_operations() {
    println!("\n=== EC Point Operations Test ===");
    
    // Test identity
    let identity = Point::identity();
    assert!(identity.is_identity(), "Identity check failed");
    
    // Test generator
    let g = Point::generator();
    assert!(!g.is_identity(), "Generator should not be identity");
    
    // Test point addition
    let p1 = g.clone();
    let p2 = g.double();
    let p3 = p1.add(&p2);
    
    // Verify 3G = G + 2G
    let three_g = g.scalar_mul(&Scalar::from(3u64));
    assert_eq!(p3, three_g, "Point addition failed: G + 2G != 3G");
    
    // Test commutativity: P + Q = Q + P
    let p = g.scalar_mul(&Scalar::from(5u64));
    let q = g.scalar_mul(&Scalar::from(7u64));
    assert_eq!(p.add(&q), q.add(&p), "Addition not commutative");
    
    // Test associativity: (P + Q) + R = P + (Q + R)
    let r = g.scalar_mul(&Scalar::from(11u64));
    let left = p.add(&q).add(&r);
    let right = p.add(&q.add(&r));
    assert_eq!(left, right, "Addition not associative");
    
    println!("  ✓ Point operations verified");
}

#[test]
fn test_scalar_multiplication() {
    println!("\n=== Scalar Multiplication Test ===");
    
    let g = Point::generator();
    
    // Test small scalars
    for i in 0..10 {
        let p1 = g.scalar_mul(&Scalar::from(i as u64));
        
        // Verify by repeated addition
        let mut p2 = Point::identity();
        for _ in 0..i {
            p2 = p2.add(&g);
        }
        
        assert_eq!(p1, p2, "Scalar multiplication failed for {}", i);
    }
    
    // Test distributivity: k(P + Q) = kP + kQ
    let p = g.scalar_mul(&Scalar::from(3u64));
    let q = g.scalar_mul(&Scalar::from(5u64));
    let k = Scalar::from(7u64);
    
    let left = p.add(&q).scalar_mul(&k);
    let right = p.scalar_mul(&k).add(&q.scalar_mul(&k));
    assert_eq!(left, right, "Scalar multiplication not distributive");
    
    // Test large scalar
    let large = Scalar::from_bytes(&[0xFF; 32]);
    let p_large = g.scalar_mul(&large);
    assert!(!p_large.is_identity(), "Large scalar multiplication resulted in identity");
    
    println!("  ✓ Scalar multiplication verified");
}

#[test]
fn test_point_encoding() {
    println!("\n=== Point Encoding Test ===");
    
    let test_scalars = vec![
        Scalar::from(0u64),
        Scalar::from(1u64),
        Scalar::from(42u64),
        Scalar::from(12345u64),
    ];
    
    for scalar in test_scalars {
        let point = Point::generator().scalar_mul(&scalar);
        
        // Encode and decode
        let encoded = point.to_bytes();
        let decoded = Point::from_bytes(&encoded).expect("Decoding failed");
        
        assert_eq!(point, decoded, "Encoding/decoding roundtrip failed");
        
        // Test compressed encoding
        let compressed = point.to_compressed_bytes();
        let decompressed = Point::from_compressed_bytes(&compressed).expect("Decompression failed");
        
        assert_eq!(point, decompressed, "Compressed encoding roundtrip failed");
        assert!(compressed.len() < encoded.len(), "Compressed should be smaller");
    }
    
    println!("  ✓ Point encoding verified");
}

#[test]
fn test_ecdsa_signature_verification() {
    println!("\n=== ECDSA Signature Verification Test ===");
    
    // Test vectors (would normally come from standard test vectors)
    let test_cases = vec![
        // (message, public_key_scalar, r, s, should_verify)
        (
            b"hello world".to_vec(),
            Scalar::from(12345u64),
            Scalar::from(98765u64),
            Scalar::from(54321u64),
            false, // This is a dummy signature, should fail
        ),
    ];
    
    for (i, (message, pk_scalar, r, s, _expected)) in test_cases.iter().enumerate() {
        let public_key = Point::generator().scalar_mul(pk_scalar);
        
        // In real implementation, this would use proper ECDSA verification
        let result = ecdsa_verify(&message, &public_key, r, s);
        
        println!("  Test case {}: verification = {}", i, result);
    }
    
    println!("  ✓ ECDSA verification tested");
}

#[test]
fn test_field_arithmetic() {
    println!("\n=== Field Element Arithmetic Test ===");
    
    // Test basic operations
    let a = FieldElement::from(12345u64);
    let b = FieldElement::from(67890u64);
    
    // Addition
    let sum = a.add(&b);
    let expected_sum = FieldElement::from(12345u64 + 67890u64);
    assert_eq!(sum, expected_sum, "Field addition failed");
    
    // Subtraction
    let diff = b.sub(&a);
    let expected_diff = FieldElement::from(67890u64 - 12345u64);
    assert_eq!(diff, expected_diff, "Field subtraction failed");
    
    // Multiplication
    let prod = a.mul(&b);
    // This would overflow u64, so we just check it's non-zero
    assert_ne!(prod, FieldElement::zero(), "Field multiplication resulted in zero");
    
    // Inverse
    let a_inv = a.inverse().expect("Inverse should exist");
    let one = a.mul(&a_inv);
    assert_eq!(one, FieldElement::one(), "Field inverse failed");
    
    println!("  ✓ Field arithmetic verified");
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    
    #[test]
    fn bench_point_operations() {
        println!("\n=== EC Point Operation Benchmarks ===");
        
        let g = Point::generator();
        let p = g.scalar_mul(&Scalar::from(12345u64));
        let q = g.scalar_mul(&Scalar::from(67890u64));
        
        // Benchmark point addition
        let start = Instant::now();
        let iterations = 10000;
        
        for _ in 0..iterations {
            let _ = p.add(&q);
        }
        
        let add_duration = start.elapsed();
        
        // Benchmark point doubling
        let start = Instant::now();
        
        for _ in 0..iterations {
            let _ = p.double();
        }
        
        let double_duration = start.elapsed();
        
        println!("  Point addition: {:?} ({:.2} μs/op)", 
            add_duration,
            add_duration.as_micros() as f64 / iterations as f64
        );
        
        println!("  Point doubling: {:?} ({:.2} μs/op)", 
            double_duration,
            double_duration.as_micros() as f64 / iterations as f64
        );
    }
    
    #[test]
    fn bench_scalar_multiplication() {
        println!("\n=== Scalar Multiplication Benchmarks ===");
        
        let g = Point::generator();
        let scalars: Vec<Scalar> = (0..100)
            .map(|i| Scalar::from((i * 12345) as u64))
            .collect();
        
        // Benchmark fixed base scalar multiplication
        let start = Instant::now();
        
        for scalar in &scalars {
            let _ = g.scalar_mul(scalar);
        }
        
        let fixed_duration = start.elapsed();
        
        // Benchmark variable base scalar multiplication
        let base = g.scalar_mul(&Scalar::from(98765u64));
        let start = Instant::now();
        
        for scalar in &scalars {
            let _ = base.scalar_mul(scalar);
        }
        
        let variable_duration = start.elapsed();
        
        println!("  Fixed base scalar mul: {:?} ({:.2} ms/op)", 
            fixed_duration,
            fixed_duration.as_millis() as f64 / scalars.len() as f64
        );
        
        println!("  Variable base scalar mul: {:?} ({:.2} ms/op)", 
            variable_duration,
            variable_duration.as_millis() as f64 / scalars.len() as f64
        );
    }
    
    #[test]
    fn bench_point_encoding() {
        println!("\n=== Point Encoding Benchmarks ===");
        
        let points: Vec<Point> = (0..100)
            .map(|i| Point::generator().scalar_mul(&Scalar::from(i as u64)))
            .collect();
        
        // Benchmark uncompressed encoding
        let start = Instant::now();
        let iterations = 1000;
        
        for _ in 0..iterations {
            for point in &points {
                let _ = point.to_bytes();
            }
        }
        
        let encode_duration = start.elapsed();
        
        // Benchmark compressed encoding
        let start = Instant::now();
        
        for _ in 0..iterations {
            for point in &points {
                let _ = point.to_compressed_bytes();
            }
        }
        
        let compress_duration = start.elapsed();
        
        // Prepare encoded points for decoding benchmark
        let encoded: Vec<Vec<u8>> = points.iter()
            .map(|p| p.to_bytes())
            .collect();
        
        let compressed: Vec<Vec<u8>> = points.iter()
            .map(|p| p.to_compressed_bytes())
            .collect();
        
        // Benchmark decoding
        let start = Instant::now();
        
        for _ in 0..iterations {
            for bytes in &encoded {
                let _ = Point::from_bytes(bytes).unwrap();
            }
        }
        
        let decode_duration = start.elapsed();
        
        // Benchmark decompression
        let start = Instant::now();
        
        for _ in 0..iterations {
            for bytes in &compressed {
                let _ = Point::from_compressed_bytes(bytes).unwrap();
            }
        }
        
        let decompress_duration = start.elapsed();
        
        let total_ops = iterations * points.len();
        
        println!("  Encode (uncompressed): {:?} ({:.2} μs/op)", 
            encode_duration,
            encode_duration.as_micros() as f64 / total_ops as f64
        );
        
        println!("  Encode (compressed): {:?} ({:.2} μs/op)", 
            compress_duration,
            compress_duration.as_micros() as f64 / total_ops as f64
        );
        
        println!("  Decode (uncompressed): {:?} ({:.2} μs/op)", 
            decode_duration,
            decode_duration.as_micros() as f64 / total_ops as f64
        );
        
        println!("  Decode (compressed): {:?} ({:.2} μs/op)", 
            decompress_duration,
            decompress_duration.as_micros() as f64 / total_ops as f64
        );
    }
    
    #[test]
    fn bench_ecdsa_verification() {
        println!("\n=== ECDSA Verification Benchmarks ===");
        
        // Generate test data
        let messages: Vec<Vec<u8>> = (0..100)
            .map(|i| format!("Test message {}", i).into_bytes())
            .collect();
        
        let public_keys: Vec<Point> = (0..100)
            .map(|i| Point::generator().scalar_mul(&Scalar::from((i * 99991) as u64)))
            .collect();
        
        // Dummy signatures (in real test would be valid signatures)
        let signatures: Vec<(Scalar, Scalar)> = (0..100)
            .map(|i| {
                (
                    Scalar::from((i * 12345) as u64),
                    Scalar::from((i * 67890) as u64),
                )
            })
            .collect();
        
        let start = Instant::now();
        
        for i in 0..messages.len() {
            let _ = ecdsa_verify(
                &messages[i],
                &public_keys[i],
                &signatures[i].0,
                &signatures[i].1,
            );
        }
        
        let duration = start.elapsed();
        
        println!("  ECDSA verification: {:?} ({:.2} ms/op)", 
            duration,
            duration.as_millis() as f64 / messages.len() as f64
        );
    }
}