#![allow(clippy::only_used_in_recursion)]

use std::collections::HashMap;
use std::fmt::Display;
use thiserror::Error;

use crate::compiler::parser::ast::*;

#[derive(Error, Debug, Clone)]
pub enum CodegenError {
    #[error("Codegen error: {0}")]
    Generic(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),
}

/// RISC-V register allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Register {
    X0, // Zero register
    X1, // Return address
    X2, // Stack pointer
    X3, // Global pointer
    X4, // Thread pointer
    // Temporary registers
    X5,
    X6,
    X7,
    // Saved registers
    X8,
    X9,
    X10,
    X11,
    X12,
    X13,
    X14,
    X15,
    // Argument/return registers
    X16,
    X17,
    X18,
    X19,
    X20,
    X21,
    X22,
    X23,
    X24,
    X25,
    // Temporary registers
    X26,
    X27,
    X28,
    X29,
    X30,
    X31,
}

impl Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Register::X0 => "zero",
            Register::X1 => "ra",
            Register::X2 => "sp",
            Register::X3 => "gp",
            Register::X4 => "tp",
            Register::X5 => "t0",
            Register::X6 => "t1",
            Register::X7 => "t2",
            Register::X8 => "s0",
            Register::X9 => "s1",
            Register::X10 => "a0",
            Register::X11 => "a1",
            Register::X12 => "a2",
            Register::X13 => "a3",
            Register::X14 => "a4",
            Register::X15 => "a5",
            Register::X16 => "a6",
            Register::X17 => "a7",
            Register::X18 => "s2",
            Register::X19 => "s3",
            Register::X20 => "s4",
            Register::X21 => "s5",
            Register::X22 => "s6",
            Register::X23 => "s7",
            Register::X24 => "s8",
            Register::X25 => "s9",
            Register::X26 => "s10",
            Register::X27 => "s11",
            Register::X28 => "t3",
            Register::X29 => "t4",
            Register::X30 => "t5",
            Register::X31 => "t6",
        };
        write!(f, "{}", name)
    }
}

impl Register {
    pub fn arg_registers() -> Vec<Register> {
        vec![
            Register::X10,
            Register::X11,
            Register::X12,
            Register::X13,
            Register::X14,
            Register::X15,
            Register::X16,
            Register::X17,
        ]
    }

    pub fn temp_registers() -> Vec<Register> {
        vec![
            Register::X5,
            Register::X6,
            Register::X7,
            Register::X28,
            Register::X29,
            Register::X30,
            Register::X31,
        ]
    }

    pub fn saved_registers() -> Vec<Register> {
        vec![
            Register::X8,
            Register::X9,
            Register::X18,
            Register::X19,
            Register::X20,
            Register::X21,
            Register::X22,
            Register::X23,
            Register::X24,
            Register::X25,
            Register::X26,
            Register::X27,
        ]
    }
}

/// RISC-V instructions
#[derive(Debug, Clone)]
pub enum Instruction {
    // Load and store
    Load(Register, Register, i32), // Load from memory, e.g., lw rd, offset(rs1)
    Store(Register, Register, i32), // Store to memory, e.g., sw rs2, offset(rs1)

    // Arithmetic
    Add(Register, Register, Register), // Add, e.g., add rd, rs1, rs2
    AddImm(Register, Register, i32),   // Add immediate, e.g., addi rd, rs1, imm
    Sub(Register, Register, Register), // Subtract, e.g., sub rd, rs1, rs2
    Mul(Register, Register, Register), // Multiply, e.g., mul rd, rs1, rs2
    Div(Register, Register, Register), // Divide, e.g., div rd, rs1, rs2
    Rem(Register, Register, Register), // Remainder, e.g., rem rd, rs1, rs2

    // Logical
    And(Register, Register, Register), // AND, e.g., and rd, rs1, rs2
    Or(Register, Register, Register),  // OR, e.g., or rd, rs1, rs2
    Xor(Register, Register, Register), // XOR, e.g., xor rd, rs1, rs2
    AndImm(Register, Register, i32),   // AND immediate, e.g., andi rd, rs1, imm
    OrImm(Register, Register, i32),    // OR immediate, e.g., ori rd, rs1, imm
    XorImm(Register, Register, i32),   // XOR immediate, e.g., xori rd, rs1, imm

    // Shifts
    ShiftLeft(Register, Register, Register), // Shift left, e.g., sll rd, rs1, rs2
    ShiftRight(Register, Register, Register), // Shift right, e.g., srl rd, rs1, rs2
    ShiftRightArith(Register, Register, Register), // Arithmetic shift right, e.g., sra rd, rs1, rs2
    ShiftLeftImm(Register, Register, i32),   // Shift left immediate, e.g., slli rd, rs1, imm
    ShiftRightImm(Register, Register, i32),  // Shift right immediate, e.g., srli rd, rs1, imm
    ShiftRightArithImm(Register, Register, i32), // Arithmetic shift right immediate, e.g., srai rd, rs1, imm

    // Comparison
    SetLessThan(Register, Register, Register), // Set if less than, e.g., slt rd, rs1, rs2
    SetLessThanU(Register, Register, Register), // Set if less than (unsigned), e.g., sltu rd, rs1, rs2
    SetLessThanImm(Register, Register, i32), // Set if less than immediate, e.g., slti rd, rs1, imm
    SetLessThanImmU(Register, Register, i32), // Set if less than immediate (unsigned), e.g., sltiu rd, rs1, imm

    // Branches
    BranchEq(Register, Register, String), // Branch if equal, e.g., beq rs1, rs2, label
    BranchNe(Register, Register, String), // Branch if not equal, e.g., bne rs1, rs2, label
    BranchLt(Register, Register, String), // Branch if less than, e.g., blt rs1, rs2, label
    BranchLe(Register, Register, String), // Branch if less than or equal, e.g., ble rs1, rs2, label
    BranchGe(Register, Register, String), // Branch if greater than or equal, e.g., bge rs1, rs2, label
    BranchLtU(Register, Register, String), // Branch if less than (unsigned), e.g., bltu rs1, rs2, label
    BranchGeU(Register, Register, String), // Branch if greater than or equal (unsigned), e.g., bgeu rs1, rs2, label

    // Jumps
    Jump(String),                            // Jump, e.g., j label
    JumpAndLink(Register, String),           // Jump and link, e.g., jal rd, label
    JumpAndLinkReg(Register, Register, i32), // Jump and link register, e.g., jalr rd, rs1, offset

    // System
    Ecall,  // Environment call
    Ebreak, // Environment break

    // Pseudo-instructions
    Li(Register, i32),       // Load immediate, e.g., li rd, imm
    La(Register, String),    // Load address, e.g., la rd, symbol
    Mv(Register, Register),  // Move, e.g., mv rd, rs1
    Not(Register, Register), // NOT, e.g., not rd, rs1
    Neg(Register, Register), // Negate, e.g., neg rd, rs1

    // Label definition (not an instruction, but useful for assembly generation)
    Label(String),

    // Comment (not an instruction, but useful for debugging)
    Comment(String),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Load(rd, rs1, offset) => {
                write!(
                    f,
                    "    lw {}, {}({})",
                    rd,
                    offset,
                    rs1
                )
            }
            Instruction::Store(rs2, rs1, offset) => {
                write!(
                    f,
                    "    sw {}, {}({})",
                    rs2,
                    offset,
                    rs1
                )
            }
            Instruction::Add(rd, rs1, rs2) => {
                write!(
                    f,
                    "    add {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::AddImm(rd, rs1, imm) => {
                write!(
                    f,
                    "    addi {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::Sub(rd, rs1, rs2) => {
                write!(
                    f,
                    "    sub {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::Mul(rd, rs1, rs2) => {
                write!(
                    f,
                    "    mul {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::Div(rd, rs1, rs2) => {
                write!(
                    f,
                    "    div {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::Rem(rd, rs1, rs2) => {
                write!(
                    f,
                    "    rem {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::And(rd, rs1, rs2) => {
                write!(
                    f,
                    "    and {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::Or(rd, rs1, rs2) => {
                write!(
                    f,
                    "    or {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::Xor(rd, rs1, rs2) => {
                write!(
                    f,
                    "    xor {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::AndImm(rd, rs1, imm) => {
                write!(
                    f,
                    "    andi {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::OrImm(rd, rs1, imm) => {
                write!(
                    f,
                    "    ori {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::XorImm(rd, rs1, imm) => {
                write!(
                    f,
                    "    xori {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::ShiftLeft(rd, rs1, rs2) => {
                write!(
                    f,
                    "    sll {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::ShiftRight(rd, rs1, rs2) => {
                write!(
                    f,
                    "    srl {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::ShiftRightArith(rd, rs1, rs2) => {
                write!(
                    f,
                    "    sra {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::ShiftLeftImm(rd, rs1, imm) => {
                write!(
                    f,
                    "    slli {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::ShiftRightImm(rd, rs1, imm) => {
                write!(
                    f,
                    "    srli {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::ShiftRightArithImm(rd, rs1, imm) => {
                write!(
                    f,
                    "    srai {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::SetLessThan(rd, rs1, rs2) => {
                write!(
                    f,
                    "    slt {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::SetLessThanU(rd, rs1, rs2) => {
                write!(
                    f,
                    "    sltu {}, {}, {}",
                    rd,
                    rs1,
                    rs2
                )
            }
            Instruction::SetLessThanImm(rd, rs1, imm) => {
                write!(
                    f,
                    "    slti {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::SetLessThanImmU(rd, rs1, imm) => {
                write!(
                    f,
                    "    sltiu {}, {}, {}",
                    rd,
                    rs1,
                    imm
                )
            }
            Instruction::BranchEq(rs1, rs2, label) => {
                write!(
                    f,
                    "    beq {}, {}, {}",
                    rs1,
                    rs2,
                    label
                )
            }
            Instruction::BranchNe(rs1, rs2, label) => {
                write!(
                    f,
                    "    bne {}, {}, {}",
                    rs1,
                    rs2,
                    label
                )
            }
            Instruction::BranchLt(rs1, rs2, label) => {
                write!(
                    f,
                    "    blt {}, {}, {}",
                    rs1,
                    rs2,
                    label
                )
            }
            Instruction::BranchLe(rs1, rs2, label) => {
                write!(
                    f,
                    "    ble {}, {}, {}",
                    rs1,
                    rs2,
                    label
                )
            }
            Instruction::BranchGe(rs1, rs2, label) => {
                write!(
                    f,
                    "    bge {}, {}, {}",
                    rs1,
                    rs2,
                    label
                )
            }
            Instruction::BranchLtU(rs1, rs2, label) => {
                write!(
                    f,
                    "    bltu {}, {}, {}",
                    rs1,
                    rs2,
                    label
                )
            }
            Instruction::BranchGeU(rs1, rs2, label) => {
                write!(
                    f,
                    "    bgeu {}, {}, {}",
                    rs1,
                    rs2,
                    label
                )
            }
            Instruction::Jump(label) => {
                write!(f, "    j {}", label)
            }
            Instruction::JumpAndLink(rd, label) => {
                write!(f, "    jal {}, {}", rd, label)
            }
            Instruction::JumpAndLinkReg(rd, rs1, offset) => {
                write!(
                    f,
                    "    jalr {}, {}, {}",
                    rd,
                    rs1,
                    offset
                )
            }
            Instruction::Ecall => {
                write!(f, "    ecall")
            }
            Instruction::Ebreak => {
                write!(f, "    ebreak")
            }
            Instruction::Li(rd, imm) => {
                write!(f, "    li {}, {}", rd, imm)
            }
            Instruction::La(rd, symbol) => {
                write!(f, "    la {}, {}", rd, symbol)
            }
            Instruction::Mv(rd, rs1) => {
                write!(f, "    mv {}, {}", rd, rs1)
            }
            Instruction::Not(rd, rs1) => {
                write!(f, "    not {}, {}", rd, rs1)
            }
            Instruction::Neg(rd, rs1) => {
                write!(f, "    neg {}, {}", rd, rs1)
            }
            Instruction::Label(label) => {
                write!(f, "{}:", label)
            }
            Instruction::Comment(comment) => {
                write!(f, "    # {}", comment)
            }
        }
    }
}

/// Code generator for RISC-V assembly
pub struct RiscVCodegen {
    /// Instructions generated
    instructions: Vec<Instruction>,

    /// Local variable mapping to stack offsets
    locals: HashMap<String, i32>,

    /// Current stack frame size
    frame_size: i32,

    /// Next available label ID for generating unique labels
    next_label_id: u32,

    /// Labels for function entry points
    function_labels: HashMap<String, String>,

    /// Current offset for next local variable
    current_local_offset: i32,
}

impl Default for RiscVCodegen {
    fn default() -> Self {
        Self::new()
    }
}

impl RiscVCodegen {
    pub fn new() -> Self {
        RiscVCodegen {
            instructions: Vec::new(),
            locals: HashMap::new(),
            frame_size: 0,
            next_label_id: 0,
            function_labels: HashMap::new(),
            current_local_offset: 0,
        }
    }

    /// Generate code for a program
    pub fn generate(&mut self, program: &Program) -> Result<Vec<Instruction>, CodegenError> {
        // Generate function labels
        for definition in &program.definitions {
            if let Definition::FunctionDef { name, .. } = definition {
                let label = self.generate_function_label(name);
                self.function_labels.insert(name.clone(), label);
            }
        }

        // Generate code for each function
        for definition in &program.definitions {
            if let Definition::FunctionDef {
                name, params, body, ..
            } = definition
            {
                self.generate_function(name, params, body)?;
            }
        }

        Ok(self.instructions.clone())
    }

    /// Generate a unique label
    fn generate_label(&mut self, prefix: &str) -> String {
        let label = format!("{}.{}", prefix, self.next_label_id);
        self.next_label_id += 1;
        label
    }

    /// Generate a function label
    fn generate_function_label(&mut self, name: &str) -> String {
        if name == "main" {
            "main".to_string()
        } else {
            format!("function.{}", name.replace('/', "_"))
        }
    }

    /// Helper to count local variables
    fn collect_locals(&self, block: &Block) -> usize {
        let mut count = 0;
        for stmt in &block.statements {
            match stmt {
                Statement::Use { .. } => count += 1,
                Statement::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    count += self.collect_locals(then_branch);
                    count += self.collect_locals(else_branch);
                }
                Statement::Match { cases, .. } => {
                    for case in cases {
                        count += self.collect_locals(&case.body);
                    }
                }
                Statement::Switch { cases, .. } => {
                    for case in cases {
                        count += self.collect_locals(&case.body);
                    }
                }
                Statement::Fold { cases, .. } => {
                    for case in cases {
                        count += self.collect_locals(&case.body);
                    }
                }
                Statement::Bend {
                    body, else_body, ..
                } => {
                    count += self.collect_locals(body);
                    if let Some(else_b) = else_body {
                        count += self.collect_locals(else_b);
                    }
                }
                Statement::LocalDef { function_def, .. } => {
                    // If we support local functions, we might need to recurse if they capture?
                    // For now, ignore.
                    if let Definition::FunctionDef { .. } = function_def.as_ref() {
                        // But local functions are separate frames?
                        // Yes, usually. So don't count their locals here.
                    }
                }
                Statement::With { body, .. } => {
                    count += self.collect_locals(body);
                }
                _ => {}
            }
        }
        count
    }

    /// Generate code for a function
    fn generate_function(
        &mut self,
        name: &str,
        params: &[Parameter],
        body: &Block,
    ) -> Result<(), CodegenError> {
        // Reset local variables and frame size
        self.locals.clear();
        self.current_local_offset = 0;

        // Calculate frame size
        // Locals: collect_locals(body) * 4
        // Params: passed by caller, but we map them to stack offsets if needed?
        // Wait, current logic maps params to [8, 12, ...].
        // The implementation assumes params are ALREADY on the stack above the return address.
        // Or it assumes we need to spill them?
        // "Allocate space for local variables and function arguments" - the comment said.
        // But usually arguments are in registers a0-a7.
        // The previous code seemed to assume we put them on stack?
        // But it didn't generate store instructions for them!
        // It just inserted them into self.locals!
        // This implies the compiler expects params to BE at those offsets.
        // This is only true if the CALLER put them there.
        // Or if we spill them in prologue.
        // Since `generate_function` prologue didn't spill a0-a7,
        // AND `Expr::Variable` loads from `offset(SP)`,
        // The only logical conclusion is that the calling convention used by this simple compiler
        // expects arguments to be passed on the stack.

        // Let's preserve this behavior.
        // Arguments are at `CallerSP + 0`, `CallerSP + 4`...
        // `SP = CallerSP - 8` (in original). So `args at SP + 8`.

        let locals_count = self.collect_locals(body);
        let locals_size = (locals_count * 4) as i32;
        let total_frame_size = locals_size + 8; // RA + alignment/padding + locals

        // Function label
        let function_label = self.function_labels.get(name).unwrap().clone();
        self.instructions.push(Instruction::Label(function_label));

        // Function prologue: save return address
        self.instructions.push(Instruction::Comment(format!(
            "Function prologue for {}",
            name
        )));

        // Allocate stack frame
        self.instructions.push(Instruction::AddImm(
            Register::X2,
            Register::X2,
            -total_frame_size,
        ));

        // Save RA at `locals_size` (just below caller args)
        self.instructions
            .push(Instruction::Store(Register::X1, Register::X2, locals_size));

        // Map params (Caller args start at `total_frame_size + 8` relative to new SP??)
        // Original: `AddImm -8`. Args at `8`.
        // `CallerSP = NewSP + 8`.
        // Args at `CallerSP + 0` = `NewSP + 8`.

        // New: `AddImm -total`.
        // `CallerSP = NewSP + total`.
        // Args at `CallerSP + 0` = `NewSP + total`.

        // Actually, if original args were at `0(CallerSP)`, `4(CallerSP)`...
        // Then yes, offset is `total_frame_size`.

        // But wait, the original code used `offset = 8`.
        // `frame_size` (which was `offset`) = 8 + params*4.

        // If I change the stack layout, I must ensure `Expr::Variable` uses correct offsets.

        let mut offset = total_frame_size;
        for param in params {
            self.locals.insert(param.name.clone(), offset);
            offset += 4;
        }

        self.frame_size = total_frame_size; // Maybe unused, but keep it correct

        // Generate code for the function body
        self.generate_block(body)?;

        // Function epilogue: restore return address and return
        self.instructions.push(Instruction::Comment(format!(
            "Function epilogue for {}",
            name
        )));
        self.instructions
            .push(Instruction::Load(Register::X1, Register::X2, locals_size)); // Restore return address
        self.instructions.push(Instruction::AddImm(
            Register::X2,
            Register::X2,
            total_frame_size,
        ));

        // Return from function
        self.instructions
            .push(Instruction::JumpAndLinkReg(Register::X0, Register::X1, 0)); // Return

        Ok(())
    }

    /// Generate code for a block
    fn generate_block(&mut self, block: &Block) -> Result<Register, CodegenError> {
        let mut result_reg = Register::X0;

        for statement in &block.statements {
            result_reg = self.generate_statement(statement)?;
        }

        Ok(result_reg)
    }

    /// Generate code for a statement
    fn generate_statement(&mut self, statement: &Statement) -> Result<Register, CodegenError> {
        match statement {
            Statement::Return { value, .. } => {
                let result_reg = self.generate_expr(value)?;
                self.instructions
                    .push(Instruction::Mv(Register::X10, result_reg)); // Move result to a0 (return value)
                Ok(Register::X10)
            }
            Statement::Assignment { pattern, value, .. } => {
                let value_reg = self.generate_expr(value)?;
                self.generate_assignment(pattern, value_reg)?;
                Ok(value_reg)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let condition_reg = self.generate_expr(condition)?;

                let then_label = self.generate_label("if_then");
                let else_label = self.generate_label("if_else");
                let end_label = self.generate_label("if_end");

                // Branch to then_label if condition is true (non-zero)
                self.instructions.push(Instruction::BranchNe(
                    condition_reg,
                    Register::X0,
                    then_label.clone(),
                ));

                // Else branch
                self.instructions
                    .push(Instruction::Jump(else_label.clone()));

                // Then branch
                self.instructions.push(Instruction::Label(then_label));
                let then_result = self.generate_block(then_branch)?;
                self.instructions.push(Instruction::Jump(end_label.clone()));

                // Else branch
                self.instructions.push(Instruction::Label(else_label));
                let _else_result = self.generate_block(else_branch)?;

                // End of if
                self.instructions.push(Instruction::Label(end_label));

                // Result of the if statement is in then_result or else_result (depending on the branch taken)
                // In a real compiler, we would need to merge the results
                Ok(then_result)
            }
            Statement::Use { name, value, .. } => {
                let val_reg = self.generate_expr(value)?;

                // Get pre-assigned offset
                let offset = self.current_local_offset;
                self.current_local_offset += 4;

                // Store to stack
                self.instructions
                    .push(Instruction::Store(val_reg, Register::X2, offset));

                // Register in locals map
                self.locals.insert(name.clone(), offset);

                Ok(val_reg)
            }
            Statement::Expr { expr, .. } => self.generate_expr(expr),
            // For brevity, not implementing all statement types
            _ => Err(CodegenError::UnsupportedFeature(
                "Statement type not yet implemented".to_string(),
            )),
        }
    }

    /// Generate code for an expression
    fn generate_expr(&mut self, expr: &Expr) -> Result<Register, CodegenError> {
        match expr {
            Expr::Variable { name, .. } => {
                // Load variable from stack frame or global storage
                if let Some(&offset) = self.locals.get(name) {
                    let reg = Register::X5; // Temporary register
                    self.instructions
                        .push(Instruction::Load(reg, Register::X2, offset));
                    Ok(reg)
                } else if let Some(function_label) = self.function_labels.get(name) {
                    // Function pointer
                    let reg = Register::X5; // Temporary register
                    self.instructions
                        .push(Instruction::La(reg, function_label.clone()));
                    Ok(reg)
                } else {
                    Err(CodegenError::UndefinedVariable(name.clone()))
                }
            }
            Expr::Literal { kind, .. } => {
                let reg = Register::X5; // Temporary register
                match kind {
                    LiteralKind::Uint(value) => {
                        if *value <= i32::MAX as u32 {
                            self.instructions.push(Instruction::Li(reg, *value as i32));
                            Ok(reg)
                        } else {
                            Err(CodegenError::InvalidOperation(format!(
                                "Literal value too large: {}",
                                value
                            )))
                        }
                    }
                    LiteralKind::Int(value) => {
                        self.instructions.push(Instruction::Li(reg, *value));
                        Ok(reg)
                    }
                    LiteralKind::Bool(value) => {
                        self.instructions
                            .push(Instruction::Li(reg, if *value { 1 } else { 0 }));
                        Ok(reg)
                    }
                    // For brevity, not implementing all literal types
                    _ => Err(CodegenError::UnsupportedFeature(
                        "Literal type not yet implemented".to_string(),
                    )),
                }
            }
            Expr::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                let left_reg = self.generate_expr(left)?;
                let right_reg = self.generate_expr(right)?;
                let result_reg = Register::X5; // Temporary register

                match operator {
                    BinaryOperator::Add => {
                        self.instructions
                            .push(Instruction::Add(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::Sub => {
                        self.instructions
                            .push(Instruction::Sub(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::Mul => {
                        self.instructions
                            .push(Instruction::Mul(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::Div => {
                        self.instructions
                            .push(Instruction::Div(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::Mod => {
                        self.instructions
                            .push(Instruction::Rem(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::BitAnd => {
                        self.instructions
                            .push(Instruction::And(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::BitOr => {
                        self.instructions
                            .push(Instruction::Or(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::BitXor => {
                        self.instructions
                            .push(Instruction::Xor(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::Equal => {
                        // x == y can be implemented as !(x - y)
                        self.instructions
                            .push(Instruction::Sub(result_reg, left_reg, right_reg));
                        self.instructions
                            .push(Instruction::SetLessThanImm(result_reg, result_reg, 1)); // 1 if x - y < 1 (i.e., x - y <= 0)
                        self.instructions.push(Instruction::SetLessThanImm(
                            Register::X6,
                            Register::X0,
                            1,
                        )); // 1 if 0 < 1 (always true)
                        self.instructions.push(Instruction::Xor(
                            result_reg,
                            result_reg,
                            Register::X6,
                        )); // Invert the result
                        Ok(result_reg)
                    }
                    BinaryOperator::NotEqual => {
                        // x != y can be implemented as (x - y) != 0
                        self.instructions
                            .push(Instruction::Sub(result_reg, left_reg, right_reg));
                        self.instructions.push(Instruction::SetLessThanImm(
                            result_reg,
                            Register::X0,
                            1,
                        )); // 1 if 0 < 1 (always true)
                        self.instructions
                            .push(Instruction::And(result_reg, result_reg, right_reg)); // 1 if x - y != 0
                        Ok(result_reg)
                    }
                    BinaryOperator::Less => {
                        self.instructions
                            .push(Instruction::SetLessThan(result_reg, left_reg, right_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::LessEqual => {
                        // x <= y can be implemented as !(y < x)
                        self.instructions
                            .push(Instruction::SetLessThan(result_reg, right_reg, left_reg));
                        self.instructions.push(Instruction::SetLessThanImm(
                            Register::X6,
                            Register::X0,
                            1,
                        )); // 1 if 0 < 1 (always true)
                        self.instructions.push(Instruction::Xor(
                            result_reg,
                            result_reg,
                            Register::X6,
                        )); // Invert the result
                        Ok(result_reg)
                    }
                    BinaryOperator::Greater => {
                        self.instructions
                            .push(Instruction::SetLessThan(result_reg, right_reg, left_reg));
                        Ok(result_reg)
                    }
                    BinaryOperator::GreaterEqual => {
                        // x >= y can be implemented as !(x < y)
                        self.instructions
                            .push(Instruction::SetLessThan(result_reg, left_reg, right_reg));
                        self.instructions.push(Instruction::SetLessThanImm(
                            Register::X6,
                            Register::X0,
                            1,
                        )); // 1 if 0 < 1 (always true)
                        self.instructions.push(Instruction::Xor(
                            result_reg,
                            result_reg,
                            Register::X6,
                        )); // Invert the result
                        Ok(result_reg)
                    }
                    // For brevity, not implementing all operators
                    _ => Err(CodegenError::UnsupportedFeature(
                        "Binary operator not yet implemented".to_string(),
                    )),
                }
            }
            Expr::FunctionCall { function, args, .. } => {
                // For simplicity, only handle direct function calls
                if let Expr::Variable { name, .. } = &**function {
                    let function_label = self.function_labels.get(name).cloned();
                    if let Some(function_label) = function_label {
                        // Load arguments into argument registers
                        let arg_registers = Register::arg_registers();
                        for (i, arg) in args.iter().enumerate() {
                            if i >= arg_registers.len() {
                                return Err(CodegenError::InvalidOperation("Too many arguments in function call".to_string()));
                            }

                            let arg_reg = self.generate_expr(arg)?;
                            self.instructions
                                .push(Instruction::Mv(arg_registers[i], arg_reg));
                        }

                        // Call the function
                        self.instructions.push(Instruction::JumpAndLink(
                            Register::X1,
                            function_label.clone(),
                        ));

                        // Result is in a0 (x10)
                        Ok(Register::X10)
                    } else {
                        Err(CodegenError::UndefinedVariable(name.clone()))
                    }
                } else {
                    Err(CodegenError::InvalidOperation("Function call with non-variable target".to_string()))
                }
            }
            // For brevity, not implementing all expression types
            _ => Err(CodegenError::UnsupportedFeature(
                "Expression type not yet implemented".to_string(),
            )),
        }
    }

    /// Generate code for an assignment
    fn generate_assignment(
        &mut self,
        pattern: &Pattern,
        value_reg: Register,
    ) -> Result<(), CodegenError> {
        match pattern {
            Pattern::Variable { name, .. } => {
                // Store value in local variable
                if let Some(&offset) = self.locals.get(name) {
                    self.instructions
                        .push(Instruction::Store(value_reg, Register::X2, offset));
                    Ok(())
                } else {
                    // Allocate a new local variable
                    self.frame_size += 4; // Assuming 4-byte (32-bit) values
                    let offset = self.frame_size;
                    self.locals.insert(name.clone(), offset);
                    self.instructions
                        .push(Instruction::Store(value_reg, Register::X2, offset));
                    Ok(())
                }
            }
            // For brevity, not implementing all pattern types
            _ => Err(CodegenError::UnsupportedFeature(
                "Pattern type not yet implemented".to_string(),
            )),
        }
    }
}
