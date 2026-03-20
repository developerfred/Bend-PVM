//! Logging Framework Module
//! Issue #24 - Event System and Logging Framework

mod compressor;
mod logger;
mod storage;

pub use compressor::LogCompressor;
pub use logger::{LogEntry, LogLevel, Logger};
pub use storage::{InMemoryLogStorage, RotationPolicy};

/// Initialize the global logger
pub fn init_logger(level: LogLevel) -> Logger {
    Logger::new(level)
}
