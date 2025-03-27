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