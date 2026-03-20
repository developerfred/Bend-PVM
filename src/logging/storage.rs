//! Log Storage Implementation
//! Issue #24 - Event System and Logging Framework

use super::logger::{LogEntry, LogLevel};

/// Rotation policy for log storage
#[derive(Debug, Clone)]
pub enum RotationPolicy {
    /// Delete oldest logs when storage is full
    DeleteOldest,
    /// Delete newest logs when storage is full
    DeleteNewest,
    /// No rotation - stop accepting new logs
    NoRotation,
}

/// In-memory log storage with rotation support
#[derive(Debug)]
pub struct InMemoryLogStorage {
    entries: Vec<LogEntry>,
    max_size: usize,
    rotation_policy: RotationPolicy,
}

impl InMemoryLogStorage {
    /// Create a new in-memory log storage
    pub fn new() -> Self {
        InMemoryLogStorage {
            entries: Vec::new(),
            max_size: 10000,
            rotation_policy: RotationPolicy::DeleteOldest,
        }
    }

    /// Set maximum storage size
    pub fn set_max_size(&mut self, size: usize) {
        self.max_size = size;
        self.enforce_max_size();
    }

    /// Set rotation policy
    pub fn set_rotation_policy(&mut self, policy: RotationPolicy) {
        self.rotation_policy = policy;
    }

    /// Append a new log entry
    pub fn append(&mut self, entry: LogEntry) {
        if self.entries.len() >= self.max_size {
            match self.rotation_policy {
                RotationPolicy::DeleteOldest => {
                    self.entries.remove(0);
                }
                RotationPolicy::DeleteNewest => {
                    self.entries.pop();
                }
                RotationPolicy::NoRotation => {
                    return;
                }
            }
        }
        self.entries.push(entry);
    }

    /// Get total number of log entries
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Get all log entries
    pub fn get_all(&self) -> Vec<LogEntry> {
        self.entries.clone()
    }

    /// Get log entries filtered by level
    pub fn get_by_level(&self, level: LogLevel) -> Vec<LogEntry> {
        self.entries
            .iter()
            .filter(|e| e.level == level)
            .cloned()
            .collect()
    }

    /// Clear all log entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    fn enforce_max_size(&mut self) {
        while self.entries.len() > self.max_size {
            match self.rotation_policy {
                RotationPolicy::DeleteOldest => {
                    self.entries.remove(0);
                }
                RotationPolicy::DeleteNewest => {
                    self.entries.pop();
                }
                RotationPolicy::NoRotation => break,
            }
        }
    }
}

impl Default for InMemoryLogStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let storage = InMemoryLogStorage::new();
        assert_eq!(storage.count(), 0);
    }

    #[test]
    fn test_storage_append() {
        let mut storage = InMemoryLogStorage::new();
        storage.append(LogEntry::new(LogLevel::Info, "test", "Msg"));
        assert_eq!(storage.count(), 1);
    }

    #[test]
    fn test_storage_max_size() {
        let mut storage = InMemoryLogStorage::new();
        storage.set_max_size(2);
        storage.append(LogEntry::new(LogLevel::Info, "1", "First"));
        storage.append(LogEntry::new(LogLevel::Info, "2", "Second"));
        storage.append(LogEntry::new(LogLevel::Info, "3", "Third"));
        assert_eq!(storage.count(), 2);
    }

    #[test]
    fn test_storage_filter_by_level() {
        let mut storage = InMemoryLogStorage::new();
        storage.append(LogEntry::new(LogLevel::Debug, "m", "Debug"));
        storage.append(LogEntry::new(LogLevel::Error, "m", "Error"));
        storage.append(LogEntry::new(LogLevel::Info, "m", "Info"));

        let errors = storage.get_by_level(LogLevel::Error);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Error");
    }

    #[test]
    fn test_storage_clear() {
        let mut storage = InMemoryLogStorage::new();
        storage.append(LogEntry::new(LogLevel::Info, "m", "Msg"));
        assert_eq!(storage.count(), 1);
        storage.clear();
        assert_eq!(storage.count(), 0);
    }

    #[test]
    fn test_rotation_policy_delete_newest() {
        let mut storage = InMemoryLogStorage::new();
        storage.set_max_size(2);
        storage.set_rotation_policy(RotationPolicy::DeleteNewest);
        storage.append(LogEntry::new(LogLevel::Info, "1", "First"));
        storage.append(LogEntry::new(LogLevel::Info, "2", "Second"));
        storage.append(LogEntry::new(LogLevel::Info, "3", "Third"));

        let logs = storage.get_all();
        assert_eq!(logs.len(), 2);
        // Should have entries 1 and 3 (entry 2 was removed as newest)
        assert_eq!(logs[0].message, "First");
        assert_eq!(logs[1].message, "Third");
    }
}
