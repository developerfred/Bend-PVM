// DEAD CODE ELIMINATION OPTIMIZATION
// Removes unused code, unreachable branches, and dead expressions

use crate::compiler::optimizer::passes::{OptimizationError, OptimizationResult};
use crate::compiler::parser::ast::*;
use std::collections::HashSet;

/// Tree pruning optimization pass - Dead Code Elimination
pub struct PrunePass {
    /// Functions that are actually used
    used_functions: HashSet<String>,
    /// Variables that are actually used
    used_variables: HashSet<(String, String)>,
    /// Optimization statistics
    removed_functions: usize,
    removed_variables: usize,
    removed_expressions: usize,
}

impl PrunePass {
    pub fn new() -> Self {
        PrunePass {
            used_functions: HashSet::new(),
            used_variables: HashSet::new(),
            removed_functions: 0,
            removed_variables: 0,
            removed_expressions: 0,
        }
    }

    /// Reset statistics for a new pass
    fn reset_stats(&mut self) {
        self.removed_functions = 0;
        self.removed_variables = 0;
        self.removed_expressions = 0;
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
        self.reset_stats();
        self.used_functions.clear();
        self.used_variables.clear();

        // Phase 1: Collect all used functions and variables
        self.collect_usage(&program);

        // Phase 2: Prune unused definitions
        let pruned_definitions: Vec<Definition> = program
            .definitions
            .into_iter()
            .filter(|def| self.should_keep_definition(def))
            .collect();

        // Phase 3: Prune expressions within kept definitions
        let pruned_definitions: Vec<Definition> = pruned_definitions
            .into_iter()
            .map(|def| self.prune_expression(def))
            .collect();

        let changed = pruned_definitions.len() != program.definitions.len();

        Ok(if changed {
            OptimizationResult::Improved(
                Program {
                    definitions: pruned_definitions,
                },
                format!(
                    "Removed {} functions, {} variables, {} expressions",
                    self.removed_functions, self.removed_variables, self.removed_expressions
                ),
            )
        } else {
            OptimizationResult::Unchanged(Program {
                definitions: pruned_definitions,
            })
        })
    }
}

impl PrunePass {
    /// Collect all function and variable usage from the program
    fn collect_usage(&mut self, program: &Program) {
        for def in &program.definitions {
            match def {
                Definition::FunctionDef { name: _, body, .. } => {
                    self.collect_expression_usage(body);
                }
                Definition::StructDef { fields, .. } => {
                    for field in fields {
                        self.used_variables
                            .insert((field.name.clone(), field.name.clone()));
                    }
                }
                _ => {}
            }
        }

        // Main function is always used
        self.used_functions.insert("main".to_string());
    }

    /// Collect usage from an expression
    fn collect_expression_usage(&mut self, expr: &Expr) {
        match expr {
            Expr::Var { name, .. } => {
                self.used_variables.insert((name.clone(), name.clone()));
            }
            Expr::Call { function, args, .. } => {
                self.used_functions.insert(function.clone());
                for arg in args {
                    self.collect_expression_usage(arg);
                }
            }
            Expr::Binary { left, right, .. } => {
                self.collect_expression_usage(left);
                self.collect_expression_usage(right);
            }
            Expr::Unary { expr, .. } => {
                self.collect_expression_usage(expr);
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                self.collect_expression_usage(condition);
                self.collect_expression_usage(then_branch);
                if let Some(else_expr) = else_branch {
                    self.collect_expression_usage(else_expr);
                }
            }
            Expr::Match { expr, cases, .. } => {
                self.collect_expression_usage(expr);
                for case in cases {
                    self.collect_expression_usage(&case.body);
                }
            }
            Expr::Lambda { body, .. } => {
                self.collect_expression_usage(body);
            }
            Expr::Let {
                name, value, body, ..
            } => {
                self.collect_expression_usage(value);
                self.collect_expression_usage(body);
            }
            Expr::Do { expressions, .. } => {
                for expr in expressions {
                    self.collect_expression_usage(expr);
                }
            }
            _ => {}
        }
    }

    /// Check if a definition should be kept
    fn should_keep_definition(&self, def: &Definition) -> bool {
        match def {
            Definition::FunctionDef { name, .. } => {
                let keep = self.used_functions.contains(name);
                if !keep {
                    self.removed_functions += 1;
                }
                keep
            }
            Definition::GlobalVar { name, .. } => {
                let keep = self.used_variables.iter().any(|(var, _)| var == name);
                if !keep {
                    self.removed_variables += 1;
                }
                keep
            }
            _ => true,
        }
    }

    /// Prune expressions within a definition
    fn prune_expression(&mut self, def: Definition) -> Definition {
        match def {
            Definition::FunctionDef {
                name,
                params,
                return_type,
                location,
                visibility,
                body,
            } => {
                let pruned_body = self.prune_expr(body);
                Definition::FunctionDef {
                    name,
                    params,
                    return_type,
                    location,
                    visibility,
                    body: pruned_body,
                }
            }
            _ => def,
        }
    }

    /// Prune an expression, removing dead branches
    fn prune_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::If {
                location,
                condition,
                then_branch,
                else_branch,
            } => {
                let pruned_condition = self.prune_expr(*condition);
                let pruned_then = self.prune_expr(*then_branch);
                let pruned_else = else_branch.map(|e| Box::new(self.prune_expr(*e)));

                // Simplify constant conditions
                if let Expr::Boolean { value: true, .. } = &pruned_condition {
                    // If condition is always true, return then_branch
                    self.removed_expressions += 1;
                    return pruned_then;
                }
                if let Expr::Boolean { value: false, .. } = &pruned_condition {
                    // If condition is always false, return else_branch or Unit
                    self.removed_expressions += 1;
                    if let Some(else_expr) = pruned_else {
                        return *else_expr;
                    }
                    return Expr::Unit {
                        location: location.clone(),
                    };
                }

                Expr::If {
                    location,
                    condition: Box::new(pruned_condition),
                    then_branch: Box::new(pruned_then),
                    else_branch: pruned_else,
                }
            }
            Expr::Match {
                location,
                expr,
                cases,
                match_type,
            } => {
                let pruned_expr = self.prune_expr(*expr);
                let pruned_cases: Vec<Case> = cases
                    .into_iter()
                    .map(|case| Case {
                        pattern: case.pattern,
                        guard: case.guard,
                        body: self.prune_expr(case.body),
                        location: case.location,
                    })
                    .collect();

                Expr::Match {
                    location,
                    expr: Box::new(pruned_expr),
                    cases: pruned_cases,
                    match_type,
                }
            }
            Expr::Binary {
                location,
                left,
                op,
                right,
            } => {
                let pruned_left = self.prune_expr(*left);
                let pruned_right = self.prune_expr(*right);

                // Constant folding for binary operations
                if let (Expr::Int { value: l, .. }, Expr::Int { value: r, .. }) =
                    (&pruned_left, &pruned_right)
                {
                    let result = match op {
                        Add => l + r,
                        Sub => l - r,
                        Mul => l * r,
                        Div => {
                            if *r != 0 {
                                l / r
                            } else {
                                return Expr::Int { value: 0, location };
                            }
                        }
                        Mod => l % r,
                        Eq => (l == r) as i128,
                        Neq => (l != r) as i128,
                        Lt => (l < r) as i128,
                        Gt => (l > r) as i128,
                        Le => (l <= r) as i128,
                        Ge => (l >= r) as i128,
                        _ => {
                            return Expr::Binary {
                                location,
                                left: Box::new(pruned_left),
                                op,
                                right: Box::new(pruned_right),
                            }
                        }
                    };
                    self.removed_expressions += 1;
                    return Expr::Int {
                        value: result,
                        location,
                    };
                }

                Expr::Binary {
                    location,
                    left: Box::new(pruned_left),
                    op,
                    right: Box::new(pruned_right),
                }
            }
            Expr::Do {
                location,
                expressions,
            } => {
                // Remove trailing Unit expressions and consecutive duplicate expressions
                let mut pruned_exprs: Vec<Expr> = Vec::new();
                let mut last_was_unit = false;

                for expr in expressions {
                    let pruned = self.prune_expr(expr);
                    match &pruned {
                        Expr::Unit { .. } => {
                            if !last_was_unit && !pruned_exprs.is_empty() {
                                pruned_exprs.push(pruned);
                            }
                            last_was_unit = true;
                        }
                        _ => {
                            // Remove duplicate consecutive expressions
                            if Some(&pruned) == pruned_exprs.last() {
                                self.removed_expressions += 1;
                                continue;
                            }
                            pruned_exprs.push(pruned);
                            last_was_unit = false;
                        }
                    }
                }

                // If only one expression, return it directly
                if pruned_exprs.len() == 1 {
                    return pruned_exprs.into_iter().next().unwrap();
                }

                Expr::Do {
                    location,
                    expressions: pruned_exprs,
                }
            }
            Expr::Let {
                location,
                name,
                var_type,
                value,
                body,
            } => {
                let pruned_value = self.prune_expr(*value);
                let pruned_body = self.prune_expr(*body);

                // Remove unused let bindings
                if self.used_variables.iter().any(|(n, _)| n == &name) {
                    Expr::Let {
                        location,
                        name,
                        var_type,
                        value: Box::new(pruned_value),
                        body: Box::new(pruned_body),
                    }
                } else {
                    self.removed_variables += 1;
                    self.removed_expressions += 1;
                    pruned_body
                }
            }
            _ => expr,
        }
    }
}
