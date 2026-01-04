//! Centralized error handling for Bend-PVM
//!
//! This module provides a comprehensive error handling system that consolidates
//! all error types used throughout the Bend-PVM compiler and runtime.

use std::fmt;
use thiserror::Error;

/// The main error type for Bend-PVM operations
#[derive(Error, Debug)]
pub enum BendError {
    /// Compilation errors
    #[error("Compilation failed: {0}")]
    Compilation(#[from] CompilationError),

    /// Runtime errors
    #[error("Runtime error: {0}")]
    Runtime(#[from] RuntimeError),

    /// IO errors
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// JSON errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Optimization errors
    #[error("Optimization error: {0}")]
    Optimization(#[from] crate::compiler::optimizer::passes::OptimizationError),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Generic errors
    #[error("{0}")]
    Generic(String),
}

/// Compilation phase errors
#[derive(Error, Debug)]
pub enum CompilationError {
    /// Lexical analysis errors
    #[error("Lexical error: {0}")]
    Lexical(String),

    /// Syntax parsing errors
    #[error("Parse error: {0}")]
    Parse(String),

    /// Semantic analysis errors
    #[error("Semantic error: {0}")]
    Semantic(String),

    /// Type checking errors
    #[error("Type error: {0}")]
    Type(String),

    /// Code generation errors
    #[error("Code generation error: {0}")]
    Codegen(String),

    /// Optimization errors
    #[error("Optimization error: {0}")]
    Optimization(String),

    /// Module system errors
    #[error("Module error: {0}")]
    Module(String),

    /// Security analysis errors
    #[error("Security error: {0}")]
    Security(String),
}

/// Runtime execution errors
#[derive(Error, Debug)]
pub enum RuntimeError {
    /// Gas exhaustion
    #[error("Gas limit exceeded: used {used}, limit {limit}")]
    GasExhausted { used: u64, limit: u64 },

    /// Memory access violations
    #[error("Memory access violation at address {address}")]
    MemoryViolation { address: u64 },

    /// Stack overflow
    #[error("Stack overflow")]
    StackOverflow,

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    /// Contract execution errors
    #[error("Contract execution failed: {0}")]
    ContractError(String),

    /// External call errors
    #[error("External call failed: {0}")]
    ExternalCallError(String),
}

/// Error severity levels for better error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Warning - non-fatal issue
    Warning,
    /// Error - compilation/runtime failure
    Error,
    /// Critical - system-level failure
    Critical,
}

impl fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorSeverity::Warning => write!(f, "warning"),
            ErrorSeverity::Error => write!(f, "error"),
            ErrorSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Enhanced error information with context
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Error severity
    pub severity: ErrorSeverity,

    /// Source file location
    pub file: Option<String>,

    /// Line number (1-indexed)
    pub line: Option<usize>,

    /// Column number (1-indexed)
    pub column: Option<usize>,

    /// Error code for categorization
    pub code: Option<String>,

    /// Additional context information
    pub context: Vec<String>,

    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(severity: ErrorSeverity) -> Self {
        ErrorContext {
            severity,
            file: None,
            line: None,
            column: None,
            code: None,
            context: Vec::new(),
            suggestion: None,
        }
    }

    /// Set the file location
    pub fn file(mut self, file: impl Into<String>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set the line and column
    pub fn position(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Set the error code
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add context information
    pub fn context(mut self, info: impl Into<String>) -> Self {
        self.context.push(info.into());
        self
    }

    /// Set a suggestion
    pub fn suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        ErrorContext::new(ErrorSeverity::Error)
    }
}

/// Enhanced error with context information
#[derive(Debug)]
pub struct ContextualError {
    /// The underlying error
    pub error: BendError,

    /// Additional context
    pub context: ErrorContext,
}

impl ContextualError {
    /// Create a new contextual error
    pub fn new(error: BendError, context: ErrorContext) -> Self {
        ContextualError { error, context }
    }

    /// Create an error with minimal context
    pub fn simple(error: BendError) -> Self {
        ContextualError {
            error,
            context: ErrorContext::default(),
        }
    }
}

impl fmt::Display for ContextualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error)?;

        if let Some(file) = &self.context.file {
            write!(f, " in {}", file)?;
        }

        if let (Some(line), Some(column)) = (self.context.line, self.context.column) {
            write!(f, " at line {}, column {}", line, column)?;
        }

        if let Some(code) = &self.context.code {
            write!(f, " [{}]", code)?;
        }

        Ok(())
    }
}

impl std::error::Error for ContextualError {}

/// Error collection and reporting
#[derive(Debug, Default)]
pub struct ErrorReporter {
    /// Collected errors
    errors: Vec<ContextualError>,

    /// Collected warnings
    warnings: Vec<ContextualError>,

    /// Maximum number of errors to collect before stopping
    max_errors: usize,
}

impl ErrorReporter {
    /// Create a new error reporter
    pub fn new() -> Self {
        ErrorReporter {
            errors: Vec::new(),
            warnings: Vec::new(),
            max_errors: 100, // Default limit
        }
    }

    /// Set the maximum number of errors to collect
    pub fn with_max_errors(mut self, max: usize) -> Self {
        self.max_errors = max;
        self
    }

    /// Report an error
    pub fn error(&mut self, error: BendError, context: ErrorContext) {
        if self.errors.len() < self.max_errors {
            self.errors.push(ContextualError::new(error, context));
        }
    }

    /// Report a warning
    pub fn warning(&mut self, error: BendError, context: ErrorContext) {
        self.warnings.push(ContextualError::new(error, context));
    }

    /// Report a simple error
    pub fn simple_error(&mut self, error: BendError) {
        self.error(error, ErrorContext::default());
    }

    /// Report a simple warning
    pub fn simple_warning(&mut self, error: BendError) {
        self.warning(error, ErrorContext::default());
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    /// Get the number of errors
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// Get the number of warnings
    pub fn warning_count(&self) -> usize {
        self.warnings.len()
    }

    /// Check if we've reached the error limit
    pub fn at_error_limit(&self) -> bool {
        self.errors.len() >= self.max_errors
    }

    /// Get all errors
    pub fn errors(&self) -> &[ContextualError] {
        &self.errors
    }

    /// Get all warnings
    pub fn warnings(&self) -> &[ContextualError] {
        &self.warnings
    }

    /// Clear all errors and warnings
    pub fn clear(&mut self) {
        self.errors.clear();
        self.warnings.clear();
    }

    /// Generate a summary report
    pub fn summary(&self) -> String {
        let mut summary = String::new();

        if self.has_errors() {
            summary.push_str(&format!("Found {} errors", self.errors.len()));
            if self.at_error_limit() {
                summary.push_str(" (error limit reached)");
            }
            summary.push('\n');
        }

        if self.has_warnings() {
            summary.push_str(&format!("Found {} warnings\n", self.warnings.len()));
        }

        if !self.has_errors() && !self.has_warnings() {
            summary.push_str("No errors or warnings found\n");
        }

        summary
    }

    /// Generate a detailed report
    pub fn detailed_report(&self) -> String {
        let mut report = self.summary();

        for error in &self.errors {
            report.push_str(&format!("Error: {}\n", error));
            if let Some(suggestion) = &error.context.suggestion {
                report.push_str(&format!("Suggestion: {}\n", suggestion));
            }
            report.push('\n');
        }

        for warning in &self.warnings {
            report.push_str(&format!("Warning: {}\n", warning));
            if let Some(suggestion) = &warning.context.suggestion {
                report.push_str(&format!("Suggestion: {}\n", suggestion));
            }
            report.push('\n');
        }

        report
    }
}

/// Result type alias for Bend-PVM operations
pub type BendResult<T> = Result<T, BendError>;

/// Result type alias for operations that can provide context
pub type ContextualResult<T> = Result<T, ContextualError>;

/// Convenience functions for creating common errors
pub mod errors {
    use super::*;

    /// Create a lexical error
    pub fn lexical(message: impl Into<String>) -> BendError {
        BendError::Compilation(CompilationError::Lexical(message.into()))
    }

    /// Create a parse error
    pub fn parse(message: impl Into<String>) -> BendError {
        BendError::Compilation(CompilationError::Parse(message.into()))
    }

    /// Create a type error
    pub fn type_error(message: impl Into<String>) -> BendError {
        BendError::Compilation(CompilationError::Type(message.into()))
    }

    /// Create a codegen error
    pub fn codegen(message: impl Into<String>) -> BendError {
        BendError::Compilation(CompilationError::Codegen(message.into()))
    }

    /// Create a runtime gas error
    pub fn gas_exhausted(used: u64, limit: u64) -> BendError {
        BendError::Runtime(RuntimeError::GasExhausted { used, limit })
    }

    /// Create a generic error
    pub fn generic(message: impl Into<String>) -> BendError {
        BendError::Generic(message.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = errors::lexical("Invalid token");
        assert!(matches!(
            error,
            BendError::Compilation(CompilationError::Lexical(_))
        ));

        let error = errors::gas_exhausted(1000, 500);
        assert!(matches!(
            error,
            BendError::Runtime(RuntimeError::GasExhausted { .. })
        ));
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new(ErrorSeverity::Error)
            .file("test.bend")
            .position(10, 5)
            .code("E001")
            .context("Additional context")
            .suggestion("Try using a different syntax");

        assert_eq!(context.file, Some("test.bend".to_string()));
        assert_eq!(context.line, Some(10));
        assert_eq!(context.column, Some(5));
        assert_eq!(context.code, Some("E001".to_string()));
        assert_eq!(context.context.len(), 1);
        assert_eq!(
            context.suggestion,
            Some("Try using a different syntax".to_string())
        );
    }

    #[test]
    fn test_error_reporter() {
        let mut reporter = ErrorReporter::new();

        // Add some errors
        reporter.simple_error(errors::lexical("Error 1"));
        reporter.simple_error(errors::parse("Error 2"));
        reporter.simple_warning(errors::generic("Warning 1"));

        assert!(reporter.has_errors());
        assert!(reporter.has_warnings());
        assert_eq!(reporter.error_count(), 2);
        assert_eq!(reporter.warning_count(), 1);

        let summary = reporter.summary();
        assert!(summary.contains("2 errors"));
        assert!(summary.contains("1 warnings"));
    }

    #[test]
    fn test_error_display() {
        let error = ContextualError::new(
            errors::type_error("Type mismatch"),
            ErrorContext::new(ErrorSeverity::Error)
                .file("test.bend")
                .position(5, 10)
                .code("T001"),
        );

        let display = format!("{}", error);
        assert!(display.contains("Type mismatch"));
        assert!(display.contains("test.bend"));
        assert!(display.contains("line 5"));
        assert!(display.contains("column 10"));
        assert!(display.contains("[T001]"));
    }

    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
    }

    #[test]
    fn test_error_limit() {
        let mut reporter = ErrorReporter::new().with_max_errors(2);

        reporter.simple_error(errors::lexical("Error 1"));
        reporter.simple_error(errors::lexical("Error 2"));
        reporter.simple_error(errors::lexical("Error 3")); // Should be ignored

        assert_eq!(reporter.error_count(), 2);
        assert!(reporter.at_error_limit());
    }
}
