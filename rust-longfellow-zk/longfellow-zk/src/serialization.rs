/// Proof Serialization and Deserialization
/// 
/// Provides efficient serialization formats for proofs with support for
/// binary, JSON, and compressed formats

use crate::{ZkProof, ProofMetadata};
use longfellow_algebra::traits::Field;
use longfellow_core::{Result, LongfellowError};
use serde::{Serialize, Deserialize};
use std::io::{Read, Write};

/// Proof format types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum ProofFormat {
    /// Binary format using bincode
    Binary = 0,
    /// JSON format
    Json = 1,
    /// MessagePack format
    MessagePack = 2,
    /// Protocol Buffers (reserved)
    Protobuf = 3,
}

/// Compression types
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum CompressionType {
    /// No compression
    None = 0,
    /// Zlib compression
    Zlib = 1,
    /// Zstandard compression
    Zstd = 2,
    /// LZ4 compression
    Lz4 = 3,
}

/// Proof container with format metadata
#[derive(Debug)]
pub struct ProofContainer<F: Field> {
    /// Magic number: 0x4C4F4E47 ("LONG")
    pub magic: u32,
    /// Version number (major.minor as u16)
    pub version: u16,
    /// Proof format
    pub format: ProofFormat,
    /// Compression type
    pub compression: CompressionType,
    /// Field element size in bytes
    pub field_size: u16,
    /// Reserved for future use
    pub reserved: [u8; 6],
    /// The actual proof
    pub proof: ZkProof<F>,
}

impl<F: Field> ProofContainer<F> {
    /// Current format version
    pub const CURRENT_VERSION: u16 = 0x0200; // 2.0
    
    /// Magic number
    pub const MAGIC: u32 = 0x4C4F4E47; // "LONG"
    
    /// Create a new proof container
    pub fn new(proof: ZkProof<F>) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::CURRENT_VERSION,
            format: ProofFormat::Binary,
            compression: CompressionType::None,
            field_size: std::mem::size_of::<F>() as u16,
            reserved: [0; 6],
            proof,
        }
    }
    
    /// Set format
    pub fn with_format(mut self, format: ProofFormat) -> Self {
        self.format = format;
        self
    }
    
    /// Set compression
    pub fn with_compression(mut self, compression: CompressionType) -> Self {
        self.compression = compression;
        self
    }
}

/// Proof serializer
pub struct ProofSerializer;

impl ProofSerializer {
    /// Serialize proof to bytes
    pub fn serialize<F: Field + Serialize>(
        proof: &ZkProof<F>,
        format: ProofFormat,
        compression: CompressionType,
    ) -> Result<Vec<u8>> {
        // Create container
        let container = ProofContainer::new(proof.clone())
            .with_format(format)
            .with_compression(compression);
        
        // Serialize header
        let mut output = Vec::new();
        output.extend_from_slice(&container.magic.to_le_bytes());
        output.extend_from_slice(&container.version.to_le_bytes());
        output.push(container.format as u8);
        output.push(container.compression as u8);
        output.extend_from_slice(&container.field_size.to_le_bytes());
        output.extend_from_slice(&container.reserved);
        
        // Serialize proof data
        let proof_data = match format {
            ProofFormat::Binary => bincode::serialize(proof)
                .map_err(|e| LongfellowError::SerializationError(e.to_string()))?,
            ProofFormat::Json => serde_json::to_vec(proof)
                .map_err(|e| LongfellowError::SerializationError(e.to_string()))?,
            ProofFormat::MessagePack => rmp_serde::to_vec(proof)
                .map_err(|e| LongfellowError::SerializationError(e.to_string()))?,
            ProofFormat::Protobuf => {
                return Err(LongfellowError::NotImplemented(
                    "Protobuf serialization not yet implemented".to_string()
                ));
            }
        };
        
        // Apply compression
        let compressed_data = match compression {
            CompressionType::None => proof_data,
            CompressionType::Zlib => {
                use flate2::write::ZlibEncoder;
                use flate2::Compression;
                
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&proof_data)
                    .map_err(|e| LongfellowError::SerializationError(e.to_string()))?;
                encoder.finish()
                    .map_err(|e| LongfellowError::SerializationError(e.to_string()))?
            }
            CompressionType::Zstd => {
                zstd::encode_all(proof_data.as_slice(), 3)
                    .map_err(|e| LongfellowError::SerializationError(e.to_string()))?
            }
            CompressionType::Lz4 => {
                lz4::block::compress(&proof_data, None, true)
                    .map_err(|e| LongfellowError::SerializationError(e.to_string()))?
            }
        };
        
        output.extend_from_slice(&compressed_data);
        Ok(output)
    }
    
    /// Deserialize proof from bytes
    pub fn deserialize<F: Field + for<'de> Deserialize<'de>>(
        data: &[u8],
    ) -> Result<ZkProof<F>> {
        if data.len() < 20 {
            return Err(LongfellowError::DeserializationError(
                "Data too short for proof header".to_string()
            ));
        }
        
        // Parse header
        let magic = u32::from_le_bytes(data[0..4].try_into().unwrap());
        if magic != ProofContainer::<F>::MAGIC {
            return Err(LongfellowError::DeserializationError(
                format!("Invalid magic number: 0x{:08X}", magic)
            ));
        }
        
        let version = u16::from_le_bytes(data[4..6].try_into().unwrap());
        let format = data[6];
        let compression = data[7];
        let _field_size = u16::from_le_bytes(data[8..10].try_into().unwrap());
        // Skip reserved bytes
        
        let proof_data = &data[20..];
        
        // Decompress if needed
        let decompressed_data = match compression {
            0 => proof_data.to_vec(),
            1 => {
                // Zlib
                use flate2::read::ZlibDecoder;
                let mut decoder = ZlibDecoder::new(proof_data);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)
                    .map_err(|e| LongfellowError::DeserializationError(e.to_string()))?;
                decompressed
            }
            2 => {
                // Zstd
                zstd::decode_all(proof_data)
                    .map_err(|e| LongfellowError::DeserializationError(e.to_string()))?
            }
            3 => {
                // LZ4
                lz4::block::decompress(proof_data, None)
                    .map_err(|e| LongfellowError::DeserializationError(e.to_string()))?
            }
            _ => {
                return Err(LongfellowError::DeserializationError(
                    format!("Unknown compression type: {}", compression)
                ));
            }
        };
        
        // Deserialize proof
        let proof = match format {
            0 => bincode::deserialize(&decompressed_data)
                .map_err(|e| LongfellowError::DeserializationError(e.to_string()))?,
            1 => serde_json::from_slice(&decompressed_data)
                .map_err(|e| LongfellowError::DeserializationError(e.to_string()))?,
            2 => rmp_serde::from_slice(&decompressed_data)
                .map_err(|e| LongfellowError::DeserializationError(e.to_string()))?,
            _ => {
                return Err(LongfellowError::DeserializationError(
                    format!("Unknown format type: {}", format)
                ));
            }
        };
        
        Ok(proof)
    }
}

/// Human-readable proof export format
#[derive(Debug, Serialize, Deserialize)]
pub struct ProofExport<F: Field> {
    /// Format version
    pub version: String,
    /// Export timestamp
    pub timestamp: u64,
    /// Proof metadata
    pub metadata: ProofMetadata,
    /// Statement (as JSON)
    pub statement: serde_json::Value,
    /// Ligero proof (base64 encoded)
    pub ligero_proof: String,
    /// Sumcheck proof (base64 encoded, optional)
    pub sumcheck_proof: Option<String>,
    /// Commitments (hex encoded)
    pub commitments: Vec<String>,
    /// Field information
    pub field_info: FieldInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldInfo {
    pub name: String,
    pub modulus: String,
    pub bits: u32,
}

impl<F: Field + Serialize> ProofExport<F> {
    /// Create export from proof
    pub fn from_proof(proof: &ZkProof<F>) -> Result<Self> {
        use base64::Engine;
        
        let ligero_bytes = bincode::serialize(&proof.ligero_proof)
            .map_err(|e| LongfellowError::SerializationError(e.to_string()))?;
        
        let sumcheck_bytes = proof.sumcheck_proof.as_ref()
            .map(|sp| bincode::serialize(sp))
            .transpose()
            .map_err(|e| LongfellowError::SerializationError(e.to_string()))?;
        
        Ok(Self {
            version: "2.0.0".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata: proof.metadata.clone(),
            statement: serde_json::to_value(&proof.statement)
                .map_err(|e| LongfellowError::SerializationError(e.to_string()))?,
            ligero_proof: base64::engine::general_purpose::STANDARD.encode(&ligero_bytes),
            sumcheck_proof: sumcheck_bytes.map(|b| 
                base64::engine::general_purpose::STANDARD.encode(&b)
            ),
            commitments: proof.commitments.iter()
                .map(|c| hex::encode(c))
                .collect(),
            field_info: FieldInfo {
                name: std::any::type_name::<F>().to_string(),
                modulus: F::MODULUS.to_string(),
                bits: F::MODULUS_BITS,
            },
        })
    }
}

/// Streaming proof writer for large proofs
pub struct ProofWriter<W: Write> {
    writer: W,
    format: ProofFormat,
    compression: CompressionType,
}

impl<W: Write> ProofWriter<W> {
    pub fn new(writer: W, format: ProofFormat, compression: CompressionType) -> Self {
        Self {
            writer,
            format,
            compression,
        }
    }
    
    pub fn write_proof<F: Field + Serialize>(&mut self, proof: &ZkProof<F>) -> Result<()> {
        let data = ProofSerializer::serialize(proof, self.format, self.compression)?;
        self.writer.write_all(&data)
            .map_err(|e| LongfellowError::IoError(e.to_string()))?;
        Ok(())
    }
}

/// Streaming proof reader for large proofs
pub struct ProofReader<R: Read> {
    reader: R,
}

impl<R: Read> ProofReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }
    
    pub fn read_proof<F: Field + for<'de> Deserialize<'de>>(&mut self) -> Result<ZkProof<F>> {
        let mut data = Vec::new();
        self.reader.read_to_end(&mut data)
            .map_err(|e| LongfellowError::IoError(e.to_string()))?;
        ProofSerializer::deserialize(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Statement, DocumentType};
    use longfellow_algebra::Fp128;
    
    fn create_test_proof() -> ZkProof<Fp128> {
        use longfellow_ligero::LigeroProof;
        
        // Create minimal test proof
        ZkProof {
            statement: Statement {
                document_type: DocumentType::Raw,
                predicates: vec![],
                revealed_fields: vec![],
                hidden_fields: vec![],
                private_fields: vec![],
            },
            ligero_proof: LigeroProof::default(), // Would need proper construction
            sumcheck_proof: None,
            commitments: vec![[1u8; 32]],
            metadata: ProofMetadata {
                version: "2.0.0".to_string(),
                created_at: 0,
                security_bits: 128,
                document_type: DocumentType::Raw,
                circuit_stats: crate::CircuitStats {
                    num_gates: 10,
                    num_wires: 20,
                    num_constraints: 15,
                    depth: 5,
                },
                proof_generation_time_ms: Some(1000),
                reed_solomon_rate: Some(0.25),
                encoding_type: Some("convolution".to_string()),
            },
        }
    }
    
    #[test]
    fn test_binary_serialization() {
        let proof = create_test_proof();
        
        // Serialize
        let data = ProofSerializer::serialize(&proof, ProofFormat::Binary, CompressionType::None)
            .unwrap();
        
        // Check header
        assert_eq!(&data[0..4], &0x4C4F4E47u32.to_le_bytes());
        
        // Deserialize
        let deserialized: ZkProof<Fp128> = ProofSerializer::deserialize(&data).unwrap();
        assert_eq!(deserialized.metadata.version, proof.metadata.version);
    }
    
    #[test]
    fn test_compressed_serialization() {
        let proof = create_test_proof();
        
        // Test different compression types
        for compression in [CompressionType::Zlib, CompressionType::Zstd, CompressionType::Lz4] {
            let data = ProofSerializer::serialize(&proof, ProofFormat::Binary, compression)
                .unwrap();
            
            let deserialized: ZkProof<Fp128> = ProofSerializer::deserialize(&data).unwrap();
            assert_eq!(deserialized.metadata.version, proof.metadata.version);
        }
    }
}