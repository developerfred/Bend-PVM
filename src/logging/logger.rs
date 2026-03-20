//! Logger Implementation
//! Issue #24 - Event System and Logging Framework

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Log severity levels ordered by importance
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "Trace"),
            LogLevel::Debug => write!(f, "Debug"),
            LogLevel::Info => write!(f, "Info"),
            LogLevel::Warning => write!(f, "Warning"),
            LogLevel::Error => write!(f, "Error"),
            LogLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// The severity level of the log
    pub level: LogLevel,
    /// The module that generated the log
    pub module: String,
    /// The log message
    pub message: String,
    /// Unix timestamp when the log was created
    pub timestamp: u64,
    /// Optional structured metadata
    pub metadata: Option<HashMap<String, String>>,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, module: &str, message: &str) -> Self {
        LogEntry {
            level,
            module: module.to_string(),
            message: message.to_string(),
            timestamp: current_timestamp(),
            metadata: None,
        }
    }

    /// Create a new log entry with metadata
    pub fn new_with_metadata(
        level: LogLevel,
        module: &str,
        message: &str,
        metadata: HashMap<String, String>,
    ) -> Self {
        LogEntry {
            level,
            module: module.to_string(),
            message: message.to_string(),
            timestamp: current_timestamp(),
            metadata: Some(metadata),
        }
    }

    /// Serialize the log entry to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

/// Logger with level filtering
#[derive(Debug)]
pub struct Logger {
    level: LogLevel,
}

impl Logger {
    /// Create a new logger with the specified minimum level
    pub fn new(level: LogLevel) -> Self {
        Logger { level }
    }

    /// Get the logger's minimum level
    pub fn get_level(&self) -> LogLevel {
        self.level.clone()
    }

    /// Check if a message at the given level should be logged
    pub fn should_log(&self, level: &LogLevel) -> bool {
        level >= &self.level
    }

    /// Log a message if it passes the level filter
    pub fn log(&self, entry: &LogEntry) -> bool {
        if self.should_log(&entry.level) {
            // In a real implementation, this would write to output
            true
        } else {
            false
        }
    }
}

/// Get current unix timestamp in seconds
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new(LogLevel::Info);
        assert_eq!(logger.get_level(), LogLevel::Info);
    }

    #[test]
    fn test_log_level_filtering() {
        let logger = Logger::new(LogLevel::Warning);

        assert!(!logger.should_log(&LogLevel::Trace));
        assert!(!logger.should_log(&LogLevel::Debug));
        assert!(!logger.should_log(&LogLevel::Info));
        assert!(logger.should_log(&LogLevel::Warning));
        assert!(logger.should_log(&LogLevel::Error));
        assert!(logger.should_log(&LogLevel::Critical));
    }

    #[test]
    fn test_log_entry_creation() {
        let entry = LogEntry::new(LogLevel::Error, "Module", "Error message");
        assert_eq!(entry.level, LogLevel::Error);
        assert_eq!(entry.module, "Module");
        assert_eq!(entry.message, "Error message");
        assert!(entry.metadata.is_none());
    }

    #[test]
    fn test_log_entry_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());
        let entry = LogEntry::new_with_metadata(LogLevel::Info, "api", "Request", metadata);
        assert!(entry.metadata.is_some());
        assert_eq!(
            entry.metadata.unwrap().get("key"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_log_entry_serialization() {
        let entry = LogEntry::new(LogLevel::Warning, "contract", "Gas limit");
        let json = entry.to_json().unwrap();
        assert!(json.contains("Warning"));
        assert!(json.contains("contract"));
    }

    #[test]
    fn test_log_filtering() {
        let logger = Logger::new(LogLevel::Error);
        let debug_entry = LogEntry::new(LogLevel::Debug, "test", "Debug msg");
        let error_entry = LogEntry::new(LogLevel::Error, "test", "Error msg");

        assert!(!logger.log(&debug_entry));
        assert!(logger.log(&error_entry));
    }
}
