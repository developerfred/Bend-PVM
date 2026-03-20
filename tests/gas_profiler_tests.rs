use bend_pvm::analyzer::gas_profiler::GasProfiler;

fn create_test_profiler() -> GasProfiler {
    GasProfiler::new()
}

fn create_simple_contract() -> &'static str {
    r#"
fn add(x: u24, y: u24) -> u24 {
    x + y
}

fn multiply(a: u24, b: u24) -> u24 {
    a * b
}
"#
}

fn create_recursive_contract() -> &'static str {
    r#"
fn factorial(n: u24) -> u24 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn sum_upto(n: u24) -> u24 {
    if n == 0 {
        0
    } else {
        n + sum_upto(n - 1)
    }
}
"#
}

mod gas_profiler_tests {
    use super::*;

    #[test]
    fn test_gas_profiler_creation() {
        let profiler = create_test_profiler();
        assert!(profiler.get_cost("storage_read") > 0);
    }

    #[test]
    fn test_gas_profiler_has_all_costs() {
        let profiler = create_test_profiler();
        assert_eq!(profiler.get_cost("storage_read"), 200);
        assert_eq!(profiler.get_cost("storage_write"), 5000);
        assert_eq!(profiler.get_cost("storage_delete"), 500);
        assert_eq!(profiler.get_cost("external_call"), 2500);
        assert_eq!(profiler.get_cost("event_emit"), 375);
        assert_eq!(profiler.get_cost("if_branch"), 10);
        assert_eq!(profiler.get_cost("bend_iteration"), 25);
        assert_eq!(profiler.get_cost("function_call"), 40);
        assert_eq!(profiler.get_cost("binary_op"), 5);
        assert_eq!(profiler.get_cost("variable_access"), 3);
        assert_eq!(profiler.get_cost("literal"), 3);
    }

    #[test]
    fn test_profile_simple_contract() {
        let profiler = create_test_profiler();
        let source = create_simple_contract();

        let result = profiler.profile_source(source, "test.bend");
        assert!(result.is_ok());

        let profile = result.unwrap();
        assert_eq!(profile.file_path, "test.bend");
        assert!(profile.total_gas > 0);
    }

    #[test]
    fn test_profile_finds_functions() {
        let profiler = create_test_profiler();
        let source = create_simple_contract();

        let result = profiler.profile_source(source, "test.bend").unwrap();

        let function_names: Vec<&str> = result.estimates.iter().map(|e| e.name.as_str()).collect();

        assert!(function_names.contains(&"add"), "Should find add function");
        assert!(
            function_names.contains(&"multiply"),
            "Should find multiply function"
        );
    }

    #[test]
    fn test_simple_function_has_base_cost() {
        let profiler = create_test_profiler();
        let source = r#"
fn empty() -> u24 {
    0
}
"#;

        let result = profiler.profile_source(source, "test.bend").unwrap();
        assert_eq!(result.estimates.len(), 1);

        let estimate = &result.estimates[0];
        assert!(
            estimate.base_cost >= 20,
            "Function overhead should be at least 20"
        );
        assert!(estimate.avg_cost >= estimate.base_cost);
    }

    #[test]
    fn test_recursive_function_detection() {
        let profiler = create_test_profiler();
        let source = create_recursive_contract();

        let result = profiler.profile_source(source, "test.bend").unwrap();

        let factorial = result
            .estimates
            .iter()
            .find(|e| e.name == "factorial")
            .expect("Should find factorial function");

        assert!(
            factorial.is_recursive,
            "Factorial should be marked as recursive"
        );
    }

    #[test]
    fn test_sum_upto_recursive_detection() {
        let profiler = create_test_profiler();
        let source = create_recursive_contract();

        let result = profiler.profile_source(source, "test.bend").unwrap();

        let sum_upto = result
            .estimates
            .iter()
            .find(|e| e.name == "sum_upto")
            .expect("Should find sum_upto function");

        assert!(
            sum_upto.is_recursive,
            "sum_upto should be marked as recursive"
        );
    }

    #[test]
    fn test_non_recursive_function_not_marked() {
        let profiler = create_test_profiler();
        let source = r#"
fn helper(n: u24) -> u24 {
    n + 1
}

fn main(x: u24) -> u24 {
    helper(x)
}
"#;

        let result = profiler.profile_source(source, "test.bend").unwrap();

        let main_fn = result
            .estimates
            .iter()
            .find(|e| e.name == "main")
            .expect("Should find main function");

        assert!(!main_fn.is_recursive, "Main should not be recursive");
    }

    #[test]
    fn test_cost_breakdown_exists() {
        let profiler = create_test_profiler();
        let source = r#"
fn test() -> u24 {
    1 + 2
}
"#;

        let result = profiler.profile_source(source, "test.bend").unwrap();

        let estimate = &result.estimates[0];
        assert!(
            !estimate.cost_breakdown.is_empty(),
            "Cost breakdown should not be empty"
        );
    }

    #[test]
    fn test_line_range_is_set() {
        let profiler = create_test_profiler();
        let source = r#"
fn test() -> u24 {
    42
}
"#;

        let result = profiler.profile_source(source, "test.bend").unwrap();

        let estimate = &result.estimates[0];
        assert!(
            estimate.line_range.1 > estimate.line_range.0,
            "Line range should be valid"
        );
    }

    #[test]
    fn test_most_expensive_function_identified() {
        let profiler = create_test_profiler();
        let source = create_simple_contract();

        let result = profiler.profile_source(source, "test.bend").unwrap();

        if let Some(expensive) = &result.most_expensive_function {
            let expensive_fn = result
                .estimates
                .iter()
                .find(|e| &e.name == expensive)
                .expect("Most expensive should exist");

            assert_eq!(
                expensive_fn.avg_cost,
                result.estimates.iter().map(|e| e.avg_cost).max().unwrap()
            );
        }
    }

    #[test]
    fn test_total_gas_sum_of_functions() {
        let profiler = create_test_profiler();
        let source = create_simple_contract();

        let result = profiler.profile_source(source, "test.bend").unwrap();

        let sum: u64 = result.estimates.iter().map(|e| e.avg_cost).sum();

        assert_eq!(result.total_gas, sum);
    }

    #[test]
    fn test_profiler_get_cost_returns_default_for_unknown() {
        let profiler = create_test_profiler();
        assert_eq!(profiler.get_cost("unknown_operation"), 5);
    }

    #[test]
    fn test_binary_operations_contribute_to_cost() {
        let profiler = create_test_profiler();
        let source = r#"
fn add(a: u24, b: u24) -> u24 {
    a + b
}
"#;

        let result = profiler.profile_source(source, "test.bend").unwrap();
        let estimate = &result.estimates[0];

        assert!(estimate.base_cost > 0);
    }

    #[test]
    fn test_max_cost_higher_for_recursive_functions() {
        let profiler = create_test_profiler();
        let source = create_recursive_contract();

        let result = profiler.profile_source(source, "test.bend").unwrap();

        let recursive_fn = result
            .estimates
            .iter()
            .find(|e| e.is_recursive)
            .expect("Should find recursive function");

        assert!(recursive_fn.max_cost >= recursive_fn.base_cost);
    }

    #[test]
    fn test_multiple_functions_profiled() {
        let profiler = create_test_profiler();
        let source = r#"
fn one() -> u24 { 1 }
fn two() -> u24 { 2 }
fn three() -> u24 { 3 }
"#;

        let result = profiler.profile_source(source, "test.bend").unwrap();
        assert_eq!(result.estimates.len(), 3);
    }

    #[test]
    fn test_function_with_if_branch() {
        let profiler = create_test_profiler();
        let source = r#"
fn max(a: u24, b: u24) -> u24 {
    if a > b {
        a
    } else {
        b
    }
}
"#;

        let result = profiler.profile_source(source, "test.bend").unwrap();
        assert_eq!(result.estimates.len(), 1);

        let estimate = &result.estimates[0];
        assert!(estimate.base_cost > 0);
    }
}
