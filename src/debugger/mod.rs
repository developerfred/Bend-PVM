pub mod breakpoint;
pub mod disassembler;
pub mod inspector;
pub mod state;

use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

use self::breakpoint::Breakpoint;
use self::state::{DebuggerState, ExecutionState};
use crate::compiler::codegen::risc_v::Instruction;
use crate::runtime::env::{Environment, ExecutionContext};

/// Debugger errors
#[derive(Error, Debug)]
pub enum DebuggerError {
    #[error("Debugger error: {0}")]
    Generic(String),

    #[error("Breakpoint error: {0}")]
    Breakpoint(String),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Execution error: {0}")]
    Execution(String),
}

/// Debugger event
#[derive(Debug, Clone)]
pub enum DebuggerEvent {
    /// Program started
    Started,

    /// Program stopped at a breakpoint
    Breakpoint(Breakpoint),

    /// Program stepped
    Stepped,

    /// Program continued
    Continued,

    /// Program finished
    Finished,

    /// Program crashed
    Crashed(String),
}

/// Debugger command
#[derive(Debug, Clone)]
pub enum DebuggerCommand {
    /// Run/continue the program
    Continue,

    /// Step to the next instruction
    Step,

    /// Step to the next line
    StepLine,

    /// Step into a function
    StepIn,

    /// Step out of a function
    StepOut,

    /// Set a breakpoint
    SetBreakpoint(Breakpoint),

    /// Remove a breakpoint
    RemoveBreakpoint(Breakpoint),

    /// Print the current state
    Print,

    /// Evaluate an expression
    Evaluate(String),

    /// Exit the debugger
    Exit,
}

/// Debug information for a program
#[derive(Debug, Clone)]
pub struct DebugInfo {
    /// Source file path
    pub source_path: PathBuf,

    /// Source code
    pub source_code: String,

    /// Line to instruction mapping
    pub line_to_instruction: HashMap<usize, Vec<usize>>,

    /// Instruction to line mapping
    pub instruction_to_line: HashMap<usize, usize>,

    /// Local variable locations
    pub locals: HashMap<String, VariableLocation>,

    /// Function ranges
    pub functions: HashMap<String, FunctionRange>,
}

/// Variable location in memory or registers
#[derive(Debug, Clone)]
pub enum VariableLocation {
    /// Stack variable
    Stack(i32),

    /// Register variable
    Register(u8),
}

/// Function range in the code
#[derive(Debug, Clone)]
pub struct FunctionRange {
    /// Function name
    pub name: String,

    /// Start instruction index
    pub start: usize,

    /// End instruction index
    pub end: usize,

    /// Start line in source
    pub start_line: usize,

    /// End line in source
    pub end_line: usize,
}

/// Debugger for Bend-PVM programs
pub struct Debugger {
    /// Debug information
    debug_info: DebugInfo,

    /// Debugger state
    state: DebuggerState,

    /// Instructions
    instructions: Vec<Instruction>,

    /// Breakpoints
    breakpoints: Vec<Breakpoint>,

    /// Environment
    environment: Environment,

    /// Event handler
    event_handler: Option<Box<dyn Fn(DebuggerEvent)>>,
}

impl Debugger {
    /// Create a new debugger
    pub fn new(
        debug_info: DebugInfo,
        instructions: Vec<Instruction>,
        context: ExecutionContext,
    ) -> Self {
        Debugger {
            debug_info,
            state: DebuggerState::new(),
            instructions,
            breakpoints: Vec::new(),
            environment: Environment::new(context),
            event_handler: None,
        }
    }

    /// Set the event handler
    pub fn set_event_handler<F>(&mut self, handler: F)
    where
        F: Fn(DebuggerEvent) + 'static,
    {
        self.event_handler = Some(Box::new(handler));
    }

    /// Add a breakpoint
    pub fn add_breakpoint(&mut self, breakpoint: Breakpoint) -> Result<(), DebuggerError> {
        // Validate the breakpoint
        match breakpoint {
            Breakpoint::Line(line) => {
                if !self.debug_info.line_to_instruction.contains_key(&line) {
                    return Err(DebuggerError::Breakpoint(format!(
                        "Line {} not found in the program",
                        line
                    )));
                }
            }
            Breakpoint::Instruction(instruction) => {
                if instruction >= self.instructions.len() {
                    return Err(DebuggerError::Breakpoint(format!(
                        "Instruction {} not found in the program",
                        instruction
                    )));
                }
            }
            Breakpoint::Function(ref function) => {
                if !self.debug_info.functions.contains_key(function) {
                    return Err(DebuggerError::Breakpoint(format!(
                        "Function {} not found in the program",
                        function
                    )));
                }
            }
        }

        self.breakpoints.push(breakpoint);

        Ok(())
    }

    /// Remove a breakpoint
    pub fn remove_breakpoint(&mut self, breakpoint: Breakpoint) -> Result<(), DebuggerError> {
        let index = self.breakpoints.iter().position(|b| b == &breakpoint);

        if let Some(index) = index {
            self.breakpoints.remove(index);
            Ok(())
        } else {
            Err(DebuggerError::Breakpoint(format!(
                "Breakpoint {:?} not found",
                breakpoint
            )))
        }
    }

    /// Run the program
    pub fn run(&mut self) -> Result<(), DebuggerError> {
        // Emit the started event
        self.emit_event(DebuggerEvent::Started);

        // Set the initial state
        self.state.execution_state = ExecutionState::Running;

        loop {
            // Check if we should stop
            if self.state.execution_state == ExecutionState::Stopped {
                break;
            }

            // Check if we're at a breakpoint
            if self.is_at_breakpoint() {
                self.state.execution_state = ExecutionState::Paused;
                self.emit_event(DebuggerEvent::Breakpoint(
                    self.current_breakpoint().unwrap(),
                ));
                break;
            }

            // Execute the next instruction
            self.step()?;
        }

        Ok(())
    }

    /// Step to the next instruction
    pub fn step(&mut self) -> Result<(), DebuggerError> {
        // Check if we're already stopped
        if self.state.execution_state == ExecutionState::Stopped {
            return Ok(());
        }

        // Get the current instruction
        let pc = self.state.pc;

        if pc >= self.instructions.len() {
            self.state.execution_state = ExecutionState::Stopped;
            self.emit_event(DebuggerEvent::Finished);
            return Ok(());
        }

        let instruction = self.instructions[pc].clone();

        // Execute the instruction
        match self.execute_instruction(&instruction) {
            Ok(_) => {
                // Increment the program counter
                self.state.pc += 1;

                // Emit the stepped event
                self.emit_event(DebuggerEvent::Stepped);

                Ok(())
            }
            Err(err) => {
                self.state.execution_state = ExecutionState::Stopped;
                self.emit_event(DebuggerEvent::Crashed(err.to_string()));

                Err(err)
            }
        }
    }

    /// Continue execution until the next breakpoint
    pub fn continue_execution(&mut self) -> Result<(), DebuggerError> {
        // Check if we're already stopped
        if self.state.execution_state == ExecutionState::Stopped {
            return Ok(());
        }

        // Set the state to running
        self.state.execution_state = ExecutionState::Running;

        // Emit the continued event
        self.emit_event(DebuggerEvent::Continued);

        // Run until the next breakpoint or end of program
        self.run()
    }

    /// Step to the next line
    pub fn step_line(&mut self) -> Result<(), DebuggerError> {
        // Check if we're already stopped
        if self.state.execution_state == ExecutionState::Stopped {
            return Ok(());
        }

        // Get the current line
        let current_line = self.current_line();

        // Step until we reach a different line
        loop {
            self.step()?;

            if self.state.execution_state == ExecutionState::Stopped {
                break;
            }

            let new_line = self.current_line();

            if new_line != current_line {
                break;
            }
        }

        Ok(())
    }

    /// Step into a function
    pub fn step_in(&mut self) -> Result<(), DebuggerError> {
        // Check if we're already stopped
        if self.state.execution_state == ExecutionState::Stopped {
            return Ok(());
        }

        // Get the current instruction
        let pc = self.state.pc;

        if pc >= self.instructions.len() {
            return Ok(());
        }

        let instruction = self.instructions[pc].clone();

        // Check if the instruction is a function call
        match instruction {
            Instruction::JumpAndLink(_, ref label) => {
                // Step once to enter the function
                self.step()?;

                // Get the function name from the label
                if let Some(function_name) = self.label_to_function(label) {
                    // Update the call stack
                    self.state.call_stack.push(function_name.clone());
                }
            }
            _ => {
                // Not a function call, just step normally
                self.step()?;
            }
        }

        Ok(())
    }

    /// Step out of a function
    pub fn step_out(&mut self) -> Result<(), DebuggerError> {
        // Check if we're already stopped
        if self.state.execution_state == ExecutionState::Stopped {
            return Ok(());
        }

        // Check if we're in a function
        if self.state.call_stack.is_empty() {
            return Ok(());
        }

        // Get the current call stack depth
        let depth = self.state.call_stack.len();

        // Step until we return from the function
        loop {
            self.step()?;

            if self.state.execution_state == ExecutionState::Stopped {
                break;
            }

            if self.state.call_stack.len() < depth {
                break;
            }
        }

        Ok(())
    }

    /// Get the current line
    fn current_line(&self) -> usize {
        let pc = self.state.pc;

        if pc >= self.instructions.len() {
            return 0;
        }

        *self.debug_info.instruction_to_line.get(&pc).unwrap_or(&0)
    }

    /// Check if we're at a breakpoint
    fn is_at_breakpoint(&self) -> bool {
        let pc = self.state.pc;

        if pc >= self.instructions.len() {
            return false;
        }

        let line = self.current_line();

        // Check all breakpoints
        for breakpoint in &self.breakpoints {
            match breakpoint {
                Breakpoint::Line(bp_line) => {
                    if *bp_line == line {
                        return true;
                    }
                }
                Breakpoint::Instruction(bp_instruction) => {
                    if *bp_instruction == pc {
                        return true;
                    }
                }
                Breakpoint::Function(bp_function) => {
                    if let Some(function_range) = self.debug_info.functions.get(bp_function) {
                        if pc == function_range.start {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Get the current breakpoint
    fn current_breakpoint(&self) -> Option<Breakpoint> {
        let pc = self.state.pc;

        if pc >= self.instructions.len() {
            return None;
        }

        let line = self.current_line();

        // Check all breakpoints
        for breakpoint in &self.breakpoints {
            match breakpoint {
                Breakpoint::Line(bp_line) => {
                    if *bp_line == line {
                        return Some(breakpoint.clone());
                    }
                }
                Breakpoint::Instruction(bp_instruction) => {
                    if *bp_instruction == pc {
                        return Some(breakpoint.clone());
                    }
                }
                Breakpoint::Function(bp_function) => {
                    if let Some(function_range) = self.debug_info.functions.get(bp_function) {
                        if pc == function_range.start {
                            return Some(breakpoint.clone());
                        }
                    }
                }
            }
        }

        None
    }

    /// Execute an instruction
    fn execute_instruction(&mut self, instruction: &Instruction) -> Result<(), DebuggerError> {
        match instruction {
            // Load from memory (lw)
            Instruction::Load(rd, rs1, offset) => {
                let base_addr = self.get_reg_value(rs1)? as i32;
                let addr = (base_addr + offset) as u32;

                // Read 4 bytes from memory (little-endian)
                let mut value: u32 = 0;
                for i in 0..4 {
                    if let Some(byte) = self.state.memory.get(&(addr + i)) {
                        value |= (*byte as u32) << (8 * i);
                    } else {
                        return Err(DebuggerError::Execution(format!(
                            "Memory read error: address 0x{:08x}",
                            addr + i
                        )));
                    }
                }

                self.set_reg_value(rd, value);
                Ok(())
            }

            // Store to memory (sw)
            Instruction::Store(rs2, rs1, offset) => {
                let value = self.get_reg_value(rs2)?;
                let base_addr = self.get_reg_value(rs1)? as i32;
                let addr = (base_addr + offset) as u32;

                // Write 4 bytes to memory (little-endian)
                for i in 0..4 {
                    self.state
                        .memory
                        .insert(addr + i, ((value >> (8 * i)) & 0xFF) as u8);
                }

                Ok(())
            }

            // Branch if equal (beq)
            Instruction::BranchEq(rs1, rs2, label) => {
                let val1 = self.get_reg_value(rs1)?;
                let val2 = self.get_reg_value(rs2)?;

                if val1 == val2 {
                    if let Some(target_pc) = self.label_to_pc(label) {
                        self.state.pc = target_pc;
                    } else {
                        return Err(DebuggerError::Execution(format!(
                            "Label not found: {}",
                            label
                        )));
                    }
                }

                Ok(())
            }

            // Branch if not equal (bne)
            Instruction::BranchNe(rs1, rs2, label) => {
                let val1 = self.get_reg_value(rs1)?;
                let val2 = self.get_reg_value(rs2)?;

                if val1 != val2 {
                    if let Some(target_pc) = self.label_to_pc(label) {
                        self.state.pc = target_pc;
                    } else {
                        return Err(DebuggerError::Execution(format!(
                            "Label not found: {}",
                            label
                        )));
                    }
                }

                Ok(())
            }

            // Branch if less than or equal (ble)
            Instruction::BranchLe(rs1, rs2, label) => {
                let val1 = self.get_reg_value(rs1)? as i32;
                let val2 = self.get_reg_value(rs2)? as i32;

                if val1 <= val2 {
                    if let Some(target_pc) = self.label_to_pc(label) {
                        self.state.pc = target_pc;
                    } else {
                        return Err(DebuggerError::Execution(format!(
                            "Label not found: {}",
                            label
                        )));
                    }
                }

                Ok(())
            }

            // All other instructions (arithmetic, logic, shifts, etc.)
            _ => {
                // For now, we just return Ok for unimplemented instructions
                Ok(())
            }
        }
    }

    /// Get register value as u32
    fn get_reg_value(
        &self,
        reg: &crate::compiler::codegen::risc_v::Register,
    ) -> Result<u32, DebuggerError> {
        let reg_name = reg.to_string();
        self.state
            .registers
            .get(&reg_name)
            .copied()
            .ok_or_else(|| DebuggerError::Execution(format!("Register {} not set", reg_name)))
    }

    /// Set register value
    fn set_reg_value(&mut self, reg: &crate::compiler::codegen::risc_v::Register, value: u32) {
        let reg_name = reg.to_string();
        self.state.registers.insert(reg_name, value);
    }

    /// Convert a label to a PC address
    fn label_to_pc(&self, label: &str) -> Option<usize> {
        // Search through instructions to find the label
        for (i, instruction) in self.instructions.iter().enumerate() {
            if let Instruction::Label(ref lbl) = instruction {
                if lbl == label {
                    return Some(i);
                }
            }
        }
        None
    }

    /// Convert a label to a function name
    fn label_to_function(&self, _label: &str) -> Option<String> {
        // In a real implementation, this would parse the label to extract the function name
        // For now, we just return None
        None
    }

    /// Emit a debugger event
    fn emit_event(&self, event: DebuggerEvent) {
        if let Some(ref handler) = self.event_handler {
            handler(event);
        }
    }
}
