use bend_pvm::debugger::{
    breakpoint::Breakpoint,
    state::{DebuggerState, ExecutionState},
    DebugInfo, FunctionRange, VariableLocation,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
mod debugger_state_tests {
    use super::*;

    #[test]
    fn test_debugger_state_new() {
        let state = DebuggerState::new();
        assert_eq!(state.execution_state, ExecutionState::Stopped);
        assert_eq!(state.pc, 0);
        assert!(state.call_stack.is_empty());
        assert!(state.registers.is_empty());
    }

    #[test]
    fn test_set_and_get_register() {
        let mut state = DebuggerState::new();
        state.set_register("x1", 42);
        assert_eq!(state.get_register("x1"), Some(42));
    }

    #[test]
    fn test_set_and_get_memory() {
        let mut state = DebuggerState::new();
        state.set_memory(0x1000, 0xAB);
        assert_eq!(state.get_memory(0x1000), Some(0xAB));
    }

    #[test]
    fn test_set_and_get_local_variable() {
        let mut state = DebuggerState::new();
        state.set_local_variable("counter", 100);
        assert_eq!(state.get_local_variable("counter"), Some(100));
    }

    #[test]
    fn test_call_stack_push_pop() {
        let mut state = DebuggerState::new();
        state.call_stack.push("main".to_string());
        state.call_stack.push("helper".to_string());
        assert_eq!(state.current_function(), Some("helper"));
        state.call_stack.pop();
        assert_eq!(state.current_function(), Some("main"));
    }

    #[test]
    fn test_reset_clears_state() {
        let mut state = DebuggerState::new();
        state.set_register("x1", 42);
        state.set_memory(0x1000, 0xAB);
        state.pc = 100;
        state.call_stack.push("test".to_string());

        state.reset();

        assert_eq!(state.execution_state, ExecutionState::Stopped);
        assert_eq!(state.pc, 0);
        assert!(state.call_stack.is_empty());
        assert!(state.registers.is_empty());
        assert!(state.memory.is_empty());
    }

    #[test]
    fn test_execution_state_transitions() {
        let mut state = DebuggerState::new();
        assert_eq!(state.execution_state, ExecutionState::Stopped);

        state.execution_state = ExecutionState::Running;
        assert_eq!(state.execution_state, ExecutionState::Running);

        state.execution_state = ExecutionState::Paused;
        assert_eq!(state.execution_state, ExecutionState::Paused);
    }
}

#[cfg(test)]
mod breakpoint_tests {
    use super::*;

    #[test]
    fn test_breakpoint_line_creation() {
        let bp = Breakpoint::line(42);
        assert_eq!(bp, Breakpoint::Line(42));
    }

    #[test]
    fn test_breakpoint_instruction_creation() {
        let bp = Breakpoint::instruction(100);
        assert_eq!(bp, Breakpoint::Instruction(100));
    }

    #[test]
    fn test_breakpoint_function_creation() {
        let bp = Breakpoint::function("main");
        assert_eq!(bp, Breakpoint::Function("main".to_string()));
    }

    #[test]
    fn test_breakpoint_description() {
        let bp = Breakpoint::Line(42);
        assert_eq!(bp.description(), "Line 42");

        let bp = Breakpoint::Instruction(100);
        assert_eq!(bp.description(), "Instruction 100");

        let bp = Breakpoint::Function("main".to_string());
        assert_eq!(bp.description(), "Function main");
    }
}

#[cfg(test)]
mod debug_info_tests {
    use super::*;

    fn create_test_debug_info() -> DebugInfo {
        DebugInfo {
            source_path: PathBuf::from("test.bend"),
            source_code: "fn main() {\n    let x = 1;\n    let y = 2;\n}".to_string(),
            line_to_instruction: HashMap::from([(1, vec![0]), (2, vec![1]), (3, vec![2])]),
            instruction_to_line: HashMap::from([(0, 1), (1, 2), (2, 3)]),
            functions: HashMap::from([(
                "main".to_string(),
                FunctionRange {
                    name: "main".to_string(),
                    start: 0,
                    end: 10,
                    start_line: 1,
                    end_line: 3,
                },
            )]),
            locals: HashMap::from([
                ("x".to_string(), VariableLocation::Stack(4)),
                ("y".to_string(), VariableLocation::Stack(8)),
            ]),
        }
    }

    #[test]
    fn test_debug_info_creation() {
        let debug_info = create_test_debug_info();
        assert_eq!(debug_info.source_path, PathBuf::from("test.bend"));
        assert_eq!(debug_info.line_to_instruction.len(), 3);
        assert_eq!(debug_info.instruction_to_line.len(), 3);
    }

    #[test]
    fn test_line_to_instruction_mapping() {
        let debug_info = create_test_debug_info();
        assert_eq!(debug_info.line_to_instruction.get(&1), Some(&vec![0]));
        assert_eq!(debug_info.line_to_instruction.get(&2), Some(&vec![1]));
    }

    #[test]
    fn test_instruction_to_line_mapping() {
        let debug_info = create_test_debug_info();
        assert_eq!(debug_info.instruction_to_line.get(&0), Some(&1));
        assert_eq!(debug_info.instruction_to_line.get(&1), Some(&2));
    }

    #[test]
    fn test_function_ranges() {
        let debug_info = create_test_debug_info();
        let func = debug_info.functions.get("main").unwrap();
        assert_eq!(func.name, "main");
        assert_eq!(func.start, 0);
        assert_eq!(func.end, 10);
    }

    #[test]
    fn test_locals_mapping() {
        let debug_info = create_test_debug_info();
        if let Some(loc) = debug_info.locals.get("x") {
            assert!(matches!(loc, VariableLocation::Stack(4)));
        } else {
            panic!("x not found in locals");
        }
        if let Some(loc) = debug_info.locals.get("y") {
            assert!(matches!(loc, VariableLocation::Stack(8)));
        } else {
            panic!("y not found in locals");
        }
    }
}

#[cfg(test)]
mod expression_evaluator_tests {
    use super::*;
    use bend_pvm::debugger::inspector::DebugInspector;

    fn create_test_inspector() -> DebugInspector {
        let debug_info = DebugInfo {
            source_path: PathBuf::from("test.bend"),
            source_code: "fn main() {\n    let x = 1;\n    let y = 2;\n}".to_string(),
            line_to_instruction: HashMap::new(),
            instruction_to_line: HashMap::new(),
            functions: HashMap::new(),
            locals: HashMap::from([
                ("x".to_string(), VariableLocation::Stack(4)),
                ("y".to_string(), VariableLocation::Stack(8)),
            ]),
        };

        let mut state = DebuggerState::new();
        state.set_local_variable("x", 10);
        state.set_local_variable("y", 20);
        state.registers.insert("x1".to_string(), 5);
        state.registers.insert("x2".to_string(), 15);

        DebugInspector::new(debug_info, state, Vec::new())
    }

    #[test]
    fn test_evaluate_literal_number() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_evaluate_simple_addition() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("5 + 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "8");
    }

    #[test]
    fn test_evaluate_simple_subtraction() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("10 - 4");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "6");
    }

    #[test]
    fn test_evaluate_simple_multiplication() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("6 * 7");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");
    }

    #[test]
    fn test_evaluate_simple_division() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("20 / 4");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5");
    }

    #[test]
    fn test_evaluate_local_variable() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("x");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "10");
    }

    #[test]
    fn test_evaluate_register() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("x1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "5");
    }

    #[test]
    fn test_evaluate_expression_with_variables() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("x + y");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "30");
    }

    #[test]
    fn test_evaluate_complex_expression() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("(x + y) * 2");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "60");
    }

    #[test]
    fn test_evaluate_comparison_equality() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("5 == 5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_evaluate_comparison_inequality() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("5 != 3");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_evaluate_comparison_less_than() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("3 < 5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_evaluate_comparison_greater_than() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("10 > 5");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true");
    }

    #[test]
    fn test_evaluate_undefined_variable_returns_error() {
        let inspector = create_test_inspector();
        let result = inspector.evaluate("undefined_var");
        assert!(result.is_err());
    }
}
