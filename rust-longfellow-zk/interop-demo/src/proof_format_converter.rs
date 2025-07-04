/// Proof format converter for Rust-to-C++ interoperability
/// 
/// This program converts Rust-generated proofs to the exact binary format
/// expected by the C++ verifier.

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;

use longfellow_util::{serialization::{BinaryWriter, to_bytes}};

#[derive(Parser, Debug)]
#[command(name = "proof_format_converter")]
#[command(about = "Convert Rust proofs to C++ binary format")]
struct Args {
    /// Input proof file (JSON)
    #[arg(short, long)]
    input: PathBuf,
    
    /// Output proof file (binary)
    #[arg(short, long)]
    output: PathBuf,
    
    /// C++ verifier configuration file
    #[arg(short, long)]
    config: Option<PathBuf>,
    
    /// Output format
    #[arg(short, long, value_enum, default_value = "cpp-binary")]
    format: OutputFormat,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum OutputFormat {
    /// C++ binary format
    CppBinary,
    /// C++ text format  
    CppText,
    /// Protocol buffers
    Protobuf,
}

/// C++ proof format structure
#[derive(Serialize, Deserialize, Debug)]
struct CppProofFormat {
    /// Magic number for format identification
    magic: u32,
    
    /// Format version
    version: u16,
    
    /// Proof type identifier
    proof_type: u8,
    
    /// Security parameter
    security_bits: u16,
    
    /// Field modulus (32 bytes)
    field_modulus: [u8; 32],
    
    /// Number of public inputs
    num_public_inputs: u32,
    
    /// Public input data
    public_inputs: Vec<CppFieldElement>,
    
    /// Proof data length
    proof_data_len: u32,
    
    /// Proof data
    proof_data: Vec<u8>,
    
    /// Verification key length
    vk_len: u32,
    
    /// Verification key data
    verification_key: Vec<u8>,
    
    /// Checksum for integrity
    checksum: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct CppFieldElement {
    /// Field element as 32-byte array (little-endian)
    data: [u8; 32],
}

impl CppFieldElement {
    fn from_hex(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str)?;
        let mut data = [0u8; 32];
        
        // Ensure we don't overflow
        let copy_len = bytes.len().min(32);
        data[..copy_len].copy_from_slice(&bytes[..copy_len]);
        
        Ok(CppFieldElement { data })
    }
    
    fn to_bytes(&self) -> Vec<u8> {
        self.data.to_vec()
    }
}

const CPP_MAGIC: u32 = 0x4C4F4E47; // "LONG" in hex
const CPP_VERSION: u16 = 0x0100;    // Version 1.0

fn main() -> Result<()> {
    let args = Args::parse();
    
    println!("🔄 Converting Rust proof to C++ format...");
    
    // Read the input proof
    let proof_json = fs::read_to_string(&args.input)
        .with_context(|| format!("Failed to read proof from {:?}", args.input))?;
    
    let rust_proof: serde_json::Value = serde_json::from_str(&proof_json)?;
    
    // Convert to C++ format
    let cpp_proof = convert_to_cpp_format(&rust_proof)?;
    
    // Write output in requested format
    match args.format {
        OutputFormat::CppBinary => write_cpp_binary(&cpp_proof, &args.output)?,
        OutputFormat::CppText => write_cpp_text(&cpp_proof, &args.output)?,
        OutputFormat::Protobuf => write_protobuf(&cpp_proof, &args.output)?,
    }
    
    println!("✅ Conversion complete!");
    println!("📄 Output: {:?}", args.output);
    print_conversion_summary(&cpp_proof);
    
    Ok(())
}

fn convert_to_cpp_format(rust_proof: &serde_json::Value) -> Result<CppProofFormat> {
    println!("🔧 Converting proof format...");
    
    // Extract basic fields
    let proof_type_str = rust_proof["proof_type"].as_str()
        .context("Missing proof_type")?;
    
    let proof_type = match proof_type_str {
        "field_arithmetic" => 1,
        "merkle_proof" => 2,
        "polynomial" => 3,
        "circuit" => 4,
        "document" => 5,
        "ligero" => 6,
        "full_zk" => 7,
        "matrix_multiplication" => 8,
        "hash_chain" => 5, // Map to document type for C++ compatibility
        "matrix" => 4, // Map to circuit type for C++ compatibility  
        _ => return Err(anyhow::anyhow!("Unknown proof type: {}", proof_type_str)),
    };
    
    let security_bits = rust_proof["security_bits"].as_u64()
        .context("Missing security_bits")? as u16;
    
    // Parse field modulus
    let field_modulus_hex = rust_proof["field_modulus"].as_str()
        .context("Missing field_modulus")?;
    let field_modulus_bytes = hex::decode(field_modulus_hex)?;
    let mut field_modulus = [0u8; 32];
    let copy_len = field_modulus_bytes.len().min(32);
    field_modulus[..copy_len].copy_from_slice(&field_modulus_bytes[..copy_len]);
    
    // Convert public inputs
    let mut public_inputs = Vec::new();
    if let Some(inputs_obj) = rust_proof["public_inputs"].as_object() {
        for (_key, value) in inputs_obj {
            if let Some(hex_str) = value.as_str() {
                // Try to parse as hex field element
                if hex_str.len() <= 64 { // Max 32 bytes as hex
                    if let Ok(field_elem) = CppFieldElement::from_hex(hex_str) {
                        public_inputs.push(field_elem);
                    }
                }
            }
        }
    }
    
    // Serialize proof data
    let proof_data = serialize_proof_data(&rust_proof["proof_data"])?;
    
    // Serialize verification key
    let verification_key = if let Some(vk) = rust_proof.get("verification_key") {
        to_bytes(vk)?
    } else {
        Vec::new()
    };
    
    // Calculate checksum
    let mut checksum_data = Vec::new();
    checksum_data.extend_from_slice(&field_modulus);
    checksum_data.extend_from_slice(&proof_data);
    checksum_data.extend_from_slice(&verification_key);
    let checksum = crc32(&checksum_data);
    
    Ok(CppProofFormat {
        magic: CPP_MAGIC,
        version: CPP_VERSION,
        proof_type,
        security_bits,
        field_modulus,
        num_public_inputs: public_inputs.len() as u32,
        public_inputs,
        proof_data_len: proof_data.len() as u32,
        proof_data,
        vk_len: verification_key.len() as u32,
        verification_key,
        checksum,
    })
}

fn serialize_proof_data(proof_data: &serde_json::Value) -> Result<Vec<u8>> {
    let mut writer = BinaryWriter::new();
    
    // Write proof type tag
    if let Some(type_obj) = proof_data.as_object() {
        if let Some(type_name) = type_obj.get("type") {
            match type_name.as_str() {
                Some("FieldArithmetic") => {
                    // Serialize field arithmetic data in the format expected by C++ verifier
                    if let Some(result) = type_obj.get("result") {
                        let result_bytes = hex::decode(result.as_str().unwrap_or(""))?;
                        // Pad to 32 bytes (field element size)
                        let mut padded_result = result_bytes;
                        padded_result.resize(32, 0);
                        writer.write_bytes(&padded_result);
                    }
                    
                    // Write number of intermediate values
                    if let Some(intermediate) = type_obj.get("intermediate_values").and_then(|i| i.as_array()) {
                        writer.write_u32_le(intermediate.len() as u32);
                    } else {
                        writer.write_u32_le(2u32); // Default: 2 intermediate values
                    }
                }
                Some("MerkleProof") => {
                    writer.write_u8(2);
                    // Serialize Merkle proof data
                    if let Some(root) = type_obj.get("root") {
                        let root_bytes = hex::decode(root.as_str().unwrap_or(""))?;
                        writer.write_bytes(&root_bytes);
                    }
                    if let Some(path) = type_obj.get("path").and_then(|p| p.as_array()) {
                        writer.write_u32_le(path.len() as u32);
                        for path_elem in path {
                            let elem_bytes = hex::decode(path_elem.as_str().unwrap_or(""))?;
                            writer.write_bytes(&elem_bytes);
                        }
                    }
                }
                Some("Circuit") => {
                    writer.write_u8(4);
                    // Serialize circuit data
                    if let Some(num_constraints) = type_obj.get("num_constraints") {
                        writer.write_u32_le(num_constraints.as_u64().unwrap_or(0) as u32);
                    }
                    if let Some(num_variables) = type_obj.get("num_variables") {
                        writer.write_u32_le(num_variables.as_u64().unwrap_or(0) as u32);
                    }
                }
                Some("Ligero") => {
                    writer.write_u8(6);
                    // Serialize Ligero proof data
                    if let Some(commitments) = type_obj.get("column_commitments").and_then(|c| c.as_array()) {
                        writer.write_u32_le(commitments.len() as u32);
                        for commitment in commitments {
                            let commit_bytes = hex::decode(commitment.as_str().unwrap_or(""))?;
                            writer.write_bytes(&commit_bytes);
                        }
                    }
                }
                Some("FullZk") => {
                    writer.write_u8(7);
                    // Serialize combined proof data
                    writer.write_u32_le(2); // Number of sub-proofs
                    
                    // Serialize Ligero sub-proof
                    if let Some(ligero_proof) = type_obj.get("ligero_proof") {
                        let ligero_data = serialize_proof_data(ligero_proof)?;
                        writer.write_u32_le(ligero_data.len() as u32);
                        writer.write_bytes(&ligero_data);
                    }
                    
                    // Serialize Sumcheck sub-proof
                    if let Some(sumcheck_proof) = type_obj.get("sumcheck_proof") {
                        let sumcheck_data = to_bytes(sumcheck_proof)?;
                        writer.write_u32_le(sumcheck_data.len() as u32);
                        writer.write_bytes(&sumcheck_data);
                    }
                }
                Some("MatrixMultiplication") => {
                    writer.write_u8(8);
                    // Serialize matrix result trace
                    if let Some(trace) = type_obj.get("result_trace").and_then(|t| t.as_array()) {
                        writer.write_u32_le(trace.len() as u32);
                        for elem in trace {
                            let elem_bytes = hex::decode(elem.as_str().unwrap_or(""))?;
                            writer.write_bytes(&elem_bytes);
                        }
                    }
                }
                Some("HashChain") => {
                    // Serialize hash chain data as document format
                    if let Some(final_hash) = type_obj.get("final_value") {
                        let hash_bytes = hex::decode(final_hash.as_str().unwrap_or(""))?;
                        let mut padded_hash = hash_bytes;
                        padded_hash.resize(32, 0);
                        writer.write_bytes(&padded_hash);
                    }
                    if let Some(iterations) = type_obj.get("iterations") {
                        writer.write_u32_le(iterations.as_u64().unwrap_or(0) as u32);
                    }
                }
                Some("Matrix") => {
                    // Serialize matrix data as circuit format
                    let empty_vec = vec![];
                    let result = type_obj.get("result").and_then(|r| r.as_array()).unwrap_or(&empty_vec);
                    writer.write_u32_le(result.len() as u32); // num_constraints
                    writer.write_u32_le((result.len() * result.len()) as u32); // num_variables
                    
                    // Flatten matrix result as witness values
                    for row in result {
                        if let Some(row_array) = row.as_array() {
                            for elem in row_array {
                                let elem_bytes = hex::decode(elem.as_str().unwrap_or(""))?;
                                let mut padded_elem = elem_bytes;
                                padded_elem.resize(32, 0);
                                writer.write_bytes(&padded_elem);
                            }
                        }
                    }
                }
                Some("Polynomial") => {
                    // Serialize polynomial data
                    if let Some(eval_point) = type_obj.get("evaluation_point") {
                        let point_bytes = hex::decode(eval_point.as_str().unwrap_or(""))?;
                        let mut padded_point = point_bytes;
                        padded_point.resize(32, 0);
                        writer.write_bytes(&padded_point);
                    }
                    if let Some(eval_result) = type_obj.get("evaluation_result") {
                        let result_bytes = hex::decode(eval_result.as_str().unwrap_or(""))?;
                        let mut padded_result = result_bytes;
                        padded_result.resize(32, 0);
                        writer.write_bytes(&padded_result);
                    }
                }
                _ => {
                    writer.write_u8(0); // Unknown type
                }
            }
        }
    }
    
    Ok(writer.into_bytes())
}

fn write_cpp_binary(proof: &CppProofFormat, output_path: &PathBuf) -> Result<()> {
    println!("📝 Writing C++ binary format...");
    
    let mut writer = BinaryWriter::new();
    
    // Write header
    writer.write_u32_le(proof.magic);
    writer.write_u16_le(proof.version);
    writer.write_u8(proof.proof_type);
    writer.write_u16_le(proof.security_bits);
    
    // Write field modulus
    writer.write_bytes(&proof.field_modulus);
    
    // Write public inputs
    writer.write_u32_le(proof.num_public_inputs);
    for input in &proof.public_inputs {
        writer.write_bytes(&input.to_bytes());
    }
    
    // Write proof data
    writer.write_u32_le(proof.proof_data_len);
    writer.write_bytes(&proof.proof_data);
    
    // Write verification key
    writer.write_u32_le(proof.vk_len);
    writer.write_bytes(&proof.verification_key);
    
    // Write checksum
    writer.write_u32_le(proof.checksum);
    
    fs::write(output_path, writer.into_bytes())?;
    Ok(())
}

fn write_cpp_text(proof: &CppProofFormat, output_path: &PathBuf) -> Result<()> {
    println!("📝 Writing C++ text format...");
    
    let mut output = String::new();
    
    // Header
    output.push_str(&format!("LONGFELLOW_PROOF_V{}\n", proof.version));
    output.push_str(&format!("TYPE={}\n", proof.proof_type));
    output.push_str(&format!("SECURITY={}\n", proof.security_bits));
    output.push_str(&format!("FIELD_MODULUS={}\n", hex::encode(&proof.field_modulus)));
    
    // Public inputs
    output.push_str(&format!("PUBLIC_INPUTS={}\n", proof.num_public_inputs));
    for (i, input) in proof.public_inputs.iter().enumerate() {
        output.push_str(&format!("INPUT_{}={}\n", i, hex::encode(&input.to_bytes())));
    }
    
    // Proof data
    output.push_str(&format!("PROOF_DATA_LEN={}\n", proof.proof_data_len));
    output.push_str(&format!("PROOF_DATA={}\n", hex::encode(&proof.proof_data)));
    
    // Verification key
    if !proof.verification_key.is_empty() {
        output.push_str(&format!("VK_LEN={}\n", proof.vk_len));
        output.push_str(&format!("VK_DATA={}\n", hex::encode(&proof.verification_key)));
    }
    
    // Checksum
    output.push_str(&format!("CHECKSUM={:08x}\n", proof.checksum));
    
    fs::write(output_path, output)?;
    Ok(())
}

fn write_protobuf(_proof: &CppProofFormat, _output_path: &PathBuf) -> Result<()> {
    // Placeholder for Protocol Buffers format
    println!("⚠️  Protocol Buffers format not yet implemented");
    Ok(())
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    
    !crc
}

fn print_conversion_summary(proof: &CppProofFormat) {
    println!("\n📋 Conversion Summary");
    println!("====================");
    println!("Magic: 0x{:08X}", proof.magic);
    println!("Version: {}.{}", proof.version >> 8, proof.version & 0xFF);
    println!("Proof Type: {}", proof.proof_type);
    println!("Security: {} bits", proof.security_bits);
    println!("Field Modulus: 0x{}...", hex::encode(&proof.field_modulus[..8]));
    println!("Public Inputs: {}", proof.num_public_inputs);
    println!("Proof Data: {} bytes", proof.proof_data_len);
    println!("Verification Key: {} bytes", proof.vk_len);
    println!("Checksum: 0x{:08X}", proof.checksum);
}