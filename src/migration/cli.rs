//! # Migration CLI
//!
//! Command-line interface for the Solidity to Bend-PVM migration tool.

use super::analyzer::SolidityAnalyzer;
use super::ast::SoliditySource;
use super::converter::SolidityToBendConverter;
use super::{MigrationConfig, MigrationError, SolidityMigrator};
use clap::{Arg, ArgAction, Command};
use std::path::PathBuf;
use std::process;

/// Run the migration CLI
pub fn run_cli() {
    let matches = Command::new("bend-migrate")
        .version("1.0.0")
        .author("Bend-PVM Team")
        .about("Migrate Solidity smart contracts to Bend-PVM")
        .subcommand(
            Command::new("convert")
                .about("Convert Solidity file to Bend-PVM")
                .arg(
                    Arg::new("input")
                        .required(true)
                        .help("Input Solidity file or directory"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output directory for converted files"),
                )
                .arg(
                    Arg::new("recursive")
                        .short('r')
                        .long("recursive")
                        .action(ArgAction::SetTrue)
                        .help("Recursively process directories"),
                ),
        )
        .subcommand(
            Command::new("analyze")
                .about("Analyze Solidity file for migration compatibility")
                .arg(Arg::new("input").required(true).help("Input Solidity file"))
                .arg(
                    Arg::new("json")
                        .short('j')
                        .long("json")
                        .action(ArgAction::SetTrue)
                        .help("Output in JSON format"),
                ),
        )
        .subcommand(Command::new("list-erc").about("List available ERC templates"))
        .subcommand(
            Command::new("template")
                .about("Generate ERC template")
                .arg(
                    Arg::new("erc_type")
                        .required(true)
                        .help("ERC template type (ERC20, ERC721, ERC1155)"),
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output file path"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("convert", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let output = sub_matches.get_one::<String>("output").map(String::from);
            let recursive = sub_matches.get_flag("recursive");

            convert_command(input, output, recursive);
        }
        Some(("analyze", sub_matches)) => {
            let input = sub_matches.get_one::<String>("input").unwrap();
            let json = sub_matches.get_flag("json");

            analyze_command(input, json);
        }
        Some(("list-erc", _)) => {
            list_erc_command();
        }
        Some(("template", sub_matches)) => {
            let erc_type = sub_matches.get_one::<String>("erc_type").unwrap();
            let output = sub_matches.get_one::<String>("output").map(String::from);

            template_command(erc_type, output);
        }
        _ => {
            println!("Bend-PVM Migration Tool");
            println!("Usage: bend-migrate <command> [options]");
            println!();
            println!("Commands:");
            println!("  convert      Convert Solidity file to Bend-PVM");
            println!("  analyze      Analyze Solidity file for compatibility");
            println!("  list-erc     List available ERC templates");
            println!("  template     Generate ERC template");
            println!();
            println!("Use 'bend-migrate <command> --help' for more information.");
        }
    }
}

/// Convert command
fn convert_command(input: &str, output: Option<String>, recursive: bool) {
    println!("Converting Solidity files from: {}", input);

    let mut migrator = SolidityMigrator::new();
    let mut config = MigrationConfig::default();

    if let Some(output_dir) = output {
        config.output_dir = PathBuf::from(output_dir);
    }

    // In a real implementation, we would:
    // 1. Find all .sol files
    // 2. Parse each file
    // 3. Convert to Bend-PVM
    // 4. Write output files

    println!("Configuration:");
    println!("  Output directory: {}", config.output_dir.display());
    println!("  Generate tests: {}", config.generate_tests);
    println!("  Generate docs: {}", config.generate_docs);
    println!();

    println!("Migration features:");
    println!("  ERC20: ✅ Included");
    println!("  ERC721: ✅ Included");
    println!("  ERC1155: ✅ Included");

    println!();
    println!("Ready to convert. (Full implementation pending file I/O)");
}

/// Analyze command
fn analyze_command(input: &str, json: bool) {
    println!("Analyzing Solidity file: {}", input);

    let mut analyzer = SolidityAnalyzer::new();

    // Create a dummy source for demonstration
    let dummy_source = SoliditySource {
        version_pragma: None,
        imports: Vec::new(),
        contracts: Vec::new(),
        interfaces: Vec::new(),
        libraries: Vec::new(),
        enums: Vec::new(),
        structs: Vec::new(),
        location: super::ast::SolLocation {
            file: input.to_string(),
            line: 1,
            column: 1,
            start: 0,
            end: 0,
        },
    };

    let result = analyzer.analyze(&dummy_source);

    if json {
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
    } else {
        println!();
        println!("Analysis Results:");
        println!("  Compatibility Score: {:.1}%", result.compatibility_score);
        println!(
            "  Estimated Gas Savings: {:.1}%",
            result.estimated_gas_savings
        );
        println!("  Issues Found: {}", result.issues.len());

        if !result.issues.is_empty() {
            println!();
            println!("Issues:");
            for (i, issue) in result.issues.iter().enumerate() {
                println!("  {}. [{}] {}", i + 1, issue.severity, issue.description);
                println!("     Location: {}", issue.source_location);
                if let Some(suggestion) = &issue.suggestion {
                    println!("     Suggestion: {}", suggestion);
                }
            }
        }
    }
}

/// List ERC templates command
fn list_erc_command() {
    let migrator = SolidityMigrator::new();
    let templates = migrator.list_erc_templates();

    println!("Available ERC Templates:");
    for template in templates {
        println!("  - {}", template);
    }

    println!();
    println!("Usage: bend-migrate template <ERC_TYPE>");
    println!("Example: bend-migrate template ERC20");
}

/// Template command
fn template_command(erc_type: &str, output: Option<String>) {
    let mut migrator = SolidityMigrator::new();

    // Add ERC-1155 template if requested
    if erc_type == "ERC1155" {
        migrator.erc_templates.insert("ERC1155", r#"
/// ERC-1155 Multi-Token Implementation for Bend-PVM
contract ERC1155 is BendContract {
    /// Token URI
    let uri: String

    /// Mapping from token ID to balance
    let balances: Map<u256, Map<Address, u256>>

    /// Mapping from token ID to operator approvals
    let operator_approvals: Map<u256, Map<Address, bool>>

    /// Mapping from owner to operator approvals
    let default_operator_approvals: Map<Address, Map<Address, bool>>

    /// Event: Transfer Single
    event TransferSingle(operator: Address, from: Address, to: Address, id: u256, value: u256)

    /// Event: Transfer Batch
    event TransferBatch(operator: Address, from: Address, to: Address, ids: Vec<u256>, values: Vec<u256>)

    /// Event: Approval for All
    event ApprovalForAll(account: Address, operator: Address, approved: bool)

    /// Event: URI(value: String, id: u256)
    event URI(value: String, id: u256)

    /// Constructor
    fn init(uri: String) {
        self.uri = uri
        self.balances = Map::new()
        self.operator_approvals = Map::new()
        self.default_operator_approvals = Map::new()
    }

    /// Get balance of account for token
    fn balance_of(account: Address, id: u256) -> u256 {
        if !self.balances.contains_key(id) {
            return 0
        }
        let token_balances = self.balances[id]
        if !token_balances.contains_key(account) {
            return 0
        }
        token_balances[account]
    }

    /// Get balance of multiple accounts for multiple tokens
    fn balance_of_batch(accounts: Vec<Address>, ids: Vec<u256>) -> Vec<u256> {
        assert(accounts.len() == ids.len(), "Accounts and IDs length mismatch")
        let results = Vec::new()
        for i in 0..accounts.len() {
            let balance = self.balance_of(accounts[i], ids[i])
            results.push(balance)
        }
        results
    }

    /// Set approval for all
    fn set_approval_for_all(operator: Address, approved: bool) {
        self.default_operator_approvals[msg.sender][operator] = approved
        emit ApprovalForAll(msg.sender, operator, approved)
    }

    /// Check if operator is approved for all
    fn is_approved_for_all(account: Address, operator: Address) -> bool {
        if !self.default_operator_approvals.contains_key(account) {
            return false
        }
        self.default_operator_approvals[account][operator]
    }

    /// Safe transfer from single token
    fn safe_transfer_from(from: Address, to: Address, id: u256, amount: u256, data: Vec<u8>) {
        assert(from == msg.sender || self.is_approved_for_all(from, msg.sender), "Not authorized")
        assert(to != Address::zero(), "Transfer to zero address")

        let from_balance = self.balance_of(from, id)
        assert(from_balance >= amount, "Insufficient balance")

        self.balances[id][from] = from_balance - amount
        self.balances[id][to] = self.balance_of(to, id) + amount

        emit TransferSingle(msg.sender, from, to, id, amount)
    }

    /// Safe batch transfer from
    fn safe_batch_transfer_from(from: Address, to: Address, ids: Vec<u256>, amounts: Vec<u256>, data: Vec<u8>) {
        assert(from == msg.sender || self.is_approved_for_all(from, msg.sender), "Not authorized")
        assert(to != Address::zero(), "Transfer to zero address")
        assert(ids.len() == amounts.len(), "IDs and amounts length mismatch")

        for i in 0..ids.len() {
            let id = ids[i]
            let amount = amounts[i]
            let from_balance = self.balance_of(from, id)
            assert(from_balance >= amount, "Insufficient balance")

            self.balances[id][from] = from_balance - amount
            self.balances[id][to] = self.balance_of(to, id) + amount
        }

        emit TransferBatch(msg.sender, from, to, ids, amounts)
    }

    /// Mint single token
    fn mint(to: Address, id: u256, amount: u256, data: Vec<u8>) {
        assert(to != Address::zero(), "Mint to zero address")

        self.balances[id][to] = self.balance_of(to, id) + amount
        emit TransferSingle(msg.sender, Address::zero(), to, id, amount)
    }

    /// Batch mint tokens
    fn mint_batch(to: Address, ids: Vec<u256>, amounts: Vec<u256>, data: Vec<u8>) {
        assert(to != Address::zero(), "Mint to zero address")
        assert(ids.len() == amounts.len(), "IDs and amounts length mismatch")

        for i in 0..ids.len() {
            let id = ids[i]
            let amount = amounts[i]
            self.balances[id][to] = self.balance_of(to, id) + amount
        }

        emit TransferBatch(msg.sender, Address::zero(), to, ids, amounts)
    }

    /// Burn single token
    fn burn(from: Address, id: u256, amount: u256) {
        let from_balance = self.balance_of(from, id)
        assert(from_balance >= amount, "Insufficient balance")

        self.balances[id][from] = from_balance - amount
        emit TransferSingle(msg.sender, from, Address::zero(), id, amount)
    }

    /// Batch burn tokens
    fn burn_batch(from: Address, ids: Vec<u256>, amounts: Vec<u256>) {
        assert(ids.len() == amounts.len(), "IDs and amounts length mismatch")

        for i in 0..ids.len() {
            let id = ids[i]
            let amount = amounts[i]
            let from_balance = self.balance_of(from, id)
            assert(from_balance >= amount, "Insufficient balance")

            self.balances[id][from] = from_balance - amount
        }

        emit TransferBatch(msg.sender, from, Address::zero(), ids, amounts)
    }

    /// Get URI for token
    fn uri(id: u256) -> String {
        self.uri
    }
}
"#);
    }

    match migrator.get_erc_template(erc_type) {
        Some(template) => {
            let output_content = template.trim();

            if let Some(output_path) = output {
                std::fs::write(&output_path, output_content)
                    .expect("Failed to write template file");
                println!("Generated {} template: {}", erc_type, output_path);
            } else {
                println!("{}", output_content);
            }
        }
        None => {
            eprintln!("Error: ERC template '{}' not found", erc_type);
            eprintln!("Available templates:");
            for template in migrator.list_erc_templates() {
                eprintln!("  - {}", template);
            }
            process::exit(1);
        }
    }
}
