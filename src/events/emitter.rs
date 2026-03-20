//! Event Emitter
//! Issue #24 - Event System and Logging Framework

use super::event::{Event, EventLevel};

/// Event subscription for filtering
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// Unique subscription ID
    pub id: u64,
    /// Allowed event levels for this subscription
    pub levels: Vec<EventLevel>,
    /// Allowed topics (supports wildcards like "contract.*")
    pub topics: Vec<String>,
}

/// Event emitter for publishing and managing events
#[derive(Debug)]
pub struct EventEmitter {
    events: Vec<Event>,
    subscriptions: Vec<EventSubscription>,
    max_history: usize,
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        EventEmitter {
            events: Vec::new(),
            subscriptions: Vec::new(),
            max_history: 1000,
        }
    }

    /// Emit a new event
    pub fn emit(&mut self, event: Event) {
        if self.events.len() >= self.max_history {
            self.events.remove(0);
        }
        self.events.push(event);
    }

    /// Get the total number of events
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get all events
    pub fn get_events(&self) -> Vec<Event> {
        self.events.clone()
    }

    /// Get events filtered by level
    pub fn get_events_with_level(&self, level: EventLevel) -> Vec<Event> {
        self.events
            .iter()
            .filter(|e| e.level == level)
            .cloned()
            .collect()
    }

    /// Get events filtered by topic
    pub fn get_events_by_topic(&self, topic: &str) -> Vec<Event> {
        self.events
            .iter()
            .filter(|e| e.topic == topic)
            .cloned()
            .collect()
    }

    /// Set maximum history size
    pub fn set_max_history(&mut self, max: usize) {
        self.max_history = max;
        while self.events.len() > max {
            self.events.remove(0);
        }
    }

    /// Subscribe to events with given filters
    pub fn subscribe(&mut self, levels: Vec<EventLevel>, topics: Vec<String>) -> Option<u64> {
        let id = (self.subscriptions.len() as u64) + 1;
        self.subscriptions
            .push(EventSubscription { id, levels, topics });
        Some(id)
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&mut self, id: u64) {
        self.subscriptions.retain(|s| s.id != id);
    }

    /// Check if subscription exists
    pub fn is_subscribed(&self, id: u64) -> bool {
        self.subscriptions.iter().any(|s| s.id == id)
    }
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_emitter_creation() {
        let emitter = EventEmitter::new();
        assert_eq!(emitter.event_count(), 0);
    }

    #[test]
    fn test_event_emission() {
        let mut emitter = EventEmitter::new();
        emitter.emit(Event::new(EventLevel::Info, "test", "message"));
        assert_eq!(emitter.event_count(), 1);
    }

    #[test]
    fn test_event_filtering_by_level() {
        let mut emitter = EventEmitter::new();
        emitter.emit(Event::new(EventLevel::Debug, "test", "Debug msg"));
        emitter.emit(Event::new(EventLevel::Warning, "test", "Warn msg"));
        emitter.emit(Event::new(EventLevel::Error, "test", "Error msg"));

        let warnings = emitter.get_events_with_level(EventLevel::Warning);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].message, "Warn msg");
    }

    #[test]
    fn test_event_filtering_by_topic() {
        let mut emitter = EventEmitter::new();
        emitter.emit(Event::new(EventLevel::Info, "contract.deploy", "Deploy 1"));
        emitter.emit(Event::new(
            EventLevel::Info,
            "contract.execute",
            "Execute 1",
        ));
        emitter.emit(Event::new(EventLevel::Info, "contract.deploy", "Deploy 2"));

        let deploys = emitter.get_events_by_topic("contract.deploy");
        assert_eq!(deploys.len(), 2);
    }

    #[test]
    fn test_event_history_limit() {
        let mut emitter = EventEmitter::new();
        emitter.set_max_history(3);

        for i in 0..5 {
            let msg = format!("Event {}", i);
            emitter.emit(Event::new(EventLevel::Info, "test", &msg));
        }

        assert_eq!(emitter.event_count(), 3);
    }

    #[test]
    fn test_subscription() {
        let mut emitter = EventEmitter::new();
        let sub_id = emitter.subscribe(
            vec![EventLevel::Error, EventLevel::Critical],
            vec!["contract.*".to_string()],
        );
        assert!(sub_id.is_some());
        let id = sub_id.unwrap();
        assert!(emitter.is_subscribed(id));

        emitter.unsubscribe(id);
        assert!(!emitter.is_subscribed(id));
    }
}
