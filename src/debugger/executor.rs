//! RISC-V instruction executor
//!
//! This module implements a real executor that can execute RISC-V instructions
//! step by step with proper state management.

use crate::debugger::state::{DebuggerState, ExecutionState};
use crate::compiler::codegen::risc_v::Instruction;
use crate::compiler::lexer::lexer::BendLexer;
use thiserror::Error;

/// Result of instruction execution
pub type ExecutionResult = Result<(), ExecutionError>;

/// Instruction executor
pub struct InstructionExecutor {
    state: DebuggerState,
}

impl InstructionExecutor {
    /// Create a new instruction executor
    pub fn new() -> Self {
        Self {
            state: DebuggerState::new(),
        }
    }

    /// Execute a single instruction
    pub fn execute_instruction(&mut self, instruction: &Instruction) -> ExecutionResult {
        match instruction {
            // Arithmetic operations
            Instruction::Add { rd, rs1, rs2, funct } => {
                let result = self.execute_add(rd, rs1, rs2);
                self.advance_pc();
                result
            }

            Instruction::Sub { rd, rs1, rs2, funct } => {
                let result = self.execute_sub(rd, rs1, rs2);
                self.advance_pc();
                result
            }

            Instruction::And { rd, rs1, rs2, funct } => {
                let result = self.execute_and(rd, rs1, rs2);
                self.advance_pc();
                result
            }

            Instruction::Or { rd, rs1, rs2, funct } => {
                let result = self.execute_or(rd, rs1, rs2);
                self.advance_pc();
                result
            }

            Instruction::Xor { rd, rs1, rs2, funct } => {
                let result = self.execute_xor(rd, rs1, rs2);
                self.advance_pc();
                result
            }

            Instruction::Sll { rd, rs1, shamt, funct } => {
                let result = self.execute_sll(rd, rs1, shamt);
                self.advance_pc();
                result
            }

            Instruction::Srl { rd, rs1, shamt, funct } => {
                let result = self.execute_srl(rd, rs1, shamt);
                self.advance_pc();
                result
            }

            Instruction::Slli { rd, rs1, shamt, funct } => {
                let result = self.execute_slli(rd, rs1, shamt);
                self.advance_pc();
                result
            }

            Instruction::Srli { rd, rs1, shamt, funct } => {
                let result = self.execute_srli(rd, rs1, shamt);
                self.advance_pc();
                result
            }

            Instruction::Lui { rd, imm, funct } => {
                let result = self.execute_lui(rd, imm);
                self.advance_pc();
                result
            }

            Instruction::Auipc { rd, imm, funct } => {
                let result = self.execute_auipc(rd, imm);
                self.advance_pc();
                result
            }

            // Load/store operations
            Instruction::Lw { rd, offset, funct } => {
                return Err(ExecutionError::MemoryAccess(
                    "Memory load not implemented yet".to_string(),
                ));
            }

            Instruction::Sw { rs1, rs2, offset, funct } => {
                return Err(ExecutionError::MemoryAccess(
                    "Memory store not implemented yet".to_string(),
                ));
            }

            Instruction::Beq { rs1, rs2, offset, funct } => {
                return Err(ExecutionError::MemoryAccess(
                    "Memory branch not implemented yet".to_string(),
                ));
            }

            Instruction::Bne { rs1, rs2, offset, funct } => {
                return Err(ExecutionError::MemoryAccess(
                    "Memory branch not implemented yet".to_string(),
                ));
            }

            // Memory operations (stubs - return errors for now)
            Instruction::Lw { .. }
            Instruction::Sw { .. }
            Instruction::Beq { .. }
            Instruction::Bne { .. }

            Instruction::Lui { rd, imm, funct } => {
                let result = self.execute_lui(rd, imm);
                self.advance_pc();
                result
            }

            Instruction::Auipc { rd, imm, funct } => {
                let result = self.execute_auipc(rd, imm);
                self.advance_pc();
                result
            }

            _ => {
                self.advance_pc();
            }
        }
    }

    // Helper methods for arithmetic operations
    fn execute_add(&mut self, rd: u8, rs1: u8, rs2: u8) -> ExecutionResult {
        let val1 = self.state.get_register(rs1).unwrap_or(0);
        let val2 = self.state.get_register(rs2).unwrap_or(0);

        // Handle wrapping
        let (result, overflow) = val1.wrapping_add(val2);
        if overflow {
            self.state.execution_state = ExecutionState::Crashed(
                "Integer overflow in add".to_string(),
            );
            return Err(ExecutionError::DivisionByZero);
        }

        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_sub(&mut self, rd: u8, rs1: u8, rs2: u8) -> ExecutionResult {
        let val1 = self.state.get_register(rs1).unwrap_or(0);
        let val2 = self.state.get_register(rs2).unwrap_or(0);

        // Handle wrapping
        let (result, underflow) = val1.wrapping_sub(val2);
        if underflow {
            self.state.execution_state = ExecutionState::Crashed(
                "Integer underflow in sub".to_string(),
            );
            return Err(ExecutionError::DivisionByZero);
        }

        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_and(&mut self, rd: u8, rs1: u8, rs2: u8) -> ExecutionResult {
        let val1 = self.state.get_register(rs1).unwrap_or(0);
        let val2 = self.state.get_register(rs2).unwrap_or(0);

        let result = val1 & val2;
        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_or(&mut self, rd: u8, rs1: u8, rs2: u8) -> ExecutionResult {
        let val1 = self.state.get_register(rs1).unwrap_or(0);
        let val2 = self.state.get_register(rs2).unwrap_or(0);

        let result = val1 | val2;
        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_xor(&mut self, rd: u8, rs1: u8, rs2: u8) -> ExecutionResult {
        let val1 = self.state.get_register(rs1).unwrap_or(0);
        let val2 = self.state.get_register(rs2).unwrap_or(0);

        let result = val1 ^ val2;
        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_sll(&mut self, rd: u8, rs1: u8, shamt: u8) -> ExecutionResult {
        let val = self.state.get_register(rs1).unwrap_or(0);
        let result = val.wrapping_shl(shamt);
        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_srl(&mut self, rd: u8, rs1: u8, shamt: u8) -> ExecutionResult {
        let val = self.state.get_register(rs1).unwrap_or(0);
        let result = val.wrapping_shr(shamt);
        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_slli(&mut self, rd: u8, rs1: u8, shamt: u8) -> ExecutionResult {
        let val = self.state.get_register(rs1).unwrap_or(0);
        let result = val.wrapping_shl(shamt);
        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_srli(&mut self, rd: u8, rs1: u8, shamt: u8) -> ExecutionResult {
        let val = self.state.get_register(rs1).unwrap_or(0);
        let result = val.wrapping_shr(shamt);
        self.state.set_register(rd, result);
        Ok(())
    }

    fn execute_lui(&mut self, rd: u8, imm: u32) -> ExecutionResult {
        let imm = imm as i32;
        self.state.set_register(rd, imm);
        Ok(())
    }

    fn execute_auipc(&mut self, rd: u8, imm: u32) -> ExecutionResult {
        let current_pc = self.state.pc as i32;
        let imm = imm as i32;
        let new_pc = (current_pc.wrapping_add(imm) as usize) % (1 << 22); // 4MB PC space
        self.state.pc = new_pc;
        Ok(())
    }

    /// Advance to next instruction
    fn advance_pc(&mut self) {
        self.state.pc += 1;
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_add_operation() {
            let mut executor = InstructionExecutor::new();

            // Set up some values
            executor.state.set_register(1, 42);
            executor.state.set_register(2, 17);

            // Test add: 42 + 17 should be 59
            assert!(executor.execute_instruction(&Instruction::Add {
                rd: 0, rs1: 1, rs2: 2, funct: 0,
            }).is_ok());

            let result = executor.state.get_register(0);
            assert_eq!(result, Some(59));
        }

        #[test]
        fn test_sub_operation() {
            let mut executor = InstructionExecutor::new();

            executor.state.set_register(1, 50);
            executor.state.set_register(2, 30);

            // Test sub: 50 - 30 should be 20
            assert!(executor.execute_instruction(&Instruction::Sub {
                rd: 0, rs1: 1, rs2: 2, funct: 0,
            }).is_ok());

            let result = executor.state.get_register(0);
            assert_eq!(result, Some(20));
        }

        #[test]
        fn test_xor_operation() {
            let mut executor = InstructionExecutor::new();

            executor.state.set_register(1, 0b10101010);
            executor.state.set_register(2, 0b11110010);

            // Test xor: 0b10101010 ^ 0b11110010 = 0b01011000
            assert!(executor.execute_instruction(&Instruction::Xor {
                rd: 0, rs1: 1, rs2: 2, funct: 0,
            }).is_ok());

            let result = executor.state.get_register(0);
            assert_eq!(result, Some(0b01011000));
        }

        #[test]
        fn test_shift_operations() {
            let mut executor = InstructionExecutor::new();

            executor.state.set_register(1, 1);

            // Test SLL: 1 << 2 = 4
            assert!(executor.execute_instruction(&Instruction::Sll {
                rd: 0, rs1: 1, shamt: 2, funct: 0,
            }).is_ok());

            let result = executor.state.get_register(0);
            assert_eq!(result, Some(4));
        }

        #[test]
        fn test_lui_operation() {
            let mut executor = InstructionExecutor::new();

            // Test LUI with immediate value 12345
            assert!(executor.execute_instruction(&Instruction::Lui {
                rd: 0, imm: 12345, funct: 0,
            }).is_ok());

            let result = executor.state.get_register(0);
            assert_eq!(result, Some(12345));
        }

        #[test]
        fn test_auipc_operation() {
            let mut executor = InstructionExecutor::new();

            executor.state.set_register(1, 100);

            // Test AUIPC: PC should advance by immediate
            assert!(executor.execute_instruction(&Instruction::Auipc {
                rd: 0, imm: 200, funct: 0,
            }).is_ok());

            let pc = executor.state.pc;
            // PC should wrap around at 4MB (422 bytes)
            assert_eq!(pc % (1 << 22), 200);
        }
    }
}
