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
