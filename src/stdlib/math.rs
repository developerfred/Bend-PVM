use crate::compiler::parser::ast::*;
use crate::runtime::env::Environment;
use crate::runtime::metering::MeteringError;
use std::f32::consts;

/// Math constants
pub struct MathConstants {
    /// Pi
    pub pi: f32,
    
    /// E (Euler's number)
    pub e: f32,
    
    /// Phi (Golden ratio)
    pub phi: f32,
    
    /// Sqrt(2)
    pub sqrt2: f32,
    
    /// Ln(2)
    pub ln2: f32,
    
    /// Ln(10)
    pub ln10: f32,
}

impl Default for MathConstants {
    fn default() -> Self {
        MathConstants {
            pi: consts::PI,
            e: consts::E,
            phi: 1.618033988749895,
            sqrt2: consts::SQRT_2,
            ln2: consts::LN_2,
            ln10: consts::LN_10,
        }
    }
}

/// Math functions implementation
pub struct MathFunctions {
    /// Constants
    pub constants: MathConstants,
}

impl MathFunctions {
    /// Create a new math functions instance
    pub fn new() -> Self {
        MathFunctions {
            constants: MathConstants::default(),
        }
    }
    
    /// Absolute value
    pub fn abs(&self, x: f32) -> f32 {
        x.abs()
    }
    
    /// Square root
    pub fn sqrt(&self, x: f32) -> f32 {
        x.sqrt()
    }
    
    /// Cube root
    pub fn cbrt(&self, x: f32) -> f32 {
        x.cbrt()
    }
    
    /// Power
    pub fn pow(&self, x: f32, y: f32) -> f32 {
        x.powf(y)
    }
    
    /// Exponential
    pub fn exp(&self, x: f32) -> f32 {
        x.exp()
    }
    
    /// Natural logarithm
    pub fn ln(&self, x: f32) -> f32 {
        x.ln()
    }
    
    /// Base 10 logarithm
    pub fn log10(&self, x: f32) -> f32 {
        x.log10()
    }
    
    /// Base 2 logarithm
    pub fn log2(&self, x: f32) -> f32 {
        x.log2()
    }
    
    /// Sine
    pub fn sin(&self, x: f32) -> f32 {
        x.sin()
    }
    
    /// Cosine
    pub fn cos(&self, x: f32) -> f32 {
        x.cos()
    }
    
    /// Tangent
    pub fn tan(&self, x: f32) -> f32 {
        x.tan()
    }
    
    /// Arcsine
    pub fn asin(&self, x: f32) -> f32 {
        x.asin()
    }
    
    /// Arccosine
    pub fn acos(&self, x: f32) -> f32 {
        x.acos()
    }
    
    /// Arctangent
    pub fn atan(&self, x: f32) -> f32 {
        x.atan()
    }
    
    /// Arctangent of y/x
    pub fn atan2(&self, y: f32, x: f32) -> f32 {
        y.atan2(x)
    }
    
    /// Hyperbolic sine
    pub fn sinh(&self, x: f32) -> f32 {
        x.sinh()
    }
    
    /// Hyperbolic cosine
    pub fn cosh(&self, x: f32) -> f32 {
        x.cosh()
    }
    
    /// Hyperbolic tangent
    pub fn tanh(&self, x: f32) -> f32 {
        x.tanh()
    }
    
    /// Floor
    pub fn floor(&self, x: f32) -> f32 {
        x.floor()
    }
    
    /// Ceiling
    pub fn ceil(&self, x: f32) -> f32 {
        x.ceil()
    }
    
    /// Round
    pub fn round(&self, x: f32) -> f32 {
        x.round()
    }
    
    /// Truncate
    pub fn trunc(&self, x: f32) -> f32 {
        x.trunc()
    }
    
    /// Fractional part
    pub fn fract(&self, x: f32) -> f32 {
        x.fract()
    }
    
    /// Minimum
    pub fn min(&self, x: f32, y: f32) -> f32 {
        x.min(y)
    }
    
    /// Maximum
    pub fn max(&self, x: f32, y: f32) -> f32 {
        x.max(y)
    }
    
    /// Clamp
    pub fn clamp(&self, x: f32, min: f32, max: f32) -> f32 {
        x.clamp(min, max)
    }
    
    /// Check if value is finite
    pub fn is_finite(&self, x: f32) -> bool {
        x.is_finite()
    }
    
    /// Check if value is infinite
    pub fn is_infinite(&self, x: f32) -> bool {
        x.is_infinite()
    }
    
    /// Check if value is NaN
    pub fn is_nan(&self, x: f32) -> bool {
        x.is_nan()
    }
    
    /// Copy sign
    pub fn copysign(&self, x: f32, y: f32) -> f32 {
        x.copysign(y)
    }
    
    /// Convert degrees to radians
    pub fn to_radians(&self, degrees: f32) -> f32 {
        degrees.to_radians()
    }
    
    /// Convert radians to degrees
    pub fn to_degrees(&self, radians: f32) -> f32 {
        radians.to_degrees()
    }
}

/// Register math functions in the runtime environment
pub fn register_math_functions(_env: &mut Environment) -> Result<(), MeteringError> {
    // In a real implementation, this would register the functions in the environment
    // For now, we just return Ok
    Ok(())
}

/// Generate AST for math library
pub fn generate_math_ast() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
    
    // Math constants
    definitions.push(Definition::FunctionDef {
        name: "Math/PI".to_string(),
        params: Vec::new(),
        return_type: Some(Type::F24 {
            location: dummy_loc.clone(),
        }),
        body: Block {
            statements: vec![
                Statement::Return {
                    value: Expr::Literal {
                        kind: LiteralKind::Float(std::f32::consts::PI),
                        location: dummy_loc.clone(),
                    },
                    location: dummy_loc.clone(),
                },
            ],
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });
    
    definitions.push(Definition::FunctionDef {
        name: "Math/E".to_string(),
        params: Vec::new(),
        return_type: Some(Type::F24 {
            location: dummy_loc.clone(),
        }),
        body: Block {
            statements: vec![
                Statement::Return {
                    value: Expr::Literal {
                        kind: LiteralKind::Float(std::f32::consts::E),
                        location: dummy_loc.clone(),
                    },
                    location: dummy_loc.clone(),
                },
            ],
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });
    
    // Math functions
    let math_funcs = vec![
        ("sin", 1), ("cos", 1), ("tan", 1),
        ("sqrt", 1), ("exp", 1), ("log", 1),
        ("floor", 1), ("ceil", 1), ("abs", 1),
        ("min", 2), ("max", 2), ("pow", 2)
    ];

    for (name, arity) in math_funcs {
let mut params = Vec::new();
        for i in 0..arity {
            params.push(Parameter {
                name: format!("x{}", i),
                ty: Type::F24 {
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            });
        }

        definitions.push(Definition::FunctionDef {
            name: format!("Math/{}", name),
            params,
            return_type: Some(Type::F24 {
                location: dummy_loc.clone(),
            }),
            body: Block {
                statements: Vec::new(), // Built-in
                location: dummy_loc.clone(),
            },
            checked: Some(true),
            location: dummy_loc.clone(),
        });
    }
    
    definitions
}
