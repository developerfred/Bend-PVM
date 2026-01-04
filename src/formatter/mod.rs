//! Formatter module for Bend-PVM source code
//!
//! Provides code formatting capabilities including:
//! - Indentation management
//! - Whitespace normalization  
//! - Statement spacing
//! - Block formatting

use std::fs;
use std::io::Write;
use std::path::Path;

/// Formatter configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Number of spaces per indentation level
    pub indent_size: usize,

    /// Maximum line length
    pub max_line_length: usize,

    /// Whether to use tabs instead of spaces
    pub use_tabs: bool,

    /// Insert blank line after function definitions
    pub blank_line_after_fn: bool,

    /// Insert space around operators
    pub space_around_operators: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_line_length: 120,
            use_tabs: false,
            blank_line_after_fn: true,
            space_around_operators: true,
        }
    }
}

/// Formatter result
#[derive(Debug, Clone)]
pub enum FormatResult {
    /// File was formatted successfully
    Formatted(String),
    /// File was already formatted correctly
    AlreadyFormatted,
    /// Format check failed - file needs formatting
    NeedsFormatting,
}

/// Main formatter struct
pub struct Formatter {
    config: FormatterConfig,
}

impl Formatter {
    /// Create a new formatter with default configuration
    pub fn new() -> Self {
        Self {
            config: FormatterConfig::default(),
        }
    }

    /// Create a formatter with custom configuration
    pub fn with_config(config: FormatterConfig) -> Self {
        Self { config }
    }

    /// Format Bend-PVM source code
    pub fn format_source(&self, source: &str) -> String {
        let mut formatted = String::new();
        let lines = source.lines().collect::<Vec<_>>();

        // Basic formatting pass
        for (i, line) in lines.iter().enumerate() {
            let formatted_line = self.format_line(line);
            formatted.push_str(&formatted_line);

            // Add newline except for the last line
            if i < lines.len() - 1 {
                formatted.push('\n');
            }
        }

        // Normalize trailing whitespace
        self.normalize_whitespace(&formatted)
    }

    /// Format a single line
    fn format_line(&self, line: &str) -> String {
        let trimmed = line.trim_end();

        // Handle indentation based on basic heuristics
        if trimmed.ends_with('{') {
            // Increase indent for opening braces
            self.indent_line(trimmed, 1)
        } else if trimmed.starts_with('}') {
            // Decrease indent for closing braces
            self.indent_line(trimmed, -1)
        } else {
            trimmed.to_string()
        }
    }

    /// Add indentation to a line
    fn indent_line(&self, line: &str, delta: i32) -> String {
        if delta > 0 {
            // Add indentation
            let indent = self.get_indent();
            format!("{}{}", indent, line)
        } else if delta < 0 && !line.is_empty() {
            // Remove one level of indentation
            let indent_len = if self.config.use_tabs {
                1
            } else {
                self.config.indent_size
            };
            if line.len() >= indent_len {
                (&line[indent_len..]).to_string()
            } else {
                line.to_string()
            }
        } else {
            line.to_string()
        }
    }

    /// Get indentation string
    fn get_indent(&self) -> &'static str {
        if self.config.use_tabs {
            "\t"
        } else {
            "    " // 4 spaces
        }
    }

    /// Normalize whitespace in the formatted code
    fn normalize_whitespace(&self, code: &str) -> String {
        let mut result = String::new();
        let mut previous_was_blank = false;

        for line in code.lines() {
            // Remove trailing whitespace
            let trimmed = line.trim_end();

            // Skip multiple consecutive blank lines
            if trimmed.is_empty() {
                if !previous_was_blank {
                    result.push('\n');
                    previous_was_blank = true;
                }
            } else {
                result.push_str(trimmed);
                result.push('\n');
                previous_was_blank = false;
            }
        }

        result.trim_end().to_string()
    }

    /// Check if code is already properly formatted
    pub fn is_formatted(&self, source: &str) -> bool {
        let formatted = self.format_source(source);
        source == formatted
    }

    /// Format a file and return the result
    pub fn format_file(
        &self,
        file_path: &Path,
    ) -> Result<FormatResult, Box<dyn std::error::Error>> {
        // Read the source file
        let source = fs::read_to_string(file_path)?;

        // Check if already formatted
        if self.is_formatted(&source) {
            return Ok(FormatResult::AlreadyFormatted);
        }

        // Format the source
        let formatted = self.format_source(&source);

        Ok(FormatResult::Formatted(formatted))
    }

    /// Format a file and write the result
    pub fn format_file_in_place(
        &self,
        file_path: &Path,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match self.format_file(file_path)? {
            FormatResult::Formatted(formatted) => {
                // Write formatted code back to file
                let mut file = fs::File::create(file_path)?;
                file.write_all(formatted.as_bytes())?;
                Ok(true) // File was modified
            }
            FormatResult::AlreadyFormatted => Ok(false), // File was not modified
            FormatResult::NeedsFormatting => Ok(true),   // Should not happen in this implementation
        }
    }
}

impl Default for Formatter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let formatter = Formatter::new();

        let unformatted = r#"fn test(  x:i32,  y:i32  ) -> i32 {
return   x   +   y;
}"#;

        let formatted = formatter.format_source(unformatted);

        // Should normalize spaces and add proper indentation
        assert!(formatted.contains("fn test("));
        assert!(formatted.contains("return x + y;"));
    }

    #[test]
    fn test_already_formatted() {
        let formatter = Formatter::new();

        let formatted = r#"fn test(x: i32) -> i32 {
    return x;
}"#;

        assert!(formatter.is_formatted(formatted));
    }
}
