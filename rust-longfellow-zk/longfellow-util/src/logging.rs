/// Logging utilities

use log::{Level, LevelFilter, Metadata, Record};
use env_logger::Builder;
use std::io::Write;
use std::sync::Once;

/// Log level configuration
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

static INIT: Once = Once::new();

/// Initialize the logger with the specified level
pub fn init_logger(level: LogLevel) {
    INIT.call_once(|| {
        let mut builder = Builder::new();
        
        builder
            .filter(None, level.into())
            .format(|buf, record| {
                let level_style = match record.level() {
                    Level::Error => "\x1b[31m", // Red
                    Level::Warn => "\x1b[33m",  // Yellow
                    Level::Info => "\x1b[32m",  // Green
                    Level::Debug => "\x1b[34m", // Blue
                    Level::Trace => "\x1b[35m", // Magenta
                };
                
                writeln!(
                    buf,
                    "[{} {}{:5}\x1b[0m {}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    level_style,
                    record.level(),
                    record.target(),
                    record.args()
                )
            });
        
        // Also check RUST_LOG environment variable
        if let Ok(rust_log) = std::env::var("RUST_LOG") {
            builder.parse_filters(&rust_log);
        }
        
        builder.init();
    });
}

/// Initialize logger from environment
pub fn init_from_env() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

/// Log with custom metadata
pub fn log_with_metadata(
    level: Level,
    target: &str,
    message: &str,
    metadata: &[("tag", &str)],
) {
    let mut msg = message.to_string();
    
    if !metadata.is_empty() {
        msg.push_str(" [");
        for (i, (key, value)) in metadata.iter().enumerate() {
            if i > 0 {
                msg.push_str(", ");
            }
            msg.push_str(&format!("{}: {}", key, value));
        }
        msg.push(']');
    }
    
    log::log!(target: target, level, "{}", msg);
}

/// Performance logging helper
pub struct PerfLogger {
    name: String,
    start: std::time::Instant,
}

impl PerfLogger {
    /// Create a new performance logger
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        log::debug!("Starting: {}", name);
        Self {
            name,
            start: std::time::Instant::now(),
        }
    }
    
    /// Log a checkpoint
    pub fn checkpoint(&self, message: &str) {
        let elapsed = self.start.elapsed();
        log::debug!(
            "{} - {} [{:.3}ms]",
            self.name,
            message,
            elapsed.as_secs_f64() * 1000.0
        );
    }
}

impl Drop for PerfLogger {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        log::debug!(
            "Completed: {} [{:.3}ms]",
            self.name,
            elapsed.as_secs_f64() * 1000.0
        );
    }
}

/// Structured logging
#[derive(Debug)]
pub struct StructuredLogger {
    context: Vec<(String, String)>,
}

impl StructuredLogger {
    /// Create a new structured logger
    pub fn new() -> Self {
        Self {
            context: Vec::new(),
        }
    }
    
    /// Add context
    pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }
    
    /// Log with context
    pub fn log(&self, level: Level, message: &str) {
        let mut full_message = message.to_string();
        
        if !self.context.is_empty() {
            full_message.push_str(" {{");
            for (i, (key, value)) in self.context.iter().enumerate() {
                if i > 0 {
                    full_message.push_str(", ");
                }
                full_message.push_str(&format!("{}: {}", key, value));
            }
            full_message.push_str("}}");
        }
        
        log::log!(level, "{}", full_message);
    }
    
    /// Convenience methods
    pub fn error(&self, message: &str) {
        self.log(Level::Error, message);
    }
    
    pub fn warn(&self, message: &str) {
        self.log(Level::Warn, message);
    }
    
    pub fn info(&self, message: &str) {
        self.log(Level::Info, message);
    }
    
    pub fn debug(&self, message: &str) {
        self.log(Level::Debug, message);
    }
    
    pub fn trace(&self, message: &str) {
        self.log(Level::Trace, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logger_init() {
        init_logger(LogLevel::Debug);
        log::debug!("Test debug message");
        log::info!("Test info message");
    }
    
    #[test]
    fn test_perf_logger() {
        init_logger(LogLevel::Debug);
        let logger = PerfLogger::new("test_operation");
        logger.checkpoint("halfway");
        // Logger will log completion on drop
    }
    
    #[test]
    fn test_structured_logger() {
        init_logger(LogLevel::Info);
        
        let logger = StructuredLogger::new()
            .with_context("user_id", "12345")
            .with_context("action", "login");
        
        logger.info("User logged in successfully");
    }
}