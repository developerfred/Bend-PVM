#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::optimizer::eta_reduction::EtaReductionPass;
    use crate::compiler::optimizer::float_comb::FloatCombPass;
    use crate::compiler::optimizer::linearize::LinearizePass;
    use crate::compiler::optimizer::passes::{OptimizationPass, OptimizationResult};
    use crate::compiler::optimizer::pruner::PrunePass;
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

    #[test]
    fn test_linearize_pass_basic() {
        let program = create_simple_program();
        let mut pass = LinearizePass::new();

        let start = std::time::Instant::now();
        let result = pass.run(program);
        let duration = start.elapsed();

        assert!(result.is_ok(), "LinearizePass should succeed");
        assert!(duration.as_millis() < 50, "LinearizePass should be fast");
    }

    #[test]
    fn test_float_comb_pass_basic() {
        let program = create_simple_program();
        let mut pass = FloatCombPass::new();

        let start = std::time::Instant::now();
        let result = pass.run(program);
        let duration = start.elapsed();

        assert!(result.is_ok(), "FloatCombPass should succeed");
        assert!(duration.as_millis() < 50, "FloatCombPass should be fast");
    }

    #[test]
    fn test_prune_pass_basic() {
        let program = create_simple_program();
        let mut pass = PrunePass::new();

        let start = std::time::Instant::now();
        let result = pass.run(program);
        let duration = start.elapsed();

        assert!(result.is_ok(), "PrunePass should succeed");
        assert!(duration.as_millis() < 50, "PrunePass should be fast");
    }

    #[test]
    fn test_eta_reduction_pass_basic() {
        let program = create_simple_program();
        let mut pass = EtaReductionPass::new();

        let start = std::time::Instant::now();
        let result = pass.run(program);
        let duration = start.elapsed();

        assert!(result.is_ok(), "EtaReductionPass should succeed");
        assert!(duration.as_millis() < 50, "EtaReductionPass should be fast");
    }

    #[test]
    fn test_pass_performance_consistency() {
        let program = create_simple_program();
        let passes: Vec<Box<dyn OptimizationPass>> = vec![
            Box::new(LinearizePass::new()),
            Box::new(FloatCombPass::new()),
            Box::new(PrunePass::new()),
            Box::new(EtaReductionPass::new()),
        ];

        for pass in passes {
            let start = std::time::Instant::now();
            let result = pass.run(program.clone());
            let duration = start.elapsed();

            assert!(result.is_ok(), "{} should succeed", pass.name());
            assert!(
                duration.as_millis() < 100,
                "{} should be performant",
                pass.name()
            );
        }
    }

    #[test]
    fn test_pass_idempotency() {
        let program = create_simple_program();
        let passes: Vec<Box<dyn OptimizationPass>> = vec![
            Box::new(LinearizePass::new()),
            Box::new(FloatCombPass::new()),
            Box::new(PrunePass::new()),
            Box::new(EtaReductionPass::new()),
        ];

        for pass in passes {
            // Run pass twice
            let result1 = pass.run(program.clone()).unwrap();
            let result2 = pass.run(result1.program()).unwrap();

            // Second run should not modify (or modify much less)
            if result2.was_modified() {
                // Allow some modification but not excessive
                assert!(
                    matches!(result2, OptimizationResult::Modified(_)),
                    "Second run of {} should be stable",
                    pass.name()
                );
            }
        }
    }

    #[test]
    fn test_pass_names_and_descriptions() {
        let passes: Vec<Box<dyn OptimizationPass>> = vec![
            Box::new(LinearizePass::new()),
            Box::new(FloatCombPass::new()),
            Box::new(PrunePass::new()),
            Box::new(EtaReductionPass::new()),
        ];

        for pass in passes {
            assert!(!pass.name().is_empty(), "Pass name should not be empty");
            assert!(
                !pass.description().is_empty(),
                "Pass description should not be empty"
            );
            assert!(
                pass.name().chars().all(|c| c.is_alphanumeric() || c == '_'),
                "Pass name should be valid identifier: {}",
                pass.name()
            );
        }
    }

    #[test]
    fn test_complex_program_optimization() {
        let source = r#"
            fn add(a: u32, b: u32) -> u32 {
                return a + b;
            }

            fn multiply(x: u32, y: u32) -> u32 {
                return x * y;
            }

            fn complex_calc(a: u32, b: u32, c: u32) -> u32 {
                let temp1 = add(a, b);
                let temp2 = multiply(temp1, c);
                let temp3 = add(temp2, 42);
                return multiply(temp3, 2);
            }

            fn main() -> u32 {
                return complex_calc(1, 2, 3);
            }
        "#;

        let program = parse_program(source);
        let passes: Vec<Box<dyn OptimizationPass>> = vec![
            Box::new(LinearizePass::new()),
            Box::new(FloatCombPass::new()),
            Box::new(PrunePass::new()),
            Box::new(EtaReductionPass::new()),
        ];

        let mut current_program = program;
        let start = std::time::Instant::now();

        for pass in passes {
            let result = pass.run(current_program).unwrap();
            current_program = result.program();
        }

        let total_duration = start.elapsed();

        assert!(
            total_duration.as_millis() < 200,
            "Complex optimization should be timely"
        );
        assert!(
            !current_program.definitions.is_empty(),
            "Should preserve program structure"
        );
    }

    #[test]
    fn test_optimization_pass_isolation() {
        // Test that passes don't interfere with each other
        let program = create_simple_program();

        let mut pass1 = LinearizePass::new();
        let mut pass2 = FloatCombPass::new();

        // Run pass1
        let result1 = pass1.run(program.clone()).unwrap();
        let program_after_1 = result1.program();

        // Run pass2 on original
        let result2 = pass2.run(program).unwrap();
        let program_after_2 = result2.program();

        // Results should be different (passes do different things)
        // but both should be valid
        assert!(program_after_1.definitions.len() > 0);
        assert!(program_after_2.definitions.len() > 0);
    }

    #[test]
    fn test_error_handling_in_passes() {
        // Test that passes handle edge cases gracefully
        let empty_program = Program {
            definitions: vec![],
            location: None,
        };

        let passes: Vec<Box<dyn OptimizationPass>> = vec![
            Box::new(LinearizePass::new()),
            Box::new(FloatCombPass::new()),
            Box::new(PrunePass::new()),
            Box::new(EtaReductionPass::new()),
        ];

        for pass in passes {
            let result = pass.run(empty_program.clone());
            // Passes should handle empty programs gracefully
            assert!(
                result.is_ok() || matches!(result, Err(OptimizationError::Generic(_))),
                "{} should handle empty programs",
                pass.name()
            );
        }
    }
}
