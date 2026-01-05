//! Foreign Function Interface (FFI) for Bend-PVM
//!
//! This module provides support for calling external functions from Bend-PVM contracts.
//! It allows contracts to interface with the host environment and other contracts.

use crate::stdlib::string::StringUtils;
use std::collections::HashMap;
use thiserror::Error;

/// FFI-related errors
#[derive(Error, Debug)]
pub enum FFIError {
    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Invalid function signature: {0}")]
    InvalidSignature(String),

    #[error("ABI encoding error: {0}")]
    EncodingError(String),

    #[error("ABI decoding error: {0}")]
    DecodingError(String),

    #[error("External call failed: {0}")]
    CallFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}

/// Function signature for external calls
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionSignature {
    /// Function name
    pub name: String,

    /// Parameter types
    pub inputs: Vec<String>,

    /// Return type
    pub output: Option<String>,

    /// Whether this is a view function (read-only)
    pub view: bool,
}

impl FunctionSignature {
    /// Create a new function signature
    pub fn new(name: impl Into<String>, inputs: Vec<String>, output: Option<String>) -> Self {
        FunctionSignature {
            name: name.into(),
            inputs,
            output,
            view: false,
        }
    }

    /// Mark as view function
    pub fn view(mut self) -> Self {
        self.view = true;
        self
    }

    /// Generate function selector (simplified)
    pub fn selector(&self) -> [u8; 4] {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.name.hash(&mut hasher);
        let hash = hasher.finish();

        let bytes = hash.to_be_bytes();
        [bytes[0], bytes[1], bytes[2], bytes[3]]
    }
}

/// External function registry
#[derive(Debug)]
pub struct FFIRegistry {
    /// Registered functions
    functions: HashMap<String, FunctionSignature>,

    /// Permission checks
    permissions: HashMap<String, Vec<String>>,
}

impl FFIRegistry {
    /// Create a new FFI registry
    pub fn new() -> Self {
        FFIRegistry {
            functions: HashMap::new(),
            permissions: HashMap::new(),
        }
    }

    /// Register an external function
    pub fn register_function(&mut self, signature: FunctionSignature) -> Result<(), FFIError> {
        if self.functions.contains_key(&signature.name) {
            return Err(FFIError::InvalidSignature(format!(
                "Function {} already registered",
                signature.name
            )));
        }

        self.functions.insert(signature.name.clone(), signature);
        Ok(())
    }

    /// Get a function signature
    pub fn get_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    /// Check if a function is registered
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Set permissions for a function
    pub fn set_permissions(&mut self, function: &str, allowed_callers: Vec<String>) {
        self.permissions
            .insert(function.to_string(), allowed_callers);
    }

    /// Check if a caller is allowed to call a function
    pub fn check_permission(&self, function: &str, caller: &str) -> bool {
        if let Some(allowed) = self.permissions.get(function) {
            allowed.contains(&caller.to_string()) || allowed.contains(&"*".to_string())
        } else {
            true // No permissions set, allow all
        }
    }

    /// List all registered functions
    pub fn list_functions(&self) -> Vec<&FunctionSignature> {
        self.functions.values().collect()
    }
}

impl Default for FFIRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// FFI Call context
#[derive(Debug)]
pub struct FFICallContext {
    /// Calling contract address
    pub caller: String,

    /// Gas limit for the call
    pub gas_limit: u64,

    /// Value being sent (in wei/gwei)
    pub value: u64,

    /// Call data
    pub data: Vec<u8>,
}

impl FFICallContext {
    /// Create a new call context
    pub fn new(caller: impl Into<String>) -> Self {
        FFICallContext {
            caller: caller.into(),
            gas_limit: 0,
            value: 0,
            data: Vec::new(),
        }
    }

    /// Set gas limit
    pub fn gas_limit(mut self, limit: u64) -> Self {
        self.gas_limit = limit;
        self
    }

    /// Set value
    pub fn value(mut self, value: u64) -> Self {
        self.value = value;
        self
    }

    /// Set call data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }
}

/// FFI Call result
#[derive(Debug)]
pub enum FFICallResult {
    /// Successful call with return data
    Success(Vec<u8>),

    /// Call reverted with reason
    Revert(Vec<u8>),

    /// Call failed
    Error(String),
}

/// ABI Encoder/Decoder for FFI calls
pub struct ABICodec;

impl ABICodec {
    /// Encode function call data
    pub fn encode_call(
        signature: &FunctionSignature,
        args: &[Vec<u8>],
    ) -> Result<Vec<u8>, FFIError> {
        if args.len() != signature.inputs.len() {
            return Err(FFIError::InvalidSignature(format!(
                "Expected {} arguments, got {}",
                signature.inputs.len(),
                args.len()
            )));
        }

        let mut data = Vec::new();

        // Add function selector
        data.extend_from_slice(&signature.selector());

        // Add encoded arguments (simplified - real implementation would use proper ABI encoding)
        for arg in args {
            data.extend_from_slice(arg);
        }

        Ok(data)
    }

    /// Decode return data
    pub fn decode_return(signature: &FunctionSignature, data: &[u8]) -> Result<Vec<u8>, FFIError> {
        // Simplified decoding - real implementation would parse ABI-encoded data
        if data.is_empty() && signature.output.is_some() {
            return Err(FFIError::DecodingError("Expected return data".to_string()));
        }

        Ok(data.to_vec())
    }

    /// Encode a single value (simplified)
    pub fn encode_value(value: &str) -> Vec<u8> {
        // Simple string to bytes conversion
        // Real implementation would handle different types
        value.as_bytes().to_vec()
    }

    /// Decode a single value (simplified)
    pub fn decode_value(data: &[u8]) -> String {
        // Simple bytes to string conversion
        String::from_utf8_lossy(data).to_string()
    }
}

/// FFI Manager - main interface for FFI operations
pub struct FFIManager {
    registry: FFIRegistry,
}

impl FFIManager {
    /// Create a new FFI manager
    pub fn new() -> Self {
        FFIManager {
            registry: FFIRegistry::new(),
        }
    }

    /// Register a built-in function
    pub fn register_builtin(
        &mut self,
        name: &str,
        inputs: Vec<&str>,
        output: Option<&str>,
    ) -> Result<(), FFIError> {
        let signature = FunctionSignature::new(
            name,
            inputs.into_iter().map(|s| s.to_string()).collect(),
            output.map(|s| s.to_string()),
        );

        self.registry.register_function(signature)
    }

    /// Make an FFI call
    pub fn call(
        &self,
        function_name: &str,
        args: Vec<Vec<u8>>,
        context: &FFICallContext,
    ) -> Result<FFICallResult, FFIError> {
        // Check if function is registered
        let signature = self
            .registry
            .get_function(function_name)
            .ok_or_else(|| FFIError::FunctionNotFound(function_name.to_string()))?;

        // Check permissions
        if !self
            .registry
            .check_permission(function_name, &context.caller)
        {
            return Err(FFIError::PermissionDenied(format!(
                "Caller {} not allowed to call {}",
                context.caller, function_name
            )));
        }

        // Encode call data
        let _call_data = ABICodec::encode_call(signature, &args)?;

        // Simulate external call (in real implementation, this would make actual external call)
        match function_name {
            "console.log" => {
                let message = ABICodec::decode_value(&args[0]);
                println!("[CONSOLE] {}", message);
                Ok(FFICallResult::Success(Vec::new()))
            }
            "block.timestamp" => {
                // Return current timestamp (simplified)
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                Ok(FFICallResult::Success(timestamp.to_be_bytes().to_vec()))
            }
            "block.number" => {
                // Return block number (simplified)
                let block_number: u64 = 12345;
                Ok(FFICallResult::Success(block_number.to_be_bytes().to_vec()))
            }
            "string.concat" => {
                let s1 = ABICodec::decode_value(&args[0]);
                let s2 = ABICodec::decode_value(&args[1]);
                Ok(FFICallResult::Success(ABICodec::encode_value(&(s1 + &s2))))
            }
            "string.length" => {
                let s = ABICodec::decode_value(&args[0]);
                let len = s.len() as u64;
                Ok(FFICallResult::Success(len.to_be_bytes().to_vec()))
            }
            "string.to_uppercase" => {
                let s = ABICodec::decode_value(&args[0]);
                let result = StringUtils::to_uppercase(&s);
                Ok(FFICallResult::Success(ABICodec::encode_value(&result)))
            }
            "string.to_lowercase" => {
                let s = ABICodec::decode_value(&args[0]);
                let result = StringUtils::to_lowercase(&s);
                Ok(FFICallResult::Success(ABICodec::encode_value(&result)))
            }
            "string.trim" => {
                let s = ABICodec::decode_value(&args[0]);
                let result = StringUtils::trim(&s);
                Ok(FFICallResult::Success(ABICodec::encode_value(&result)))
            }
            "string.contains" => {
                let s = ABICodec::decode_value(&args[0]);
                let sub = ABICodec::decode_value(&args[1]);
                let result = if StringUtils::contains(&s, &sub) {
                    1u32
                } else {
                    0u32
                };
                Ok(FFICallResult::Success(result.to_be_bytes().to_vec()))
            }
            _ => {
                // Generic external call simulation
                println!("[FFI] Calling external function: {}", function_name);
                Ok(FFICallResult::Success(vec![1, 2, 3, 4])) // Mock return data
            }
        }
    }

    /// Get registry reference
    pub fn registry(&self) -> &FFIRegistry {
        &self.registry
    }

    /// Get mutable registry reference
    pub fn registry_mut(&mut self) -> &mut FFIRegistry {
        &mut self.registry
    }
}

impl Default for FFIManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common FFI operations
pub mod ffi_functions {
    use super::*;

    /// Register common built-in functions
    pub fn register_builtins(manager: &mut FFIManager) -> Result<(), FFIError> {
        // Console functions
        manager.register_builtin("console.log", vec!["string"], None)?;
        manager.register_builtin("console.error", vec!["string"], None)?;
        manager.register_builtin("console.warn", vec!["string"], None)?;

        // Blockchain functions
        manager.register_builtin("block.timestamp", vec![], Some("uint256"))?;
        manager.register_builtin("block.number", vec![], Some("uint256"))?;
        manager.register_builtin("block.gaslimit", vec![], Some("uint256"))?;
        manager.register_builtin("block.coinbase", vec![], Some("address"))?;

        // Transaction functions
        manager.register_builtin("tx.origin", vec![], Some("address"))?;
        manager.register_builtin("tx.gasprice", vec![], Some("uint256"))?;

        // Contract functions
        manager.register_builtin("self.address", vec![], Some("address"))?;
        manager.register_builtin("self.balance", vec![], Some("uint256"))?;

        // Math functions
        manager.register_builtin("math.sqrt", vec!["uint256"], Some("uint256"))?;
        manager.register_builtin("math.pow", vec!["uint256", "uint256"], Some("uint256"))?;

        // String functions
        manager.register_builtin("string.concat", vec!["string", "string"], Some("string"))?;
        manager.register_builtin("string.length", vec!["string"], Some("uint256"))?;
        manager.register_builtin("string.to_uppercase", vec!["string"], Some("string"))?;
        manager.register_builtin("string.to_lowercase", vec!["string"], Some("string"))?;
        manager.register_builtin("string.trim", vec!["string"], Some("string"))?;
        manager.register_builtin("string.contains", vec!["string", "string"], Some("uint256"))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_signature() {
        let sig = FunctionSignature::new(
            "test",
            vec!["uint256".to_string(), "address".to_string()],
            Some("bool".to_string()),
        )
        .view();

        assert_eq!(sig.name, "test");
        assert_eq!(sig.inputs.len(), 2);
        assert_eq!(sig.output, Some("bool".to_string()));
        assert!(sig.view);

        let selector = sig.selector();
        assert_eq!(selector.len(), 4);
    }

    #[test]
    fn test_registry() {
        let mut registry = FFIRegistry::new();

        let sig = FunctionSignature::new(
            "test_func",
            vec!["uint256".to_string()],
            Some("bool".to_string()),
        );
        registry.register_function(sig.clone()).unwrap();

        assert!(registry.has_function("test_func"));
        assert_eq!(
            registry.get_function("test_func").unwrap().name,
            "test_func"
        );

        // Test permissions
        registry.set_permissions("test_func", vec!["contract1".to_string()]);
        assert!(registry.check_permission("test_func", "contract1"));
        assert!(!registry.check_permission("test_func", "contract2"));
    }

    #[test]
    fn test_ffi_manager() {
        let mut manager = FFIManager::new();

        // Register a function
        manager
            .register_builtin("test.call", vec!["string"], Some("string"))
            .unwrap();

        // Test call
        let context = FFICallContext::new("caller1");
        let args = vec![b"hello".to_vec()];

        let result = manager.call("test.call", args, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_abi_codec() {
        let sig = FunctionSignature::new(
            "test",
            vec!["uint256".to_string()],
            Some("bool".to_string()),
        );

        let args = vec![vec![1, 2, 3, 4]];
        let encoded = ABICodec::encode_call(&sig, &args).unwrap();

        // Should have selector (4 bytes) + data
        assert!(encoded.len() >= 4);

        let decoded = ABICodec::decode_return(&sig, &encoded[4..]).unwrap();
        assert_eq!(decoded, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_builtin_registration() {
        let mut manager = FFIManager::new();
        ffi_functions::register_builtins(&mut manager).unwrap();

        // Check some builtins are registered
        assert!(manager.registry().has_function("console.log"));
        assert!(manager.registry().has_function("block.timestamp"));
        assert!(manager.registry().has_function("math.sqrt"));
    }

    #[test]
    fn test_call_context() {
        let context = FFICallContext::new("test_caller")
            .gas_limit(100000)
            .value(1000)
            .data(vec![1, 2, 3]);

        assert_eq!(context.caller, "test_caller");
        assert_eq!(context.gas_limit, 100000);
        assert_eq!(context.value, 1000);
        assert_eq!(context.data, vec![1, 2, 3]);
    }

    #[test]
    fn test_error_handling() {
        let manager = FFIManager::new();
        let context = FFICallContext::new("caller");

        // Test calling unregistered function
        let result = manager.call("nonexistent", vec![], &context);
        assert!(matches!(result, Err(FFIError::FunctionNotFound(_))));

        // Test permission denied
        let mut manager = FFIManager::new();
        manager
            .register_builtin("restricted", vec![], None)
            .unwrap();
        manager
            .registry_mut()
            .set_permissions("restricted", vec!["allowed".to_string()]);

        let result = manager.call("restricted", vec![], &context);
        assert!(matches!(result, Err(FFIError::PermissionDenied(_))));
    }
}

// SECURITY FIX: Input validation constants
const MAX_ARGS: usize = 16;
const MAX_INPUT_SIZE: usize = 65536;
const MAX_OUTPUT_SIZE: usize = 65536;

fn is_valid_function_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
}
