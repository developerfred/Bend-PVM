//! Event System Module
//! Issue #24 - Event System and Logging Framework

mod emitter;
mod event;

pub use emitter::{EventEmitter, EventSubscription};
pub use event::{Event, EventLevel};

use std::collections::HashMap;

/// Initialize the global event system
pub fn init_event_system() -> EventEmitter {
    EventEmitter::new()
}
