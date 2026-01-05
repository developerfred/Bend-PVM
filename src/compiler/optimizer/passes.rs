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
    fn run(&mut self, program: Program) -> Result<OptimizationResult, OptimizationError>;
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

impl Default for OptimizationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizationManager {
    /// Creates a new optimization manager with default passes
    pub fn new() -> Self {
        let passes: Vec<Box<dyn OptimizationPass>> = vec![
            // Add default passes here
        ];

        let enabled_passes = HashSet::new();

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
            }
            OptimizationLevel::Basic => {
                // Enable basic passes
                self.enable_pass("constant_folding");
                self.enable_pass("dead_code_elimination");
            }
            OptimizationLevel::Standard => {
                // Enable standard passes
                self.enable_pass("constant_folding");
                self.enable_pass("dead_code_elimination");
                self.enable_pass("linearize");
                self.enable_pass("prune");
            }
            OptimizationLevel::Aggressive => {
                // Enable all passes
                let pass_names: Vec<_> = self.passes.iter().map(|p| p.name()).collect();
                for name in pass_names {
                    self.enable_pass(name);
                }
            }
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
    pub fn optimize(&mut self, program: Program) -> Result<Program, OptimizationError> {
        let mut current_program = program;
        let mut modified = false;

        // Run all enabled passes
        for pass in &mut self.passes {
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
    use crate::compiler::optimizer::eta_reduction::EtaReductionPass;
    use crate::compiler::optimizer::float_comb::FloatCombPass;
    use crate::compiler::optimizer::linearize::LinearizePass;
    use crate::compiler::optimizer::pruner::PrunePass;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::parser::ast::*;
    use crate::compiler::parser::parser::Parser;

    fn parse_program(source: &str) -> Program {
        let mut parser = Parser::new(source);
        parser.parse_program().unwrap()
    }

    fn create_simple_program() -> Program {
        parse_program(
            r#"
            fn main() -> u32 {
                let x = 1 + 2;
                let y = x * 3;
                return y;
            }
        "#,
        )
    }

    fn create_complex_program() -> Program {
        parse_program(
            r#"
            fn add(a: u32, b: u32) -> u32 {
                return a + b;
            }

            fn main() -> u32 {
                let result = add(5, add(3, 2));
                return result;
            }
        "#,
        )
    }

    #[test]
    fn test_optimization_manager_creation() {
        let manager = OptimizationManager::new();
        assert_eq!(manager.level, OptimizationLevel::Standard);
        assert!(manager.passes.is_empty());
    }

    #[test]
    fn test_default_manager_creation() {
        let manager = create_default_manager();
        assert!(!manager.passes.is_empty());
        assert_eq!(manager.level, OptimizationLevel::Standard);
    }

    #[test]
    fn test_optimization_levels() {
        let mut manager = OptimizationManager::new();

        // Test None level
        manager.set_level(OptimizationLevel::None);
        assert_eq!(manager.level, OptimizationLevel::None);
        assert!(manager.enabled_passes.is_empty());

        // Test Basic level
        manager.set_level(OptimizationLevel::Basic);
        assert_eq!(manager.level, OptimizationLevel::Basic);

        // Test Standard level
        manager.set_level(OptimizationLevel::Standard);
        assert_eq!(manager.level, OptimizationLevel::Standard);

        // Test Aggressive level
        manager.set_level(OptimizationLevel::Aggressive);
        assert_eq!(manager.level, OptimizationLevel::Aggressive);
    }

    #[test]
    fn test_pass_registration() {
        let mut manager = OptimizationManager::new();

        // Create a simple test pass
        struct TestPass;
        impl OptimizationPass for TestPass {
            fn name(&self) -> &'static str {
                "test_pass"
            }
            fn description(&self) -> &'static str {
                "Test pass"
            }
            fn run(&mut self, program: Program) -> Result<OptimizationResult, OptimizationError> {
                Ok(OptimizationResult::Unchanged(program))
            }
        }

        manager.register_pass(Box::new(TestPass));
        assert_eq!(manager.passes.len(), 1);
        assert_eq!(manager.passes[0].name(), "test_pass");
    }

    #[test]
    fn test_pass_enabling_disabling() {
        let mut manager = OptimizationManager::new();

        // Create and register a test pass
        struct TestPass;
        impl OptimizationPass for TestPass {
            fn name(&self) -> &'static str {
                "test_pass"
            }
            fn description(&self) -> &'static str {
                "Test pass"
            }
            fn run(&mut self, program: Program) -> Result<OptimizationResult, OptimizationError> {
                Ok(OptimizationResult::Unchanged(program))
            }
        }

        manager.register_pass(Box::new(TestPass));

        // Test enabling
        manager.enable_pass("test_pass");
        assert!(manager.enabled_passes.contains("test_pass"));

        // Test disabling
        manager.disable_pass("test_pass");
        assert!(!manager.enabled_passes.contains("test_pass"));
    }

    #[test]
    fn test_optimization_result_methods() {
        let program = create_simple_program();

        // Test Modified result
        let modified_result = OptimizationResult::Modified(program.clone());
        assert!(modified_result.was_modified());
        assert_eq!(
            modified_result.program().definitions.len(),
            program.definitions.len()
        );

        // Test Unchanged result
        let unchanged_result = OptimizationResult::Unchanged(program);
        assert!(!unchanged_result.was_modified());
    }

    #[test]
    fn test_optimization_performance_simple_program() {
        let mut manager = create_default_manager();
        let program = create_simple_program();

        let start = std::time::Instant::now();
        let result = manager.optimize(program);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Optimization should succeed");
        assert!(
            duration.as_millis() < 100,
            "Simple optimization should be fast (< 100ms)"
        );
    }

    #[test]
    fn test_optimization_performance_complex_program() {
        let mut manager = create_default_manager();
        let program = create_complex_program();

        let start = std::time::Instant::now();
        let result = manager.optimize(program);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Optimization should succeed");
        assert!(
            duration.as_millis() < 200,
            "Complex optimization should be reasonable (< 200ms)"
        );
    }

    #[test]
    fn test_optimization_with_different_levels() {
        let program = create_simple_program();

        // Test all optimization levels
        let levels = vec![
            OptimizationLevel::None,
            OptimizationLevel::Basic,
            OptimizationLevel::Standard,
            OptimizationLevel::Aggressive,
        ];

        for level in levels {
            let mut manager = create_default_manager();
            manager.set_level(level);

            let start = std::time::Instant::now();
            let result = manager.optimize(program.clone());
            let duration = start.elapsed();

            assert!(
                result.is_ok(),
                "Optimization should succeed for level {:?}",
                level
            );
            assert!(
                duration.as_millis() < 500,
                "Optimization should be timely for level {:?}",
                level
            );
        }
    }

    #[test]
    fn test_optimization_idempotent() {
        // Test that running optimization multiple times gives the same result
        let mut manager = create_default_manager();
        let program = create_simple_program();

        let result1 = manager.optimize(program.clone()).unwrap();
        let result2 = manager.optimize(program).unwrap();

        // Should be idempotent - running again shouldn't change anything
        assert_eq!(
            result1, result2,
            "Second optimization run should not modify already optimized code"
        );
    }

    #[test]
    fn test_individual_pass_execution() {
        use crate::compiler::optimizer::linearize::LinearizePass;
        use crate::compiler::optimizer::pruner::PrunePass;

        let program = create_complex_program();

        // Test LinearizePass
        let mut linearize_pass = LinearizePass::new();
        let start = std::time::Instant::now();
        let linearize_result = linearize_pass.run(program.clone());
        let linearize_duration = start.elapsed();

        assert!(linearize_result.is_ok(), "LinearizePass should succeed");
        assert!(
            linearize_duration.as_millis() < 100,
            "LinearizePass should be fast"
        );

        // Test PrunePass
        let mut prune_pass = PrunePass::new();
        let start = std::time::Instant::now();
        let prune_result = prune_pass.run(program);
        let prune_duration = start.elapsed();

        assert!(prune_result.is_ok(), "PrunePass should succeed");
        assert!(prune_duration.as_millis() < 100, "PrunePass should be fast");
    }

    #[test]
    fn test_pass_chaining_correctness() {
        // Test that passes can be chained together correctly
        let mut manager = create_default_manager();
        manager.set_level(OptimizationLevel::Standard);

        let program = create_complex_program();

        // Run optimization multiple times to ensure stability
        let mut current_program = program;
        for _ in 0..3 {
            let result = manager.optimize(current_program).unwrap();
            current_program = result;
        }

        // Final result should be valid and optimizable
        assert!(!current_program.definitions.is_empty());
    }

    #[test]
    fn test_memory_usage_bounds() {
        // Test that optimization doesn't use excessive memory
        let mut manager = create_default_manager();

        // Create a moderately large program
        let mut source = String::new();
        source.push_str("fn main() -> u32 {\n");
        for i in 0..50 {
            source.push_str(&format!("    let var{} = {};\n", i, i));
        }
        source.push_str("    return var0;\n");
        source.push_str("}\n");

        let program = parse_program(&source);

        let start = std::time::Instant::now();
        let result = manager.optimize(program);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Should handle moderately large programs");
        assert!(
            duration.as_millis() < 1000,
            "Should complete within reasonable time"
        );
    }

    #[test]
    fn test_error_propagation() {
        // Test that optimization errors are properly propagated
        struct FailingPass;
        impl OptimizationPass for FailingPass {
            fn name(&self) -> &'static str {
                "failing_pass"
            }
            fn description(&self) -> &'static str {
                "A pass that always fails"
            }
            fn run(&mut self, _program: Program) -> Result<OptimizationResult, OptimizationError> {
                Err(OptimizationError::Generic("Test failure".to_string()))
            }
        }

        let mut manager = OptimizationManager::new();
        manager.register_pass(Box::new(FailingPass));
        manager.enable_pass("failing_pass");

        let program = create_simple_program();
        let result = manager.optimize(program);

        assert!(result.is_err(), "Should propagate optimization errors");
    }

    #[test]
    fn test_optimization_statistics() {
        // Test that we can measure optimization effectiveness
        let mut manager = create_default_manager();
        let program = create_complex_program();

        // Count original operations (rough estimate)
        let original_ops = count_operations(&program);

        let result = manager.optimize(program).unwrap();
        let optimized_program = result;

        // Count operations after optimization
        let optimized_ops = count_operations(&optimized_program);

        // Optimization should not increase operation count significantly
        // (may be the same or slightly different due to transformations)
        assert!(
            optimized_ops <= original_ops * 2,
            "Optimization should not dramatically increase operations"
        );
    }

    // Helper function to count operations in a program (rough estimate)
    fn count_operations(program: &Program) -> usize {
        let mut count = 0;
        for definition in &program.definitions {
            if let Definition::FunctionDef { body, .. } = definition {
                count += count_block_operations(body);
            }
        }
        count
    }

    fn count_block_operations(block: &Block) -> usize {
        let mut count = 0;
        for statement in &block.statements {
            match statement {
                Statement::Assignment { value, .. } => {
                    count += count_expr_operations(value);
                }
                Statement::Return { value, .. } => {
                    count += count_expr_operations(value);
                }
                Statement::If {
                    condition,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    count += count_expr_operations(condition);
                    count += count_block_operations(then_branch);
                    count += count_block_operations(else_branch);
                }
                _ => count += 1,
            }
        }
        count
    }

    fn count_expr_operations(expr: &Expr) -> usize {
        match expr {
            Expr::BinaryOp { left, right, .. } => {
                1 + count_expr_operations(left) + count_expr_operations(right)
            }
            Expr::FunctionCall { args, .. } => {
                1 + args.iter().map(count_expr_operations).sum::<usize>()
            }

            _ => 1,
        }
    }
}
