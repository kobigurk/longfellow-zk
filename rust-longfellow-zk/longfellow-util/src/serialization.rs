/// Serialization utilities

use serde::{Serialize, Deserialize};
use longfellow_core::{LongfellowError, Result};

/// Serialize to bytes using bincode
pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    bincode::serialize(value)
        .map_err(|e| LongfellowError::SerializationError(format!("Bincode error: {}", e)))
}

/// Deserialize from bytes using bincode
pub fn from_bytes<T: for<'de> Deserialize<'de>>(bytes: &[u8]) -> Result<T> {
    bincode::deserialize(bytes)
        .map_err(|e| LongfellowError::SerializationError(format!("Bincode error: {}", e)))
}

/// Serialize to JSON string
pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value)
        .map_err(|e| LongfellowError::SerializationError(format!("JSON error: {}", e)))
}

/// Serialize to pretty JSON string
pub fn to_json_pretty<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string_pretty(value)
        .map_err(|e| LongfellowError::SerializationError(format!("JSON error: {}", e)))
}

/// Deserialize from JSON string
pub fn from_json<T: for<'de> Deserialize<'de>>(s: &str) -> Result<T> {
    serde_json::from_str(s)
        .map_err(|e| LongfellowError::SerializationError(format!("JSON error: {}", e)))
}

/// Write to file
pub fn write_to_file<T: Serialize>(path: impl AsRef<std::path::Path>, value: &T) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let bytes = to_bytes(value)?;
    let mut file = File::create(path)
        .map_err(|e| LongfellowError::IoError(e))?;
    file.write_all(&bytes)
        .map_err(|e| LongfellowError::IoError(e))?;
    
    Ok(())
}

/// Read from file
pub fn read_from_file<T: for<'de> Deserialize<'de>>(path: impl AsRef<std::path::Path>) -> Result<T> {
    use std::fs;
    
    let bytes = fs::read(path)
        .map_err(|e| LongfellowError::IoError(e))?;
    from_bytes(&bytes)
}

/// Write JSON to file
pub fn write_json_to_file<T: Serialize>(
    path: impl AsRef<std::path::Path>,
    value: &T,
    pretty: bool,
) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let json = if pretty {
        to_json_pretty(value)?
    } else {
        to_json(value)?
    };
    
    let mut file = File::create(path)
        .map_err(|e| LongfellowError::IoError(e))?;
    file.write_all(json.as_bytes())
        .map_err(|e| LongfellowError::IoError(e))?;
    
    Ok(())
}

/// Read JSON from file
pub fn read_json_from_file<T: for<'de> Deserialize<'de>>(
    path: impl AsRef<std::path::Path>,
) -> Result<T> {
    use std::fs;
    
    let json = fs::read_to_string(path)
        .map_err(|e| LongfellowError::IoError(e))?;
    from_json(&json)
}

/// Compress data using zlib
pub fn compress(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::Compression;
    use flate2::write::ZlibEncoder;
    use std::io::Write;
    
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)
        .map_err(|e| LongfellowError::CompressionError(e.to_string()))?;
    encoder.finish()
        .map_err(|e| LongfellowError::CompressionError(e.to_string()))
}

/// Decompress data using zlib
pub fn decompress(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    use std::io::Read;
    
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .map_err(|e| LongfellowError::CompressionError(e.to_string()))?;
    
    Ok(decompressed)
}

/// Binary reader helper
pub struct BinaryReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BinaryReader<'a> {
    /// Create a new binary reader
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }
    
    /// Read u8
    pub fn read_u8(&mut self) -> Result<u8> {
        if self.pos >= self.data.len() {
            return Err(LongfellowError::ParseError("Unexpected end of data".to_string()));
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }
    
    /// Read u16 (little endian)
    pub fn read_u16_le(&mut self) -> Result<u16> {
        if self.pos + 2 > self.data.len() {
            return Err(LongfellowError::ParseError("Unexpected end of data".to_string()));
        }
        let value = u16::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
        ]);
        self.pos += 2;
        Ok(value)
    }
    
    /// Read u32 (little endian)
    pub fn read_u32_le(&mut self) -> Result<u32> {
        if self.pos + 4 > self.data.len() {
            return Err(LongfellowError::ParseError("Unexpected end of data".to_string()));
        }
        let value = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(value)
    }
    
    /// Read bytes
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.pos + len > self.data.len() {
            return Err(LongfellowError::ParseError("Unexpected end of data".to_string()));
        }
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }
    
    /// Remaining bytes
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }
}

/// Binary writer helper
pub struct BinaryWriter {
    data: Vec<u8>,
}

impl BinaryWriter {
    /// Create a new binary writer
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    /// Create with capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }
    
    /// Write u8
    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value);
    }
    
    /// Write u16 (little endian)
    pub fn write_u16_le(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }
    
    /// Write u32 (little endian)
    pub fn write_u32_le(&mut self, value: u32) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }
    
    /// Write bytes
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }
    
    /// Get the written data
    pub fn into_bytes(self) -> Vec<u8> {
        self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct TestStruct {
        a: u32,
        b: String,
        c: Vec<u8>,
    }
    
    #[test]
    fn test_bincode_serialization() {
        let test = TestStruct {
            a: 42,
            b: "hello".to_string(),
            c: vec![1, 2, 3],
        };
        
        let bytes = to_bytes(&test).unwrap();
        let decoded: TestStruct = from_bytes(&bytes).unwrap();
        
        assert_eq!(test, decoded);
    }
    
    #[test]
    fn test_json_serialization() {
        let test = TestStruct {
            a: 42,
            b: "hello".to_string(),
            c: vec![1, 2, 3],
        };
        
        let json = to_json(&test).unwrap();
        let decoded: TestStruct = from_json(&json).unwrap();
        
        assert_eq!(test, decoded);
    }
    
    #[test]
    fn test_binary_reader_writer() {
        let mut writer = BinaryWriter::new();
        writer.write_u8(0x42);
        writer.write_u16_le(0x1234);
        writer.write_u32_le(0x567890AB);
        writer.write_bytes(b"hello");
        
        let data = writer.into_bytes();
        
        let mut reader = BinaryReader::new(&data);
        assert_eq!(reader.read_u8().unwrap(), 0x42);
        assert_eq!(reader.read_u16_le().unwrap(), 0x1234);
        assert_eq!(reader.read_u32_le().unwrap(), 0x567890AB);
        assert_eq!(reader.read_bytes(5).unwrap(), b"hello");
        assert_eq!(reader.remaining(), 0);
    }
}