use std::collections::HashMap;

/// Execution state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExecutionState {
    /// Program is running
    Running,

    /// Program is paused
    Paused,

    /// Program is stopped
    Stopped,
}

/// Debugger state
#[derive(Debug, Clone)]
pub struct DebuggerState {
    /// Execution state
    pub execution_state: ExecutionState,

    /// Program counter
    pub pc: usize,

    /// Call stack
    pub call_stack: Vec<String>,

    /// Register values
    pub registers: HashMap<String, u32>,

    /// Memory values
    pub memory: HashMap<u32, u8>,

    /// Local variables
    pub local_variables: HashMap<String, u32>,
}

impl Default for DebuggerState {
    fn default() -> Self {
        Self::new()
    }
}

impl DebuggerState {
    /// Create a new debugger state
    pub fn new() -> Self {
        DebuggerState {
            execution_state: ExecutionState::Stopped,
            pc: 0,
            call_stack: Vec::new(),
            registers: HashMap::new(),
            memory: HashMap::new(),
            local_variables: HashMap::new(),
        }
    }

    /// Get a register value
    pub fn get_register(&self, register: &str) -> Option<u32> {
        self.registers.get(register).cloned()
    }

    /// Set a register value
    pub fn set_register(&mut self, register: &str, value: u32) {
        self.registers.insert(register.to_string(), value);
    }

    /// Get a memory value
    pub fn get_memory(&self, address: u32) -> Option<u8> {
        self.memory.get(&address).cloned()
    }

    /// Set a memory value
    pub fn set_memory(&mut self, address: u32, value: u8) {
        self.memory.insert(address, value);
    }

    /// Get a local variable value
    pub fn get_local_variable(&self, name: &str) -> Option<u32> {
        self.local_variables.get(name).cloned()
    }

    /// Set a local variable value
    pub fn set_local_variable(&mut self, name: &str, value: u32) {
        self.local_variables.insert(name.to_string(), value);
    }

    /// Get current function
    pub fn current_function(&self) -> Option<&str> {
        self.call_stack.last().map(|s| s.as_str())
    }

    /// Reset the state
    pub fn reset(&mut self) {
        self.execution_state = ExecutionState::Stopped;
        self.pc = 0;
        self.call_stack.clear();
        self.registers.clear();
        self.memory.clear();
        self.local_variables.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_state_variants() {
        assert!(matches!(ExecutionState::Running, ExecutionState::Running));
        assert!(matches!(ExecutionState::Paused, ExecutionState::Paused));
        assert!(matches!(ExecutionState::Stopped, ExecutionState::Stopped));
    }

    #[test]
    fn test_debugger_state_new() {
        let state = DebuggerState::new();
        assert_eq!(state.execution_state, ExecutionState::Stopped);
        assert_eq!(state.pc, 0);
        assert!(state.call_stack.is_empty());
        assert!(state.registers.is_empty());
        assert!(state.memory.is_empty());
        assert!(state.local_variables.is_empty());
    }

    #[test]
    fn test_register_operations() {
        let mut state = DebuggerState::new();
        state.set_register("rax", 42);
        assert_eq!(state.get_register("rax"), Some(42));

        state.set_register("rax", 100);
        assert_eq!(state.get_register("rax"), Some(100));

        assert_eq!(state.get_register("unknown"), None);
    }

    #[test]
    fn test_memory_operations() {
        let mut state = DebuggerState::new();
        state.set_memory(0x1000, 0xAB);
        assert_eq!(state.get_memory(0x1000), Some(0xAB));

        state.set_memory(0x1000, 0xCD);
        assert_eq!(state.get_memory(0x1000), Some(0xCD));

        assert_eq!(state.get_memory(0xFFFF), None);
    }

    #[test]
    fn test_local_variables() {
        let mut state = DebuggerState::new();
        state.set_local_variable("x", 10);
        state.set_local_variable("y", 20);

        assert_eq!(state.get_local_variable("x"), Some(10));
        assert_eq!(state.get_local_variable("y"), Some(20));
        assert_eq!(state.get_local_variable("z"), None);
    }

    #[test]
    fn test_call_stack() {
        let mut state = DebuggerState::new();
        state.call_stack.push("main".to_string());
        state.call_stack.push("foo".to_string());

        assert_eq!(state.current_function(), Some("foo"));

        state.call_stack.pop();
        assert_eq!(state.current_function(), Some("main"));

        state.call_stack.pop();
        assert_eq!(state.current_function(), None);
    }

    #[test]
    fn test_reset() {
        let mut state = DebuggerState::new();
        state.pc = 100;
        state.set_register("rax", 42);
        state.set_memory(0x1000, 0xAB);
        state.call_stack.push("test".to_string());

        state.reset();

        assert_eq!(state.pc, 0);
        assert_eq!(state.execution_state, ExecutionState::Stopped);
        assert!(state.registers.is_empty());
        assert!(state.memory.is_empty());
        assert!(state.call_stack.is_empty());
    }
}
