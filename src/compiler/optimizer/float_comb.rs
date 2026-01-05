use crate::compiler::optimizer::passes::{OptimizationError, OptimizationPass, OptimizationResult};
use crate::compiler::parser::ast::*;

/// Float combination optimization pass
///
/// This pass combines floating-point operations to reduce the number of operations
/// and improve performance. It performs constant folding for floating-point
/// expressions and simplifies mathematical expressions.
pub struct FloatCombPass;

impl FloatCombPass {
    /// Creates a new float combination pass
    pub fn new() -> Self {
        FloatCombPass
    }

    /// Optimize a block by combining floating-point operations
    fn optimize_block(&self, block: &Block) -> (Block, bool) {
        let mut new_statements = Vec::new();
        let mut modified = false;

        for statement in &block.statements {
            let (optimized_statement, stmt_modified) = self.optimize_statement(statement);
            new_statements.push(optimized_statement);
            modified = modified || stmt_modified;
        }

        (
            Block {
                statements: new_statements,
                location: block.location.clone(),
            },
            modified,
        )
    }

    /// Optimize a statement by combining floating-point operations
    fn optimize_statement(&self, statement: &Statement) -> (Statement, bool) {
        match statement {
            Statement::Assignment {
                pattern,
                value,
                location,
            } => {
                let (optimized_expr, expr_modified) = self.optimize_expr(value);

                (
                    Statement::Assignment {
                        pattern: pattern.clone(),
                        value: optimized_expr,
                        location: location.clone(),
                    },
                    expr_modified,
                )
            }
            Statement::Return { value, location } => {
                let (optimized_expr, expr_modified) = self.optimize_expr(value);

                (
                    Statement::Return {
                        value: optimized_expr,
                        location: location.clone(),
                    },
                    expr_modified,
                )
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                location,
            } => {
                let (optimized_condition, condition_modified) = self.optimize_expr(condition);
                let (optimized_then, then_modified) = self.optimize_block(then_branch);
                let (optimized_else, else_modified) = self.optimize_block(else_branch);

                let modified = condition_modified || then_modified || else_modified;

                (
                    Statement::If {
                        condition: optimized_condition,
                        then_branch: optimized_then,
                        else_branch: optimized_else,
                        location: location.clone(),
                    },
                    modified,
                )
            }
            Statement::Expr { expr, location } => {
                let (optimized_expr, expr_modified) = self.optimize_expr(expr);

                (
                    Statement::Expr {
                        expr: optimized_expr,
                        location: location.clone(),
                    },
                    expr_modified,
                )
            }
            // For other statement types, implement optimization logic
            // This is a simplified implementation
            _ => (statement.clone(), false),
        }
    }

    /// Optimize an expression by combining floating-point operations
    fn optimize_expr(&self, expr: &Expr) -> (Expr, bool) {
        match expr {
            Expr::BinaryOp {
                left,
                operator,
                right,
                location,
            } => {
                let (optimized_left, left_modified) = self.optimize_expr(left);
                let (optimized_right, right_modified) = self.optimize_expr(right);

                // Try to fold constants
                if let Some(folded) =
                    self.fold_constants(&optimized_left, operator, &optimized_right, location)
                {
                    return (folded, true);
                }

                // Try algebraic simplifications
                if let Some(simplified) =
                    self.simplify_algebra(&optimized_left, operator, &optimized_right, location)
                {
                    return (simplified, true);
                }

                let modified = left_modified || right_modified;

                (
                    Expr::BinaryOp {
                        left: Box::new(optimized_left),
                        operator: operator.clone(),
                        right: Box::new(optimized_right),
                        location: location.clone(),
                    },
                    modified,
                )
            }
            Expr::FunctionCall {
                function,
                args,
                named_args,
                location,
            } => {
                let (optimized_function, function_modified) = self.optimize_expr(function);

                let mut optimized_args = Vec::new();
                let mut args_modified = false;

                for arg in args {
                    let (optimized_arg, arg_modified) = self.optimize_expr(arg);
                    optimized_args.push(optimized_arg);
                    args_modified = args_modified || arg_modified;
                }

                let mut optimized_named_args = named_args.clone();
                let mut named_args_modified = false;

                for (name, arg) in named_args {
                    let (optimized_arg, arg_modified) = self.optimize_expr(arg);
                    if arg_modified {
                        optimized_named_args.insert(name.clone(), optimized_arg);
                        named_args_modified = true;
                    }
                }

                let modified = function_modified || args_modified || named_args_modified;

                (
                    Expr::FunctionCall {
                        function: Box::new(optimized_function),
                        args: optimized_args,
                        named_args: optimized_named_args.clone(),
                        location: location.clone(),
                    },
                    modified,
                )
            }
            // For other expression types, no optimization needed
            _ => (expr.clone(), false),
        }
    }

    /// Folds constant floating-point operations
    fn fold_constants(
        &self,
        left: &Expr,
        operator: &BinaryOperator,
        right: &Expr,
        location: &Location,
    ) -> Option<Expr> {
        match (left, operator, right) {
            // Float + Float -> Float
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Add,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Float(left_val + right_val),
                location: location.clone(),
            }),
            // Float - Float -> Float
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Sub,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Float(left_val - right_val),
                location: location.clone(),
            }),
            // Float * Float -> Float
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Mul,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Float(left_val * right_val),
                location: location.clone(),
            }),
            // Float / Float -> Float
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Div,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => {
                if *right_val != 0.0 {
                    Some(Expr::Literal {
                        kind: LiteralKind::Float(left_val / right_val),
                        location: location.clone(),
                    })
                } else {
                    // Division by zero, don't fold
                    None
                }
            }
            // Float % Float -> Float
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Mod,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => {
                if *right_val != 0.0 {
                    Some(Expr::Literal {
                        kind: LiteralKind::Float(left_val % right_val),
                        location: location.clone(),
                    })
                } else {
                    // Modulo by zero, don't fold
                    None
                }
            }
            // Float ** Float -> Float (Power)
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Pow,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Float(left_val.powf(*right_val)),
                location: location.clone(),
            }),
            // Float comparison operators
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Equal,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Uint(if left_val == right_val { 1 } else { 0 }),
                location: location.clone(),
            }),
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::NotEqual,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Uint(if left_val != right_val { 1 } else { 0 }),
                location: location.clone(),
            }),
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Less,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Uint(if left_val < right_val { 1 } else { 0 }),
                location: location.clone(),
            }),
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::LessEqual,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Uint(if left_val <= right_val { 1 } else { 0 }),
                location: location.clone(),
            }),
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::Greater,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Uint(if left_val > right_val { 1 } else { 0 }),
                location: location.clone(),
            }),
            (
                Expr::Literal {
                    kind: LiteralKind::Float(left_val),
                    ..
                },
                BinaryOperator::GreaterEqual,
                Expr::Literal {
                    kind: LiteralKind::Float(right_val),
                    ..
                },
            ) => Some(Expr::Literal {
                kind: LiteralKind::Uint(if left_val >= right_val { 1 } else { 0 }),
                location: location.clone(),
            }),
            // No constant folding possible
            _ => None,
        }
    }

    /// Simplifies algebraic expressions
    fn simplify_algebra(
        &self,
        left: &Expr,
        operator: &BinaryOperator,
        right: &Expr,
        location: &Location,
    ) -> Option<Expr> {
        match (left, operator, right) {
            // x + 0 -> x
            (
                expr,
                BinaryOperator::Add,
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    ..
                },
            ) if *val == 0.0 => Some(expr.clone()),
            // 0 + x -> x
            (
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    ..
                },
                BinaryOperator::Add,
                expr,
            ) if *val == 0.0 => Some(expr.clone()),
            // x - 0 -> x
            (
                expr,
                BinaryOperator::Sub,
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    ..
                },
            ) if *val == 0.0 => Some(expr.clone()),
            // x * 1 -> x
            (
                expr,
                BinaryOperator::Mul,
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    ..
                },
            ) if *val == 1.0 => Some(expr.clone()),
            // 1 * x -> x
            (
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    ..
                },
                BinaryOperator::Mul,
                expr,
            ) if *val == 1.0 => Some(expr.clone()),
            // x * 0 -> 0
            (
                _,
                BinaryOperator::Mul,
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    location: right_loc,
                    ..
                },
            ) if *val == 0.0 => Some(Expr::Literal {
                kind: LiteralKind::Float(0.0),
                location: right_loc.clone(),
            }),
            // 0 * x -> 0
            (
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    location: left_loc,
                    ..
                },
                BinaryOperator::Mul,
                _,
            ) if *val == 0.0 => Some(Expr::Literal {
                kind: LiteralKind::Float(0.0),
                location: left_loc.clone(),
            }),
            // x / 1 -> x
            (
                expr,
                BinaryOperator::Div,
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    ..
                },
            ) if *val == 1.0 => Some(expr.clone()),
            // x ** 1 -> x
            (
                expr,
                BinaryOperator::Pow,
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    ..
                },
            ) if *val == 1.0 => Some(expr.clone()),
            // x ** 0 -> 1
            (
                _,
                BinaryOperator::Pow,
                Expr::Literal {
                    kind: LiteralKind::Float(val),
                    ..
                },
            ) if *val == 0.0 => Some(Expr::Literal {
                kind: LiteralKind::Float(1.0),
                location: location.clone(),
            }),
            // No simplification possible
            _ => None,
        }
    }
}

impl Default for FloatCombPass {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationPass for FloatCombPass {
    fn name(&self) -> &'static str {
        "float_comb"
    }

    fn description(&self) -> &'static str {
        "Combines floating-point operations for better performance"
    }

    fn run(&mut self, program: Program) -> Result<OptimizationResult, OptimizationError> {
        let mut modified = false;
        let mut new_definitions = Vec::new();

        // Optimize each definition
        for definition in &program.definitions {
            match definition {
                Definition::FunctionDef {
                    name,
                    params,
                    return_type,
                    body,
                    checked,
                    location,
                } => {
                    // Optimize the function body
                    let (optimized_body, body_modified) = self.optimize_block(body);
                    modified = modified || body_modified;

                    new_definitions.push(Definition::FunctionDef {
                        name: name.clone(),
                        params: params.clone(),
                        return_type: return_type.clone(),
                        body: optimized_body,
                        checked: *checked,
                        location: location.clone(),
                    });
                }
                // Other definition types don't need optimization
                _ => new_definitions.push(definition.clone()),
            }
        }

        // Return the result
        if modified {
            Ok(OptimizationResult::Modified(Program {
                imports: program.imports.clone(),
                definitions: new_definitions,
                location: program.location.clone(),
            }))
        } else {
            Ok(OptimizationResult::Unchanged(program))
        }
    }
}
