/// Breakpoint types for the debugger
#[derive(Debug, Clone, PartialEq)]
pub enum Breakpoint {
    /// Line breakpoint
    Line(usize),

    /// Instruction breakpoint
    Instruction(usize),

    /// Function breakpoint
    Function(String),
}

impl Breakpoint {
    /// Create a line breakpoint
    pub fn line(line: usize) -> Self {
        Breakpoint::Line(line)
    }

    /// Create an instruction breakpoint
    pub fn instruction(instruction: usize) -> Self {
        Breakpoint::Instruction(instruction)
    }

    /// Create a function breakpoint
    pub fn function(function: &str) -> Self {
        Breakpoint::Function(function.to_string())
    }

    /// Get the breakpoint description
    pub fn description(&self) -> String {
        match self {
            Breakpoint::Line(line) => format!("Line {}", line),
            Breakpoint::Instruction(instruction) => format!("Instruction {}", instruction),
            Breakpoint::Function(function) => format!("Function {}", function),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_breakpoint() {
        let bp = Breakpoint::line(42);
        assert!(matches!(bp, Breakpoint::Line(n) if n == 42));
    }

    #[test]
    fn test_instruction_breakpoint() {
        let bp = Breakpoint::instruction(100);
        assert!(matches!(bp, Breakpoint::Instruction(n) if n == 100));
    }

    #[test]
    fn test_function_breakpoint() {
        let bp = Breakpoint::function("main");
        assert!(matches!(bp, Breakpoint::Function(ref s) if s == "main"));
    }

    #[test]
    fn test_breakpoint_description() {
        let line_bp = Breakpoint::line(10);
        assert_eq!(line_bp.description(), "Line 10");

        let instr_bp = Breakpoint::instruction(50);
        assert_eq!(instr_bp.description(), "Instruction 50");

        let func_bp = Breakpoint::function("foo");
        assert_eq!(func_bp.description(), "Function foo");
    }

    #[test]
    fn test_breakpoint_clone() {
        let bp = Breakpoint::line(42);
        let cloned = bp.clone();
        assert_eq!(bp, cloned);
    }

    #[test]
    fn test_breakpoint_partial_eq() {
        let bp1 = Breakpoint::line(42);
        let bp2 = Breakpoint::line(42);
        let bp3 = Breakpoint::line(43);

        assert_eq!(bp1, bp2);
        assert_ne!(bp1, bp3);
    }
}
