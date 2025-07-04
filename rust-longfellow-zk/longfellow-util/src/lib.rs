/// Utility functions and helpers for longfellow-zk

pub mod crypto;
pub mod logging;
pub mod serialization;
pub mod timing;

// Re-export commonly used items
pub use crypto::{sha256, sha3_256, verify_sha256};
pub use logging::{init_logger, LogLevel};
pub use serialization::{to_bytes, from_bytes, to_json, from_json};
pub use timing::{Timer, time_operation};

/// Initialize all utilities with default settings
pub fn init() {
    init_logger(LogLevel::Info);
}