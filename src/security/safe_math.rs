/// SafeMath module - Overflow/Underflow Protection
/// 
/// Provides safe arithmetic operations that prevent integer overflow and underflow
/// which are common sources of security vulnerabilities in smart contracts.

use crate::compiler::parser::ast::*;
use thiserror::Error;

/// SafeMath errors
#[derive(Error, Debug)]
pub enum SafeMathError {
    #[error("Integer overflow: {operation} {left} {right}")]
    Overflow { operation: String, left: String, right: String },
    
    #[error("Integer underflow: {operation} {left} {right}")]
    Underflow { operation: String, left: String, right: String },
    
    #[error("Division by zero")]
    DivisionByZero,
}

/// Safe arithmetic trait
pub trait SafeArithmetic {
    type Output;
    
    fn safe_add(self, other: Self) -> Result<Self::Output, SafeMathError>;
    fn safe_sub(self, other: Self) -> Result<Self::Output, SafeMathError>;
    fn safe_mul(self, other: Self) -> Result<Self::Output, SafeMathError>;
    fn safe_div(self, other: Self) -> Result<Self::Output, SafeMathError>;
    fn safe_mod(self, other: Self) -> Result<Self::Output, SafeMathError>;
}

impl SafeArithmetic for i64 {
    type Output = i64;
    
    fn safe_add(self, other: i64) -> Result<i64, SafeMathError> {
        self.checked_add(other)
            .ok_or_else(|| SafeMathError::Overflow {
                operation: "add".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
    
    fn safe_sub(self, other: i64) -> Result<i64, SafeMathError> {
        self.checked_sub(other)
            .ok_or_else(|| SafeMathError::Underflow {
                operation: "sub".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
    
    fn safe_mul(self, other: i64) -> Result<i64, SafeMathError> {
        self.checked_mul(other)
            .ok_or_else(|| SafeMathError::Overflow {
                operation: "mul".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
    
    fn safe_div(self, other: i64) -> Result<i64, SafeMathError> {
        if other == 0 {
            return Err(SafeMathError::DivisionByZero);
        }
        self.checked_div(other)
            .ok_or_else(|| SafeMathError::Overflow {
                operation: "div".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
    
    fn safe_mod(self, other: i64) -> Result<i64, SafeMathError> {
        if other == 0 {
            return Err(SafeMathError::DivisionByZero);
        }
        self.checked_rem(other)
            .ok_or_else(|| SafeMathError::Overflow {
                operation: "mod".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
}

impl SafeArithmetic for u64 {
    type Output = u64;
    
    fn safe_add(self, other: u64) -> Result<u64, SafeMathError> {
        self.checked_add(other)
            .ok_or_else(|| SafeMathError::Overflow {
                operation: "add".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
    
    fn safe_sub(self, other: u64) -> Result<u64, SafeMathError> {
        self.checked_sub(other)
            .ok_or_else(|| SafeMathError::Underflow {
                operation: "sub".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
    
    fn safe_mul(self, other: u64) -> Result<u64, SafeMathError> {
        self.checked_mul(other)
            .ok_or_else(|| SafeMathError::Overflow {
                operation: "mul".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
    
    fn safe_div(self, other: u64) -> Result<u64, SafeMathError> {
        if other == 0 {
            return Err(SafeMathError::DivisionByZero);
        }
        self.checked_div(other)
            .ok_or_else(|| SafeMathError::Overflow {
                operation: "div".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
    
    fn safe_mod(self, other: u64) -> Result<u64, SafeMathError> {
        if other == 0 {
            return Err(SafeMathError::DivisionByZero);
        }
        self.checked_rem(other)
            .ok_or_else(|| SafeMathError::Overflow {
                operation: "mod".to_string(),
                left: self.to_string(),
                right: other.to_string(),
            })
    }
}

/// SafeMath wrapper for type safety
pub struct SafeMath;

impl SafeMath {
    /// Safe addition for integers
    pub fn add<T: SafeArithmetic>(a: T, b: T) -> Result<T::Output, SafeMathError> {
        a.safe_add(b)
    }
    
    /// Safe subtraction for integers
    pub fn sub<T: SafeArithmetic>(a: T, b: T) -> Result<T::Output, SafeMathError> {
        a.safe_sub(b)
    }
    
    /// Safe multiplication for integers
    pub fn mul<T: SafeArithmetic>(a: T, b: T) -> Result<T::Output, SafeMathError> {
        a.safe_mul(b)
    }
    
    /// Safe division for integers
    pub fn div<T: SafeArithmetic>(a: T, b: T) -> Result<T::Output, SafeMathError> {
        a.safe_div(b)
    }
    
    /// Safe modulo for integers
    pub fn mod_<T: SafeArithmetic>(a: T, b: T) -> Result<T::Output, SafeMathError> {
        a.safe_mod(b)
    }
    
    /// Check if addition would overflow
    pub fn would_overflow_add<T: SafeArithmetic>(a: T, b: T) -> bool {
        a.safe_add(b).is_err()
    }
    
    /// Check if subtraction would underflow
    pub fn would_underflow_sub<T: SafeArithmetic>(a: T, b: T) -> bool {
        a.safe_sub(b).is_err()
    }
    
    /// Check if multiplication would overflow
    pub fn would_overflow_mul<T: SafeArithmetic>(a: T, b: T) -> bool {
        a.safe_mul(b).is_err()
    }
}

/// Register SafeMath functions in the AST
pub fn register_safe_math() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
    
    let int_type = Type::Named {
        name: "Int".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    // Safe addition
    definitions.push(Definition::FunctionDef {
        name: "SafeMath/add".to_string(),
        params: vec![
            Parameter {
                name: "a".to_string(),
                ty: int_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "b".to_string(),
                ty: int_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(int_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Safe subtraction
    definitions.push(Definition::FunctionDef {
        name: "SafeMath/sub".to_string(),
        params: vec![
            Parameter {
                name: "a".to_string(),
                ty: int_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "b".to_string(),
                ty: int_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(int_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Safe multiplication
    definitions.push(Definition::FunctionDef {
        name: "SafeMath/mul".to_string(),
        params: vec![
            Parameter {
                name: "a".to_string(),
                ty: int_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "b".to_string(),
                ty: int_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(int_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Safe division
    definitions.push(Definition::FunctionDef {
        name: "SafeMath/div".to_string(),
        params: vec![
            Parameter {
                name: "a".to_string(),
                ty: int_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "b".to_string(),
                ty: int_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(int_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    definitions
}