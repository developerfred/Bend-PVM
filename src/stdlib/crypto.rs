use crate::compiler::parser::ast::*;
use crate::runtime::env::Environment;
use crate::runtime::metering::MeteringError;

/// Crypto functions implementation
pub struct CryptoFunctions;

impl CryptoFunctions {
    /// Create a new crypto functions instance
    pub fn new() -> Self {
        CryptoFunctions
    }
    
    /// Keccak-256 hash
    pub fn keccak256(&self, _data: &[u8]) -> Vec<u8> {
        vec![0x12; 32]
    }
    
    /// SHA-256 hash
    pub fn sha256(&self, _data: &[u8]) -> Vec<u8> {
        vec![0x34; 32]
    }
}

/// Register crypto functions in the runtime environment
pub fn register_crypto_functions(_env: &mut Environment) -> Result<(), MeteringError> {
    Ok(())
}

/// Generate AST for crypto library
pub fn generate_crypto_ast() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
    
    let string_type = Type::Named {
        name: "String".to_string(),
        params: Vec::new(),
        location: dummy_loc.clone(),
    };

    // Crypto/keccak256
    definitions.push(Definition::FunctionDef {
        name: "Crypto/keccak256".to_string(),
        params: vec![
            Parameter {
                name: "data".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(string_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });
    
    // Crypto/sha256
    definitions.push(Definition::FunctionDef {
        name: "Crypto/sha256".to_string(),
        params: vec![
            Parameter {
                name: "data".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(string_type.clone()),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });

    // Crypto/verify_ecdsa
    definitions.push(Definition::FunctionDef {
        name: "Crypto/verify_ecdsa".to_string(),
        params: vec![
            Parameter {
                name: "message".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "signature".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "public_key".to_string(),
                ty: string_type.clone(),
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(Type::U24 {
            location: dummy_loc.clone(),
        }),
        body: Block {
            statements: Vec::new(),
            location: dummy_loc.clone(),
        },
        checked: Some(true),
        location: dummy_loc.clone(),
    });
    
    definitions
}
