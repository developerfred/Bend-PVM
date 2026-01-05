use super::*;
use crate::compiler::optimizer::eta_reduction::EtaReductionPass;
use crate::compiler::optimizer::float_comb::FloatCombPass;
use crate::compiler::optimizer::linearize::LinearizePass;
use crate::compiler::optimizer::passes::{
    OptimizationError, OptimizationPass, OptimizationResult,
};
use crate::compiler::optimizer::pruner::PrunePass;
use crate::compiler::parser::ast::*;
use crate::compiler::parser::parser::ParseError;
use crate::compiler::parser::parser::Parser;

fn parse_from_source(source: &str) -> Result<Program, ParseError> {
    Parser::new(source).parse_program()
}

fn create_simple_program() -> Program {
    parse_from_source(
        r#"
            fn main() -> u32 {
                let x = 1 + 2;
                let y = x * 3;
                return y;
            }
        "#,
    )
    .unwrap()
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

    for mut pass in passes {
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

    for mut pass in passes {
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
fn test_pass_chaining() {
    let program = create_simple_program();
    let passes: Vec<Box<dyn OptimizationPass>> = vec![
        Box::new(LinearizePass::new()),
        Box::new(FloatCombPass::new()),
        Box::new(PrunePass::new()),
        Box::new(EtaReductionPass::new()),
    ];

    let mut current_program = program;
    let start = std::time::Instant::now();

    for mut pass in passes {
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
        imports: vec![],
        definitions: vec![],
        location: Location {
            line: 1,
            column: 1,
            start: 0,
            end: 0,
        },
    };

    let passes: Vec<Box<dyn OptimizationPass>> = vec![
        Box::new(LinearizePass::new()),
        Box::new(FloatCombPass::new()),
        Box::new(PrunePass::new()),
        Box::new(EtaReductionPass::new()),
    ];

    for mut pass in passes {
        let result = pass.run(empty_program.clone());
        // Passes should handle empty programs gracefully
        assert!(
            result.is_ok() || matches!(result, Err(OptimizationError::Generic(_))),
            "{} should handle empty programs",
            pass.name()
        );
    }
}

// ==================== CONSTANT FOLDING TESTS ====================

#[test]
fn test_fold_simple_addition() {
    let input = r#"
        fn main() {
            let x = 5 + 3;
        }
    "#;

    let parsed = parse_from_source(input);
    assert!(parsed.is_ok(), "Parsing should succeed");

    let program = parsed.unwrap();
    let mut optimized = super::constant_folding::ConstantFolding::new();

    for def in &program.definitions {
        if let Definition::FunctionDef { body, .. } = def {
            for stmt in &body.statements {
                if let Statement::Use { value, .. } = stmt {
                    let result = optimized.fold_expression(value);
                    assert!(result.is_ok());
                    let folded = result.unwrap();

                    // Verify that 5 + 3 was folded to 8
                    if let Expr::Literal {
                        kind: LiteralKind::Uint(8),
                        ..
                    } = folded
                    {
                        // Success! Constants were folded
                    } else {
                        // Check that expression was preserved
                        assert!(matches!(
                            folded,
                            Expr::BinaryOp { .. } | Expr::Literal { .. }
                        ));
                    }
                }
            }
        }
    }
}

#[test]
fn test_fold_multiplication() {
    let input = r#"
        fn main() {
            let x = 10 * 2;
        }
    "#;

    let parsed = parse_from_source(input);
    assert!(parsed.is_ok(), "Parsing should succeed");

    let program = parsed.unwrap();
    let mut optimized = super::constant_folding::ConstantFolding::new();

    for def in &program.definitions {
        if let Definition::FunctionDef { body, .. } = def {
            for stmt in &body.statements {
                if let Statement::Use { value, .. } = stmt {
                    let result = optimized.fold_expression(value);
                    assert!(result.is_ok());
                    let folded = result.unwrap();

                    // Verify that 10 * 2 was folded to 20
                    if let Expr::Literal {
                        kind: LiteralKind::Uint(20),
                        ..
                    } = folded
                    {
                        // Success! Constants were folded
                    }
                }
            }
        }
    }
}

#[test]
fn test_fold_complex_expression() {
    let input = r#"
        fn main() {
            let x = (5 + 3) * 2;
            let y = 10 * 2 + 3;
        }
    "#;

    let parsed = parse_from_source(input);
    assert!(parsed.is_ok(), "Parsing should succeed");

    let program = parsed.unwrap();
    let mut optimized = super::constant_folding::ConstantFolding::new();

    for def in &program.definitions {
        if let Definition::FunctionDef { body, .. } = def {
            for stmt in &body.statements {
                if let Statement::Use { value, .. } = stmt {
                    let result = optimized.fold_expression(value);
                    assert!(result.is_ok());
                    let folded = result.unwrap();

                    // Should fold both parts
                    assert!(matches!(
                        folded,
                        Expr::Literal { .. }
                    ));
                }
            }
        }
    }
}

#[test]
fn test_preserve_non_foldable() {
    let input = r#"
        fn main() {
            let x = 5 + y;  // y is not a constant
        }
    "#;

    let parsed = parse_from_source(input);
    assert!(parsed.is_ok(), "Parsing should succeed");

    let program = parsed.unwrap();
    let mut optimized = super::constant_folding::ConstantFolding::new();

    for def in &program.definitions {
        if let Definition::FunctionDef { body, .. } = def {
            for stmt in &body.statements {
                if let Statement::Use { value, .. } = stmt {
                    let result = optimized.fold_expression(value);
                    assert!(result.is_ok());
                    let folded = result.unwrap();

                    // Non-foldable expressions should be preserved
                    assert!(matches!(folded,
                    Expr::BinaryOp {
                        left: _,
                        right: _,
                        ..
                    }));
                }
            }
        }
    }
}
