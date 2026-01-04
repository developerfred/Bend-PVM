//! # Solidity Contract Analyzer
//!
//! This module provides analysis capabilities for Solidity contracts
//! during migration, including compatibility checking and issue detection.

use super::ast::*;
use super::{IssueSeverity, MigrationIssue};
use serde::Serialize;

/// Analyzer for Solidity contracts
pub struct SolidityAnalyzer {
    /// Issues found during analysis
    issues: Vec<MigrationIssue>,
    /// Feature support matrix
    support_matrix: HashMap<String, SupportLevel>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SupportLevel {
    Supported,
    Partial,
    Unsupported,
    ManualRequired,
}

impl SolidityAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        let mut analyzer = SolidityAnalyzer {
            issues: Vec::new(),
            support_matrix: HashMap::new(),
        };
        analyzer.initialize_support_matrix();
        analyzer
    }

    /// Initialize the feature support matrix
    fn initialize_support_matrix(&mut self) {
        // Core features - fully supported
        self.support_matrix
            .insert("functions".to_string(), SupportLevel::Supported);
        self.support_matrix
            .insert("state_variables".to_string(), SupportLevel::Supported);
        self.support_matrix
            .insert("events".to_string(), SupportLevel::Supported);
        self.support_matrix
            .insert("enums".to_string(), SupportLevel::Supported);
        self.support_matrix
            .insert("structs".to_string(), SupportLevel::Supported);
        self.support_matrix
            .insert("mappings".to_string(), SupportLevel::Supported);
        self.support_matrix
            .insert("arrays".to_string(), SupportLevel::Supported);

        // Partial support features
        self.support_matrix
            .insert("inheritance".to_string(), SupportLevel::Partial);
        self.support_matrix
            .insert("modifiers".to_string(), SupportLevel::Partial);
        self.support_matrix
            .insert("interfaces".to_string(), SupportLevel::Partial);
        self.support_matrix
            .insert("libraries".to_string(), SupportLevel::Partial);

        // Manual required features
        self.support_matrix
            .insert("inline_assembly".to_string(), SupportLevel::ManualRequired);
        self.support_matrix
            .insert("assembly_blocks".to_string(), SupportLevel::ManualRequired);
        self.support_matrix.insert(
            "fallback_functions".to_string(),
            SupportLevel::ManualRequired,
        );
        self.support_matrix
            .insert("receive_ether".to_string(), SupportLevel::ManualRequired);

        // Unsupported features
        self.support_matrix
            .insert("dynamic_memory".to_string(), SupportLevel::Unsupported);
        self.support_matrix
            .insert("pointer_arithmetic".to_string(), SupportLevel::Unsupported);
        self.support_matrix.insert(
            "bit-level_operations".to_string(),
            SupportLevel::Unsupported,
        );
    }

    /// Analyze a Solidity source file
    pub fn analyze(&mut self, source: &SoliditySource) -> AnalysisResult {
        self.issues.clear();

        // Analyze each contract
        for contract in &source.contracts {
            self.analyze_contract(contract);
        }

        AnalysisResult {
            issues: self.issues.clone(),
            compatibility_score: self.calculate_compatibility_score(),
            estimated_gas_savings: self.estimate_gas_savings(source),
        }
    }

    /// Analyze a contract definition
    fn analyze_contract(&mut self, contract: &ContractDefinition) {
        // Check contract kind
        match contract.kind {
            ContractKind::Contract => {
                self.check_feature("inheritance", &contract.location);
            }
            ContractKind::Interface => {
                self.check_feature("interfaces", &contract.location);
            }
            ContractKind::Library => {
                self.check_feature("libraries", &contract.location);
            }
        }

        // Analyze base contracts
        for base in &contract.base_contracts {
            self.analyze_base_contract(base);
        }

        // Analyze state variables
        for var in &contract.state_variables {
            self.analyze_state_variable(var);
        }

        // Analyze functions
        for func in &contract.functions {
            self.analyze_function(func);
        }

        // Analyze modifiers
        for modifier in &contract.modifiers {
            self.analyze_modifier(modifier);
        }
    }

    /// Analyze a base contract reference
    fn analyze_base_contract(&mut self, base: &BaseContract) {
        // Check for arguments in base contract
        if !base.arguments.is_empty() {
            self.issues.push(MigrationIssue {
                description: format!("Base contract '{}' has constructor arguments", base.name),
                source_location: format!("{}:{}", base.location.line, base.location.column),
                severity: IssueSeverity::Manual,
                suggestion: Some(
                    "Ensure base contract constructor is called with correct arguments".to_string(),
                ),
            });
        }
    }

    /// Analyze a state variable
    fn analyze_state_variable(&mut self, var: &StateVariable) {
        // Check for mappings
        if let TypeName::Mapping(_) = &var.type_name {
            // Mappings are supported
        }

        // Check for arrays
        if let TypeName::Array(_) = &var.type_name {
            // Dynamic arrays need special handling
            self.issues.push(MigrationIssue {
                description: format!("Dynamic array '{}' may need migration", var.name),
                source_location: format!("{}:{}", var.location.line, var.location.column),
                severity: IssueSeverity::Partial,
                suggestion: Some("Consider using fixed-size arrays or List type".to_string()),
            });
        }

        // Check for public visibility with auto-generated getters
        if var.visibility == Visibility::Public {
            self.issues.push(MigrationIssue {
                description: format!("Public state variable '{}' needs getter function", var.name),
                source_location: format!("{}:{}", var.location.line, var.location.column),
                severity: IssueSeverity::Partial,
                suggestion: Some("Add getter function for public variable".to_string()),
            });
        }
    }

    /// Analyze a function
    fn analyze_function(&mut self, func: &FunctionDefinition) {
        // Check for special functions
        if func.is_fallback {
            self.check_feature("fallback_functions", &func.location);
            self.issues.push(MigrationIssue {
                description: "Fallback function requires special handling".to_string(),
                source_location: format!("{}:{}", func.location.line, func.location.column),
                severity: IssueSeverity::Manual,
                suggestion: Some("Implement as entry point function".to_string()),
            });
        }

        if func.is_receive {
            self.check_feature("receive_ether", &func.location);
        }

        // Check for external calls
        let has_external_calls = func
            .body
            .as_ref()
            .map(|body| self.detect_external_calls(body))
            .unwrap_or(false);

        if has_external_calls {
            self.issues.push(MigrationIssue {
                description: format!("Function '{}' may have external calls", func.name),
                source_location: format!("{}:{}", func.location.line, func.location.column),
                severity: IssueSeverity::Partial,
                suggestion: Some("Review call permissions and gas handling".to_string()),
            });
        }

        // Check for events
        let has_events = func
            .body
            .as_ref()
            .map(|body| self.detect_events(body))
            .unwrap_or(false);

        if has_events {
            self.issues.push(MigrationIssue {
                description: format!("Function '{}' emits events", func.name),
                source_location: format!("{}:{}", func.location.line, func.location.column),
                severity: IssueSeverity::Supported,
                suggestion: None,
            });
        }
    }

    /// Analyze a modifier
    fn analyze_modifier(&mut self, modifier: &ModifierDefinition) {
        self.check_feature("modifiers", &modifier.location);

        self.issues.push(MigrationIssue {
            description: format!(
                "Modifier '{}' needs conversion to predicate function",
                modifier.name
            ),
            source_location: format!("{}:{}", modifier.location.line, modifier.location.column),
            severity: IssueSeverity::Partial,
            suggestion: Some("Convert modifier to pre/post-condition check functions".to_string()),
        });
    }

    /// Check if a feature is supported
    fn check_feature(&mut self, feature: &str, location: &SolLocation) {
        match self.support_matrix.get(feature) {
            Some(SupportLevel::Supported) => {}
            Some(SupportLevel::Partial) => {
                self.issues.push(MigrationIssue {
                    description: format!("Feature '{}' has limited support", feature),
                    source_location: format!("{}:{}", location.line, location.column),
                    severity: IssueSeverity::Partial,
                    suggestion: Some(format!(
                        "Review {} implementation for compatibility",
                        feature
                    )),
                });
            }
            Some(SupportLevel::ManualRequired) => {
                self.issues.push(MigrationIssue {
                    description: format!("Feature '{}' requires manual conversion", feature),
                    source_location: format!("{}:{}", location.line, location.column),
                    severity: IssueSeverity::Manual,
                    suggestion: Some(format!("Manual implementation needed for {}", feature)),
                });
            }
            Some(SupportLevel::Unsupported) => {
                self.issues.push(MigrationIssue {
                    description: format!("Feature '{}' is not supported", feature),
                    source_location: format!("{}:{}", location.line, location.column),
                    severity: IssueSeverity::Unsupported,
                    suggestion: Some(format!("Find alternative for {}", feature)),
                });
            }
            None => {
                self.issues.push(MigrationIssue {
                    description: format!("Unknown feature: {}", feature),
                    source_location: format!("{}:{}", location.line, location.column),
                    severity: IssueSeverity::Manual,
                    suggestion: Some("Review feature compatibility".to_string()),
                });
            }
        }
    }

    /// Detect external calls in a block
    fn detect_external_calls(&self, block: &Block) -> bool {
        for stmt in &block.statements {
            if self.statement_has_external_call(stmt) {
                return true;
            }
        }
        false
    }

    /// Check if a statement contains external calls
    fn statement_has_external_call(&self, stmt: &Statement) -> bool {
        match stmt {
            Statement::Expression(expr_stmt) => {
                self.expression_is_external_call(&expr_stmt.expression)
            }
            Statement::If(if_stmt) => {
                self.expression_is_external_call(&if_stmt.condition)
                    || self.statement_has_external_call(&*if_stmt.true_body)
                    || if_stmt
                        .false_body
                        .as_ref()
                        .map(|s| self.statement_has_external_call(&**s))
                        .unwrap_or(false)
            }
            Statement::For(for_stmt) => {
                for_stmt
                    .initialization
                    .as_ref()
                    .map(|s| self.statement_has_external_call(s))
                    .unwrap_or(false)
                    || for_stmt
                        .condition
                        .as_ref()
                        .map(|c| self.expression_is_external_call(c))
                        .unwrap_or(false)
                    || self.statement_has_external_call(&*for_stmt.body)
            }
            Statement::While(while_stmt) => {
                self.expression_is_external_call(&while_stmt.condition)
                    || self.statement_has_external_call(&*while_stmt.body)
            }
            _ => false,
        }
    }

    /// Check if an expression is an external call
    fn expression_is_external_call(&self, expr: &Expression) -> bool {
        match expr {
            Expression::FunctionCall(func_call) => {
                if let Expression::Identifier(id) = &*func_call.expression {
                    !id.name.starts_with("this") && !id.name.starts_with("super")
                } else {
                    true
                }
            }
            Expression::MemberAccess(member) => {
                if let Expression::Identifier(id) = &*member.expression {
                    id.name == "this" || id.name == "super"
                } else {
                    true
                }
            }
            _ => false,
        }
    }

    /// Detect events in a block
    fn detect_events(&self, block: &Block) -> bool {
        for stmt in &block.statements {
            if matches!(stmt, Statement::Emit(_)) {
                return true;
            }
        }
        false
    }

    /// Calculate compatibility score
    fn calculate_compatibility_score(&self) -> f64 {
        let total = self.issues.len();
        if total == 0 {
            return 100.0;
        }

        let supported = self
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Supported)
            .count();

        let partial = self
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Partial)
            .count();

        let manual = self
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Manual)
            .count();

        let unsupported = self
            .issues
            .iter()
            .filter(|i| i.severity == IssueSeverity::Unsupported)
            .count();

        // Weighted score
        let score = (supported as f64 * 1.0
            + partial as f64 * 0.7
            + manual as f64 * 0.4
            + unsupported as f64 * 0.0)
            / (total as f64)
            * 100.0;

        score.max(0.0).min(100.0)
    }

    /// Estimate gas savings
    fn estimate_gas_savings(&self, source: &SoliditySource) -> f64 {
        // Simple estimation based on complexity
        let total_functions: usize = source.contracts.iter().map(|c| c.functions.len()).sum();

        let total_complexity = self.issues.len() as f64;

        // Rough estimate: 20% gas savings on average
        let base_savings = 0.20;

        // Adjust based on complexity
        let complexity_factor = (100.0 - total_complexity.min(50.0)) / 100.0;

        base_savings * complexity_factor * total_functions as f64
    }
}

/// Analysis result
#[derive(Debug, Serialize)]
pub struct AnalysisResult {
    /// Issues found during analysis
    pub issues: Vec<MigrationIssue>,
    /// Compatibility score (0-100)
    pub compatibility_score: f64,
    /// Estimated gas savings percentage
    pub estimated_gas_savings: f64,
}

impl Default for SolidityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = SolidityAnalyzer::new();
        assert!(!analyzer.support_matrix.is_empty());
    }

    #[test]
    fn test_support_matrix() {
        let analyzer = SolidityAnalyzer::new();
        assert_eq!(
            analyzer.support_matrix.get("functions"),
            Some(&SupportLevel::Supported)
        );
        assert_eq!(
            analyzer.support_matrix.get("inheritance"),
            Some(&SupportLevel::Partial)
        );
        assert_eq!(
            analyzer.support_matrix.get("inline_assembly"),
            Some(&SupportLevel::ManualRequired)
        );
    }

    #[test]
    fn test_compatibility_score_empty() {
        let analyzer = SolidityAnalyzer::new();
        let score = analyzer.calculate_compatibility_score();
        assert_eq!(score, 100.0);
    }
}
