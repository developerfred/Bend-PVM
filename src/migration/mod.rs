//! # Solidity to Bend-PVM Transpiler
//!
//! This module provides utilities for converting Solidity smart contracts
//! to Bend-PVM format, enabling migration from Ethereum ecosystem.

use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

pub mod analyzer;
pub mod ast;
pub mod cli;
pub mod converter;

/// Errors that can occur during migration
#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Translation error: {0}")]
    TranslationError(String),

    #[error("Unsupported Solidity feature: {0}")]
    UnsupportedFeature(String),

    #[error("Contract analysis error: {0}")]
    AnalysisError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Compatibility issue at {location}: {message}")]
    CompatibilityIssue {
        location: String,
        message: String,
        severity: IssueSeverity,
    },
}

/// Severity levels for compatibility issues
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum IssueSeverity {
    /// Feature is fully supported
    Supported,
    /// Feature is partially supported with limitations
    Partial,
    /// Feature needs manual intervention
    Manual,
    /// Feature is not supported
    Unsupported,
}

impl std::fmt::Display for IssueSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueSeverity::Supported => write!(f, "Supported"),
            IssueSeverity::Partial => write!(f, "Partial"),
            IssueSeverity::Manual => write!(f, "Manual"),
            IssueSeverity::Unsupported => write!(f, "Unsupported"),
        }
    }
}

/// Migration statistics
#[derive(Debug, Default)]
pub struct MigrationStats {
    /// Total contracts processed
    pub contracts_processed: usize,
    /// Total functions translated
    pub functions_translated: usize,
    /// Total lines of code
    pub lines_of_code: usize,
    /// Issues found during migration
    pub issues: Vec<MigrationIssue>,
    /// Estimated gas savings (if applicable)
    pub gas_savings_estimate: f64,
}

/// A single migration issue
#[derive(Debug, Clone, Serialize)]
pub struct MigrationIssue {
    /// Issue description
    pub description: String,
    /// Source location in Solidity
    pub source_location: String,
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Suggested workaround
    pub suggestion: Option<String>,
}

/// Migration configuration
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Output directory for translated code
    pub output_dir: PathBuf,
    /// Generate tests for migrated contracts
    pub generate_tests: bool,
    /// Generate documentation
    pub generate_docs: bool,
    /// Keep original comments
    pub keep_comments: bool,
    /// Generate compatibility report
    pub generate_report: bool,
    /// ERC standards to include
    pub include_erc: Vec<String>,
    /// Custom mappings
    pub custom_mappings: HashMap<String, String>,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        MigrationConfig {
            output_dir: PathBuf::from("./bend-output"),
            generate_tests: true,
            generate_docs: true,
            keep_comments: true,
            generate_report: true,
            include_erc: vec![
                "ERC20".to_string(),
                "ERC721".to_string(),
                "ERC1155".to_string(),
            ],
            custom_mappings: HashMap::new(),
        }
    }
}

/// Main migration orchestrator
pub struct SolidityMigrator {
    config: MigrationConfig,
    stats: MigrationStats,
    erc_templates: HashMap<String, String>,
}

impl SolidityMigrator {
    /// Create a new migrator with default configuration
    pub fn new() -> Self {
        Self::with_config(MigrationConfig::default())
    }

    /// Create a migrator with custom configuration
    pub fn with_config(config: MigrationConfig) -> Self {
        let mut migrator = SolidityMigrator {
            config,
            stats: MigrationStats::default(),
            erc_templates: HashMap::new(),
        };

        // Initialize ERC templates
        migrator.initialize_erc_templates();

        migrator
    }

    /// Initialize built-in ERC templates
    fn initialize_erc_templates(&mut self) {
        // ERC-20 template
        self.erc_templates.insert(
            "ERC20".to_string(),
            r#"
/// ERC-20 Token Implementation for Bend-PVM
contract ERC20 is BendContract {
    /// Token name
    let name: String
    
    /// Token symbol  
    let symbol: String
    
    /// Total supply
    let total_supply: u256
    
    /// Balance mapping
    let balances: Map<Address, u256>
    
    /// Allowance mapping
    let allowances: Map<Address, Map<Address, u256>>
    
    /// Event: Transfer
    event Transfer(from: Address, to: Address, value: u256)
    
    /// Event: Approval
    event Approval(owner: Address, spender: Address, value: u256)
    
    /// Constructor
    fn init(name: String, symbol: String, initial_supply: u256) {
        self.name = name
        self.symbol = symbol
        self.total_supply = initial_supply
        self.balances[msg.sender] = initial_supply
    }
    
    /// Get token name
    fn get_name() -> String {
        self.name
    }
    
    /// Get token symbol
    fn get_symbol() -> String {
        self.symbol
    }
    
    /// Get decimals (default 18)
    fn get_decimals() -> u8 {
        18
    }
    
    /// Get total supply
    fn get_total_supply() -> u256 {
        self.total_supply
    }
    
    /// Get balance of account
    fn balance_of(account: Address) -> u256 {
        self.balances[account]
    }
    
    /// Transfer tokens
    fn transfer(to: Address, amount: u256) -> bool {
        let from = msg.sender
        assert(self.balances[from] >= amount, "Insufficient balance")
        
        self.balances[from] = self.balances[from] - amount
        self.balances[to] = self.balances[to] + amount
        
        emit Transfer(from, to, amount)
        true
    }
    
    /// Approve spender
    fn approve(spender: Address, amount: u256) -> bool {
        self.allowances[msg.sender][spender] = amount
        emit Approval(msg.sender, spender, amount)
        true
    }
    
    /// Transfer from (with allowance)
    fn transfer_from(from: Address, to: Address, amount: u256) -> bool {
        assert(self.allowances[from][msg.sender] >= amount, "Insufficient allowance")
        assert(self.balances[from] >= amount, "Insufficient balance")
        
        self.allowances[from][msg.sender] = self.allowances[from][msg.sender] - amount
        self.balances[from] = self.balances[from] - amount
        self.balances[to] = self.balances[to] + amount
        
        emit Transfer(from, to, amount)
        true
    }
    
    /// Get allowance
    fn allowance(owner: Address, spender: Address) -> u256 {
        self.allowances[owner][spender]
    }
}
"#
            .to_string(),
        );

        // ERC-721 template
        self.erc_templates.insert(
            "ERC721".to_string(),
            r#"
/// ERC-721 Non-Fungible Token Implementation for Bend-PVM
contract ERC721 is BendContract {
    /// Token name
    let name: String
    
    /// Token symbol
    let symbol: String
    
    /// Mapping from token ID to owner
    let owners: Map<u256, Address>
    
    /// Mapping from owner to token count
    let balances: Map<Address, u256>
    
    /// Mapping from token ID to approved address
    let token_approvals: Map<u256, Address>
    
    /// Mapping from owner to operator approvals
    let operator_approvals: Map<Address, Map<Address, bool>>
    
    /// Event: Transfer
    event Transfer(from: Address, to: Address, token_id: u256)
    
    /// Event: Approval
    event Approval(owner: Address, approved: Address, token_id: u256)
    
    /// Event: Approval for all
    event ApprovalForAll(owner: Address, operator: Address, approved: bool)
    
    /// Constructor
    fn init(name: String, symbol: String) {
        self.name = name
        self.symbol = symbol
    }
    
    /// Get token name
    fn get_name() -> String {
        self.name
    }
    
    /// Get token symbol
    fn get_symbol() -> String {
        self.symbol
    }
    
    /// Get balance of owner
    fn balance_of(owner: Address) -> u256 {
        self.balances[owner]
    }
    
    /// Get owner of token
    fn owner_of(token_id: u256) -> Address {
        self.owners[token_id]
    }
    
    /// Approve token
    fn approve(to: Address, token_id: u256) {
        let owner = self.owners[token_id]
        assert(msg.sender == owner || self.operator_approvals[owner][msg.sender], "Not authorized")
        self.token_approvals[token_id] = to
        emit Approval(owner, to, token_id)
    }
    
    /// Approve all
    fn set_approval_for_all(operator: Address, approved: bool) {
        self.operator_approvals[msg.sender][operator] = approved
        emit ApprovalForAll(msg.sender, operator, approved)
    }
    
    /// Get approved address
    fn get_approved(token_id: u256) -> Address {
        self.token_approvals[token_id]
    }
    
    /// Check if operator is approved
    fn is_approved_for_all(owner: Address, operator: Address) -> bool {
        self.operator_approvals[owner][operator]
    }
    
    /// Transfer token
    fn transfer_from(from: Address, to: Address, token_id: u256) {
        assert(
            msg.sender == from || 
            self.token_approvals[token_id] == msg.sender ||
            self.operator_approvals[from][msg.sender],
            "Not authorized"
        )
        assert(self.owners[token_id] == from, "Not token owner")
        
        self.owners[token_id] = to
        self.balances[from] = self.balances[from] - 1
        self.balances[to] = self.balances[to] + 1
        
        emit Transfer(from, to, token_id)
    }
    
    /// Safe transfer from
    fn safe_transfer_from(from: Address, to: Address, token_id: u256) {
        self.transfer_from(from, to, token_id)
        // Add receiver contract check if needed
    }
    
    /// Mint token (should be protected)
    fn mint(to: Address, token_id: u256) {
        assert(self.owners[token_id] == Address::zero(), "Token already exists")
        self.owners[token_id] = to
        self.balances[to] = self.balances[to] + 1
        emit Transfer(Address::zero(), to, token_id)
    }
    
    /// Burn token (should be protected)
    fn burn(token_id: u256) {
        let owner = self.owners[token_id]
        assert(owner != Address::zero(), "Invalid token")
        
        self.owners[token_id] = Address::zero()
        self.balances[owner] = self.balances[owner] - 1
        
        emit Transfer(owner, Address::zero(), token_id)
    }
}
"#
            .to_string(),
        );
    }

    /// Get an ERC template by name
    pub fn get_erc_template(&self, name: &str) -> Option<&String> {
        self.erc_templates.get(name)
    }

    /// List available ERC templates
    pub fn list_erc_templates(&self) -> Vec<String> {
        self.erc_templates.keys().cloned().collect()
    }
}

/// Helper function to create a new migrator
pub fn create_migrator() -> SolidityMigrator {
    SolidityMigrator::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrator_creation() {
        let migrator = SolidityMigrator::new();
        assert!(migrator.list_erc_templates().contains(&"ERC20".to_string()));
        assert!(migrator
            .list_erc_templates()
            .contains(&"ERC721".to_string()));
    }

    #[test]
    fn test_erc_templates_exist() {
        let migrator = SolidityMigrator::new();
        assert!(migrator.get_erc_template("ERC20").is_some());
        assert!(migrator.get_erc_template("ERC721").is_some());
        assert!(migrator.get_erc_template("ERC1155").is_none()); // Not implemented yet
    }
}
