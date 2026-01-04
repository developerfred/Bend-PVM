// FUNCTION INLINING OPTIMIZATION
// Replaces function calls with the function body

use crate::compiler::optimizer::passes::{OptimizationError, OptimizationResult};
use crate::compiler::parser::ast::*;
use std::collections::HashMap;

/// Function inlining optimization pass
pub struct InlinePass {
    /// Function definitions indexed by name
    functions: HashMap<String, Definition>,
    /// Maximum inline depth to prevent infinite recursion
    max_depth: usize,
    /// Current inline depth
    current_depth: usize,
    /// Statistics
    inlined_calls: usize,
    original_size: usize,
    new_size: usize,
}

impl InlinePass {
    pub fn new() -> Self {
        InlinePass {
            functions: HashMap::new(),
            max_depth: 10,
            current_depth: 0,
            inlined_calls: 0,
            original_size: 0,
            new_size: 0,
        }
    }

    fn reset_stats(&mut self) {
        self.inlined_calls = 0;
        self.original_size = 0;
        self.new_size = 0;
    }
}

impl crate::compiler::optimizer::passes::OptimizationPass for InlinePass {
    fn name(&self) -> &'static str {
        "inline"
    }

    fn description(&self) -> &'static str {
        "Inlines small functions to reduce call overhead"
    }

    fn run(&mut self, program: Program) -> Result<OptimizationResult, OptimizationError> {
        self.reset_stats();
        self.functions.clear();

        // Collect all function definitions
        for def in &program.definitions {
            if let Definition::FunctionDef { name, body, .. } = def {
                self.functions.insert(name.clone(), def.clone());
                self.original_size += estimate_size(body);
            }
        }

        // Inline functions in program
        let inlined_definitions: Vec<Definition> = program
            .definitions
            .into_iter()
            .map(|def| self.inline_definition(def))
            .collect();

        self.new_size = inlined_definitions
            .iter()
            .filter_map(|d| {
                if let Definition::FunctionDef { body, .. } = d {
                    Some(estimate_size(body))
                } else {
                    Some(0)
                }
            })
            .sum();

        let improvement = self.original_size > self.new_size;
        let report = format!(
            "Inlined {} calls, size: {} -> {} ({}% reduction)",
            self.inlined_calls,
            self.original_size,
            self.new_size,
            if self.original_size > 0 {
                (self.original_size - self.new_size) * 100 / self.original_size
            } else {
                0
            }
        );

        Ok(if improvement || self.inlined_calls > 0 {
            OptimizationResult::Improved(
                Program {
                    definitions: inlined_definitions,
                },
                report,
            )
        } else {
            OptimizationResult::Unchanged(Program {
                definitions: inlined_definitions,
            })
        })
    }
}

impl InlinePass {
    /// Inline functions within a definition
    fn inline_definition(&mut self, def: Definition) -> Definition {
        match def {
            Definition::FunctionDef {
                name,
                params,
                return_type,
                location,
                visibility,
                body,
            } => {
                let inlined_body = self.inline_expression(*body);
                Definition::FunctionDef {
                    name,
                    params,
                    return_type,
                    location,
                    visibility,
                    body: inlined_body,
                }
            }
            _ => def,
        }
    }

    /// Inline functions within an expression
    fn inline_expression(&mut self, expr: Expr) -> Expr {
        if self.current_depth >= self.max_depth {
            return expr;
        }

        match expr {
            Expr::Call {
                location,
                function,
                args,
                generic_args,
            } => {
                // Check if function should be inlined
                if let Some(target_def) = self.functions.get(&function) {
                    if let Definition::FunctionDef {
                        name: _,
                        params: target_params,
                        body: target_body,
                        ..
                    } = target_def
                    {
                        // Only inline small functions
                        if estimate_size(target_body) <= 50 {
                            self.current_depth += 1;
                            self.inlined_calls += 1;

                            // Create parameter mapping
                            let param_map: HashMap<String, Expr> = target_params
                                .iter()
                                .zip(args.into_iter())
                                .map(|(p, a)| (p.name.clone(), *a))
                                .collect();

                            // Inline the body with parameter substitution
                            let inlined_body =
                                self.substitute_params(target_body.clone(), &param_map);

                            self.current_depth -= 1;
                            return inlined_body;
                        }
                    }
                }
                Expr::Call {
                    location,
                    function,
                    args,
                    generic_args,
                }
            }
            Expr::Binary {
                location,
                left,
                op,
                right,
            } => Expr::Binary {
                location,
                left: Box::new(self.inline_expression(*left)),
                op,
                right: Box::new(self.inline_expression(*right)),
            },
            Expr::If {
                location,
                condition,
                then_branch,
                else_branch,
            } => Expr::If {
                location,
                condition: Box::new(self.inline_expression(*condition)),
                then_branch: Box::new(self.inline_expression(*then_branch)),
                else_branch: else_branch.map(|b| Box::new(self.inline_expression(*b))),
            },
            Expr::Match {
                location,
                expr,
                cases,
                match_type,
            } => Expr::Match {
                location,
                expr: Box::new(self.inline_expression(*expr)),
                cases: cases
                    .into_iter()
                    .map(|case| Case {
                        pattern: case.pattern,
                        guard: case.guard,
                        body: self.inline_expression(case.body),
                        location: case.location,
                    })
                    .collect(),
                match_type,
            },
            Expr::Do {
                location,
                expressions,
            } => Expr::Do {
                location,
                expressions: expressions
                    .into_iter()
                    .map(|e| self.inline_expression(e))
                    .collect(),
            },
            Expr::Let {
                location,
                name,
                var_type,
                value,
                body,
            } => Expr::Let {
                location,
                name,
                var_type,
                value: Box::new(self.inline_expression(*value)),
                body: Box::new(self.inline_expression(*body)),
            },
            Expr::Lambda {
                location,
                params,
                return_type,
                body,
            } => Expr::Lambda {
                location,
                params,
                return_type,
                body: Box::new(self.inline_expression(*body)),
            },
            _ => expr,
        }
    }

    /// Substitute parameters in an expression
    fn substitute_params(&self, expr: Expr, param_map: &HashMap<String, Expr>) -> Expr {
        match expr {
            Expr::Var { location, name } => {
                if let Some(subst) = param_map.get(&name) {
                    subst.clone()
                } else {
                    Expr::Var { location, name }
                }
            }
            Expr::Binary {
                location,
                left,
                op,
                right,
            } => Expr::Binary {
                location,
                left: Box::new(self.substitute_params(*left, param_map)),
                op,
                right: Box::new(self.substitute_params(*right, param_map)),
            },
            Expr::If {
                location,
                condition,
                then_branch,
                else_branch,
            } => Expr::If {
                location,
                condition: Box::new(self.substitute_params(*condition, param_map)),
                then_branch: Box::new(self.substitute_params(*then_branch, param_map)),
                else_branch: else_branch.map(|b| Box::new(self.substitute_params(*b, param_map))),
            },
            Expr::Let {
                location,
                name,
                var_type,
                value,
                body,
            } => Expr::Let {
                location,
                name,
                var_type,
                value: Box::new(self.substitute_params(*value, param_map)),
                body: Box::new(self.substitute_params(*body, param_map)),
            },
            _ => expr,
        }
    }
}

/// Estimate the size of an expression for inlining decisions
fn estimate_size(expr: &Expr) -> usize {
    match expr {
        Expr::Int { .. } => 1,
        Expr::Var { .. } => 1,
        Expr::Boolean { .. } => 1,
        Expr::Unit { .. } => 1,
        Expr::String { value, .. } => value.len(),
        Expr::Call { args, .. } => 1 + args.iter().map(estimate_size).sum::<usize>(),
        Expr::Binary { left, right, .. } => 1 + estimate_size(left) + estimate_size(right),
        Expr::Unary { expr, .. } => 1 + estimate_size(expr),
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            1 + estimate_size(condition)
                + estimate_size(then_branch)
                + else_branch.as_ref().map(estimate_size).unwrap_or(0)
        }
        Expr::Match { expr, cases, .. } => {
            1 + estimate_size(expr) + cases.iter().map(|c| estimate_size(&c.body)).sum::<usize>()
        }
        Expr::Do { expressions, .. } => expressions.iter().map(estimate_size).sum::<usize>(),
        Expr::Let { value, body, .. } => estimate_size(value) + estimate_size(body),
        Expr::Lambda { body, .. } => 1 + estimate_size(body),
        Expr::List { elements, .. } => elements.len(),
        Expr::Record { fields, .. } => fields.len(),
        Expr::FieldAccess { expr, .. } => estimate_size(expr),
        Expr::IndexAccess { expr, index, .. } => estimate_size(expr) + estimate_size(index),
        Expr::Break { .. } => 1,
        Expr::Continue { .. } => 1,
        Expr::Return { expr, .. } => 1 + expr.as_ref().map(estimate_size).unwrap_or(0),
    }
}
