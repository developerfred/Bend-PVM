pub mod compiler {
    pub mod lexer {
        pub mod token;
        pub mod lexer;
    }
    
    pub mod parser {
        pub mod ast;
        pub mod parser;
    }
    
    pub mod analyzer {
        pub mod type_checker;
    }
    
    pub mod optimizer {
        pub mod passes;
        pub mod linearize;
        pub mod float_comb;
        pub mod pruner;
        pub mod eta_reduction;
    }
    
    pub mod codegen {
        pub mod ir;
        pub mod risc_v;
        pub mod metadata;
    }
    
    pub mod polkavm {
        pub mod bridge;
        pub mod abi;
        pub mod host;
    }
}

pub mod runtime {
    pub mod env;
    pub mod metering;
    pub mod memory;
    pub mod storage;
}

use std::path::{Path, PathBuf};
use std::fs;

use thiserror::Error;

use compiler::lexer::lexer::BendLexer;
use compiler::parser::parser::Parser;
use compiler::analyzer::type_checker::TypeChecker;
use compiler::codegen::risc_v::RiscVCodegen;
use compiler::polkavm::bridge::{PolkaVMModule, compile_to_polkavm};

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
        }
    }
}

/// Compile a Bend contract
pub fn compile<P: AsRef<Path>>(file_path: P, options: CompilerOptions) -> Result<(), CompilerError> {
    // Read the source file
    let source = fs::read_to_string(file_path.as_ref())?;
    
    // Lex the source
    let mut lexer = BendLexer::new(&source);
    
    // Parse the source
    let mut parser = Parser::new(&source);
    let program = parser.parse_program()
        .map_err(|e| CompilerError::Parse(e.to_string()))?;
    
    // Type check the program (if enabled)
    if options.type_check {
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program)
            .map_err(|e| CompilerError::Type(e.to_string()))?;
    }
    
    // Generate code
    let mut codegen = RiscVCodegen::new();
    let instructions = codegen.generate(&program)
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
    let program = parser.parse_program()
        .map_err(|e| CompilerError::Parse(e.to_string()))?;
    
    // Type check the program (if enabled)
    if options.type_check {
        let mut type_checker = TypeChecker::new();
        type_checker.check_program(&program)
            .map_err(|e| CompilerError::Type(e.to_string()))?;
    }
    
    // Generate code
    let mut codegen = RiscVCodegen::new();
    let instructions = codegen.generate(&program)
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