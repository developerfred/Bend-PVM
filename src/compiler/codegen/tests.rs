#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::codegen::risc_v::{CodegenError, Instruction, Register, RiscVCodegen};
    use crate::compiler::parser::ast::*;
    use crate::compiler::parser::parser::Parser;

    fn parse_program(source: &str) -> Program {
        let mut parser = Parser::new(source);
        parser.parse_program().unwrap()
    }

    fn generate_code(source: &str) -> Result<Vec<Instruction>, CodegenError> {
        let mut codegen = RiscVCodegen::new();
        let program = parse_program(source);
        codegen.generate(&program)
    }

    #[test]
    fn test_simple_function_codegen() {
        let source = r#"
            fn main() -> u32 {
                return 42;
            }
        "#;

        let instructions = generate_code(source).unwrap();

        // Should generate at least the main label and return instruction
        assert!(!instructions.is_empty());

        // Check for main label
        let main_label = instructions
            .iter()
            .find(|inst| matches!(inst, Instruction::Label(label) if label == "main"));
        assert!(main_label.is_some(), "Should generate main label");

        // Check for return value loading
        let li_42 = instructions
            .iter()
            .find(|inst| matches!(inst, Instruction::Li(reg, 42) if reg.to_string() == "t0"));
        assert!(li_42.is_some(), "Should generate li t0, 42");

        // Check for return value move to a0
        let mv_a0 = instructions.iter().find(|inst| {
            matches!(inst, Instruction::Mv(dest, src) if dest.to_string() == "a0" && src.to_string() == "t0")
        });
        assert!(mv_a0.is_some(), "Should generate mv a0, t0");
    }

    #[test]
    fn test_function_with_parameters() {
        let source = r#"
            fn add(a: u32, b: u32) -> u32 {
                return a + b;
            }
        "#;

        let instructions = generate_code(source).unwrap();
        assert!(!instructions.is_empty());

        // Check for function label
        let add_label = instructions
            .iter()
            .find(|inst| matches!(inst, Instruction::Label(label) if label == "function.add"));
        assert!(add_label.is_some(), "Should generate function.add label");
    }

    #[test]
    fn test_binary_operations() {
        let source = r#"
            fn calc(x: u32, y: u32) -> u32 {
                return x + y;
            }
        "#;

        let instructions = generate_code(source).unwrap();

        // Should contain addition instruction
        let add_inst = instructions
            .iter()
            .find(|inst| matches!(inst, Instruction::Add(_, _, _)));
        assert!(add_inst.is_some(), "Should generate add instruction");
    }

    #[test]
    fn test_variable_assignment() {
        let source = r#"
            fn test() -> u32 {
                let x = 10;
                return x;
            }
        "#;

        let instructions = generate_code(source).unwrap();
        assert!(!instructions.is_empty());

        // Should contain store and load instructions for variable
        let store_inst = instructions
            .iter()
            .find(|inst| matches!(inst, Instruction::Store(_, _, _)));
        assert!(
            store_inst.is_some(),
            "Should generate store instruction for variable"
        );

        let load_inst = instructions
            .iter()
            .find(|inst| matches!(inst, Instruction::Load(_, _, _)));
        assert!(
            load_inst.is_some(),
            "Should generate load instruction for variable"
        );
    }

    #[test]
    fn test_function_call() {
        let source = r#"
            fn add(a: u32, b: u32) -> u32 {
                return a + b;
            }

            fn main() -> u32 {
                return add(5, 3);
            }
        "#;

        let instructions = generate_code(source).unwrap();

        // Should contain jal instruction for function call
        let jal_inst = instructions
            .iter()
            .find(|inst| matches!(inst, Instruction::JumpAndLink(_, _)));
        assert!(
            jal_inst.is_some(),
            "Should generate jal instruction for function call"
        );
    }

    #[test]
    fn test_if_statement() {
        let source = r#"
            fn test(x: u32) -> u32 {
                if x > 0 {
                    return 1;
                } else {
                    return 0;
                }
            }
        "#;

        let instructions = generate_code(source).unwrap();

        // Should contain branch instructions
        let branch_inst = instructions.iter().find(|inst| {
            matches!(
                inst,
                Instruction::BranchNe(_, _, _)
                    | Instruction::BranchLt(_, _, _)
                    | Instruction::BranchGe(_, _, _)
            )
        });
        assert!(
            branch_inst.is_some(),
            "Should generate branch instruction for if statement"
        );

        // Should contain labels for then/else branches
        let labels: Vec<_> = instructions
            .iter()
            .filter(|inst| matches!(inst, Instruction::Label(_)))
            .collect();
        assert!(labels.len() >= 3, "Should generate labels for if branches"); // main, then, else, end
    }

    #[test]
    fn test_multiple_functions() {
        let source = r#"
            fn helper() -> u32 {
                return 42;
            }

            fn main() -> u32 {
                return helper();
            }
        "#;

        let instructions = generate_code(source).unwrap();

        // Should contain both function labels
        let labels: Vec<_> = instructions
            .iter()
            .filter_map(|inst| match inst {
                Instruction::Label(label) => Some(label.as_str()),
                _ => None,
            })
            .collect();

        assert!(labels.contains(&"main"), "Should contain main label");
        assert!(
            labels.contains(&"function.helper"),
            "Should contain helper function label"
        );
    }

    #[test]
    fn test_code_generation_error_handling() {
        let source = r#"
            fn test() -> u32 {
                return undefined_var;
            }
        "#;

        let result = generate_code(source);
        assert!(
            result.is_err(),
            "Should return error for undefined variable"
        );
    }

    #[test]
    fn test_register_allocation() {
        // Test that registers are properly allocated and don't conflict
        let source = r#"
            fn complex_calc(a: u32, b: u32, c: u32) -> u32 {
                let x = a + b;
                let y = x * c;
                return y - a;
            }
        "#;

        let instructions = generate_code(source).unwrap();

        // Count register usage - should use different registers for different operations
        let mut register_usage = std::collections::HashMap::new();

        for inst in &instructions {
            match inst {
                Instruction::Add(rd, _, _)
                | Instruction::Sub(rd, _, _)
                | Instruction::Mul(rd, _, _)
                | Instruction::Load(rd, _, _)
                | Instruction::Li(rd, _) => {
                    *register_usage.entry(rd.to_string()).or_insert(0) += 1;
                }
                _ => {}
            }
        }

        // Should use multiple registers
        assert!(
            register_usage.len() > 1,
            "Should use multiple registers for complex operations"
        );
    }

    #[test]
    fn test_stack_frame_management() {
        let source = r#"
            fn test() -> u32 {
                let a = 1;
                let b = 2;
                let c = 3;
                return a + b + c;
            }
        "#;

        let instructions = generate_code(source).unwrap();

        // Should contain stack pointer adjustments
        let sp_adjustments: Vec<_> = instructions.iter().filter(|inst| {
            matches!(inst, Instruction::AddImm(reg, _, offset) if reg == &Register::X2 && *offset < 0)
        }).collect();

        assert!(
            !sp_adjustments.is_empty(),
            "Should adjust stack pointer for local variables"
        );
    }

    #[test]
    fn test_performance_complex_expression() {
        // Test code generation performance with complex expressions
        let source = r#"
            fn complex() -> u32 {
                let a = 1;
                let b = 2;
                let c = 3;
                let d = 4;
                let e = 5;
                return ((a + b) * c - d) / e;
            }
        "#;

        let start = std::time::Instant::now();
        let instructions = generate_code(source).unwrap();
        let duration = start.elapsed();

        // Should generate reasonable number of instructions
        assert!(
            instructions.len() > 10,
            "Should generate sufficient instructions for complex expression"
        );

        // Performance check - should be fast
        assert!(
            duration.as_millis() < 100,
            "Code generation should be fast (< 100ms)"
        );
    }

    #[test]
    fn test_performance_large_program() {
        // Generate a program with many functions
        let mut source = String::new();
        source.push_str("fn main() -> u32 { return 0; }\n");

        for i in 1..20 {
            source.push_str(&format!(
                "fn func{}(x: u32) -> u32 {{ return x + {}; }}\n",
                i, i
            ));
        }

        let start = std::time::Instant::now();
        let instructions = generate_code(&source).unwrap();
        let duration = start.elapsed();

        // Should generate reasonable number of instructions
        assert!(
            instructions.len() > 50,
            "Should generate instructions for all functions"
        );

        // Performance check - should handle large programs reasonably
        assert!(
            duration.as_millis() < 500,
            "Should handle large programs within reasonable time (< 500ms)"
        );
    }

    #[test]
    fn test_instruction_output_format() {
        let source = r#"
            fn test() -> u32 {
                return 42;
            }
        "#;

        let instructions = generate_code(source).unwrap();

        // Test that instructions can be formatted to string
        for inst in instructions {
            let formatted = format!("{}", inst);
            assert!(
                !formatted.is_empty(),
                "Instruction should format to non-empty string"
            );

            // Should not contain internal debug info
            assert!(
                !formatted.contains("Instruction::"),
                "Formatted instruction should not contain internal type names"
            );
        }
    }

    #[test]
    fn test_label_generation_uniqueness() {
        let source = r#"
            fn test() -> u32 {
                if true {
                    return 1;
                } else {
                    return 0;
                }
            }
        "#;

        let instructions = generate_code(source).unwrap();

        let labels: Vec<_> = instructions
            .iter()
            .filter_map(|inst| match inst {
                Instruction::Label(label) => Some(label.clone()),
                _ => None,
            })
            .collect();

        // All labels should be unique
        let mut seen = std::collections::HashSet::new();
        for label in labels {
            assert!(!seen.contains(&label), "Label {} should be unique", label);
            seen.insert(label);
        }
    }

    #[test]
    fn test_unsupported_features_error() {
        // Test that unsupported features return appropriate errors
        // This would depend on what features are not yet implemented
        // For now, we'll test with a feature that should be implemented
        let source = r#"
            fn test() -> u32 {
                return 42;
            }
        "#;

        let result = generate_code(source);
        assert!(result.is_ok(), "Basic features should be supported");
    }
}
