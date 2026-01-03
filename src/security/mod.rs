/// Bend-PVM Security Framework
/// 
/// This module provides comprehensive security features for the Bend-PVM ecosystem:
/// - SafeMath: Overflow/underflow protection for arithmetic operations
/// - Input Validation: Sanitization and validation of user inputs
/// - Access Control: Role-based access control (RBAC) system
/// - Reentrancy Guard: Protection against reentrancy attacks
/// - Gas Metering: Gas limit enforcement and optimization
/// - Security Scanner: Vulnerability detection and analysis
/// - Static Analysis: Code security analysis
/// - Fuzz Testing: Automated security testing framework

pub mod safe_math;
pub mod validation;
pub mod access_control;
pub mod reentrancy_guard;
pub mod gas_metering;
pub mod security_scanner;
pub mod static_analysis;
pub mod fuzz_testing;

use crate::compiler::parser::ast::*;
use crate::runtime::env::Environment;
use crate::runtime::metering::MeteringError;
use thiserror::Error;

/// Security severity levels
#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Security framework errors
#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Access denied: {0}")]
    AccessDenied(String),
    
    #[error("Reentrancy detected")]
    ReentrancyDetected,
    
    #[error("Gas limit exceeded: {0}")]
    GasLimitExceeded(u64),
    
    #[error("Input validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Static analysis error: {0}")]
    StaticAnalysisError(String),
}

// Implement From<TestError> for SecurityError
impl From<fuzz_testing::TestError> for SecurityError {
    fn from(test_error: fuzz_testing::TestError) -> Self {
        SecurityError::SecurityViolation(format!("Test error: {:?}", test_error))
    }
}

/// Security configuration
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub gas_limit: u64,
    pub max_call_depth: u32,
    pub enable_access_control: bool,
    pub enable_reentrancy_guard: bool,
    pub enable_input_validation: bool,
    pub enable_static_analysis: bool,
    pub enable_fuzz_testing: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            gas_limit: 10_000_000,
            max_call_depth: 100,
            enable_access_control: true,
            enable_reentrancy_guard: true,
            enable_input_validation: true,
            enable_static_analysis: true,
            enable_fuzz_testing: false, // Disabled by default for performance
        }
    }
}

/// Main security manager for Bend-PVM
pub struct SecurityManager {
    pub config: SecurityConfig,
    gas_meter: gas_metering::GasMeter,
    access_control: access_control::AccessControl,
    reentrancy_guard: reentrancy_guard::ReentrancyGuard,
    input_validator: validation::InputValidator,
    security_scanner: security_scanner::SecurityScanner,
    static_analyzer: static_analysis::StaticAnalyzer,
    fuzz_tester: fuzz_testing::FuzzTester,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            config: config.clone(),
            gas_meter: gas_metering::GasMeter::new(config.gas_limit),
            access_control: access_control::AccessControl::new(),
            reentrancy_guard: reentrancy_guard::ReentrancyGuard::new(),
            input_validator: validation::InputValidator::new(),
            security_scanner: security_scanner::SecurityScanner::new(),
            static_analyzer: static_analysis::StaticAnalyzer::new(),
            fuzz_tester: fuzz_testing::FuzzTester::new(),
        }
    }

    /// Validate a program before execution
    pub fn validate_program(&mut self, program: &Program) -> Result<(), SecurityError> {
        // Input validation
        if self.config.enable_input_validation {
            self.input_validator.validate_program(program)?;
        }

        // Static analysis
        if self.config.enable_static_analysis {
            self.static_analyzer.analyze_program(program)?;
        }

        // Security scanning
        self.security_scanner.scan_program(program)?;

        Ok(())
    }

    /// Check access permissions
    pub fn check_access(&mut self, caller: &[u8], resource: &str, operation: &str) -> Result<(), SecurityError> {
        if !self.config.enable_access_control {
            return Ok(());
        }
        self.access_control.check_permission(caller, resource, operation)
    }

    /// Enter secure execution context
    pub fn enter_execution(&mut self, function: &str) -> Result<(), SecurityError> {
        // Check reentrancy
        if self.config.enable_reentrancy_guard {
            self.reentrancy_guard.enter_function(function)?;
        }

        // Check gas limits
        self.gas_meter.check_gas_limit()?;

        Ok(())
    }

    /// Exit secure execution context
    pub fn exit_execution(&mut self, function: &str) {
        if self.config.enable_reentrancy_guard {
            self.reentrancy_guard.exit_function(function);
        }
    }

    /// Consume gas
    pub fn consume_gas(&mut self, amount: u64) -> Result<(), SecurityError> {
        self.gas_meter.consume_gas(amount)
    }

    /// Get remaining gas
    pub fn remaining_gas(&self) -> u64 {
        self.gas_meter.remaining_gas()
    }

    /// Run security tests
    pub fn run_security_tests(&mut self, program: &Program) -> Result<(), SecurityError> {
        if self.config.enable_fuzz_testing {
            self.fuzz_tester.fuzz_program(program)?;
        }
        Ok(())
    }

    /// Get security report
    pub fn get_security_report(&self) -> SecurityReport {
        SecurityReport {
            gas_consumed: self.gas_meter.total_consumed(),
            reentrancy_attempts: self.reentrancy_guard.get_attempt_count(),
            validation_failures: self.input_validator.get_failure_count(),
            security_violations: self.security_scanner.get_violation_count(),
        }
    }
}

/// Security report
#[derive(Debug, Clone)]
pub struct SecurityReport {
    pub gas_consumed: u64,
    pub reentrancy_attempts: u32,
    pub validation_failures: u32,
    pub security_violations: u32,
}

/// Initialize security framework in environment
pub fn init_security(env: &mut Environment, config: SecurityConfig) -> Result<(), MeteringError> {
    // Security framework is integrated into the runtime environment
    // Additional initialization can be added here
    Ok(())
}

/// Register security modules
pub fn register_security_modules() -> Vec<Definition> {
    let mut definitions = Vec::new();
    
    // Register safe math operations
    definitions.extend(safe_math::register_safe_math());
    
    // Register validation functions
    definitions.extend(validation::register_validation_functions());
    
    // Register access control functions
    definitions.extend(access_control::register_access_control_functions());
    
    definitions
}