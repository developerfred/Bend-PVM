/// Reentrancy Guard module
///
/// Provides protection against reentrancy attacks, which are a common and dangerous
/// type of security vulnerability in smart contracts and distributed systems.
use crate::compiler::parser::ast::*;
use crate::security::SecurityError;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Call depth tracking
#[derive(Debug, Clone)]
struct CallStackEntry {
    function: String,
    timestamp: u64,
    caller: Vec<u8>, // Address of the caller
}

/// Reentrancy guard state
#[derive(Debug, Clone)]
enum GuardState {
    Unlocked,
    Locked {
        lock_holder: Vec<u8>,
        lock_time: u64,
    },
}

/// Reentrancy protection modes
#[derive(Debug, Clone)]
pub enum ProtectionMode {
    /// No protection
    None,
    /// Basic function-level reentrancy guard
    FunctionLevel,
    /// Contract-level reentrancy guard
    ContractLevel,
    /// Time-based reentrancy guard
    TimeBased { min_interval: u64 },
    /// Address-based reentrancy guard
    AddressBased,
}

/// Reentrancy guard
pub struct ReentrancyGuard {
    call_stack: Vec<CallStackEntry>,
    locked_functions: HashMap<String, GuardState>,
    visited_addresses: HashMap<String, HashSet<Vec<u8>>>,
    max_call_depth: u32,
    attempt_count: u32,
    mode: ProtectionMode,
}

impl Default for ReentrancyGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl ReentrancyGuard {
    /// Create a new reentrancy guard
    pub fn new() -> Self {
        Self {
            call_stack: Vec::new(),
            locked_functions: HashMap::new(),
            visited_addresses: HashMap::new(),
            max_call_depth: 100,
            attempt_count: 0,
            mode: ProtectionMode::FunctionLevel,
        }
    }

    /// Set the protection mode
    pub fn set_mode(&mut self, mode: ProtectionMode) {
        self.mode = mode;
    }

    /// Set maximum call depth
    pub fn set_max_depth(&mut self, depth: u32) {
        self.max_call_depth = depth;
    }

    /// Enter a function (check for reentrancy)
    pub fn enter_function(&mut self, function: &str) -> Result<(), SecurityError> {
        self.attempt_count += 1;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check call stack depth
        if self.call_stack.len() >= self.max_call_depth as usize {
            return Err(SecurityError::ReentrancyDetected);
        }

        match &self.mode {
            ProtectionMode::None => {
                // No protection, just track the call
            }
            ProtectionMode::FunctionLevel => {
                // Check if function is already in call stack (reentrancy)
                if self
                    .call_stack
                    .iter()
                    .any(|entry| entry.function == function)
                {
                    return Err(SecurityError::ReentrancyDetected);
                }
            }
            ProtectionMode::ContractLevel => {
                // This would need contract address information
                // For now, treat similar to function level
                if self
                    .call_stack
                    .iter()
                    .any(|entry| entry.function.split('/').next() == function.split('/').next())
                {
                    return Err(SecurityError::ReentrancyDetected);
                }
            }
            ProtectionMode::TimeBased { min_interval } => {
                // Check time-based reentrancy
                if let Some(last_entry) = self.call_stack.last() {
                    if current_time - last_entry.timestamp < *min_interval {
                        return Err(SecurityError::ReentrancyDetected);
                    }
                }
            }
            ProtectionMode::AddressBased => {
                // Check address-based reentrancy
                // This would need the caller's address
                // For now, use function-level check
                if self
                    .call_stack
                    .iter()
                    .any(|entry| entry.function == function)
                {
                    return Err(SecurityError::ReentrancyDetected);
                }
            }
        }

        // Add entry to call stack
        self.call_stack.push(CallStackEntry {
            function: function.to_string(),
            timestamp: current_time,
            caller: Vec::new(), // Would be set by caller
        });

        Ok(())
    }

    /// Exit a function
    pub fn exit_function(&mut self, function: &str) {
        // Remove the most recent matching entry from call stack
        if let Some(pos) = self
            .call_stack
            .iter()
            .rposition(|entry| entry.function == function)
        {
            self.call_stack.remove(pos);
        }
    }

    /// Set caller address for current function
    pub fn set_caller(&mut self, function: &str, caller: &[u8]) {
        if let Some(entry) = self
            .call_stack
            .iter_mut()
            .rfind(|entry| entry.function == function)
        {
            entry.caller = caller.to_vec();
        }
    }

    /// Check if address has called this function recently
    pub fn check_address_frequency(
        &mut self,
        function: &str,
        address: &[u8],
        max_frequency: u32,
    ) -> Result<(), SecurityError> {
        let key = function.to_string();
        let address_set = self
            .visited_addresses
            .entry(key)
            .or_insert_with(HashSet::new);

        if address_set.len() >= max_frequency as usize && !address_set.contains(address) {
            return Err(SecurityError::ReentrancyDetected);
        }

        address_set.insert(address.to_vec());
        Ok(())
    }

    /// Lock a function temporarily
    pub fn lock_function(
        &mut self,
        function: &str,
        lock_holder: &[u8],
        duration: u64,
    ) -> Result<(), SecurityError> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let guard_state = GuardState::Locked {
            lock_holder: lock_holder.to_vec(),
            lock_time: current_time + duration,
        };

        if let Some(existing_lock) = self.locked_functions.get(function) {
            match existing_lock {
                GuardState::Locked { lock_time, .. } if *lock_time > current_time => {
                    return Err(SecurityError::ReentrancyDetected);
                }
                _ => {}
            }
        }

        self.locked_functions
            .insert(function.to_string(), guard_state);
        Ok(())
    }

    /// Unlock a function
    pub fn unlock_function(
        &mut self,
        function: &str,
        lock_holder: &[u8],
    ) -> Result<(), SecurityError> {
        if let Some(guard_state) = self.locked_functions.get_mut(function) {
            match guard_state {
                GuardState::Locked {
                    lock_holder: holder,
                    ..
                } if holder == lock_holder => {
                    self.locked_functions.remove(function);
                    Ok(())
                }
                _ => Err(SecurityError::AccessDenied(
                    "Not the lock holder".to_string(),
                )),
            }
        } else {
            Ok(())
        }
    }

    /// Check if function is currently locked
    pub fn is_function_locked(&self, function: &str) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(guard_state) = self.locked_functions.get(function) {
            match guard_state {
                GuardState::Locked { lock_time, .. } => *lock_time > current_time,
                GuardState::Unlocked => false,
            }
        } else {
            false
        }
    }

    /// Get current call stack depth
    pub fn current_depth(&self) -> u32 {
        self.call_stack.len() as u32
    }

    /// Get call stack information
    pub fn get_call_stack(&self) -> &[CallStackEntry] {
        &self.call_stack
    }

    /// Check for potential reentrancy patterns
    pub fn check_reentrancy_patterns(&self, functions: &[String]) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for recursive calls
        for func in functions {
            let call_count = self
                .call_stack
                .iter()
                .filter(|entry| &entry.function == func)
                .count();
            if call_count > 1 {
                warnings.push(format!("Potential recursive call detected: {}", func));
            }
        }

        // Check for deep call stacks
        if self.call_stack.len() > 10 {
            warnings.push(format!(
                "Deep call stack detected: {} levels",
                self.call_stack.len()
            ));
        }

        // Check for time-based reentrancy
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut time_deltas = Vec::new();
        for entry in &self.call_stack {
            time_deltas.push(current_time - entry.timestamp);
        }

        if time_deltas.len() > 1 {
            let avg_delta = time_deltas.iter().sum::<u64>() / time_deltas.len() as u64;
            if avg_delta < 1 {
                warnings.push(
                    "Very rapid successive calls detected (potential reentrancy)".to_string(),
                );
            }
        }

        warnings
    }

    /// Reset the reentrancy guard
    pub fn reset(&mut self) {
        self.call_stack.clear();
        self.locked_functions.clear();
        self.visited_addresses.clear();
        self.attempt_count = 0;
    }

    /// Get attempt statistics
    pub fn get_attempt_count(&self) -> u32 {
        self.attempt_count
    }

    /// Analyze call patterns for security assessment
    pub fn analyze_call_patterns(&self) -> SecurityAnalysis {
        let mut patterns = HashMap::new();
        let mut total_calls = 0;
        let mut unique_functions = HashSet::new();

        for entry in &self.call_stack {
            *patterns.entry(entry.function.clone()).or_insert(0) += 1;
            unique_functions.insert(entry.function.clone());
            total_calls += 1;
        }

        let max_call_frequency = patterns.values().max().copied().unwrap_or(0);
        let avg_call_frequency = if total_calls > 0 {
            total_calls / unique_functions.len() as u32
        } else {
            0
        };

        SecurityAnalysis {
            total_calls,
            unique_functions: unique_functions.len(),
            max_call_frequency,
            avg_call_frequency,
            reentrancy_risk: if max_call_frequency > 1 {
                "HIGH".to_string()
            } else if max_call_frequency > 0 {
                "MEDIUM".to_string()
            } else {
                "LOW".to_string()
            },
        }
    }
}

/// Security analysis result
#[derive(Debug, Clone)]
pub struct SecurityAnalysis {
    pub total_calls: u32,
    pub unique_functions: usize,
    pub max_call_frequency: u32,
    pub avg_call_frequency: u32,
    pub reentrancy_risk: String,
}

/// Register reentrancy protection functions in AST
pub fn register_reentrancy_functions() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location {
        line: 0,
        column: 0,
        start: 0,
        end: 0,
    };

    let string_type = Type::Named {
        name: "String".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    let bool_type = Type::Named {
        name: "Bool".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    let int_type = Type::Named {
        name: "Int".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    // Check if function is reentrant-safe
    definitions.push(Definition::FunctionDef {
        name: "ReentrancyGuard/checkSafe".to_string(),
        params: vec![Parameter {
            name: "function".to_string(),
            ty: string_type.clone(),
            location: dummy_loc.clone(),
        }],
        return_type: Some(bool_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Get current call depth
    definitions.push(Definition::FunctionDef {
        name: "ReentrancyGuard/getDepth".to_string(),
        params: vec![],
        return_type: Some(int_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Reset reentrancy guard
    definitions.push(Definition::FunctionDef {
        name: "ReentrancyGuard/reset".to_string(),
        params: vec![],
        return_type: None,
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    definitions
}
