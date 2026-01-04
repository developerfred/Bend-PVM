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

/// Safe math operations for contract development
pub struct SafeMath;

impl SafeMath {
    /// Safe addition with overflow check
    pub fn add(a: u128, b: u128) -> Result<u128, String> {
        a.checked_add(b).ok_or("Addition overflow".to_string())
    }

    /// Safe subtraction with underflow check
    pub fn sub(a: u128, b: u128) -> Result<u128, String> {
        a.checked_sub(b).ok_or("Subtraction underflow".to_string())
    }

    /// Safe multiplication with overflow check
    pub fn mul(a: u128, b: u128) -> Result<u128, String> {
        a.checked_mul(b)
            .ok_or("Multiplication overflow".to_string())
    }

    /// Safe division
    pub fn div(a: u128, b: u128) -> Result<u128, String> {
        if b == 0 {
            Err("Division by zero".to_string())
        } else {
            Ok(a / b)
        }
    }

    /// Safe modulus
    pub fn mod_(a: u128, b: u128) -> Result<u128, String> {
        if b == 0 {
            Err("Modulus by zero".to_string())
        } else {
            Ok(a % b)
        }
    }

    /// Safe power with overflow check
    pub fn pow(base: u128, exp: u32) -> Result<u128, String> {
        base.checked_pow(exp).ok_or("Power overflow".to_string())
    }

    /// Safe addition with wraparound
    pub fn add_wrapped(a: u128, b: u128) -> u128 {
        a.wrapping_add(b)
    }

    /// Safe subtraction with wraparound
    pub fn sub_wrapped(a: u128, b: u128) -> u128 {
        a.wrapping_sub(b)
    }

    /// Safe multiplication with wraparound
    pub fn mul_wrapped(a: u128, b: u128) -> u128 {
        a.wrapping_mul(b)
    }
}

/// Big integer operations
pub struct BigIntMath;

impl BigIntMath {
    /// Create big int from u128
    pub fn from_u128(value: u128) -> Vec<u8> {
        value.to_le_bytes().to_vec()
    }

    /// Create big int from i128
    pub fn from_i128(value: i128) -> Vec<u8> {
        let unsigned = if value < 0 {
            ((value as i128) * -1) as u128
        } else {
            value as u128
        };
        let mut bytes = unsigned.to_le_bytes().to_vec();
        if value < 0 {
            bytes.push(0x80); // Sign bit
        }
        bytes
    }

    /// Convert big int to u128 with overflow check
    pub fn to_u128(bytes: &[u8]) -> Result<u128, String> {
        if bytes.len() > 16 {
            return Err("BigInt too large for u128".to_string());
        }
        let mut padded = [0u8; 16];
        padded[..bytes.len()].copy_from_slice(bytes);
        Ok(u128::from_le_bytes(padded))
    }

    /// Big int addition
    pub fn add(a: &[u8], b: &[u8]) -> Vec<u8> {
        let a_int = Self::to_u128(a).unwrap_or(0);
        let b_int = Self::to_u128(b).unwrap_or(0);
        Self::from_u128(a_int.wrapping_add(b_int))
    }

    /// Big int subtraction
    pub fn sub(a: &[u8], b: &[u8]) -> Vec<u8> {
        let a_int = Self::to_u128(a).unwrap_or(0);
        let b_int = Self::to_u128(b).unwrap_or(0);
        Self::from_u128(a_int.wrapping_sub(b_int))
    }

    /// Big int multiplication
    pub fn mul(a: &[u8], b: &[u8]) -> Vec<u8> {
        let a_int = Self::to_u128(a).unwrap_or(0);
        let b_int = Self::to_u128(b).unwrap_or(0);
        Self::from_u128(a_int.wrapping_mul(b_int))
    }

    /// Big int comparison: returns -1 if a < b, 0 if a == b, 1 if a > b
    pub fn cmp(a: &[u8], b: &[u8]) -> i8 {
        let a_int = Self::to_u128(a).unwrap_or(0);
        let b_int = Self::to_u128(b).unwrap_or(0);
        if a_int < b_int {
            -1
        } else if a_int > b_int {
            1
        } else {
            0
        }
    }
}

/// Bitwise operations for contracts
pub struct BitwiseMath;

impl BitwiseMath {
    /// Bitwise AND
    pub fn and(a: u128, b: u128) -> u128 {
        a & b
    }

    /// Bitwise OR
    pub fn or(a: u128, b: u128) -> u128 {
        a | b
    }

    /// Bitwise XOR
    pub fn xor(a: u128, b: u128) -> u128 {
        a ^ b
    }

    /// Bitwise NOT
    pub fn not(a: u128) -> u128 {
        !a
    }

    /// Left shift
    pub fn shl(a: u128, shift: u32) -> u128 {
        a << shift
    }

    /// Right shift
    pub fn shr(a: u128, shift: u32) -> u128 {
        a >> shift
    }

    /// Rotate left
    pub fn rotl(a: u128, shift: u32, bits: u32) -> u128 {
        let shift = shift % bits;
        (a << shift) | (a >> (bits - shift))
    }

    /// Rotate right
    pub fn rotr(a: u128, shift: u32, bits: u32) -> u128 {
        let shift = shift % bits;
        (a >> shift) | (a << (bits - shift))
    }

    /// Count trailing zeros
    pub fn ctz(a: u128) -> u32 {
        a.trailing_zeros()
    }

    /// Count leading zeros
    pub fn clz(a: u128) -> u32 {
        a.leading_zeros()
    }

    /// Population count (Hamming weight)
    pub fn popcount(a: u128) -> u32 {
        a.count_ones()
    }
}

/// Random number generation for contracts
pub struct Random;

impl Random {
    /// Generate random u128 using environmental entropy
    pub fn rand() -> u128 {
        let mut buf = [0u8; 16];
        getrandom::getrandom(&mut buf).unwrap_or_default();
        u128::from_le_bytes(buf)
    }

    /// Generate random u128 in range [min, max]
    pub fn range(min: u128, max: u128) -> u128 {
        if min >= max {
            return min;
        }
        let range = max - min + 1;
        let rand = Self::rand();
        min + (rand % range)
    }

    /// Generate random boolean
    pub fn bool() -> bool {
        Self::rand() % 2 == 1
    }

    /// Generate random bytes
    pub fn bytes(len: usize) -> Vec<u8> {
        let mut buf = vec![0u8; len];
        getrandom::getrandom(&mut buf).unwrap_or_default();
        buf
    }

    /// Shuffle a vector randomly
    pub fn shuffle<T>(vec: &mut Vec<T>) {
        let _rng = Self::rand();
        for i in (1..vec.len()).rev() {
            let j = Self::range(0, i as u128) as usize;
            vec.swap(i, j);
        }
    }
}

/// Percentage and ratio calculations
pub struct Percentage;

impl Percentage {
    /// Calculate percentage: (value * 100) / total
    pub fn of(value: u128, total: u128) -> Result<u128, String> {
        if total == 0 {
            Err("Total cannot be zero".to_string())
        } else {
            Ok((value * 100) / total)
        }
    }

    /// Calculate basis points (1/100 of 1%)
    pub fn bps(value: u128, total: u128) -> Result<u128, String> {
        if total == 0 {
            Err("Total cannot be zero".to_string())
        } else {
            Ok((value * 10000) / total)
        }
    }

    /// Apply percentage: value * percent / 100
    pub fn apply(value: u128, percent: u128) -> u128 {
        (value * percent) / 100
    }

    /// Apply basis points: value * bps / 10000
    pub fn apply_bps(value: u128, bps: u128) -> u128 {
        (value * bps) / 10000
    }

    /// Calculate growth rate: (new - old) * 100 / old
    pub fn growth_rate(old: u128, new: u128) -> Result<i128, String> {
        if old == 0 {
            Err("Old value cannot be zero".to_string())
        } else {
            let diff = new as i128 - old as i128;
            Ok((diff * 100) / old as i128)
        }
    }

    /// Linear interpolation: start + (end - start) * percent / 100
    pub fn lerp(start: u128, end: u128, percent: u128) -> u128 {
        start + ((end - start) * percent / 100)
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
    let dummy_loc = Location {
        line: 0,
        column: 0,
        start: 0,
        end: 0,
    };

    // Math constants
    definitions.push(Definition::FunctionDef {
        name: "Math/PI".to_string(),
        params: Vec::new(),
        return_type: Some(Type::F24 {
            location: dummy_loc.clone(),
        }),
        body: Block {
            statements: vec![Statement::Return {
                value: Expr::Literal {
                    kind: LiteralKind::Float(std::f32::consts::PI),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            }],
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
            statements: vec![Statement::Return {
                value: Expr::Literal {
                    kind: LiteralKind::Float(std::f32::consts::E),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            }],
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Math functions
    let math_funcs = vec![
        ("sin", 1),
        ("cos", 1),
        ("tan", 1),
        ("sqrt", 1),
        ("exp", 1),
        ("log", 1),
        ("floor", 1),
        ("ceil", 1),
        ("abs", 1),
        ("min", 2),
        ("max", 2),
        ("pow", 2),
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
