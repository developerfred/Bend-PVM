use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod documentation_completeness_tests {
    use super::*;

    const REQUIRED_SECTIONS: &[&str] = &[
        "# Bend-PVM Language Reference",
        "## Introduction",
        "## Syntax Basics",
        "### Comments",
        "### Data Types",
        "### Variables and Assignments",
        "### Functions",
        "### Control Flow",
        "## Resource Model",
        "## Monadic Composition",
        "## Pattern Matching",
        "## Libraries and Imports",
        "## Standard Library",
        "## Contract Structure",
        "## Best Practices",
        "## Advanced Features",
        "## Conclusion",
    ];

    const REQUIRED_SUBSECTIONS: &[&str] = &[
        "### Import Statements",
        "#### If Statements",
        "#### Match Statements",
        "#### Bend Statements",
        "### Recursive Types",
        "### Higher-Order Functions",
        "### Lambda Expressions",
        "### Object Definitions",
        "### Type Definitions",
    ];

    fn get_doc_path() -> PathBuf {
        PathBuf::from("docs/language_reference.md")
    }

    #[test]
    fn test_doc_file_exists() {
        let path = get_doc_path();
        assert!(path.exists(), "docs/language_reference.md must exist");
    }

    #[test]
    fn test_doc_has_title() {
        let content = fs::read_to_string(get_doc_path()).unwrap();
        assert!(
            content.contains("# Bend-PVM Language Reference"),
            "Document must have title"
        );
    }

    #[test]
    fn test_doc_has_all_required_sections() {
        let content = fs::read_to_string(get_doc_path()).unwrap();
        let mut missing = Vec::new();

        for section in REQUIRED_SECTIONS {
            if !content.contains(section) {
                missing.push(*section);
            }
        }

        assert!(missing.is_empty(), "Missing sections: {:?}", missing);
    }

    #[test]
    fn test_doc_has_all_required_subsections() {
        let content = fs::read_to_string(get_doc_path()).unwrap();
        let mut missing = Vec::new();

        for section in REQUIRED_SUBSECTIONS {
            if !content.contains(section) {
                missing.push(*section);
            }
        }

        assert!(missing.is_empty(), "Missing subsections: {:?}", missing);
    }

    #[test]
    fn test_doc_has_table_of_contents() {
        let content = fs::read_to_string(get_doc_path()).unwrap();
        assert!(
            content.contains("## Table of Contents") || content.contains("## Contents"),
            "Document must have a table of contents"
        );
    }

    #[test]
    fn test_doc_has_error_handling_section() {
        let content = fs::read_to_string(get_doc_path()).unwrap();
        assert!(
            content.to_lowercase().contains("error") && content.contains("Result"),
            "Document must cover error handling with Result type"
        );
    }

    #[test]
    fn test_doc_has_performance_tips() {
        let content = fs::read_to_string(get_doc_path()).unwrap();
        assert!(
            content.to_lowercase().contains("performance")
                || content.to_lowercase().contains("optimization")
                || content.to_lowercase().contains("gas"),
            "Document must cover performance/optimization"
        );
    }
}

#[cfg(test)]
mod code_example_tests {
    use super::*;

    #[test]
    fn test_comment_example_exists() {
        let content = fs::read_to_string("docs/language_reference.md").unwrap();
        assert!(
            content.contains("```bend") && content.contains("# ") && content.contains("#{"),
            "Must have code examples with comments"
        );
    }

    #[test]
    fn test_function_example_has_return_keyword() {
        let content = fs::read_to_string("docs/language_reference.md").unwrap();
        assert!(
            content.contains("def ") && content.contains("return"),
            "Function examples must show return keyword"
        );
    }

    #[test]
    fn test_match_statement_example_exists() {
        let content = fs::read_to_string("docs/language_reference.md").unwrap();
        assert!(
            content.contains("match ") && content.contains("case "),
            "Must have match statement examples"
        );
    }

    #[test]
    fn test_type_definition_example_exists() {
        let content = fs::read_to_string("docs/language_reference.md").unwrap();
        assert!(
            content.contains("type ") && content.contains(":"),
            "Must have type definition examples"
        );
    }

    #[test]
    fn test_contract_example_exists() {
        let content = fs::read_to_string("docs/language_reference.md").unwrap();
        assert!(
            content.contains("def main()") || content.contains("contract"),
            "Must have contract structure example"
        );
    }

    #[test]
    fn test_all_code_blocks_have_language_tag() {
        let content = fs::read_to_string("docs/language_reference.md").unwrap();
        let mut issues = Vec::new();

        let mut in_block = false;
        for line in content.lines() {
            if line.starts_with("```") {
                if in_block {
                    in_block = false;
                } else {
                    in_block = true;
                    if !line.contains("```bend")
                        && !line.contains("```rust")
                        && !line.contains("```")
                    {
                        issues.push(line);
                    }
                }
            }
        }

        assert!(
            issues.is_empty(),
            "All code blocks should be tagged with language: {:?}",
            issues
        );
    }
}

#[cfg(test)]
mod migration_guide_tests {
    use super::*;

    #[test]
    fn test_solidity_migration_guide_exists() {
        let path = Path::new("docs/solidity_migration_guide.md");
        assert!(path.exists(), "docs/solidity_migration_guide.md must exist");
    }

    #[test]
    fn test_migration_guide_has_sections() {
        let content = fs::read_to_string("docs/solidity_migration_guide.md").unwrap();
        assert!(
            content.contains("##") && content.contains("Solidity"),
            "Migration guide must have sections"
        );
    }
}
