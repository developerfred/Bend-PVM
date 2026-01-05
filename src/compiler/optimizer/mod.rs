//! Optimization Passes for the Bend-PVM compiler
//!
//! This module provides various compiler optimization passes that improve
//! the quality and performance of generated RISC-V code.

pub mod constant_folding;

pub use crate::compiler::codegen::Instruction;
pub use crate::compiler::parser::ast::{BinaryOperator, Expr, Location};

/// Optimization pass manager
pub struct OptimizationManager {
    /// List of optimization passes to apply
    pub passes: Vec<Box<dyn OptimizationPass>>,
}

impl OptimizationManager {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
        }
    }

    pub fn add_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        self.passes.push(pass);
    }

    /// Apply all optimization passes to a program
    pub fn optimize(&mut self, program: &crate::compiler::parser::ast::Program) -> Result<crate::compiler::parser::ast::Program, String> {
        let mut optimized_program = program.clone();

        for definition in &mut optimized_program.definitions {
            match definition {
                crate::compiler::parser::ast::Definition::FunctionDef { body, .. } => {
                    // Apply optimizations to function body
                    let optimized_body = self.optimize_block(body)?;
                    optimized_program.definitions.push(crate::compiler::parser::ast::Definition::FunctionDef {
                        name: definition.name().clone(),
                        type_params: definition.type_params().clone(),
                        parameters: definition.parameters().clone(),
                        return_type: definition.return_type().clone(),
                        body: optimized_body,
                        location: definition.location().clone(),
                        attributes: definition.attributes().clone(),
                    });
                }
                _ => {
                    // For now, keep other definitions unchanged
                    optimized_program.definitions.push(definition.clone());
                }
            }
        }

        Ok(optimized_program)
    }

    fn optimize_block(&mut self, block: &crate::compiler::parser::ast::Block) -> Result<crate::compiler::parser::ast::Block, String> {
        let mut optimized_statements = Vec::new();

        for statement in &block.statements {
            let optimized_stmt = self.optimize_statement(statement)?;
            optimized_statements.push(optimized_stmt);
        }

        Ok(crate::compiler::parser::ast::Block {
            statements: optimized_statements,
            location: block.location().clone(),
        })
    }

    fn optimize_statement(&mut self, stmt: &crate::compiler::parser::ast::Statement) -> Result<crate::compiler::parser::ast::Statement, String> {
        match stmt {
            crate::compiler::parser::ast::Statement::Use { value, .. } => {
                // TODO: Apply optimizations
                Ok(stmt.clone())
            }
            _ => Ok(stmt.clone()),
        }
    }
}

/// Trait for optimization passes
pub trait OptimizationPass {
    fn name(&self) -> &str;
    fn apply(&mut self, program: &crate::compiler::parser::ast::Program) -> Result<crate::compiler::parser::ast::Program, String>;
}
