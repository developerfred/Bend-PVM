use std::collections::HashMap;
use crate::compiler::parser::ast::*;
use crate::compiler::module::ModuleError;

/// Represents an import in a namespace
#[derive(Debug, Clone)]
pub struct Import {
    /// Original name in the source module
    pub original_name: String,
    
    /// Alias in this module
    pub alias: String,
    
    /// Source module name
    pub source_module: String,
}

/// Represents a namespace in a module
#[derive(Debug, Clone)]
pub struct Namespace {
    /// Namespace name (usually the module name)
    pub name: String,
    
    /// Defined symbols (name -> definition)
    pub definitions: HashMap<String, Definition>,
    
    /// Imported symbols (alias -> import)
    pub imports: HashMap<String, Import>,
}

impl Namespace {
    /// Create a new namespace
    pub fn new(name: String) -> Self {
        Namespace {
            name,
            definitions: HashMap::new(),
            imports: HashMap::new(),
        }
    }
    
    /// Add a definition to the namespace
    pub fn add_definition(&mut self, name: String, definition: Definition) -> Result<(), ModuleError> {
        // Check if the definition already exists
        if self.definitions.contains_key(&name) {
            return Err(ModuleError::DuplicateSymbol(name));
        }
        
        // Add the definition
        self.definitions.insert(name, definition);
        
        Ok(())
    }
    
    /// Add an import to the namespace
    pub fn add_import(
        &mut self,
        original_name: String,
        alias: String,
        source_module: String,
    ) -> Result<(), ModuleError> {
        // Check if the alias already exists
        if self.imports.contains_key(&alias) {
            return Err(ModuleError::DuplicateSymbol(alias));
        }
        
        // Add the import
        self.imports.insert(
            alias.clone(),
            Import {
                original_name,
                alias,
                source_module,
            },
        );
        
        Ok(())
    }
    
    /// Look up a symbol in the namespace
    pub fn lookup(&self, name: &str) -> Option<&Definition> {
        // First, check local definitions
        if let Some(definition) = self.definitions.get(name) {
            return Some(definition);
        }
        
        // Then, check imports
        if let Some(import) = self.imports.get(name) {
            // For imports, we just return None since we need to look up
            // the definition in the source module, which we don't have here
            return None;
        }
        
        None
    }
    
    /// Check if a name is already defined in this namespace
    pub fn contains(&self, name: &str) -> bool {
        self.definitions.contains_key(name) || self.imports.contains_key(name)
    }
    
    /// Get all defined names in this namespace
    pub fn defined_names(&self) -> Vec<String> {
        self.definitions.keys().cloned().collect()
    }
    
    /// Get all imported names in this namespace
    pub fn imported_names(&self) -> Vec<String> {
        self.imports.keys().cloned().collect()
    }
}