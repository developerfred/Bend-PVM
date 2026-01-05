use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

use bend_pvm::compiler::parser::ast::*;
use bend_pvm::compiler::parser::parser::Parser;

/// Error types for gas profiling
#[derive(Error, Debug)]
pub enum ProfilerError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Gas cost estimate for a function
#[derive(Debug, Clone)]
pub struct GasEstimate {
    /// Function name
    pub name: String,

    /// Estimated base gas cost (minimum cost)
    pub base_cost: u64,

    /// Estimated maximum gas cost
    pub max_cost: u64,

    /// Estimated average gas cost
    pub avg_cost: u64,

    /// Breakdown of gas costs by operation type
    pub cost_breakdown: HashMap<String, u64>,

    /// Whether the function is recursive
    pub is_recursive: bool,

    /// Whether the function calls external contracts
    pub has_external_calls: bool,

    /// Line range in the source code
    pub line_range: (usize, usize),
}

/// Result of gas profiling
#[derive(Debug)]
pub struct GasProfile {
    /// Gas estimates for functions
    pub estimates: Vec<GasEstimate>,

    /// Path to the profiled file
    pub file_path: String,

    /// Total estimated gas usage
    pub total_gas: u64,

    /// Most expensive function
    pub most_expensive_function: Option<String>,
}

/// Gas profiler for Bend contracts
pub struct GasProfiler {
    /// Gas costs for various operations
    costs: HashMap<String, u64>,
}

impl GasProfiler {
    /// Create a new gas profiler
    pub fn new() -> Self {
        let mut costs = HashMap::new();

        // Base costs for various operations
        costs.insert("storage_read".to_string(), 200);
        costs.insert("storage_write".to_string(), 5000);
        costs.insert("storage_delete".to_string(), 500);
        costs.insert("external_call".to_string(), 2500);
        costs.insert("event_emit".to_string(), 375);
        costs.insert("if_branch".to_string(), 10);
        costs.insert("bend_iteration".to_string(), 25);
        costs.insert("match_branch".to_string(), 15);
        costs.insert("function_call".to_string(), 40);
        costs.insert("binary_op".to_string(), 5);
        costs.insert("variable_access".to_string(), 3);
        costs.insert("literal".to_string(), 3);
        costs.insert("tuple".to_string(), 10);
        costs.insert("list".to_string(), 15);
        costs.insert("constructor".to_string(), 20);
        costs.insert("return".to_string(), 5);

        GasProfiler { costs }
    }

    /// Profile a file for gas usage
    pub fn profile_file<P: AsRef<Path>>(&self, file_path: P) -> Result<GasProfile, ProfilerError> {
        let source = fs::read_to_string(&file_path)?;
        let file_path_str = file_path.as_ref().to_string_lossy().to_string();

        self.profile_source(&source, &file_path_str)
    }

    /// Profile source code for gas usage
    pub fn profile_source(
        &self,
        source: &str,
        file_path: &str,
    ) -> Result<GasProfile, ProfilerError> {
        // Parse the source
        let mut parser = Parser::new(source);
        let program = parser
            .parse_program()
            .map_err(|e| ProfilerError::Parse(e.to_string()))?;

        // Profile each function
        let mut estimates = Vec::new();

        for definition in &program.definitions {
            match definition {
                Definition::FunctionDef { name, body, .. } => {
                    // Skip internal functions (starting with underscore)
                    if name.starts_with('_') {
                        continue;
                    }

                    // Profile the function
                    let estimate = self.profile_function(name, body, &program);
                    estimates.push(estimate);
                }
                _ => {}
            }
        }

        // Calculate total gas and find the most expensive function
        let total_gas = estimates.iter().map(|e| e.avg_cost).sum();

        let most_expensive_function = estimates
            .iter()
            .max_by_key(|e| e.avg_cost)
            .map(|e| e.name.clone());

        Ok(GasProfile {
            estimates,
            file_path: file_path.to_string(),
            total_gas,
            most_expensive_function,
        })
    }

    /// Profile a function for gas usage
    fn profile_function(&self, name: &str, body: &Block, program: &Program) -> GasEstimate {
        let mut cost_breakdown = HashMap::new();

        // Add base cost for the function
        cost_breakdown.insert("function_overhead".to_string(), 20);

        // Calculate costs for the function body
        let body_cost = self.profile_block(body, &mut cost_breakdown);

        // Determine if the function is recursive
        let is_recursive = self.is_function_recursive(name, body);

        // Determine if the function calls external contracts
        let has_external_calls = self.has_external_calls(body);

        // Calculate total costs
        let base_cost = cost_breakdown.values().sum();
        let avg_cost = base_cost;
        let max_cost = if is_recursive || has_external_calls {
            // For recursive or external calling functions, max cost is harder to estimate
            // For simplicity, we multiply the base cost by a factor
            base_cost * 5
        } else {
            base_cost
        };

        GasEstimate {
            name: name.to_string(),
            base_cost,
            max_cost,
            avg_cost,
            cost_breakdown,
            is_recursive,
            has_external_calls,
            line_range: (body.location.line, body.location.line + count_lines(body)),
        }
    }

    /// Profile a block for gas usage
    fn profile_block(&self, block: &Block, cost_breakdown: &mut HashMap<String, u64>) -> u64 {
        let mut total_cost = 0;

        for statement in &block.statements {
            total_cost += self.profile_statement(statement, cost_breakdown);
        }

        total_cost
    }

    /// Profile a statement for gas usage
    fn profile_statement(
        &self,
        statement: &Statement,
        cost_breakdown: &mut HashMap<String, u64>,
    ) -> u64 {
        match statement {
            Statement::Return { value, .. } => {
                let return_cost = self.get_cost("return");
                let value_cost = self.profile_expr(value, cost_breakdown);

                *cost_breakdown.entry("return".to_string()).or_insert(0) += return_cost;

                return_cost + value_cost
            }
            Statement::Assignment { pattern, value, .. } => {
                let assignment_cost = 10;
                let pattern_cost = 5; // Simplified, should depend on pattern complexity
                let value_cost = self.profile_expr(value, cost_breakdown);

                *cost_breakdown.entry("assignment".to_string()).or_insert(0) += assignment_cost;

                assignment_cost + pattern_cost + value_cost
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let if_cost = self.get_cost("if_branch");
                let condition_cost = self.profile_expr(condition, cost_breakdown);
                let then_cost = self.profile_block(then_branch, cost_breakdown);
                let else_cost = self.profile_block(else_branch, cost_breakdown);

                // Assume average case: half the time we take the then branch, half the time the else branch
                let avg_branch_cost = (then_cost + else_cost) / 2;

                *cost_breakdown.entry("if_branch".to_string()).or_insert(0) += if_cost;

                if_cost + condition_cost + avg_branch_cost
            }
            Statement::Bend {
                condition,
                initial_states,
                body,
                else_body,
                ..
            } => {
                let bend_cost = self.get_cost("bend_iteration");
                let condition_cost = self.profile_expr(condition, cost_breakdown);

                let mut initial_states_cost = 0;
                for (_, expr) in initial_states {
                    initial_states_cost += self.profile_expr(expr, cost_breakdown);
                }

                let body_cost = self.profile_block(body, cost_breakdown);
                let else_cost = if let Some(else_body) = else_body {
                    self.profile_block(else_body, cost_breakdown)
                } else {
                    0
                };

                // Estimate number of iterations (very rough estimate)
                let estimated_iterations = 5;

                *cost_breakdown
                    .entry("bend_iteration".to_string())
                    .or_insert(0) += bend_cost * estimated_iterations;

                bend_cost * estimated_iterations
                    + condition_cost
                    + initial_states_cost
                    + (body_cost * estimated_iterations)
                    + else_cost
            }
            Statement::Match { value, cases, .. } => {
                let match_cost = self.get_cost("match_branch");
                let value_cost = self.profile_expr(value, cost_breakdown);

                let mut cases_cost = 0;
                for case in cases {
                    cases_cost += self.profile_block(&case.body, cost_breakdown);
                }

                // Assume average case: each branch has equal probability
                let avg_case_cost = if !cases.is_empty() {
                    cases_cost / cases.len() as u64
                } else {
                    0
                };

                *cost_breakdown
                    .entry("match_branch".to_string())
                    .or_insert(0) += match_cost;

                match_cost + value_cost + avg_case_cost
            }
            Statement::With {
                monad_type, body, ..
            } => {
                let with_cost = 15;
                let body_cost = self.profile_block(body, cost_breakdown);

                // If this is an IO monad, it's likely to have external calls
                let external_cost = if monad_type == "IO" {
                    100 // Additional cost for IO operations
                } else {
                    0
                };

                *cost_breakdown.entry("with_block".to_string()).or_insert(0) += with_cost;
                if external_cost > 0 {
                    *cost_breakdown
                        .entry("io_operations".to_string())
                        .or_insert(0) += external_cost;
                }

                with_cost + body_cost + external_cost
            }
            Statement::Expr { expr, .. } => self.profile_expr(expr, cost_breakdown),
            // Add gas estimates for other statement types
            _ => 10, // Default cost for unhandled statement types
        }
    }

    /// Profile an expression for gas usage
    fn profile_expr(&self, expr: &Expr, cost_breakdown: &mut HashMap<String, u64>) -> u64 {
        match expr {
            Expr::Variable { .. } => {
                let cost = self.get_cost("variable_access");
                *cost_breakdown
                    .entry("variable_access".to_string())
                    .or_insert(0) += cost;
                cost
            }
            Expr::Literal { .. } => {
                let cost = self.get_cost("literal");
                *cost_breakdown.entry("literal".to_string()).or_insert(0) += cost;
                cost
            }
            Expr::FunctionCall { function, args, .. } => {
                let call_cost = self.get_cost("function_call");
                let function_cost = self.profile_expr(function, cost_breakdown);

                let mut args_cost = 0;
                for arg in args {
                    args_cost += self.profile_expr(arg, cost_breakdown);
                }

                // Check for special functions with known gas costs
                let special_cost = if let Expr::Variable { name, .. } = &**function {
                    if name.starts_with("IO/storage_") {
                        if name == "IO/storage_read" {
                            self.get_cost("storage_read")
                        } else if name == "IO/storage_write" {
                            self.get_cost("storage_write")
                        } else if name == "IO/storage_delete" {
                            self.get_cost("storage_delete")
                        } else {
                            0
                        }
                    } else if name.starts_with("IO/call") || name.starts_with("IO/static_call") {
                        self.get_cost("external_call")
                    } else if name.starts_with("IO/emit_event") {
                        self.get_cost("event_emit")
                    } else {
                        0
                    }
                } else {
                    0
                };

                if special_cost > 0 {
                    let operation = if let Expr::Variable { name, .. } = &**function {
                        name.to_string()
                    } else {
                        "special_operation".to_string()
                    };
                    *cost_breakdown.entry(operation).or_insert(0) += special_cost;
                } else {
                    *cost_breakdown
                        .entry("function_call".to_string())
                        .or_insert(0) += call_cost;
                }

                call_cost + function_cost + args_cost + special_cost
            }
            Expr::BinaryOp {
                left,
                operator,
                right,
                ..
            } => {
                let op_cost = self.get_cost("binary_op");
                let left_cost = self.profile_expr(left, cost_breakdown);
                let right_cost = self.profile_expr(right, cost_breakdown);

                *cost_breakdown.entry("binary_op".to_string()).or_insert(0) += op_cost;

                op_cost + left_cost + right_cost
            }
            Expr::Tuple { elements, .. } => {
                let tuple_cost = self.get_cost("tuple");

                let mut elements_cost = 0;
                for element in elements {
                    elements_cost += self.profile_expr(element, cost_breakdown);
                }

                *cost_breakdown.entry("tuple".to_string()).or_insert(0) += tuple_cost;

                tuple_cost + elements_cost
            }
            Expr::List { elements, .. } => {
                let list_cost = self.get_cost("list");

                let mut elements_cost = 0;
                for element in elements {
                    elements_cost += self.profile_expr(element, cost_breakdown);
                }

                *cost_breakdown.entry("list".to_string()).or_insert(0) += list_cost;

                list_cost + elements_cost
            }
            Expr::Constructor { args, .. } => {
                let constructor_cost = self.get_cost("constructor");

                let mut args_cost = 0;
                for arg in args {
                    args_cost += self.profile_expr(arg, cost_breakdown);
                }

                *cost_breakdown.entry("constructor".to_string()).or_insert(0) += constructor_cost;

                constructor_cost + args_cost
            }
            // Add gas estimates for other expression types
            _ => 5, // Default cost for unhandled expression types
        }
    }

    /// Check if a function is recursive
    fn is_function_recursive(&self, name: &str, body: &Block) -> bool {
        // Simplified check: look for calls to the function itself
        self.contains_call_to(body, name)
    }

    /// Check if a block contains a call to a specific function
    fn contains_call_to(&self, block: &Block, function_name: &str) -> bool {
        for statement in &block.statements {
            match statement {
                Statement::Expr { expr, .. } => {
                    if self.expr_calls_function(expr, function_name) {
                        return true;
                    }
                }
                Statement::Return { value, .. } => {
                    if self.expr_calls_function(value, function_name) {
                        return true;
                    }
                }
                Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    if self.expr_calls_function(condition, function_name)
                        || self.contains_call_to(then_branch, function_name)
                        || self.contains_call_to(else_branch, function_name)
                    {
                        return true;
                    }
                }
                Statement::Match { value, cases, .. } => {
                    if self.expr_calls_function(value, function_name) {
                        return true;
                    }

                    for case in cases {
                        if self.contains_call_to(&case.body, function_name) {
                            return true;
                        }
                    }
                }
                Statement::Bend {
                    condition,
                    initial_states,
                    body,
                    else_body,
                    ..
                } => {
                    if self.expr_calls_function(condition, function_name) {
                        return true;
                    }

                    for (_, expr) in initial_states {
                        if self.expr_calls_function(expr, function_name) {
                            return true;
                        }
                    }

                    if self.contains_call_to(body, function_name) {
                        return true;
                    }

                    if else_body.map_or(false, |b| self.contains_call_to(&b, function_name)) {
                        return true;
                    }
                }
                Statement::With { body, .. } => {
                    if self.contains_call_to(body, function_name) {
                        return true;
                    }
                }
                // Check other statement types
                _ => {}
            }
        }

        false
    }

    /// Check if an expression calls a specific function
    fn expr_calls_function(&self, expr: &Expr, function_name: &str) -> bool {
        match expr {
            Expr::FunctionCall { function, args, .. } => {
                let calls_function = if let Expr::Variable { name, .. } = &**function {
                    name == function_name
                } else {
                    self.expr_calls_function(function, function_name)
                };

                if calls_function {
                    return true;
                }

                for arg in args {
                    if self.expr_calls_function(arg, function_name) {
                        return true;
                    }
                }

                false
            }
            Expr::BinaryOp { left, right, .. } => {
                self.expr_calls_function(left, function_name)
                    || self.expr_calls_function(right, function_name)
            }
            Expr::Tuple { elements, .. } => elements
                .iter()
                .any(|e| self.expr_calls_function(e, function_name)),
            Expr::List { elements, .. } => elements
                .iter()
                .any(|e| self.expr_calls_function(e, function_name)),
            Expr::Constructor { args, .. } => args
                .iter()
                .any(|e| self.expr_calls_function(e, function_name)),
            // Check other expression types
            _ => false,
        }
    }

    /// Check if a block has external calls
    fn has_external_calls(&self, block: &Block) -> bool {
        for statement in &block.statements {
            match statement {
                Statement::Expr { expr, .. } => {
                    if self.expr_has_external_call(expr) {
                        return true;
                    }
                }
                Statement::Return { value, .. } => {
                    if self.expr_has_external_call(value) {
                        return true;
                    }
                }
                Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    if self.expr_has_external_call(condition)
                        || self.has_external_calls(then_branch)
                        || self.has_external_calls(else_branch)
                    {
                        return true;
                    }
                }
                Statement::Match { value, cases, .. } => {
                    if self.expr_has_external_call(value) {
                        return true;
                    }

                    for case in cases {
                        if self.has_external_calls(&case.body) {
                            return true;
                        }
                    }
                }
                Statement::Bend {
                    condition,
                    initial_states,
                    body,
                    else_body,
                    ..
                } => {
                    if self.expr_has_external_call(condition) {
                        return true;
                    }

                    for (_, expr) in initial_states {
                        if self.expr_has_external_call(expr) {
                            return true;
                        }
                    }

                    if self.has_external_calls(body) {
                        return true;
                    }

                    if else_body.map_or(false, |b| self.has_external_calls(&b)) {
                        return true;
                    }
                }
                Statement::With { body, .. } => {
                    if self.has_external_calls(body) {
                        return true;
                    }
                }
                // Check other statement types
                _ => {}
            }
        }

        false
    }

    /// Check if an expression has an external call
    fn expr_has_external_call(&self, expr: &Expr) -> bool {
        match expr {
            Expr::FunctionCall { function, args, .. } => {
                // Check if this is an external call function
                let is_external_call = if let Expr::Variable { name, .. } = &**function {
                    name.starts_with("IO/call")
                        || name.starts_with("IO/static_call")
                        || name.starts_with("IO/delegatecall")
                        || name.starts_with("transfer")
                } else {
                    false
                };

                if is_external_call {
                    return true;
                }

                // Check recursively
                if self.expr_has_external_call(function) {
                    return true;
                }

                for arg in args {
                    if self.expr_has_external_call(arg) {
                        return true;
                    }
                }

                false
            }
            Expr::BinaryOp { left, right, .. } => {
                self.expr_has_external_call(left) || self.expr_has_external_call(right)
            }
            Expr::Tuple { elements, .. } => elements.iter().any(|e| self.expr_has_external_call(e)),
            Expr::List { elements, .. } => elements.iter().any(|e| self.expr_has_external_call(e)),
            Expr::Constructor { args, .. } => args.iter().any(|e| self.expr_has_external_call(e)),
            // Check other expression types
            _ => false,
        }
    }

    /// Get cost for an operation
    fn get_cost(&self, operation: &str) -> u64 {
        *self.costs.get(operation).unwrap_or(&5)
    }
}

/// Count the number of lines in a block
fn count_lines(block: &Block) -> usize {
    let mut line_count = 0;

    for statement in &block.statements {
        line_count += match statement {
            Statement::If {
                then_branch,
                else_branch,
                ..
            } => 2 + count_lines(then_branch) + count_lines(else_branch),
            Statement::Match { cases, .. } => {
                1 + cases
                    .iter()
                    .map(|case| 1 + count_lines(&case.body))
                    .sum::<usize>()
            }
            Statement::Bend {
                body, else_body, ..
            } => 2 + count_lines(body) + else_body.map_or(0, |b| count_lines(&b)),
            Statement::With { body, .. } => 1 + count_lines(body),
            _ => 1,
        };
    }

    line_count
}

/// Print a gas profile
pub fn print_profile(profile: &GasProfile) {
    println!("Gas profile report for {}", profile.file_path);
    println!("-------------------------------------");

    if profile.estimates.is_empty() {
        println!("No functions found to profile.");
    } else {
        println!("Function gas estimates:");

        for (i, estimate) in profile.estimates.iter().enumerate() {
            println!(
                "{}. {} (lines {}-{})",
                i + 1,
                estimate.name,
                estimate.line_range.0,
                estimate.line_range.1
            );
            println!("   Base gas: {}", estimate.base_cost);
            println!("   Max gas: {}", estimate.max_cost);
            println!("   Avg gas: {}", estimate.avg_cost);

            if estimate.is_recursive {
                println!("   ⚠️ Recursive function (gas estimate may be inaccurate)");
            }

            if estimate.has_external_calls {
                println!("   ⚠️ Contains external calls (gas depends on called contracts)");
            }

            println!("   Cost breakdown:");
            for (operation, cost) in &estimate.cost_breakdown {
                println!("     - {}: {} gas", operation, cost);
            }

            println!();
        }

        println!("Total estimated gas: {}", profile.total_gas);

        if let Some(most_expensive) = &profile.most_expensive_function {
            println!("Most expensive function: {}", most_expensive);
        }
    }
}
