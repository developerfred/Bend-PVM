//! # Parser Tests
//!
//! Tests for the Bend-PVM parser implementation.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_basic_function() {
        let source = r#"
fn add(x: u24, y: u24) -> u24 {
    x + y
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.definitions.len(), 1);
        match &program.definitions[0] {
            Definition::FunctionDef {
                name,
                params,
                return_type,
                body,
                ..
            } => {
                assert_eq!(name, "add");
                assert_eq!(params.len(), 2);
                assert!(return_type.is_some());
                assert_eq!(body.statements.len(), 1);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parser_type_definition() {
        let source = r#"
type Option<T> {
    None,
    Some(T),
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.definitions.len(), 1);
        match &program.definitions[0] {
            Definition::TypeDef {
                name,
                type_params,
                variants,
                ..
            } => {
                assert_eq!(name, "Option");
                assert_eq!(type_params.len(), 1);
                assert_eq!(variants.len(), 2);
            }
            _ => panic!("Expected type definition"),
        }
    }

    #[test]
    fn test_parser_complex_expression() {
        let source = r#"
fn test() -> u24 {
    let x = 5 + 3 * 2;
    let y = if x > 10 { x } else { 0 };
    y
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.definitions.len(), 1);
        match &program.definitions[0] {
            Definition::FunctionDef { body, .. } => {
                assert_eq!(body.statements.len(), 3);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parser_pattern_matching() {
        let source = r#"
fn test(value: Option<u24>) -> u24 {
    match value {
        None => 0,
        Some(x) => x,
    }
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.definitions.len(), 1);
        match &program.definitions[0] {
            Definition::FunctionDef { body, .. } => {
                assert_eq!(body.statements.len(), 1);
                match &body.statements[0] {
                    Statement::Match { cases, .. } => {
                        assert_eq!(cases.len(), 2);
                    }
                    _ => panic!("Expected match statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parser_lambda_expression() {
        let source = r#"
fn test() -> u24 {
    let double = |x: u24| x * 2;
    double(5)
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.definitions.len(), 1);
        match &program.definitions[0] {
            Definition::FunctionDef { body, .. } => {
                assert_eq!(body.statements.len(), 2);
                match &body.statements[0] {
                    Statement::Use { value, .. } => match value {
                        Expr::Lambda { params, .. } => {
                            assert_eq!(params.len(), 1);
                        }
                        _ => panic!("Expected lambda expression"),
                    },
                    _ => panic!("Expected use statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parser_import_statement() {
        let source = r#"
from Math import sin, cos as cosine;
import Utils;

fn test() {
    sin(0)
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.imports.len(), 2);
        assert_eq!(program.definitions.len(), 1);

        // Test from import
        match &program.imports[0] {
            Import::FromImport { path, names, .. } => {
                assert_eq!(path, "Math");
                assert_eq!(names.len(), 2);
                assert_eq!(names[0].name, "sin");
                assert!(names[0].alias.is_none());
                assert_eq!(names[1].name, "cos");
                assert_eq!(names[1].alias.as_ref().unwrap(), "cosine");
            }
            _ => panic!("Expected from import"),
        }

        // Test direct import
        match &program.imports[1] {
            Import::DirectImport { names, .. } => {
                assert_eq!(names.len(), 1);
                assert_eq!(names[0], "Utils");
            }
            _ => panic!("Expected direct import"),
        }
    }

    #[test]
    fn test_parser_error_handling() {
        let source = "fn test() {";
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        // Should fail due to unclosed brace
        assert!(result.is_err());
    }

    #[test]
    fn test_parser_empty_program() {
        let source = "";
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.imports.len(), 0);
        assert_eq!(program.definitions.len(), 0);
    }

    #[test]
    fn test_parser_object_definition() {
        let source = r#"
object Counter {
    let value: u24;
    let max_value: u24;

    fn init(initial: u24, max: u24) {
        self.value = initial;
        self.max_value = max;
    }

    fn increment() -> bool {
        if self.value < self.max_value {
            self.value = self.value + 1;
            true
        } else {
            false
        }
    }
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.definitions.len(), 1);
        match &program.definitions[0] {
            Definition::ObjectDef {
                name,
                fields,
                functions,
                ..
            } => {
                assert_eq!(name, "Counter");
                assert_eq!(fields.len(), 2);
                assert_eq!(functions.len(), 2);
            }
            _ => panic!("Expected object definition"),
        }
    }

    #[test]
    fn test_parser_array_and_map_types() {
        let source = r#"
fn test() {
    let arr: List<u24> = [1, 2, 3];
    let map: Map<String, u24> = Map::new();
    let tuple: (u24, String) = (42, "hello");
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.definitions.len(), 1);
        match &program.definitions[0] {
            Definition::FunctionDef { body, .. } => {
                assert_eq!(body.statements.len(), 3);
            }
            _ => panic!("Expected function definition"),
        }
    }

    #[test]
    fn test_parser_bend_construct() {
        let source = r#"
fn factorial(n: u24) -> u24 {
    bend x = 1, acc = 1 {
        if acc > n {
            x
        } else {
            factorial(acc + 1, x * acc)
        }
    }
}
"#;
        let mut parser = Parser::new(source);
        let result = parser.parse_program();

        assert!(result.is_ok());
        let program = result.unwrap();

        assert_eq!(program.definitions.len(), 1);
        match &program.definitions[0] {
            Definition::FunctionDef { body, .. } => {
                assert_eq!(body.statements.len(), 1);
                match &body.statements[0] {
                    Statement::Bend { initial_states, .. } => {
                        assert_eq!(initial_states.len(), 2);
                    }
                    _ => panic!("Expected bend statement"),
                }
            }
            _ => panic!("Expected function definition"),
        }
    }
}
