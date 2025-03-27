use std::path::Path;
use std::fs;
use std::collections::HashMap;
use thiserror::Error;

use bend_pvm::compiler::parser::ast::*;
use bend_pvm::compiler::parser::parser::Parser;

/// Error types for security scanning
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Unsafe function call: {function} at line {line}, column {column}")]
    UnsafeCall {
        function: String,
        line: usize,
        column: usize,
    },
    
    #[error("Unbounded loop: at line {line}, column {column}")]
    UnboundedLoop {
        line: usize,
        column: usize,
    },
    
    #[error("Reentrancy vulnerability: at line {line}, column {column}")]
    Reentrancy {
        line: usize,
        column: usize,
    },
    
    #[error("Unchecked return value: at line {line}, column {column}")]
    UncheckedReturnValue {
        line: usize,
        column: usize,
    },
    
    #[error("Integer overflow/underflow vulnerability: at line {line}, column {column}")]
    IntegerOverflow {
        line: usize,
        column: usize,
    },
}

/// Result of security scanning
#[derive(Debug)]
pub struct SecurityReport {
    /// Issues found during scanning
    pub issues: Vec<SecurityError>,
    
    /// Path to the scanned file
    pub file_path: String,
    
    /// Scan summary
    pub summary: String,
}

/// Security scanner for Bend contracts
pub struct SecurityScanner {
    /// List of unsafe function calls
    unsafe_functions: HashMap<String, String>,
    
    /// List of functions that require return value checking
    requires_check: HashMap<String, String>,
}

impl SecurityScanner {
    /// Create a new security scanner
    pub fn new() -> Self {
        let mut unsafe_functions = HashMap::new();
        unsafe_functions.insert("IO/call_raw".to_string(), "Can execute arbitrary code".to_string());
        unsafe_functions.insert("IO/delegatecall".to_string(), "Delegatecall is dangerous".to_string());
        unsafe_functions.insert("IO/selfdestruct".to_string(), "Can destroy the contract".to_string());
        
        let mut requires_check = HashMap::new();
        requires_check.insert("transfer".to_string(), "Transfer can fail".to_string());
        requires_check.insert("IO/call".to_string(), "Call can fail".to_string());
        
        SecurityScanner {
            unsafe_functions,
            requires_check,
        }
    }
    
    /// Scan a file for security vulnerabilities
    pub fn scan_file<P: AsRef<Path>>(&self, file_path: P) -> Result<SecurityReport, SecurityError> {
        let source = fs::read_to_string(&file_path)?;
        let file_path_str = file_path.as_ref().to_string_lossy().to_string();
        
        self.scan_source(&source, &file_path_str)
    }
    
    /// Scan source code for security vulnerabilities
    pub fn scan_source(&self, source: &str, file_path: &str) -> Result<SecurityReport, SecurityError> {
        // Parse the source
        let mut parser = Parser::new(source);
        let program = parser.parse_program()
            .map_err(|e| SecurityError::Parse(e.to_string()))?;
        
        // Scan for issues
        let mut issues = Vec::new();
        
        // Scan each definition
        for definition in &program.definitions {
            match definition {
                Definition::FunctionDef { name, body, .. } => {
                    // Scan the function body
                    self.scan_block(body, &mut issues);
                }
                _ => {}
            }
        }
        
        // Create the report
        let summary = if issues.is_empty() {
            format!("No security issues found in {}", file_path)
        } else {
            format!("Found {} security issues in {}", issues.len(), file_path)
        };
        
        Ok(SecurityReport {
            issues,
            file_path: file_path.to_string(),
            summary,
        })
    }
    
    /// Scan a block for security vulnerabilities
    fn scan_block(&self, block: &Block, issues: &mut Vec<SecurityError>) {
        for statement in &block.statements {
            self.scan_statement(statement, issues);
        }
    }
    
    /// Scan a statement for security vulnerabilities
    fn scan_statement(&self, statement: &Statement, issues: &mut Vec<SecurityError>) {
        match statement {
            Statement::Expr { expr, .. } => {
                self.scan_expr(expr, issues);
            }
            Statement::FunctionCall { function, args, location, .. } => {
                // Check for unsafe function calls
                if let Expr::Variable { name, .. } = &**function {
                    if let Some(reason) = self.unsafe_functions.get(name) {
                        issues.push(SecurityError::UnsafeCall {
                            function: name.clone(),
                            line: location.line,
                            column: location.column,
                        });
                    }
                    
                    // Check for unchecked return values
                    if let Some(reason) = self.requires_check.get(name) {
                        // Check if this call is being checked
                        if !self.is_call_checked(statement) {
                            issues.push(SecurityError::UncheckedReturnValue {
                                line: location.line,
                                column: location.column,
                            });
                        }
                    }
                }
                
                // Scan function expression
                self.scan_expr(function, issues);
                
                // Scan arguments
                for arg in args {
                    self.scan_expr(arg, issues);
                }
            }
            Statement::If { condition, then_branch, else_branch, .. } => {
                self.scan_expr(condition, issues);
                self.scan_block(then_branch, issues);
                if let Some(else_branch) = else_branch {
                    self.scan_block(else_branch, issues);
                }
            }
            Statement::Bend { condition, initial_states, body, else_body, location, .. } => {
                // Check for unbounded loops
                if !self.has_termination_condition(condition, initial_states) {
                    issues.push(SecurityError::UnboundedLoop {
                        line: location.line,
                        column: location.column,
                    });
                }
                
                // Scan condition and branches
                self.scan_expr(condition, issues);
                for (_, expr) in initial_states {
                    self.scan_expr(expr, issues);
                }
                self.scan_block(body, issues);
                if let Some(else_body) = else_body {
                    self.scan_block(else_body, issues);
                }
            }
            Statement::BinaryOp { left, operator, right, location, .. } => {
                // Check for integer overflow/underflow
                match operator {
                    BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul => {
                        // Check if there's overflow protection
                        if !self.has_overflow_protection(left, right, operator) {
                            issues.push(SecurityError::IntegerOverflow {
                                line: location.line,
                                column: location.column,
                            });
                        }
                    }
                    _ => {}
                }
                
                // Scan operands
                self.scan_expr(left, issues);
                self.scan_expr(right, issues);
            }
            Statement::With { monad_type, body, location, .. } => {
                // Check for reentrancy vulnerabilities
                if monad_type == "IO" && self.has_external_calls(body) && self.modifies_state_after_call(body) {
                    issues.push(SecurityError::Reentrancy {
                        line: location.line,
                        column: location.column,
                    });
                }
                
                // Scan body
                self.scan_block(body, issues);
            }
            // Add checks for other statement types
            _ => {}
        }
    }
    
    /// Scan an expression for security vulnerabilities
    fn scan_expr(&self, expr: &Expr, issues: &mut Vec<SecurityError>) {
        match expr {
            Expr::FunctionCall { function, args, location, .. } => {
                // Check for unsafe function calls
                if let Expr::Variable { name, .. } = &**function {
                    if let Some(reason) = self.unsafe_functions.get(name) {
                        issues.push(SecurityError::UnsafeCall {
                            function: name.clone(),
                            line: location.line,
                            column: location.column,
                        });
                    }
                    
                    // Check for unchecked return values
                    if let Some(reason) = self.requires_check.get(name) {
                        // Check if this call is being checked
                        if !self.is_expr_checked(expr) {
                            issues.push(SecurityError::UncheckedReturnValue {
                                line: location.line,
                                column: location.column,
                            });
                        }
                    }
                }
                
                // Scan function expression
                self.scan_expr(function, issues);
                
                // Scan arguments
                for arg in args {
                    self.scan_expr(arg, issues);
                }
            }
            Expr::BinaryOp { left, operator, right, location, .. } => {
                // Check for integer overflow/underflow
                match operator {
                    BinaryOperator::Add | BinaryOperator::Sub | BinaryOperator::Mul => {
                        // Check if there's overflow protection
                        if !self.has_overflow_protection(left, right, operator) {
                            issues.push(SecurityError::IntegerOverflow {
                                line: location.line,
                                column: location.column,
                            });
                        }
                    }
                    _ => {}
                }
                
                // Scan operands
                self.scan_expr(left, issues);
                self.scan_expr(right, issues);
            }
            // Add checks for other expression types
            _ => {}
        }
    }
    
    /// Check if a call is being checked for errors
    fn is_call_checked(&self, statement: &Statement) -> bool {
        // This is a simplified check that would need to be more sophisticated in a real implementation
        // For example, it could check if the call result is used in an if statement
        false
    }
    
    /// Check if an expression is being checked for errors
    fn is_expr_checked(&self, expr: &Expr) -> bool {
        // This is a simplified check that would need to be more sophisticated in a real implementation
        false
    }
    
    /// Check if a bend loop has a termination condition
    fn has_termination_condition(&self, condition: &Expr, initial_states: &[(String, Expr)]) -> bool {
        // This is a simplified check that would need to be more sophisticated in a real implementation
        // For example, it could check if there's a condition that will eventually become false
        true
    }
    
    /// Check if there's overflow protection for an arithmetic operation
    fn has_overflow_protection(&self, left: &Expr, right: &Expr, operator: &BinaryOperator) -> bool {
        // This is a simplified check that would need to be more sophisticated in a real implementation
        // For example, it could check if there's a bounds check before the operation
        false
    }
    
    /// Check if a block has external calls
    fn has_external_calls(&self, block: &Block) -> bool {
        // This is a simplified check that would need to be more sophisticated in a real implementation
        // For example, it could check for calls to external contracts
        false
    }
    
    /// Check if a block modifies state after an external call (reentrancy vulnerability)
    fn modifies_state_after_call(&self, block: &Block) -> bool {
        // This is a simplified check that would need to be more sophisticated in a real implementation
        // For example, it could check if state is modified after an external call
        false
    }
}

/// Print a security report
pub fn print_report(report: &SecurityReport) {
    println!("Security scan report for {}", report.file_path);
    println!("-------------------------------------");
    
    if report.issues.is_empty() {
        println!("No security issues found.");
    } else {
        println!("Found {} security issues:", report.issues.len());
        
        for (i, issue) in report.issues.iter().enumerate() {
            println!("{}. {}", i + 1, issue);
        }
    }
    
    println!("\nSummary: {}", report.summary);
}