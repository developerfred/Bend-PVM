//! Event Types
//! Issue #24 - Event System and Logging Framework

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event severity levels ordered by importance
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warning = 3,
    Error = 4,
    Critical = 5,
}

impl std::fmt::Display for EventLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventLevel::Trace => write!(f, "Trace"),
            EventLevel::Debug => write!(f, "Debug"),
            EventLevel::Info => write!(f, "Info"),
            EventLevel::Warning => write!(f, "Warning"),
            EventLevel::Error => write!(f, "Error"),
            EventLevel::Critical => write!(f, "Critical"),
        }
    }
}

/// A single event in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// The severity level of the event
    pub level: EventLevel,
    /// The topic/category of the event (e.g., "contract.deployed")
    pub topic: String,
    /// The event message
    pub message: String,
    /// Optional structured data associated with the event
    pub data: Option<HashMap<String, String>>,
    /// Unix timestamp when the event was created
    pub timestamp: u64,
}

impl Event {
    /// Create a new event with the given level, topic, and message
    pub fn new(level: EventLevel, topic: &str, message: &str) -> Self {
        Event {
            level,
            topic: topic.to_string(),
            message: message.to_string(),
            data: None,
            timestamp: current_timestamp(),
        }
    }

    /// Create a new event with structured data
    pub fn new_with_data(level: EventLevel, topic: &str, data: HashMap<String, String>) -> Self {
        Event {
            level,
            topic: topic.to_string(),
            message: String::new(),
            data: Some(data),
            timestamp: current_timestamp(),
        }
    }

    /// Serialize the event to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
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
    fn test_event_creation() {
        let event = Event::new(EventLevel::Info, "test.topic", "Test message");
        assert_eq!(event.level, EventLevel::Info);
        assert_eq!(event.topic, "test.topic");
        assert_eq!(event.message, "Test message");
        assert!(event.data.is_none());
    }

    #[test]
    fn test_event_with_data() {
        let mut data = HashMap::new();
        data.insert("key".to_string(), "value".to_string());
        let event = Event::new_with_data(EventLevel::Debug, "test.topic", data);
        assert!(event.data.is_some());
        assert_eq!(event.data.unwrap().get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_event_level_ordering() {
        assert!(EventLevel::Trace < EventLevel::Debug);
        assert!(EventLevel::Debug < EventLevel::Info);
        assert!(EventLevel::Info < EventLevel::Warning);
        assert!(EventLevel::Warning < EventLevel::Error);
        assert!(EventLevel::Error < EventLevel::Critical);
    }

    #[test]
    fn test_event_serialization() {
        let event = Event::new(EventLevel::Info, "contract.deployed", "Contract deployed");
        let json = event.to_json().unwrap();
        assert!(json.contains("contract.deployed"));
        assert!(json.contains("Info"));
    }
}
