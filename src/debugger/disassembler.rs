use crate::compiler::codegen::risc_v::Instruction;
use crate::debugger::DebugInfo;

/// Disassembler for RISC-V instructions
pub struct Disassembler {
    /// Debug information
    debug_info: DebugInfo,

    /// Instructions
    instructions: Vec<Instruction>,
}

impl Disassembler {
    /// Create a new disassembler
    pub fn new(debug_info: DebugInfo, instructions: Vec<Instruction>) -> Self {
        Disassembler {
            debug_info,
            instructions,
        }
    }

    /// Disassemble a single instruction
    pub fn disassemble_instruction(&self, index: usize) -> Option<DisassembledInstruction> {
        if index >= self.instructions.len() {
            return None;
        }

        let instruction = &self.instructions[index];
        let source_line = self.debug_info.instruction_to_line.get(&index).cloned();

        Some(DisassembledInstruction {
            index,
            instruction: instruction.clone(),
            source_line,
            address: index * 4, // Assuming 4 bytes per instruction
        })
    }

    /// Disassemble a range of instructions
    pub fn disassemble_range(&self, start: usize, end: usize) -> Vec<DisassembledInstruction> {
        let mut result = Vec::new();

        for i in start..end {
            if let Some(disassembled) = self.disassemble_instruction(i) {
                result.push(disassembled);
            }
        }

        result
    }

    /// Disassemble a function
    pub fn disassemble_function(
        &self,
        function_name: &str,
    ) -> Option<Vec<DisassembledInstruction>> {
        let function_range = self.debug_info.functions.get(function_name)?;

        Some(self.disassemble_range(function_range.start, function_range.end))
    }

    /// Disassemble instructions around the current position
    pub fn disassemble_context(
        &self,
        pc: usize,
        context_count: usize,
    ) -> Vec<DisassembledInstruction> {
        let start = if pc > context_count {
            pc - context_count
        } else {
            0
        };

        let end = if pc + context_count < self.instructions.len() {
            pc + context_count + 1
        } else {
            self.instructions.len()
        };

        self.disassemble_range(start, end)
    }

    /// Get source line for an instruction
    pub fn source_line_for_instruction(&self, index: usize) -> Option<usize> {
        self.debug_info.instruction_to_line.get(&index).cloned()
    }

    /// Get instructions for a source line
    pub fn instructions_for_source_line(&self, line: usize) -> Option<Vec<usize>> {
        self.debug_info.line_to_instruction.get(&line).cloned()
    }

    /// Get function for an instruction
    pub fn function_for_instruction(&self, index: usize) -> Option<String> {
        for (name, range) in &self.debug_info.functions {
            if index >= range.start && index < range.end {
                return Some(name.clone());
            }
        }

        None
    }
}

/// Disassembled instruction
#[derive(Debug, Clone)]
pub struct DisassembledInstruction {
    /// Instruction index
    pub index: usize,

    /// Instruction
    pub instruction: Instruction,

    /// Source line
    pub source_line: Option<usize>,

    /// Instruction address
    pub address: usize,
}

impl DisassembledInstruction {
    /// Get a human-readable representation of the instruction
    pub fn to_string(&self) -> String {
        format!("{:08x}: {}", self.address, self.instruction)
    }

    /// Get a human-readable representation with source line
    pub fn to_string_with_source(&self) -> String {
        if let Some(line) = self.source_line {
            format!(
                "{:08x}: {} // line {}",
                self.address, self.instruction, line
            )
        } else {
            format!("{:08x}: {}", self.address, self.instruction)
        }
    }
}
