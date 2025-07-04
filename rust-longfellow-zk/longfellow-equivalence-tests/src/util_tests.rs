/// Equivalence tests for util module

use longfellow_util::{
    crypto::{sha256, sha3_256, hmac_sha256, kdf_sha256, ct_equal, base64_encode, base64_decode, hex_encode, hex_decode},
    serialization::{to_bytes, from_bytes, to_json, from_json, BinaryReader, BinaryWriter},
    timing::{Timer, time_operation, time_with_stats},
};
use serde::{Serialize, Deserialize};
use std::time::Instant;

#[test]
fn test_crypto_hashing() {
    println!("\n=== Crypto Hashing Test ===");
    
    // Test SHA-256
    let test_vectors = vec![
        (b"", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        (b"abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        (b"The quick brown fox jumps over the lazy dog", 
         "d7a8fbb307d7809469ca9abcb0082e4f8d5651e46d3cdb762d02d0bf37c9e592"),
    ];
    
    for (input, expected_hex) in test_vectors {
        let hash = sha256(input);
        let actual_hex = hex_encode(&hash);
        assert_eq!(actual_hex, expected_hex, "SHA-256 mismatch for {:?}", input);
    }
    
    // Test SHA3-256
    let sha3_hash = sha3_256(b"hello world");
    assert_eq!(sha3_hash.len(), 32, "SHA3-256 should be 32 bytes");
    
    // Test different inputs produce different hashes
    let hash1 = sha256(b"input1");
    let hash2 = sha256(b"input2");
    assert_ne!(hash1, hash2, "Different inputs should produce different hashes");
    
    println!("  ✓ Hashing functions verified");
}

#[test]
fn test_hmac_and_kdf() {
    println!("\n=== HMAC and KDF Test ===");
    
    // Test HMAC-SHA256
    let key = b"secret_key";
    let message = b"message to authenticate";
    let mac = hmac_sha256(key, message);
    
    // Verify MAC is deterministic
    let mac2 = hmac_sha256(key, message);
    assert_eq!(mac, mac2, "HMAC should be deterministic");
    
    // Different key produces different MAC
    let mac3 = hmac_sha256(b"different_key", message);
    assert_ne!(mac, mac3, "Different keys should produce different MACs");
    
    // Test KDF
    let secret = b"master_secret";
    let salt = b"salt123";
    let info = b"application_specific";
    
    let key1 = kdf_sha256(secret, salt, info, 32);
    assert_eq!(key1.len(), 32, "KDF should produce requested length");
    
    let key2 = kdf_sha256(secret, salt, info, 64);
    assert_eq!(key2.len(), 64, "KDF should produce requested length");
    assert_eq!(&key1[..], &key2[..32], "KDF should be consistent");
    
    println!("  ✓ HMAC and KDF verified");
}

#[test]
fn test_encoding_functions() {
    println!("\n=== Encoding Functions Test ===");
    
    let test_data = b"Hello, World! 123 @#$";
    
    // Test Base64
    let b64 = base64_encode(test_data);
    let decoded = base64_decode(&b64).unwrap();
    assert_eq!(decoded, test_data, "Base64 roundtrip failed");
    
    // Test Hex
    let hex = hex_encode(test_data);
    let decoded = hex_decode(&hex).unwrap();
    assert_eq!(decoded, test_data, "Hex roundtrip failed");
    
    // Test invalid decoding
    assert!(base64_decode("invalid!@#$").is_err(), "Invalid base64 should fail");
    assert!(hex_decode("invalid_hex").is_err(), "Invalid hex should fail");
    
    println!("  ✓ Encoding functions verified");
}

#[test]
fn test_constant_time_comparison() {
    println!("\n=== Constant Time Comparison Test ===");
    
    // Test equal arrays
    let a = b"secret_password";
    let b = b"secret_password";
    assert!(ct_equal(a, b), "Equal arrays should compare equal");
    
    // Test different arrays
    let c = b"different_pass";
    assert!(!ct_equal(a, c), "Different arrays should not compare equal");
    
    // Test different lengths
    let d = b"short";
    assert!(!ct_equal(a, d), "Different length arrays should not compare equal");
    
    // Verify it's actually comparing all bytes
    let e = b"secret_passwore"; // One char different at end
    assert!(!ct_equal(a, e), "Should detect difference at any position");
    
    println!("  ✓ Constant time comparison verified");
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct TestStruct {
    id: u64,
    name: String,
    data: Vec<u8>,
    nested: NestedStruct,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct NestedStruct {
    value: f64,
    flag: bool,
}

#[test]
fn test_serialization() {
    println!("\n=== Serialization Test ===");
    
    let test_obj = TestStruct {
        id: 12345,
        name: "Test Object".to_string(),
        data: vec![1, 2, 3, 4, 5],
        nested: NestedStruct {
            value: 3.14159,
            flag: true,
        },
    };
    
    // Test binary serialization
    let bytes = to_bytes(&test_obj).unwrap();
    let decoded: TestStruct = from_bytes(&bytes).unwrap();
    assert_eq!(decoded, test_obj, "Binary serialization roundtrip failed");
    
    // Test JSON serialization
    let json = to_json(&test_obj).unwrap();
    let decoded: TestStruct = from_json(&json).unwrap();
    assert_eq!(decoded, test_obj, "JSON serialization roundtrip failed");
    
    // Verify JSON is human-readable
    assert!(json.contains("Test Object"), "JSON should be readable");
    assert!(json.contains("3.14159"), "JSON should contain float value");
    
    println!("  ✓ Serialization verified");
}

#[test]
fn test_binary_reader_writer() {
    println!("\n=== Binary Reader/Writer Test ===");
    
    let mut writer = BinaryWriter::new();
    
    // Write various types
    writer.write_u8(0x42);
    writer.write_u16_le(0x1234);
    writer.write_u32_le(0x567890AB);
    writer.write_bytes(b"hello");
    
    let data = writer.into_bytes();
    
    // Read back
    let mut reader = BinaryReader::new(&data);
    
    assert_eq!(reader.read_u8().unwrap(), 0x42);
    assert_eq!(reader.read_u16_le().unwrap(), 0x1234);
    assert_eq!(reader.read_u32_le().unwrap(), 0x567890AB);
    assert_eq!(reader.read_bytes(5).unwrap(), b"hello");
    assert_eq!(reader.remaining(), 0);
    
    // Test reading past end
    assert!(reader.read_u8().is_err(), "Reading past end should fail");
    
    println!("  ✓ Binary reader/writer verified");
}

#[test]
fn test_timing_utilities() {
    println!("\n=== Timing Utilities Test ===");
    
    // Test Timer
    let mut timer = Timer::new();
    std::thread::sleep(std::time::Duration::from_millis(10));
    timer.checkpoint("after_sleep");
    
    assert!(timer.elapsed().as_millis() >= 10, "Timer should measure elapsed time");
    assert_eq!(timer.checkpoints().len(), 1, "Should have one checkpoint");
    
    // Test time_operation
    let (result, duration) = time_operation("test_op", || {
        std::thread::sleep(std::time::Duration::from_millis(5));
        42
    });
    
    assert_eq!(result, 42, "Should return function result");
    assert!(duration.as_millis() >= 5, "Should measure operation time");
    
    println!("  ✓ Timing utilities verified");
}

#[cfg(test)]
mod benchmarks {
    use super::*;
    
    #[test]
    fn bench_hashing_functions() {
        println!("\n=== Hashing Function Benchmarks ===");
        
        let sizes = vec![64, 256, 1024, 4096, 16384];
        
        for size in sizes {
            let data = vec![0x42u8; size];
            
            // Benchmark SHA-256
            let start = Instant::now();
            let iterations = 1000;
            
            for _ in 0..iterations {
                let _ = sha256(&data);
            }
            
            let sha256_duration = start.elapsed();
            
            // Benchmark SHA3-256
            let start = Instant::now();
            
            for _ in 0..iterations {
                let _ = sha3_256(&data);
            }
            
            let sha3_duration = start.elapsed();
            
            let throughput_sha256 = (iterations * size) as f64 / sha256_duration.as_secs_f64() / 1_000_000.0;
            let throughput_sha3 = (iterations * size) as f64 / sha3_duration.as_secs_f64() / 1_000_000.0;
            
            println!("  {} bytes:", size);
            println!("    SHA-256: {:?} ({:.2} MB/s)", sha256_duration, throughput_sha256);
            println!("    SHA3-256: {:?} ({:.2} MB/s)", sha3_duration, throughput_sha3);
        }
    }
    
    #[test]
    fn bench_encoding_functions() {
        println!("\n=== Encoding Function Benchmarks ===");
        
        let sizes = vec![64, 256, 1024, 4096];
        
        for size in sizes {
            let data = vec![0x42u8; size];
            let iterations = 1000;
            
            // Benchmark Base64 encode
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = base64_encode(&data);
            }
            let b64_encode_duration = start.elapsed();
            
            // Benchmark Hex encode
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = hex_encode(&data);
            }
            let hex_encode_duration = start.elapsed();
            
            // Prepare encoded data for decode benchmarks
            let b64_encoded = base64_encode(&data);
            let hex_encoded = hex_encode(&data);
            
            // Benchmark Base64 decode
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = base64_decode(&b64_encoded).unwrap();
            }
            let b64_decode_duration = start.elapsed();
            
            // Benchmark Hex decode
            let start = Instant::now();
            for _ in 0..iterations {
                let _ = hex_decode(&hex_encoded).unwrap();
            }
            let hex_decode_duration = start.elapsed();
            
            println!("  {} bytes:", size);
            println!("    Base64 encode: {:?} ({:.2} MB/s)", 
                b64_encode_duration,
                (iterations * size) as f64 / b64_encode_duration.as_secs_f64() / 1_000_000.0
            );
            println!("    Base64 decode: {:?} ({:.2} MB/s)", 
                b64_decode_duration,
                (iterations * size) as f64 / b64_decode_duration.as_secs_f64() / 1_000_000.0
            );
            println!("    Hex encode: {:?} ({:.2} MB/s)", 
                hex_encode_duration,
                (iterations * size) as f64 / hex_encode_duration.as_secs_f64() / 1_000_000.0
            );
            println!("    Hex decode: {:?} ({:.2} MB/s)", 
                hex_decode_duration,
                (iterations * size) as f64 / hex_decode_duration.as_secs_f64() / 1_000_000.0
            );
        }
    }
    
    #[test]
    fn bench_serialization() {
        println!("\n=== Serialization Benchmarks ===");
        
        let test_objects: Vec<TestStruct> = (0..100)
            .map(|i| TestStruct {
                id: i as u64,
                name: format!("Object {}", i),
                data: vec![i as u8; 100],
                nested: NestedStruct {
                    value: i as f64 * 3.14,
                    flag: i % 2 == 0,
                },
            })
            .collect();
        
        // Benchmark binary serialization
        let start = Instant::now();
        let iterations = 100;
        
        for _ in 0..iterations {
            for obj in &test_objects {
                let _ = to_bytes(obj).unwrap();
            }
        }
        
        let binary_ser_duration = start.elapsed();
        
        // Benchmark JSON serialization
        let start = Instant::now();
        
        for _ in 0..iterations {
            for obj in &test_objects {
                let _ = to_json(obj).unwrap();
            }
        }
        
        let json_ser_duration = start.elapsed();
        
        // Prepare serialized data
        let binary_data: Vec<Vec<u8>> = test_objects.iter()
            .map(|obj| to_bytes(obj).unwrap())
            .collect();
        
        let json_data: Vec<String> = test_objects.iter()
            .map(|obj| to_json(obj).unwrap())
            .collect();
        
        // Benchmark binary deserialization
        let start = Instant::now();
        
        for _ in 0..iterations {
            for data in &binary_data {
                let _: TestStruct = from_bytes(data).unwrap();
            }
        }
        
        let binary_de_duration = start.elapsed();
        
        // Benchmark JSON deserialization
        let start = Instant::now();
        
        for _ in 0..iterations {
            for data in &json_data {
                let _: TestStruct = from_json(data).unwrap();
            }
        }
        
        let json_de_duration = start.elapsed();
        
        let total_ops = iterations * test_objects.len();
        
        println!("  Binary serialization: {:?} ({:.2} μs/op)", 
            binary_ser_duration,
            binary_ser_duration.as_micros() as f64 / total_ops as f64
        );
        println!("  Binary deserialization: {:?} ({:.2} μs/op)", 
            binary_de_duration,
            binary_de_duration.as_micros() as f64 / total_ops as f64
        );
        println!("  JSON serialization: {:?} ({:.2} μs/op)", 
            json_ser_duration,
            json_ser_duration.as_micros() as f64 / total_ops as f64
        );
        println!("  JSON deserialization: {:?} ({:.2} μs/op)", 
            json_de_duration,
            json_de_duration.as_micros() as f64 / total_ops as f64
        );
    }
}