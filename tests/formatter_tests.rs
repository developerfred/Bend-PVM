//! Tests for Bend-PVM Formatter - TDD approach
//!
//! RED PHASE: These tests define expected behavior
//! All tests should FAIL initially, then pass after implementation

use bend_pvm::formatter::{FormatResult, Formatter, FormatterConfig};

#[cfg(test)]
mod formatter_config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FormatterConfig::default();
        assert_eq!(config.indent_size, 4);
        assert_eq!(config.max_line_length, 120);
        assert!(!config.use_tabs);
        assert!(config.blank_line_after_fn);
        assert!(config.space_around_operators);
    }

    #[test]
    fn test_custom_config() {
        let config = FormatterConfig {
            indent_size: 2,
            max_line_length: 100,
            use_tabs: true,
            blank_line_after_fn: false,
            space_around_operators: false,
        };
        assert_eq!(config.indent_size, 2);
        assert!(config.use_tabs);
    }
}

#[cfg(test)]
mod basic_formatting_tests {
    use super::*;

    #[test]
    fn test_removes_extra_spaces_between_tokens() {
        let mut formatter = Formatter::new();
        let input = "fn   test(  x:i32,   y:i32  ) ->  i32  {";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("fn test(x: i32, y: i32) -> i32 {"));
        let content_without_indent = result.trim();
        assert!(!content_without_indent.contains("  "));
    }

    #[test]
    fn test_removes_trailing_whitespace() {
        let mut formatter = Formatter::new();
        let input = "fn test() {    \n    return 1;     \n}";
        let result = formatter.format_source(input).unwrap();
        assert!(!result.contains("     \n")); // no trailing whitespace before newline
    }

    #[test]
    fn test_normalizes_operator_spacing() {
        let mut formatter = Formatter::new();
        let input = "let x = a+b;";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("let x = a + b;"));
    }
}

#[cfg(test)]
mod indentation_tests {
    use super::*;

    #[test]
    fn test_indents_function_body() {
        let mut formatter = Formatter::new();
        let input = "fn test() {\nreturn 1;\n}";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("    return 1")); // 4 spaces
    }

    #[test]
    fn test_indents_nested_blocks() {
        let mut formatter = Formatter::new();
        let input = "fn outer() {\nif true {\nreturn 1;\n}\n}";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("        return 1")); // 8 spaces (nested)
    }

    #[test]
    fn test_dedent_closing_brace() {
        let mut formatter = Formatter::new();
        let input = "fn test() {\nreturn 1;\n    }";
        let result = formatter.format_source(input).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        // closing brace should have less indent than body
        let closing_line = lines.last().unwrap();
        assert!(!closing_line.starts_with("        ")); // should dedent
    }

    #[test]
    fn test_custom_indent_size() {
        let config = FormatterConfig {
            indent_size: 2,
            ..Default::default()
        };
        let mut formatter = Formatter::with_config(config);
        let input = "fn test() {\nreturn 1;\n}";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("  return 1")); // 2 spaces
    }

    #[test]
    fn test_tabs_indent_when_configured() {
        let config = FormatterConfig {
            use_tabs: true,
            ..Default::default()
        };
        let mut formatter = Formatter::with_config(config);
        let input = "fn test() {\nreturn 1;\n}";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("\treturn 1")); // tab character
    }
}

#[cfg(test)]
mod blank_lines_tests {
    use super::*;

    #[test]
    fn test_collapse_multiple_blank_lines() {
        let mut formatter = Formatter::new();
        let input = "fn test() {\n\n\n\nreturn 1;\n}";
        let result = formatter.format_source(input).unwrap();
        let blank_count = result.matches("\n\n\n").count();
        assert_eq!(blank_count, 0); // should collapse to max 2
    }

    #[test]
    fn test_single_blank_line_preserved() {
        let mut formatter = Formatter::new();
        let input = "fn a() {}\n\nfn b() {}";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("\n\n")); // single blank line preserved
    }

    #[test]
    fn test_no_leading_blank_line() {
        let mut formatter = Formatter::new();
        let input = "\n\nfn test() {}";
        let result = formatter.format_source(input).unwrap();
        assert!(!result.starts_with("\n\n")); // no leading blank
    }
}

#[cfg(test)]
mod function_definition_tests {
    use super::*;

    #[test]
    fn test_function_params_spacing() {
        let mut formatter = Formatter::new();
        let input = "fn test(x:i32,y:i32)->i32{}";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("fn test(x: i32, y: i32) -> i32"));
    }

    #[test]
    fn test_blank_line_after_fn_definition() {
        let config = FormatterConfig {
            blank_line_after_fn: true,
            ..Default::default()
        };
        let mut formatter = Formatter::with_config(config);
        let input = "fn test() {}\nfn other() {}";
        let result = formatter.format_source(input).unwrap();
        println!("DEBUG blank_line: input = {:?}", input);
        println!("DEBUG blank_line: result = {:?}", result);
        // Should have blank line between functions
        assert!(result.contains("}\n\nfn other()"));
    }
}

#[cfg(test)]
mod is_formatted_tests {
    use super::*;

    #[test]
    fn test_is_formatted_returns_true_for_correct_code() {
        let mut formatter = Formatter::new();
        let formatted = "fn test(x: i32) -> i32 {\n    return x;\n}";
        let result = formatter.format_source(formatted).unwrap();
        println!("DEBUG: input = {:?}", formatted);
        println!("DEBUG: output = {:?}", result);
        println!("DEBUG: input.trim() = {:?}", formatted.trim());
        println!("DEBUG: output.trim() = {:?}", result.trim());
        println!("DEBUG: equal = {}", formatted.trim() == result.trim());
        assert!(formatter.is_formatted(formatted));
    }

    #[test]
    fn test_format_already_formatted_returns_already_formatted() {
        let mut formatter = Formatter::new();
        let formatted = "fn test() {\n    return 1;\n}";
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_formatter.bend");
        std::fs::write(&temp_file, formatted).unwrap();

        let result = formatter.format_file(&temp_file).unwrap();
        assert!(matches!(result, FormatResult::AlreadyFormatted));

        std::fs::remove_file(temp_file).ok();
    }

    #[test]
    fn test_is_formatted_is_idempotent() {
        let mut formatter = Formatter::new();
        let input = "    fn test() {\n\n        return 1;\n    }";
        let formatted1 = formatter.format_source(input).unwrap();
        let formatted2 = formatter.format_source(&formatted1).unwrap();
        assert_eq!(formatted1, formatted2);
    }
}

#[cfg(test)]
mod format_result_tests {
    use super::*;

    #[test]
    fn test_format_already_formatted_returns_already_formatted() {
        let mut formatter = Formatter::new();
        let formatted = "fn test() {\n    return 1;\n}";
        // Create a temp file to test format_file
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("test_formatter.bend");
        std::fs::write(&temp_file, formatted).unwrap();

        let result = formatter.format_file(&temp_file).unwrap();
        assert!(matches!(result, FormatResult::AlreadyFormatted));

        std::fs::remove_file(temp_file).ok();
    }
}

#[cfg(test)]
mod regression_tests {
    use super::*;

    #[test]
    fn test_issue_22_001_no_double_spaces_after_keywords() {
        let mut formatter = Formatter::new();
        let input = "if   (x > 0) {";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("if (x > 0) {"));
        assert!(!result.contains("if   "));
    }

    #[test]
    fn test_issue_22_002_arrow_spacing() {
        let mut formatter = Formatter::new();
        let input = "fn test()->i32{}";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("fn test() -> i32"));
    }

    #[test]
    fn test_issue_22_003_curly_brace_spacing() {
        let mut formatter = Formatter::new();
        let input = "if (true)  {";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("if (true) {"));
    }

    #[test]
    fn test_issue_22_004_comma_spacing() {
        let mut formatter = Formatter::new();
        let input = "fn test(a:i32,b:i32){}";
        let result = formatter.format_source(input).unwrap();
        assert!(result.contains("fn test(a: i32, b: i32)"));
    }

    #[test]
    fn test_issue_22_005_empty_lines_at_end() {
        let mut formatter = Formatter::new();
        let input = "fn test() {\n    return 1;\n}\n\n";
        let result = formatter.format_source(input).unwrap();
        assert!(!result.ends_with("\n\n\n")); // no extra blank lines at end
    }
}
