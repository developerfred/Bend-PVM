/// Security Scanner module
///
/// Provides comprehensive vulnerability detection and security scanning
/// for Bend-PVM programs to identify potential security risks.
use crate::compiler::parser::ast::*;
use crate::security::SecurityError;
use regex::Regex;
use std::collections::{HashMap, HashSet, VecDeque};

/// Security vulnerability types
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum VulnerabilityType {
    IntegerOverflow,
    IntegerUnderflow,
    Reentrancy,
    UncheckedCallReturn,
    TimestampDependence,
    BlockNumberDependence,
    UnprotectedExternalCall,
    StateVariableShadowing,
    UnprotectedSelfdestruct,
    DoS,
    AccessControl,
    InputValidation,
    OracleManipulation,
    FrontRunning,
    ReplayAttack,
    UnboundedLoop,
    MemoryArraySize,
    UncheckedArrayAccess,
    UnprotectedFallback,
    IntegerDivision,
    FloatingPoint,
    UncheckedLowLevelCall,
    DeprecatedFunctions,
    MaliciousLibraries,
    UninitializedStorage,
    UnprotectedEtherWithdrawal,
    TimeManipulation,
    Randomness,
    SignatureReplay,
    UncheckedSend,
    StateChangeAfterExternalCall,
    UnprotectedDelegateCall,
    GasLimitManipulation,
    MEV,
    UnlimitedApprove,
    IntegerCasting,
    UnexpectedRevert,
    UnrecoverableError,
}

/// Security vulnerability
#[derive(Debug, Clone)]
pub struct Vulnerability {
    pub vuln_type: VulnerabilityType,
    pub severity: SecuritySeverity,
    pub location: Location,
    pub description: String,
    pub recommendation: String,
    pub confidence: f64, // 0.0 to 1.0
}

/// Security severity levels
#[derive(Debug, Clone, PartialEq)]
pub enum SecuritySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Security scan result
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub total_vulnerabilities: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub info_count: usize,
    pub vulnerabilities: Vec<Vulnerability>,
    pub scan_duration_ms: u64,
    pub coverage_percentage: f64,
}

/// Security scanner
pub struct SecurityScanner {
    vuln_patterns: HashMap<VulnerabilityType, Vec<Regex>>,
    scan_history: VecDeque<ScanResult>,
    violation_count: u32,
    ignored_patterns: HashSet<String>,
}

impl Default for SecurityScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityScanner {
    /// Create a new security scanner
    pub fn new() -> Self {
        let mut scanner = Self {
            vuln_patterns: HashMap::new(),
            scan_history: VecDeque::new(),
            violation_count: 0,
            ignored_patterns: HashSet::new(),
        };

        scanner.initialize_patterns();
        scanner
    }

    /// Initialize vulnerability detection patterns
    fn initialize_patterns(&mut self) {
        // Integer overflow patterns
        let overflow_patterns = vec![
            Regex::new(r"\+\s*\d+").unwrap(),
            Regex::new(r"\*\s*\d+").unwrap(),
        ];
        self.vuln_patterns
            .insert(VulnerabilityType::IntegerOverflow, overflow_patterns);

        // Integer underflow patterns
        let underflow_patterns = vec![Regex::new(r"-\s*\d+").unwrap()];
        self.vuln_patterns
            .insert(VulnerabilityType::IntegerUnderflow, underflow_patterns);

        // Reentrancy patterns
        let reentrancy_patterns = vec![
            Regex::new(r"call\(").unwrap(),
            Regex::new(r"delegatecall\(").unwrap(),
            Regex::new(r"callcode\(").unwrap(),
        ];
        self.vuln_patterns
            .insert(VulnerabilityType::Reentrancy, reentrancy_patterns);

        // Unchecked call return patterns
        let unchecked_patterns = vec![
            Regex::new(r"\.call\(").unwrap(),
            Regex::new(r"\.send\(").unwrap(),
            Regex::new(r"\.transfer\(").unwrap(),
        ];
        self.vuln_patterns
            .insert(VulnerabilityType::UncheckedCallReturn, unchecked_patterns);

        // Timestamp dependence patterns
        let timestamp_patterns = vec![
            Regex::new(r"block\.timestamp").unwrap(),
            Regex::new(r"now").unwrap(),
            Regex::new(r"block\.number").unwrap(),
        ];
        self.vuln_patterns
            .insert(VulnerabilityType::TimestampDependence, timestamp_patterns);

        // Unprotected external call patterns
        let external_call_patterns = vec![
            Regex::new(r"call\(").unwrap(),
            Regex::new(r"delegatecall\(").unwrap(),
            Regex::new(r"staticcall\(").unwrap(),
        ];
        self.vuln_patterns.insert(
            VulnerabilityType::UnprotectedExternalCall,
            external_call_patterns,
        );

        // Unbounded loop patterns
        let unbounded_patterns = vec![
            Regex::new(r"for\s*\(").unwrap(),
            Regex::new(r"while\s*\(").unwrap(),
        ];
        self.vuln_patterns
            .insert(VulnerabilityType::UnboundedLoop, unbounded_patterns);

        // Array access patterns
        let array_patterns = vec![
            Regex::new(r"\[\s*\]").unwrap(),
            Regex::new(r"\.push\(").unwrap(),
            Regex::new(r"\.pop\(").unwrap(),
        ];
        self.vuln_patterns
            .insert(VulnerabilityType::UncheckedArrayAccess, array_patterns);
    }

    /// Scan a program for vulnerabilities
    pub fn scan_program(&mut self, program: &Program) -> Result<ScanResult, SecurityError> {
        let start_time = std::time::Instant::now();
        let mut vulnerabilities = Vec::new();

        // Scan all definitions in the program
        for definition in &program.definitions {
            self.scan_definition(definition, &mut vulnerabilities)?;
        }

        // Calculate severity counts
        let mut critical_count = 0;
        let mut high_count = 0;
        let mut medium_count = 0;
        let mut low_count = 0;
        let mut info_count = 0;

        for vuln in &vulnerabilities {
            match vuln.severity {
                SecuritySeverity::Critical => critical_count += 1,
                SecuritySeverity::High => high_count += 1,
                SecuritySeverity::Medium => medium_count += 1,
                SecuritySeverity::Low => low_count += 1,
                SecuritySeverity::Info => info_count += 1,
            }
        }

        let scan_duration = start_time.elapsed().as_millis() as u64;

        // Estimate coverage (simplified)
        let coverage_percentage = 75.0; // Placeholder calculation

        let scan_result = ScanResult {
            total_vulnerabilities: vulnerabilities.len(),
            critical_count,
            high_count,
            medium_count,
            low_count,
            info_count,
            vulnerabilities,
            scan_duration_ms: scan_duration,
            coverage_percentage,
        };

        // Store in history
        self.scan_history.push_back(scan_result.clone());
        if self.scan_history.len() > 10 {
            self.scan_history.pop_front();
        }

        self.violation_count += scan_result.total_vulnerabilities as u32;

        Ok(scan_result)
    }

    /// Scan a definition for vulnerabilities
    fn scan_definition(
        &self,
        definition: &Definition,
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        match definition {
            Definition::FunctionDef {
                name, body, params, ..
            } => {
                self.scan_function(name, body, params, vulnerabilities)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Scan a function for vulnerabilities
    fn scan_function(
        &self,
        name: &str,
        body: &Block,
        params: &[Parameter],
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        // Scan function body
        self.scan_block(body, vulnerabilities)?;

        // Check for parameter-related vulnerabilities
        self.scan_parameters(params, vulnerabilities)?;

        // Check for function-specific patterns
        self.check_function_patterns(name, body, vulnerabilities)?;

        Ok(())
    }

    /// Scan a block for vulnerabilities
    fn scan_block(
        &self,
        block: &Block,
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        for statement in &block.statements {
            self.scan_statement(statement, vulnerabilities)?;
        }
        Ok(())
    }

    /// Scan a statement for vulnerabilities
    fn scan_statement(
        &self,
        statement: &Statement,
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        match statement {
            Statement::Assignment { pattern, value, .. } => {
                self.scan_expression(value, vulnerabilities)?;
                self.check_assignment_patterns(pattern, value, vulnerabilities)?;
            }
            Statement::Expr { expr, .. } => {
                self.scan_expression(expr, vulnerabilities)?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Scan an expression for vulnerabilities
    fn scan_expression(
        &self,
        expr: &Expr,
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        // Pattern-based scanning
        let expr_str = format!("{:?}", expr);

        for (vuln_type, patterns) in &self.vuln_patterns {
            for pattern in patterns {
                if pattern.is_match(&expr_str) {
                    self.create_vulnerability(
                        vuln_type,
                        expr.location().clone(),
                        &expr_str,
                        vulnerabilities,
                    )?;
                }
            }
        }

        // Recursive scanning for nested expressions
        match expr {
            Expr::BinaryOp { left, right, .. } => {
                self.scan_expression(left, vulnerabilities)?;
                self.scan_expression(right, vulnerabilities)?;
            }
            Expr::FunctionCall { function, args, .. } => {
                self.scan_expression(function, vulnerabilities)?;
                for arg in args {
                    self.scan_expression(arg, vulnerabilities)?;
                }
            }
            Expr::List { elements, .. } => {
                for element in elements {
                    self.scan_expression(element, vulnerabilities)?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Scan function parameters for vulnerabilities
    fn scan_parameters(
        &self,
        params: &[Parameter],
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        for param in params {
            // Check for parameters that might be used unsafely
            if param.name.to_lowercase().contains("value")
                || param.name.to_lowercase().contains("amount")
            {
                self.create_vulnerability(
                    &VulnerabilityType::InputValidation,
                    param.location.clone(),
                    &format!("Parameter '{}' should be validated", param.name),
                    vulnerabilities,
                )?;
            }
        }
        Ok(())
    }

    /// Check for function-specific vulnerability patterns
    fn check_function_patterns(
        &self,
        name: &str,
        body: &Block,
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        // Check for unprotected selfdestruct
        if name.to_lowercase().contains("kill") || name.to_lowercase().contains("destroy") {
            self.create_vulnerability(
                &VulnerabilityType::UnprotectedSelfdestruct,
                body.location.clone(),
                "Function that may selfdestruct should have access controls",
                vulnerabilities,
            )?;
        }

        // Check for fallback function
        if name == "fallback" || name == "receive" {
            self.create_vulnerability(
                &VulnerabilityType::UnprotectedFallback,
                body.location.clone(),
                "Fallback function should have proper access controls and validation",
                vulnerabilities,
            )?;
        }

        Ok(())
    }

    /// Check assignment patterns for vulnerabilities
    fn check_assignment_patterns(
        &self,
        target: &Pattern,
        value: &Expr,
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        // Check for state variable assignment patterns
        if let Pattern::Variable { name, .. } = target {
            if name.starts_with('_') || name.to_lowercase().contains("state") {
                // This might be a state variable assignment
                if let Expr::FunctionCall { function, .. } = value {
                    if let Expr::Variable {
                        name: func_name, ..
                    } = function.as_ref()
                    {
                        if func_name == "msg.sender" || func_name == "tx.origin" {
                            self.create_vulnerability(
                                &VulnerabilityType::AccessControl,
                                value.location().clone(),
                                "State variable assignment based on msg.sender should be validated",
                                vulnerabilities,
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Create a vulnerability entry
    fn create_vulnerability(
        &self,
        vuln_type: &VulnerabilityType,
        location: Location,
        context: &str,
        vulnerabilities: &mut Vec<Vulnerability>,
    ) -> Result<(), SecurityError> {
        let (severity, description, recommendation, confidence) =
            self.get_vulnerability_info(vuln_type, context);

        vulnerabilities.push(Vulnerability {
            vuln_type: vuln_type.clone(),
            severity,
            location,
            description,
            recommendation,
            confidence,
        });

        Ok(())
    }

    /// Get vulnerability information based on type
    fn get_vulnerability_info(
        &self,
        vuln_type: &VulnerabilityType,
        context: &str,
    ) -> (SecuritySeverity, String, String, f64) {
        match vuln_type {
            VulnerabilityType::IntegerOverflow => (
                SecuritySeverity::High,
                format!("Potential integer overflow: {}", context),
                "Use SafeMath or checked arithmetic operations".to_string(),
                0.8,
            ),
            VulnerabilityType::IntegerUnderflow => (
                SecuritySeverity::High,
                format!("Potential integer underflow: {}", context),
                "Use SafeMath or checked arithmetic operations".to_string(),
                0.8,
            ),
            VulnerabilityType::Reentrancy => (
                SecuritySeverity::Critical,
                format!("Potential reentrancy vulnerability: {}", context),
                "Use reentrancy guards and check-effects-interactions pattern".to_string(),
                0.9,
            ),
            VulnerabilityType::UncheckedCallReturn => (
                SecuritySeverity::High,
                format!("Unchecked external call return value: {}", context),
                "Always check return values of external calls".to_string(),
                0.9,
            ),
            VulnerabilityType::TimestampDependence => (
                SecuritySeverity::Medium,
                format!("Timestamp dependence detected: {}", context),
                "Avoid using block.timestamp for critical logic".to_string(),
                0.7,
            ),
            VulnerabilityType::UnprotectedExternalCall => (
                SecuritySeverity::High,
                format!("Unprotected external call: {}", context),
                "Add access controls and validation for external calls".to_string(),
                0.8,
            ),
            VulnerabilityType::UnboundedLoop => (
                SecuritySeverity::Medium,
                format!("Potential unbounded loop: {}", context),
                "Ensure loops have bounded iterations".to_string(),
                0.6,
            ),
            VulnerabilityType::UncheckedArrayAccess => (
                SecuritySeverity::Medium,
                format!("Potential unchecked array access: {}", context),
                "Always check array bounds before access".to_string(),
                0.7,
            ),
            VulnerabilityType::AccessControl => (
                SecuritySeverity::High,
                format!("Access control issue: {}", context),
                "Implement proper access controls and validation".to_string(),
                0.8,
            ),
            VulnerabilityType::InputValidation => (
                SecuritySeverity::Medium,
                format!("Input validation needed: {}", context),
                "Validate all user inputs before use".to_string(),
                0.7,
            ),
            VulnerabilityType::UnprotectedFallback => (
                SecuritySeverity::High,
                format!("Unprotected fallback function: {}", context),
                "Implement proper validation and access controls in fallback".to_string(),
                0.9,
            ),
            VulnerabilityType::UnprotectedSelfdestruct => (
                SecuritySeverity::Critical,
                format!("Unprotected selfdestruct: {}", context),
                "Add access controls to prevent unauthorized selfdestruct".to_string(),
                0.9,
            ),
            _ => (
                SecuritySeverity::Low,
                format!("Potential security issue: {}", context),
                "Review this code for security implications".to_string(),
                0.5,
            ),
        }
    }

    /// Get scan history
    pub fn get_scan_history(&self) -> &VecDeque<ScanResult> {
        &self.scan_history
    }

    /// Get total violation count
    pub fn get_violation_count(&self) -> u32 {
        self.violation_count
    }

    /// Clear scan history
    pub fn clear_history(&mut self) {
        self.scan_history.clear();
    }

    /// Add pattern to ignore list
    pub fn ignore_pattern(&mut self, pattern: &str) {
        self.ignored_patterns.insert(pattern.to_string());
    }

    /// Remove pattern from ignore list
    pub fn unignore_pattern(&mut self, pattern: &str) {
        self.ignored_patterns.remove(pattern);
    }

    /// Get security score for a scan result
    pub fn calculate_security_score(&self, scan_result: &ScanResult) -> f64 {
        let total_issues = scan_result.total_vulnerabilities;
        if total_issues == 0 {
            return 100.0;
        }

        let weighted_score = (scan_result.critical_count * 10)
            + (scan_result.high_count * 5)
            + (scan_result.medium_count * 2)
            + (scan_result.low_count * 1);

        let max_possible_score = 100.0;
        let score = max_possible_score - (weighted_score as f64 * 2.0);
        score.max(0.0)
    }
}
