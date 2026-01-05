// DEAD CODE ELIMINATION OPTIMIZATION - MINIMAL VERSION
// This is a simplified version that compiles with the current AST structure

use crate::compiler::optimizer::passes::{OptimizationError, OptimizationResult};
use crate::compiler::parser::ast::*;
use std::collections::HashSet;

/// Tree pruning optimization pass - Dead Code Elimination
pub struct PrunePass {
    /// Functions that are actually used
    used_functions: HashSet<String>,
}

impl Default for PrunePass {
    fn default() -> Self {
        Self::new()
    }
}

impl PrunePass {
    pub fn new() -> Self {
        PrunePass {
            used_functions: HashSet::new(),
        }
    }
}

impl crate::compiler::optimizer::passes::OptimizationPass for PrunePass {
    fn name(&self) -> &'static str {
        "prune"
    }

    fn description(&self) -> &'static str {
        "Removes dead code, unused functions, and unreachable branches"
    }

    fn run(&mut self, program: Program) -> Result<OptimizationResult, OptimizationError> {
        // Collect used functions
        self.used_functions.insert("main".to_string());

        // Collect function names from calls in the program
        self.collect_functions(&program);

        // Filter definitions to keep only used functions
        let pruned_definitions: Vec<Definition> = program
            .definitions
            .iter()
            .filter(|def| match def {
                Definition::FunctionDef { name, .. } => {
                    self.used_functions.contains(name) || name == "main"
                }
                _ => true,
            })
            .cloned()
            .collect();

        // Check if anything was removed
        let _changed = pruned_definitions.len() != program.definitions.len();

        Ok(OptimizationResult::Unchanged(Program {
            imports: program.imports.clone(),
            definitions: pruned_definitions,
            location: program.location.clone(),
        }))
    }
}

impl PrunePass {
    /// Collect function names from function calls in the program
    fn collect_functions(&mut self, program: &Program) {
        for def in &program.definitions {
            if let Definition::FunctionDef { body, .. } = def {
                self.collect_block_functions(body);
            }
        }
    }

    /// Collect function calls from a block
    fn collect_block_functions(&mut self, block: &Block) {
        for stmt in &block.statements {
            self.collect_statement_functions(stmt);
        }
    }

    /// Collect function calls from a statement
    fn collect_statement_functions(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Return { value, .. } => {
                self.collect_expression_functions(value);
            }
            Statement::Assignment { value, .. } => {
                self.collect_expression_functions(value);
            }
            Statement::Expr { expr, .. } => {
                self.collect_expression_functions(expr);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.collect_expression_functions(condition);
                self.collect_block_functions(then_branch);
                self.collect_block_functions(else_branch);
            }
            Statement::Match { value, cases, .. } => {
                self.collect_expression_functions(value);
                for case in cases {
                    self.collect_block_functions(&case.body);
                }
            }
            Statement::Bend {
                initial_states,
                condition,
                body,
                else_body,
                ..
            } => {
                for (_, expr) in initial_states {
                    self.collect_expression_functions(expr);
                }
                self.collect_expression_functions(condition);
                self.collect_block_functions(body);
                if let Some(else_b) = else_body {
                    self.collect_block_functions(else_b);
                }
            }
            Statement::Fold { value, cases, .. } => {
                self.collect_expression_functions(value);
                for case in cases {
                    self.collect_block_functions(&case.body);
                }
            }
            Statement::Use { value, .. } => {
                self.collect_expression_functions(value);
            }
            Statement::Switch { value, cases, .. } => {
                self.collect_expression_functions(value);
                for case in cases {
                    self.collect_block_functions(&case.body);
                }
            }
            _ => {}
        }
    }

    /// Collect function calls from an expression
    fn collect_expression_functions(&mut self, expr: &Expr) {
        match expr {
            Expr::FunctionCall { function, args, .. } => {
                if let Expr::Variable { name, .. } = function.as_ref() {
                    self.used_functions.insert(name.clone());
                }
                for arg in args {
                    self.collect_expression_functions(arg);
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                self.collect_expression_functions(left);
                self.collect_expression_functions(right);
            }
            Expr::Lambda { body, .. } => {
                self.collect_expression_functions(body);
            }
            Expr::Block { block, .. } => {
                self.collect_block_functions(block);
            }
            Expr::Tuple { elements, .. } => {
                for elem in elements {
                    self.collect_expression_functions(elem);
                }
            }
            Expr::List { elements, .. } => {
                for elem in elements {
                    self.collect_expression_functions(elem);
                }
            }
            Expr::Constructor { args, .. } => {
                for arg in args {
                    self.collect_expression_functions(arg);
                }
            }
            _ => {}
        }
    }
}
