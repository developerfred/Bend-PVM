#![allow(dead_code)]
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
        #[cfg(test)]
        mod tests;
    }
    pub mod analyzer {
        pub mod type_checker;
        pub mod type_inference;
    }
    pub mod optimizer {
        pub mod eta_reduction;
        pub mod float_comb;
        pub mod inline;
        pub mod linearize;
        pub mod passes;
        pub mod pruner;
        #[cfg(test)]
        mod tests;
    }
    pub mod codegen {
        pub mod ir;
        pub mod metadata;
        pub mod risc_v;
        #[cfg(test)]
        mod tests;
    }
    pub mod module;
    pub mod polkavm {
        pub mod abi;
        pub mod bridge;
        pub mod host;
    }
}

pub mod runtime {
    pub mod env;
    pub mod memory;
    pub mod metering;
    pub mod storage;
}

pub mod stdlib;
pub mod testing;
pub mod security;
pub mod debugger;
pub mod formatter;
pub mod migration;

use std::path::PathBuf;
use thiserror::Error;

use compiler::lexer::lexer::BendLexer;
use compiler::parser::parser::Parser;
use compiler::analyzer::type_checker::TypeChecker;
use compiler::optimizer::passes::create_default_manager;
use compiler::codegen::risc_v::RiscVCodegen;
use compiler::polkavm::bridge::compile_to_polkavm;

/// Compiler error type
#[derive(Error, Debug)]
pub enum CompileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Type error: {0}")]
    Type(String),

    #[error("Optimization error: {0}")]
    Optimization(String),

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
            security_scan: false,
            static_analysis: false,
            fuzz_testing: false,
            security_level: 1,
        }
    }
}

/// Compile a Bend source file
pub fn compile(source_path: &PathBuf, options: CompilerOptions) -> Result<(), CompileError> {
    // Read source file
    let source = std::fs::read_to_string(source_path)?;

    // Parse
    let _lexer = BendLexer::new(&source);
    let mut parser = Parser::new(&source);
    let program = parser.parse_program().map_err(|e| CompileError::Parse(e.to_string()))?;

    // Type Check
    if options.type_check {
        let mut type_checker = TypeChecker::new();
        type_checker
            .check_program(&program)
            .map_err(|e| CompileError::Type(e.to_string()))?;
    }

    // Optimize
    let optimized_program = if options.optimize {
        let mut manager = create_default_manager();
        manager
            .optimize(program)
            .map_err(|e| CompileError::Optimization(e.to_string()))? 
    } else {
        program
    };

    // Generate Code
    let mut generator = RiscVCodegen::new();
    let code = generator
        .generate(&optimized_program)
        .map_err(|e| CompileError::Codegen(e.to_string()))?;

    // Output Assembly
    if options.assembly {
        let asm_path = if let Some(output) = &options.output {
            let mut p = output.clone();
            p.set_extension("s");
            p
        } else {
            let mut p = source_path.clone();
            p.set_extension("s");
            p
        };

        // Convert bytecode to assembly string (mock implementation)
        let asm_content = format!("; Assembly for {}\n{:?}", source_path.display(), code);
        std::fs::write(asm_path, asm_content)?;
    }

    // Compile to PolkaVM
    let polkavm_module = compile_to_polkavm(&code, None)
        .map_err(|e| CompileError::PolkaVM(e.to_string()))?;
    
    // Output Binary
    let bin_path = if let Some(output) = &options.output {
        output.clone()
    } else {
        let mut p = source_path.clone();
        p.set_extension("bin");
        p
    };
    
    std::fs::write(bin_path, polkavm_module.binary.as_ref().unwrap())?;

    Ok(())
}

/// Helper function to parse a Bend source string (for testing/tools)
pub fn parse_source(source: &str) -> Result<compiler::parser::ast::Program, CompileError> {
    let _lexer = BendLexer::new(source);
    let mut parser = Parser::new(source);
    parser.parse_program().map_err(|e| CompileError::Parse(e.to_string()))
}

/// Returns the current version of the compiler
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
