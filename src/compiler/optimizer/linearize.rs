use crate::compiler::optimizer::passes::{OptimizationError, OptimizationPass, OptimizationResult};
use crate::compiler::parser::ast::*;
use std::collections::HashMap;

/// Linearization optimization pass
///
/// This pass simplifies the control flow by linearizing nested expressions,
/// extracting complex subexpressions into separate statements, and reducing
/// the nesting depth of the AST.
pub struct LinearizePass {
    /// Counter for generating unique variable names
    counter: u32,
}

impl LinearizePass {
    /// Creates a new linearization pass
    pub fn new() -> Self {
        LinearizePass { counter: 0 }
    }

    /// Generates a unique variable name
    fn fresh_var(&mut self) -> String {
        self.counter += 1;
        format!("__lin_{}", self.counter)
    }

    /// Linearizes a block by extracting complex subexpressions
    fn linearize_block(&mut self, block: &Block) -> (Block, bool) {
        let mut new_statements = Vec::new();
        let mut modified = false;

        for statement in &block.statements {
            let (linearized_statements, stmt_modified) = self.linearize_statement(statement);
            new_statements.extend(linearized_statements);
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

    /// Linearizes a statement by extracting complex subexpressions
    fn linearize_statement(&mut self, statement: &Statement) -> (Vec<Statement>, bool) {
        match statement {
            Statement::Assignment {
                pattern,
                value,
                location,
            } => {
                let (linearized_expr, pre_statements, expr_modified) = self.linearize_expr(value);

                let mut statements = pre_statements;
                statements.push(Statement::Assignment {
                    pattern: pattern.clone(),
                    value: linearized_expr,
                    location: location.clone(),
                });

                (statements, expr_modified)
            }
            Statement::Return { value, location } => {
                let (linearized_expr, pre_statements, expr_modified) = self.linearize_expr(value);

                let mut statements = pre_statements;
                statements.push(Statement::Return {
                    value: linearized_expr,
                    location: location.clone(),
                });

                (statements, expr_modified)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                location,
            } => {
                // Linearize the condition
                let (linearized_condition, mut pre_statements, condition_modified) =
                    self.linearize_expr(condition);

                // Linearize the branches
                let (linearized_then, then_modified) = self.linearize_block(then_branch);
                let (linearized_else, else_modified) = self.linearize_block(else_branch);

                let modified = condition_modified || then_modified || else_modified;

                let if_statement = Statement::If {
                    condition: linearized_condition,
                    then_branch: linearized_then,
                    else_branch: linearized_else,
                    location: location.clone(),
                };

                pre_statements.push(if_statement);

                (pre_statements, modified)
            }
            Statement::Expr { expr, location } => {
                let (linearized_expr, mut pre_statements, expr_modified) =
                    self.linearize_expr(expr);

                pre_statements.push(Statement::Expr {
                    expr: linearized_expr,
                    location: location.clone(),
                });

                (pre_statements, expr_modified)
            }
            // For other statement types, we would need to implement linearization logic
            // This is a simplified implementation
            _ => (vec![statement.clone()], false),
        }
    }

    /// Linearizes an expression by extracting complex subexpressions
    fn linearize_expr(&mut self, expr: &Expr) -> (Expr, Vec<Statement>, bool) {
        match expr {
            Expr::BinaryOp {
                left,
                operator,
                right,
                location,
            } => {
                // Linearize the operands
                let (linearized_left, mut left_statements, left_modified) =
                    self.linearize_expr(left);
                let (linearized_right, mut right_statements, right_modified) =
                    self.linearize_expr(right);

                let mut pre_statements = Vec::new();
                pre_statements.append(&mut left_statements);
                pre_statements.append(&mut right_statements);

                // If the operands are complex, extract them into temporary variables
                let (final_left, mut left_temp_stmts) = self.maybe_extract_expr(&linearized_left);
                let (final_right, mut right_temp_stmts) =
                    self.maybe_extract_expr(&linearized_right);

                pre_statements.append(&mut left_temp_stmts);
                pre_statements.append(&mut right_temp_stmts);

                let modified = left_modified
                    || right_modified
                    || final_left != linearized_left
                    || final_right != linearized_right;

                let result = Expr::BinaryOp {
                    left: Box::new(final_left),
                    operator: operator.clone(),
                    right: Box::new(final_right),
                    location: location.clone(),
                };

                (result, pre_statements, modified)
            }
            Expr::FunctionCall {
                function,
                args,
                named_args,
                location,
            } => {
                // Linearize the function
                let (linearized_function, function_statements, function_modified) =
                    self.linearize_expr(function);

                // Linearize the arguments
                let mut linearized_args = Vec::new();
                let mut args_modified = false;
                let mut pre_statements = function_statements;

                for arg in args {
                    let (linearized_arg, mut arg_statements, arg_modified) =
                        self.linearize_expr(arg);
                    pre_statements.append(&mut arg_statements);
                    linearized_args.push(linearized_arg);
                    args_modified = args_modified || arg_modified;
                }

                // Linearize named arguments
                let mut linearized_named_args = HashMap::new();
                let mut named_args_modified = false;

                for (name, arg) in named_args {
                    let (linearized_arg, mut arg_statements, arg_modified) =
                        self.linearize_expr(arg);
                    pre_statements.append(&mut arg_statements);
                    linearized_named_args.insert(name.clone(), linearized_arg);
                    named_args_modified = named_args_modified || arg_modified;
                }

                // If the function is complex, extract it into a temporary variable
                let (final_function, mut function_temp_stmts) =
                    self.maybe_extract_expr(&linearized_function);
                pre_statements.append(&mut function_temp_stmts);

                // If any arguments are complex, extract them into temporary variables
                let mut final_args = Vec::new();
                for arg in linearized_args {
                    let (final_arg, mut arg_temp_stmts) = self.maybe_extract_expr(&arg);
                    pre_statements.append(&mut arg_temp_stmts);
                    final_args.push(final_arg);
                }

                let mut final_named_args = HashMap::new();
                for (name, arg) in linearized_named_args {
                    let (final_arg, mut arg_temp_stmts) = self.maybe_extract_expr(&arg);
                    pre_statements.append(&mut arg_temp_stmts);
                    final_named_args.insert(name, final_arg);
                }

                let modified = function_modified
                    || args_modified
                    || named_args_modified
                    || final_function != linearized_function;

                let result = Expr::FunctionCall {
                    function: Box::new(final_function),
                    args: final_args,
                    named_args: final_named_args,
                    location: location.clone(),
                };

                (result, pre_statements, modified)
            }
            // For other expression types, we would need to implement linearization logic
            // This is a simplified implementation
            _ => (expr.clone(), Vec::new(), false),
        }
    }

    /// Extracts a complex expression into a temporary variable
    fn maybe_extract_expr(&mut self, expr: &Expr) -> (Expr, Vec<Statement>) {
        if self.is_complex(expr) {
            // Generate a fresh variable name
            let var_name = self.fresh_var();

            // Create a new variable expression
            let var_expr = Expr::Variable {
                name: var_name.clone(),
                location: expr.location().clone(),
            };

            // Create an assignment statement
            let assignment = Statement::Assignment {
                pattern: Pattern::Variable {
                    name: var_name,
                    location: expr.location().clone(),
                },
                value: expr.clone(),
                location: expr.location().clone(),
            };

            (var_expr, vec![assignment])
        } else {
            (expr.clone(), Vec::new())
        }
    }

    /// Checks if an expression is complex and should be extracted
    fn is_complex(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Variable { .. } | Expr::Literal { .. } | Expr::Eraser { .. } => false,
            Expr::BinaryOp { .. } | Expr::FunctionCall { .. } => true,
            // For other expression types, determine if they're complex
            _ => false,
        }
    }
}

impl OptimizationPass for LinearizePass {
    fn name(&self) -> &'static str {
        "linearize"
    }

    fn description(&self) -> &'static str {
        "Linearizes the AST by extracting complex subexpressions into separate statements"
    }

    fn run(&mut self, program: Program) -> Result<OptimizationResult, OptimizationError> {
        let mut linearizer = LinearizePass::new();
        let mut modified = false;
        let mut new_definitions = Vec::new();

        // Linearize each definition
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
                    // Linearize the function body
                    let (linearized_body, body_modified) = linearizer.linearize_block(body);
                    modified = modified || body_modified;

                    new_definitions.push(Definition::FunctionDef {
                        name: name.clone(),
                        params: params.clone(),
                        return_type: return_type.clone(),
                        body: linearized_body,
                        checked: *checked,
                        location: location.clone(),
                    });
                }
                // Other definition types don't need linearization
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
