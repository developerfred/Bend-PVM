use std::collections::HashSet;

use crate::compiler::parser::ast::*;
use thiserror::Error;

/// Errors that can occur during optimization
#[derive(Error, Debug, Clone)]
pub enum OptimizationError {
    #[error("Optimization error: {0}")]
    Generic(String),
    
    #[error("Failed to linearize: {0}")]
    Linearize(String),
    
    #[error("Failed to combine float operations: {0}")]
    FloatComb(String),
    
    #[error("Failed to prune: {0}")]
    Prune(String),
    
    #[error("Failed to apply eta reduction: {0}")]
    EtaReduction(String),
}

/// Represents the result of an optimization pass
pub enum OptimizationResult {
    /// The AST was modified
    Modified(Program),
    
    /// The AST was not modified
    Unchanged(Program),
}

impl OptimizationResult {
    /// Returns the program, regardless of whether it was modified
    pub fn program(self) -> Program {
        match self {
            OptimizationResult::Modified(program) => program,
            OptimizationResult::Unchanged(program) => program,
        }
    }
    
    /// Returns true if the program was modified
    pub fn was_modified(&self) -> bool {
        matches!(self, OptimizationResult::Modified(_))
    }
}

/// Trait for optimization passes
pub trait OptimizationPass {
    /// Name of the optimization pass
    fn name(&self) -> &'static str;
    
    /// Description of the optimization pass
    fn description(&self) -> &'static str;
    
    /// Applies the optimization pass to the program
    fn run(&self, program: Program) -> Result<OptimizationResult, OptimizationError>;
}

/// Optimization pass manager
pub struct OptimizationManager {
    /// Available passes
    passes: Vec<Box<dyn OptimizationPass>>,
    
    /// Enabled passes
    enabled_passes: HashSet<String>,
    
    /// Optimization level
    level: OptimizationLevel,
}

/// Optimization levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptimizationLevel {
    /// No optimizations
    None,
    
    /// Basic optimizations
    Basic,
    
    /// Standard optimizations
    Standard,
    
    /// Aggressive optimizations
    Aggressive,
}

impl OptimizationManager {
    /// Creates a new optimization manager with default passes
    pub fn new() -> Self {
        let passes: Vec<Box<dyn OptimizationPass>> = vec![
            // Add default passes here
        ];
        
        let mut enabled_passes = HashSet::new();
        
        OptimizationManager {
            passes,
            enabled_passes,
            level: OptimizationLevel::Standard,
        }
    }
    
    /// Sets the optimization level
    pub fn set_level(&mut self, level: OptimizationLevel) {
        self.level = level;
        
        // Update enabled passes based on level
        self.enabled_passes.clear();
        
        match level {
            OptimizationLevel::None => {
                // No passes enabled
            },
            OptimizationLevel::Basic => {
                // Enable basic passes
                self.enable_pass("constant_folding");
                self.enable_pass("dead_code_elimination");
            },
            OptimizationLevel::Standard => {
                // Enable standard passes
                self.enable_pass("constant_folding");
                self.enable_pass("dead_code_elimination");
                self.enable_pass("linearize");
                self.enable_pass("prune");
            },
            OptimizationLevel::Aggressive => {
                // Enable all passes
                for pass in &self.passes {
                    self.enable_pass(pass.name());
                }
            },
        }
    }
    
    /// Enables an optimization pass
    pub fn enable_pass(&mut self, name: &str) {
        self.enabled_passes.insert(name.to_string());
    }
    
    /// Disables an optimization pass
    pub fn disable_pass(&mut self, name: &str) {
        self.enabled_passes.remove(name);
    }
    
    /// Registers a new optimization pass
    pub fn register_pass(&mut self, pass: Box<dyn OptimizationPass>) {
        self.passes.push(pass);
    }
    
    /// Runs all enabled optimization passes on the program
    pub fn optimize(&self, program: Program) -> Result<Program, OptimizationError> {
        let mut current_program = program;
        let mut modified = false;
        
        // Run all enabled passes
        for pass in &self.passes {
            if self.enabled_passes.contains(pass.name()) {
                let result = pass.run(current_program)?;
                
                if result.was_modified() {
                    modified = true;
                }
                
                current_program = result.program();
            }
        }
        
        // If any pass modified the program, run the optimization again
        // until no more changes are made
        if modified {
            self.optimize(current_program)
        } else {
            Ok(current_program)
        }
    }
}

/// Creates an optimization manager with the default set of passes
pub fn create_default_manager() -> OptimizationManager {
    use crate::compiler::optimizer::linearize::LinearizePass;
    use crate::compiler::optimizer::float_comb::FloatCombPass;
    use crate::compiler::optimizer::pruner::PrunePass;
    use crate::compiler::optimizer::eta_reduction::EtaReductionPass;
    
    let mut manager = OptimizationManager::new();
    
    // Register passes
    manager.register_pass(Box::new(LinearizePass::new()));
    manager.register_pass(Box::new(FloatCombPass::new()));
    manager.register_pass(Box::new(PrunePass::new()));
    manager.register_pass(Box::new(EtaReductionPass::new()));
    
    // Set default level
    manager.set_level(OptimizationLevel::Standard);
    
    manager
}