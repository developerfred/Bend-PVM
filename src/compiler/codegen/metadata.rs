use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata for a contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Contract name
    pub name: String,

    /// Contract version
    pub version: String,

    /// Contract author
    pub author: Option<String>,

    /// Contract license
    pub license: Option<String>,

    /// Contract description
    pub description: Option<String>,

    /// Contract functions (name -> function metadata)
    pub functions: HashMap<String, FunctionMetadata>,

    /// Contract types (name -> type metadata)
    pub types: HashMap<String, TypeMetadata>,

    /// Contract objects (name -> object metadata)
    pub objects: HashMap<String, ObjectMetadata>,

    /// Contract source files
    pub sources: Vec<SourceMetadata>,
}

/// Metadata for a contract function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionMetadata {
    /// Function name
    pub name: String,

    /// Function selector (for external calls)
    pub selector: [u8; 4],

    /// Function visibility (public, external, internal, private)
    pub visibility: FunctionVisibility,

    /// Function parameters
    pub params: Vec<ParameterMetadata>,

    /// Function return type
    pub return_type: Option<String>,

    /// Function gas cost (estimate)
    pub gas_cost: Option<u64>,

    /// Function documentation
    pub documentation: Option<String>,

    /// Source location
    pub source_location: Option<SourceLocation>,
}

/// Function visibility
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FunctionVisibility {
    /// Public function (can be called from outside the contract)
    Public,

    /// External function (can only be called from outside the contract)
    External,

    /// Internal function (can only be called from within the contract)
    Internal,

    /// Private function (can only be called from the function where it's defined)
    Private,
}

/// Metadata for a function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterMetadata {
    /// Parameter name
    pub name: String,

    /// Parameter type
    pub type_name: String,

    /// Parameter documentation
    pub documentation: Option<String>,
}

/// Metadata for a contract type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMetadata {
    /// Type name
    pub name: String,

    /// Type parameters
    pub type_params: Vec<String>,

    /// Type variants
    pub variants: Vec<VariantMetadata>,

    /// Type documentation
    pub documentation: Option<String>,

    /// Source location
    pub source_location: Option<SourceLocation>,
}

/// Metadata for a type variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantMetadata {
    /// Variant name
    pub name: String,

    /// Variant fields
    pub fields: Vec<FieldMetadata>,

    /// Variant documentation
    pub documentation: Option<String>,
}

/// Metadata for a contract object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    /// Object name
    pub name: String,

    /// Object type parameters
    pub type_params: Vec<String>,

    /// Object fields
    pub fields: Vec<FieldMetadata>,

    /// Object documentation
    pub documentation: Option<String>,

    /// Source location
    pub source_location: Option<SourceLocation>,
}

/// Metadata for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMetadata {
    /// Field name
    pub name: String,

    /// Field type
    pub type_name: String,

    /// Whether the field is recursive
    pub is_recursive: bool,

    /// Field documentation
    pub documentation: Option<String>,
}

/// Metadata for a source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMetadata {
    /// Source file name
    pub name: String,

    /// Content hash (for verification)
    pub content_hash: String,
}

/// Source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Source file index
    pub file_index: usize,

    /// Start line
    pub start_line: usize,

    /// Start column
    pub start_column: usize,

    /// End line
    pub end_line: usize,

    /// End column
    pub end_column: usize,
}

/// Build metadata for a contract from a parsed AST
pub fn build_metadata(
    name: &str,
    version: &str,
    sources: &[(&str, &str)],
    functions: HashMap<String, FunctionMetadata>,
    types: HashMap<String, TypeMetadata>,
    objects: HashMap<String, ObjectMetadata>,
) -> ContractMetadata {
    // Create source metadata
    let mut source_metadata = Vec::new();
    for (name, content) in sources {
        // In a real implementation, compute a proper hash
        let content_hash = format!("hash_{}", name.replace('.', "_"));

        source_metadata.push(SourceMetadata {
            name: name.to_string(),
            content_hash,
        });
    }

    ContractMetadata {
        name: name.to_string(),
        version: version.to_string(),
        author: None,
        license: None,
        description: None,
        functions,
        types,
        objects,
        sources: source_metadata,
    }
}

/// Compute a function selector (similar to Ethereum)
pub fn compute_function_selector(name: &str, params: &[ParameterMetadata]) -> [u8; 4] {
    // In a real implementation, this would compute a proper function selector
    // by hashing the function signature (name and parameter types)
    let mut selector = [0u8; 4];

    // Simple approach: use first 4 bytes of the function name
    let name_bytes = name.as_bytes();
    for i in 0..std::cmp::min(4, name_bytes.len()) {
        selector[i] = name_bytes[i];
    }

    selector
}
