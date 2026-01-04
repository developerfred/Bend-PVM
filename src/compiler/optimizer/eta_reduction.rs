use crate::compiler::optimizer::passes::{OptimizationError, OptimizationPass, OptimizationResult};
use crate::compiler::parser::ast::*;
use std::collections::HashSet;

/// Eta reduction optimization pass
///
/// This pass performs eta reduction, which is a transformation that simplifies
/// functions that just wrap other functions without adding functionality.
/// For example, `lambda x: f(x)` can be simplified to just `f`.
pub struct EtaReductionPass;

impl EtaReductionPass {
    /// Creates a new eta reduction pass
    pub fn new() -> Self {
        EtaReductionPass
    }

    /// Optimize a block using eta reduction
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

    /// Optimize a statement using eta reduction
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

    /// Optimize an expression using eta reduction
    fn optimize_expr(&self, expr: &Expr) -> (Expr, bool) {
        match expr {
            Expr::Lambda {
                params,
                body,
                location,
            } => {
                // First, recursively optimize the body
                let (optimized_body, body_modified) = self.optimize_expr(body);

                // Check for eta-reducible form: lambda x: f(x)
                if let Expr::FunctionCall {
                    function,
                    args,
                    named_args,
                    ..
                } = &optimized_body
                {
                    // Check if this is a simple function call that can be eta-reduced
                    if named_args.is_empty() && args.len() == params.len() {
                        // Check that each argument is a parameter variable in the same order
                        let mut can_eta_reduce = true;
                        let mut param_names = HashSet::new();

                        // Collect parameter names
                        for param in params.iter() {
                            param_names.insert(&param.name);
                        }

                        // Check that each argument is a parameter variable
                        for (i, arg) in args.iter().enumerate() {
                            if let Expr::Variable { name, .. } = arg {
                                if !param_names.contains(name) || &params[i].name != name {
                                    can_eta_reduce = false;
                                    break;
                                }
                            } else {
                                can_eta_reduce = false;
                                break;
                            }
                        }

                        if can_eta_reduce {
                            // Eta-reduce the lambda to the function
                            return ((**function).clone(), true);
                        }
                    }
                }

                // If not eta-reducible, return the optimized lambda
                (
                    Expr::Lambda {
                        params: params.clone(),
                        body: Box::new(optimized_body),
                        location: location.clone(),
                    },
                    body_modified,
                )
            }
            Expr::UnsccopedLambda {
                params,
                body,
                location,
            } => {
                // Recursively optimize the body
                let (optimized_body, body_modified) = self.optimize_expr(body);

                // Check for eta-reducible form: lambda x: f(x)
                if let Expr::FunctionCall {
                    function,
                    args,
                    named_args,
                    ..
                } = &optimized_body
                {
                    // Check if this is a simple function call that can be eta-reduced
                    if named_args.is_empty() && args.len() == params.len() {
                        // Check that each argument is a parameter variable in the same order
                        let mut can_eta_reduce = true;
                        let param_set: HashSet<&String> = params.iter().collect();

                        // Check that each argument is a parameter variable
                        for (i, arg) in args.iter().enumerate() {
                            if let Expr::Variable { name, .. } = arg {
                                if !param_set.contains(name) || &params[i] != name {
                                    can_eta_reduce = false;
                                    break;
                                }
                            } else {
                                can_eta_reduce = false;
                                break;
                            }
                        }

                        if can_eta_reduce {
                            // Eta-reduce the lambda to the function
                            return ((**function).clone(), true);
                        }
                    }
                }

                // If not eta-reducible, return the optimized lambda
                (
                    Expr::UnsccopedLambda {
                        params: params.clone(),
                        body: Box::new(optimized_body),
                        location: location.clone(),
                    },
                    body_modified,
                )
            }
            Expr::BinaryOp {
                left,
                operator,
                right,
                location,
            } => {
                let (optimized_left, left_modified) = self.optimize_expr(left);
                let (optimized_right, right_modified) = self.optimize_expr(right);

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
                        named_args: optimized_named_args,
                        location: location.clone(),
                    },
                    modified,
                )
            }
            Expr::Block { block, location } => {
                // Optimize the block
                let (optimized_block, block_modified) = self.optimize_block(block);

                (
                    Expr::Block {
                        block: optimized_block,
                        location: location.clone(),
                    },
                    block_modified,
                )
            }
            // For other expression types, no optimization needed
            _ => (expr.clone(), false),
        }
    }
}

impl OptimizationPass for EtaReductionPass {
    fn name(&self) -> &'static str {
        "eta_reduction"
    }

    fn description(&self) -> &'static str {
        "Performs eta reduction for functions that just wrap other functions"
    }

    fn run(&self, program: Program) -> Result<OptimizationResult, OptimizationError> {
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
