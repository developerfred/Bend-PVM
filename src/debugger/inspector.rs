use std::collections::HashMap;

use crate::debugger::{DebugInfo, DebuggerState, DebuggerError};
use crate::compiler::codegen::risc_v::Instruction;
use crate::compiler::parser::ast::{Expr, Statement};

/// Debug inspector for examining program state
pub struct DebugInspector {
    /// Debug information
    debug_info: DebugInfo,
    
    /// Current debugger state
    state: DebuggerState,
    
    /// Instructions
    instructions: Vec<Instruction>,
}

impl DebugInspector {
    /// Create a new debug inspector
    pub fn new(
        debug_info: DebugInfo,
        state: DebuggerState,
        instructions: Vec<Instruction>,
    ) -> Self {
        DebugInspector {
            debug_info,
            state,
            instructions,
        }
    }
    
    /// Get the source line at the current position
    pub fn current_source_line(&self) -> Option<String> {
        let pc = self.state.pc;
        
        if pc >= self.instructions.len() {
            return None;
        }
        
        let line_num = *self.debug_info.instruction_to_line.get(&pc)?;
        
        // Get the source line from the source code
        let lines: Vec<&str> = self.debug_info.source_code.lines().collect();
        
        if line_num >= 1 && line_num <= lines.len() {
            Some(lines[line_num - 1].to_string())
        } else {
            None
        }
    }
    
    /// Get the current function
    pub fn current_function(&self) -> Option<String> {
        self.state.current_function().map(|s| s.to_string())
    }
    
    /// Get the current instruction
    pub fn current_instruction(&self) -> Option<&Instruction> {
        let pc = self.state.pc;
        
        if pc >= self.instructions.len() {
            None
        } else {
            Some(&self.instructions[pc])
        }
    }
    
    /// Get local variables and their values
    pub fn local_variables(&self) -> HashMap<String, String> {
        let mut result = HashMap::new();
        
        // Get the current function
        let function_name = match self.current_function() {
            Some(name) => name,
            None => return result, // No function, no locals
        };
        
        // Get the function's local variables
        for (name, location) in &self.debug_info.locals {
            // In a real implementation, we would look up the variable's value
            // based on its location (stack or register)
            // For now, just use the values from the state
            if let Some(value) = self.state.get_local_variable(name) {
                result.insert(name.clone(), format!("{}", value));
            }
        }
        
        result
    }
    
    /// Evaluate an expression in the current context
    pub fn evaluate(&self, expression: &str) -> Result<String, DebuggerError> {
        // In a real implementation, this would parse and evaluate the expression
        // in the context of the current state
        // For now, just return an error
        Err(DebuggerError::Generic(format!("Expression evaluation not implemented: {}", expression)))
    }
    
    /// Get the call stack
    pub fn call_stack(&self) -> Vec<String> {
        self.state.call_stack.clone()
    }
    
    /// Get source context around the current line
    pub fn source_context(&self, context_lines: usize) -> Vec<(usize, String, bool)> {
        let pc = self.state.pc;
        
        if pc >= self.instructions.len() {
            return Vec::new();
        }
        
        let current_line = match self.debug_info.instruction_to_line.get(&pc) {
            Some(&line) => line,
            None => return Vec::new(),
        };
        
        let lines: Vec<&str> = self.debug_info.source_code.lines().collect();
        let mut result = Vec::new();
        
        // Calculate the range of lines to display
        let start_line = if current_line > context_lines {
            current_line - context_lines
        } else {
            1
        };
        
        let end_line = if current_line + context_lines <= lines.len() {
            current_line + context_lines
        } else {
            lines.len()
        };
        
        // Add lines to the result
        for line_num in start_line..=end_line {
            if line_num >= 1 && line_num <= lines.len() {
                let is_current = line_num == current_line;
                result.push((line_num, lines[line_num - 1].to_string(), is_current));
            }
        }
        
        result
    }
    
    /// Get a pretty-printed representation of memory
    pub fn memory_dump(&self, address: u32, size: usize) -> Vec<(u32, Vec<u8>)> {
        let mut result = Vec::new();
        
        for i in 0..size {
            let addr = address + i as u32;
            let row_addr = addr & !0xF; // Align to 16 bytes
            
            // Create a new row if needed
            if result.is_empty() || result.last().unwrap().0 != row_addr {
                result.push((row_addr, vec![0; 16]));
            }
            
            // Update the row with the memory value
            let row = result.last_mut().unwrap();
            let offset = (addr - row_addr) as usize;
            
            row.1[offset] = self.state.get_memory(addr).unwrap_or(0);
        }
        
        result
    }
    
    /// Get register values
    pub fn registers(&self) -> HashMap<String, u32> {
        self.state.registers.clone()
    }
    
    /// Get breakpoint information
    pub fn breakpoint_info(&self, line: usize) -> Option<String> {
        if !self.debug_info.line_to_instruction.contains_key(&line) {
            return None;
        }
        
        Some(format!("Breakpoint set at line {}", line))
    }
}