use serde::{Serialize, Deserialize};

use crate::compiler::codegen::metadata::{ContractMetadata, FunctionMetadata};

/// Represents the ABI for a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractABI {
    /// Contract name
    pub name: String,
    
    /// Contract version
    pub version: String,
    
    /// Contract methods
    pub methods: Vec<MethodABI>,
    
    /// Contract events
    pub events: Vec<EventABI>,
    
    /// Contract errors
    pub errors: Vec<ErrorABI>,
    
    /// Contract state variables
    pub state_variables: Vec<StateVariableABI>,
    
    /// Custom types used in the contract
    pub types: Vec<TypeABI>,
}

/// Represents a method in the contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodABI {
    /// Method name
    pub name: String,
    
    /// Method selector (4-byte hash of the method signature)
    pub selector: String,
    
    /// Method type (function, constructor, etc.)
    pub type_: MethodType,
    
    /// Method inputs
    pub inputs: Vec<ParameterABI>,
    
    /// Method outputs
    pub outputs: Vec<ParameterABI>,
    
    /// Method state mutability
    pub state_mutability: StateMutability,
    
    /// Whether the method is payable
    pub payable: bool,
}

/// Represents a parameter in the contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterABI {
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub type_: String,
    
    /// Parameter components (for complex types)
    pub components: Option<Vec<ParameterABI>>,
    
    /// Whether the parameter is indexed (for events)
    pub indexed: Option<bool>,
}

/// Represents an event in the contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventABI {
    /// Event name
    pub name: String,
    
    /// Event inputs
    pub inputs: Vec<ParameterABI>,
    
    /// Whether the event is anonymous
    pub anonymous: bool,
}

/// Represents an error in the contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorABI {
    /// Error name
    pub name: String,
    
    /// Error inputs
    pub inputs: Vec<ParameterABI>,
}

/// Represents a state variable in the contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVariableABI {
    /// State variable name
    pub name: String,
    
    /// State variable type
    pub type_: String,
    
    /// Whether the state variable is public
    pub public: bool,
    
    /// Whether the state variable is constant
    pub constant: bool,
}

/// Represents a custom type in the contract ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeABI {
    /// Type name
    pub name: String,
    
    /// Type kind (struct, enum, etc.)
    pub kind: TypeKind,
    
    /// Type components (for complex types)
    pub components: Vec<ParameterABI>,
}

/// Method types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MethodType {
    /// Regular function
    #[serde(rename = "function")]
    Function,
    
    /// Contract constructor
    #[serde(rename = "constructor")]
    Constructor,
    
    /// Receive function (for receiving native currency)
    #[serde(rename = "receive")]
    Receive,
    
    /// Fallback function (called when no other function matches)
    #[serde(rename = "fallback")]
    Fallback,
}

/// State mutability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateMutability {
    /// Function does not read or modify state
    #[serde(rename = "pure")]
    Pure,
    
    /// Function reads but does not modify state
    #[serde(rename = "view")]
    View,
    
    /// Function may modify state
    #[serde(rename = "nonpayable")]
    NonPayable,
    
    /// Function may receive native currency
    #[serde(rename = "payable")]
    Payable,
}

/// Type kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeKind {
    /// Struct type
    #[serde(rename = "struct")]
    Struct,
    
    /// Enum type
    #[serde(rename = "enum")]
    Enum,
    
    /// Tuple type
    #[serde(rename = "tuple")]
    Tuple,
}

/// Generate an ABI from contract metadata
pub fn generate_abi(metadata: &ContractMetadata) -> ContractABI {
    let mut methods = Vec::new();
    
    // Add methods from the metadata
    for (name, function) in &metadata.functions {
        methods.push(function_to_method_abi(name, function));
    }
    
    // For this example, we're not implementing events, errors, state variables, or types
    
    ContractABI {
        name: metadata.name.clone(),
        version: metadata.version.clone(),
        methods,
        events: Vec::new(),
        errors: Vec::new(),
        state_variables: Vec::new(),
        types: Vec::new(),
    }
}

/// Convert a function metadata to a method ABI
fn function_to_method_abi(name: &str, function: &FunctionMetadata) -> MethodABI {
    // Convert function parameters to ABI parameters
    let mut inputs = Vec::new();
    for param in &function.params {
        inputs.push(ParameterABI {
            name: param.name.clone(),
            type_: param.type_name.clone(),
            components: None,
            indexed: None,
        });
    }
    
    // Convert function return type to ABI parameters
    let outputs = if let Some(return_type) = &function.return_type {
        vec![ParameterABI {
            name: "".to_string(),
            type_: return_type.clone(),
            components: None,
            indexed: None,
        }]
    } else {
        Vec::new()
    };
    
    // Convert function selector to hex string
    let selector = hex::encode(function.selector);
    
    MethodABI {
        name: name.to_string(),
        selector: format!("0x{}", selector),
        type_: MethodType::Function,
        inputs,
        outputs,
        state_mutability: StateMutability::NonPayable, // Default to non-payable
        payable: false,
    }
}

/// Parse an ABI from JSON
pub fn parse_abi(json: &str) -> Result<ContractABI, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serialize an ABI to JSON
pub fn serialize_abi(abi: &ContractABI) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(abi)
}