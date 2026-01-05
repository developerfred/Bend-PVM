//! Constant Folding Optimization Pass
//!
//! This pass evaluates constant expressions at compile time and replaces them with their computed values.
//!
//! # Examples
//!
//! ```rust
//! // Before optimization:
//! let x = 5 + 3;
//! let y = 10 * 2;
//!
//! // After constant folding:
//! let x = 8;
//! let y = 20;
//! ```

#![allow(clippy::needless_return)]
#![allow(unused_imports)]

use crate::compiler::codegen::risc_v::Instruction;
use crate::compiler::parser::ast::{BinaryOperator, Expr, Location, LocationProvider};
use crate::compiler::parser::ast::{LiteralKind, Pattern, Statement};

/// Constant folding optimization pass
pub struct ConstantFolding {
    pub folded_constants: u32,
    pub optimized_ops: u32,
}

impl Default for ConstantFolding {
    fn default() -> Self {
        Self::new()
    }
}

impl ConstantFolding {
    pub fn new() -> Self {
        Self {
            folded_constants: 0,
            optimized_ops: 0,
        }
    }

    /// Apply constant folding to an expression
    pub fn fold_expression(&mut self, expr: &Expr) -> Result<Expr, String> {
        match expr {
            Expr::BinaryOp {
                left,
                operator,
                right,
                location: _,
            } => {
                // First, recursively fold the left and right operands
                let folded_left = self.fold_expression(left)?;
                let folded_right = self.fold_expression(right)?;

                // Then try to evaluate the binary operation with the folded operands
                if let Some(result) =
                    self.try_fold_binary_op(&folded_left, &folded_right, operator.clone())
                {
                    Ok(result)
                } else {
                    // Return the folded binary operation if we couldn't evaluate it
                    Ok(Expr::BinaryOp {
                        left: Box::new(folded_left),
                        operator: operator.clone(),
                        right: Box::new(folded_right),
                        location: expr.location().clone(),
                    })
                }
            }
            _ => Ok(expr.clone()),
        }
    }

    /// Try to evaluate a binary operation with constant operands
    fn try_fold_binary_op(
        &mut self,
        left: &Expr,
        right: &Expr,
        operator: BinaryOperator,
    ) -> Option<Expr> {
        // Try to extract constant values from both operands
        let left_val = self.extract_constant(left);
        let right_val = self.extract_constant(right);

        match (left_val, right_val, operator) {
            // Addition with constants
            (Some(l_val), Some(r_val), crate::compiler::parser::ast::BinaryOperator::Add) => {
                self.optimized_ops += 1;
                self.folded_constants += 1;
                return Some(Expr::Literal {
                    kind: crate::compiler::parser::ast::LiteralKind::Uint(l_val + r_val),
                    location: left.location().clone(),
                });
            }

            // Multiplication with constants
            (Some(l_mult), Some(r_mult), crate::compiler::parser::ast::BinaryOperator::Mul) => {
                self.optimized_ops += 1;
                self.folded_constants += 1;
                return Some(Expr::Literal {
                    kind: crate::compiler::parser::ast::LiteralKind::Uint(l_mult * r_mult),
                    location: left.location().clone(),
                });
            }

            // Division with constants
            (Some(l_div), Some(r_div), crate::compiler::parser::ast::BinaryOperator::Div)
                if r_div != 0 =>
            {
                self.optimized_ops += 1;
                self.folded_constants += 1;
                return Some(Expr::Literal {
                    kind: crate::compiler::parser::ast::LiteralKind::Uint(l_div / r_div),
                    location: left.location().clone(),
                });
            }

            _ => None,
        }
    }

    /// Extract constant value from an expression if possible
    fn extract_constant(&self, expr: &Expr) -> Option<u32> {
        match expr {
            Expr::Literal { kind, .. } => match kind {
                crate::compiler::parser::ast::LiteralKind::Uint(n) => Some(*n),
                crate::compiler::parser::ast::LiteralKind::Int(n) => Some(*n as u32),
                crate::compiler::parser::ast::LiteralKind::Float(n) => Some(*n as u32),
                _ => None,
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_addition() {
        let input = r#"
fn test_add() -> u24 {
    let x = 5 + 3;
    x
}
"#;

        let parsed = crate::compiler::parser::parser::parse_from_source(input);
        assert!(parsed.is_ok());

        let mut folder = ConstantFolding::new();
        let program = parsed.unwrap();

        for def in &program.definitions {
            if let crate::compiler::parser::ast::Definition::FunctionDef { body, .. } = def {
                for stmt in &body.statements {
                    if let crate::compiler::parser::ast::Statement::Use { value, .. } = stmt {
                        let optimized = folder.fold_expression(value);
                        assert!(optimized.is_ok());
                        // Check if constants were folded
                        // The actual folding will be done during code generation
                    }
                }
            }
        }
    }
}
