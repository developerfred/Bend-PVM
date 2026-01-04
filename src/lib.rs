pub mod build;
pub mod error;
pub mod ffi;

pub mod compiler {
    pub mod lexer {
        pub mod lexer;
        pub mod token;
    }

    pub mod parser {
        pub mod ast;
        pub mod parser;
    }

    pub mod analyzer {
        pub mod type_checker;
    }

    pub mod optimizer {
        pub mod eta_reduction;
        pub mod float_comb;
        pub mod linearize;
        pub mod passes;
        pub mod pruner;
    }

    pub mod codegen {
        pub mod ir;
        pub mod metadata;
        pub mod risc_v;
    }

    pub mod polkavm {
        pub mod abi;
        pub mod bridge;
        pub mod host;
    }

    pub mod module;
}

pub mod stdlib;

pub mod security;

pub mod runtime {
    pub mod env;
    pub mod memory;
    pub mod metering;
    pub mod storage;
}

use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use compiler::analyzer::type_checker::TypeChecker;
use compiler::codegen::risc_v::RiscVCodegen;
use compiler::lexer::lexer::BendLexer;
use compiler::parser::parser::Parser;
use compiler::polkavm::bridge::{compile_to_polkavm, PolkaVMModule};

use security::{SecurityConfig, SecurityManager};

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Codegen error: {0}")]
    Codegen(String),

    #[error("PolkaVM error: {0}")]
    PolkaVM(String),

    #[error("Security error: {0}")]
    Security(String),
}

/// Options for the compiler
pub struct CompilerOptions {
    /// Output file path
    pub output: Option<PathBuf>,

    /// Whether to optimize the code
    pub optimize: bool,

    /// Whether to generate debug information
    pub debug: bool,

    /// Whether to perform type checking
    pub type_check: bool,

    /// Whether to output assembly
    pub assembly: bool,

    /// Whether to output metadata
    pub metadata: bool,

    /// Whether to output ABI
    pub abi: bool,

    /// Whether to enable security scanning
    pub security_scan: bool,

    /// Whether to enable static analysis
    pub static_analysis: bool,

    /// Whether to enable fuzz testing
    pub fuzz_testing: bool,

    /// Security level (0=None, 1=Basic, 2=Enhanced, 3=Maximum)
    pub security_level: u8,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        CompilerOptions {
            output: None,
            optimize: true,
            debug: false,
            type_check: true,
            assembly: false,
            metadata: true,
            abi: true,
            security_scan: true,
            static_analysis: true,
            fuzz_testing: false, // Disabled by default for performance
            security_level: 2,   // Enhanced security by default
        }
    }
}

/// Compile a Bend contract
pub fn compile<P: AsRef<Path>>(
    file_path: P,
    options: CompilerOptions,
) -> Result<(), CompilerError> {
    // Read the source file
    let source = fs::read_to_string(file_path.as_ref())?;

    // Lex the source
    let mut lexer = BendLexer::new(&source);

    // Parse the source
    let mut parser = Parser::new(&source);
    let program = parser
        .parse_program()
        .map_err(|e| CompilerError::Parse(e.to_string()))?;

    // Type check the program (if enabled)
    if options.type_check {
        let mut type_checker = TypeChecker::new();
        type_checker
            .check_program(&program)
            .map_err(|e| CompilerError::Type(e.to_string()))?;
    }

    // Security validation (if enabled)
    if options.security_level > 0 {
        let security_config = SecurityConfig {
            gas_limit: 10_000_000,
            max_call_depth: 100,
            enable_access_control: options.security_level >= 1,
            enable_reentrancy_guard: options.security_level >= 2,
            enable_input_validation: options.security_level >= 1,
            enable_static_analysis: options.static_analysis,
            enable_fuzz_testing: options.fuzz_testing,
        };

        let mut security_manager = SecurityManager::new(security_config);
        security_manager
            .validate_program(&program)
            .map_err(|e| CompilerError::Security(e.to_string()))?;
    }

    // Generate code
    let mut codegen = RiscVCodegen::new();
    let instructions = codegen
        .generate(&program)
        .map_err(|e| CompilerError::Codegen(e.to_string()))?;

    // Compile to PolkaVM
    let output_path = options.output.as_ref().map(|p| p.as_path());
    let module = compile_to_polkavm(&instructions, output_path)
        .map_err(|e| CompilerError::PolkaVM(e.to_string()))?;

    // Output additional files if requested
    if let Some(output_path) = output_path {
        if options.assembly {
            // Write assembly to file
            let asm_path = output_path.with_extension("s");
            fs::write(&asm_path, &module.assembly)?;
        }

        if options.metadata {
            // Generate and write metadata to file
            let metadata_path = output_path.with_extension("metadata.json");
            // In a real implementation, this would generate metadata from the program
            let metadata = "{\"name\": \"dummy_metadata\"}";
            fs::write(&metadata_path, metadata)?;
        }

        if options.abi {
            // Generate and write ABI to file
            let abi_path = output_path.with_extension("abi.json");
            // In a real implementation, this would generate ABI from the program
            let abi = "{\"methods\": []}";
            fs::write(&abi_path, abi)?;
        }
    }

    Ok(())
}

/// Compile a Bend contract from source
pub fn compile_from_source(source: &str, options: CompilerOptions) -> Result<(), CompilerError> {
    // Lex the source
    let mut lexer = BendLexer::new(source);

    // Parse the source
    let mut parser = Parser::new(source);
    let program = parser
        .parse_program()
        .map_err(|e| CompilerError::Parse(e.to_string()))?;

    // Type check the program (if enabled)
    if options.type_check {
        let mut type_checker = TypeChecker::new();
        type_checker
            .check_program(&program)
            .map_err(|e| CompilerError::Type(e.to_string()))?;
    }

    // Generate code
    let mut codegen = RiscVCodegen::new();
    let instructions = codegen
        .generate(&program)
        .map_err(|e| CompilerError::Codegen(e.to_string()))?;

    // Compile to PolkaVM
    let output_path = options.output.as_ref().map(|p| p.as_path());
    let module = compile_to_polkavm(&instructions, output_path)
        .map_err(|e| CompilerError::PolkaVM(e.to_string()))?;

    // Output additional files if requested
    if let Some(output_path) = output_path {
        if options.assembly {
            // Write assembly to file
            let asm_path = output_path.with_extension("s");
            fs::write(&asm_path, &module.assembly)?;
        }

        if options.metadata {
            // Generate and write metadata to file
            let metadata_path = output_path.with_extension("metadata.json");
            // In a real implementation, this would generate metadata from the program
            let metadata = "{\"name\": \"dummy_metadata\"}";
            fs::write(&metadata_path, metadata)?;
        }

        if options.abi {
            // Generate and write ABI to file
            let abi_path = output_path.with_extension("abi.json");
            // In a real implementation, this would generate ABI from the program
            let abi = "{\"methods\": []}";
            fs::write(&abi_path, abi)?;
        }
    }

    Ok(())
}

/// Version of the compiler
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
