use std::collections::{HashMap, HashSet};
use crate::compiler::parser::ast::*;
use crate::compiler::module::{ModuleError, namespace::Namespace};

/// Name resolver for resolving names in a program
pub struct NameResolver {
    /// Namespaces (name -> namespace)
    namespaces: HashMap<String, Namespace>,
    
    /// Stack of active namespaces
    namespace_stack: Vec<String>,
    
    /// Set of defined names in the current scope
    defined_names: HashSet<String>,
    
    /// Map of name to fully qualified name
    name_mapping: HashMap<String, String>,
}

impl NameResolver {
    /// Create a new name resolver
    pub fn new() -> Self {
        NameResolver {
            namespaces: HashMap::new(),
            namespace_stack: Vec::new(),
            defined_names: HashSet::new(),
            name_mapping: HashMap::new(),
        }
    }
    
    /// Push a namespace onto the stack
    pub fn push_namespace(&mut self, namespace: Namespace) {
        let name = namespace.name.clone();
        self.namespaces.insert(name.clone(), namespace);
        self.namespace_stack.push(name);
    }
    
    /// Pop a namespace from the stack
    pub fn pop_namespace(&mut self) {
        self.namespace_stack.pop();
    }
    
    /// Add an imported namespace
    pub fn add_import(&mut self, name: String, namespace: Namespace) {
        self.namespaces.insert(name, namespace);
    }
    
    /// Resolve names in a program
    pub fn resolve_program(&mut self, program: &mut Program) -> Result<(), ModuleError> {
        // Process imports
        for import in &program.imports {
            self.resolve_import(import)?;
        }
        
        // Process definitions
        for definition in &mut program.definitions {
            self.resolve_definition(definition)?;
        }
        
        Ok(())
    }
    
    /// Resolve an import
    fn resolve_import(&mut self, import: &Import) -> Result<(), ModuleError> {
        match import {
            Import::FromImport { path, names, .. } => {
                // For each imported name, add it to the name mapping
                for name in names {
                    let original_name = &name.name;
                    let alias = if let Some(alias) = &name.alias {
                        alias
                    } else {
                        original_name
                    };
                    
                    // Map the alias to the fully qualified name
                    let qualified_name = format!("{}/{}", path, original_name);
                    self.name_mapping.insert(alias.clone(), qualified_name);
                    
                    // Add the name to the set of defined names
                    self.defined_names.insert(alias.clone());
                }
            },
            Import::DirectImport { names, .. } => {
                // For direct imports, we map the module name to itself
                for name in names {
                    self.name_mapping.insert(name.clone(), name.clone());
                }
            },
        }
        
        Ok(())
    }
    
    /// Resolve a definition
    fn resolve_definition(&mut self, definition: &mut Definition) -> Result<(), ModuleError> {
        match definition {
            Definition::FunctionDef { name, params, body, .. } => {
                // Add the function name to the set of defined names
                self.defined_names.insert(name.clone());
                
                // Create a new scope for the function
                let mut scope = self.defined_names.clone();
                
                // Add parameters to the scope
                for param in params {
                    scope.insert(param.name.clone());
                }
                
                // Save the current scope
                let old_scope = std::mem::replace(&mut self.defined_names, scope);
                
                // Resolve names in the function body
                self.resolve_block(body)?;
                
                // Restore the old scope
                self.defined_names = old_scope;
            },
            Definition::TypeDef { name, .. } => {
                // Add the type name to the set of defined names
                self.defined_names.insert(name.clone());
            },
            Definition::ObjectDef { name, .. } => {
                // Add the object name to the set of defined names
                self.defined_names.insert(name.clone());
            },
        }
        
        Ok(())
    }
    
    /// Resolve names in a block
    fn resolve_block(&mut self, block: &mut Block) -> Result<(), ModuleError> {
        // Create a new scope for the block
        let old_scope = self.defined_names.clone();
        
        // Resolve names in each statement
        for statement in &mut block.statements {
            self.resolve_statement(statement)?;
        }
        
        // Restore the old scope
        self.defined_names = old_scope;
        
        Ok(())
    }
    
    /// Resolve names in a statement
    fn resolve_statement(&mut self, statement: &mut Statement) -> Result<(), ModuleError> {
        match statement {
            Statement::Return { value, .. } => {
                self.resolve_expr(value)?;
            },
            Statement::Assignment { pattern, value, .. } => {
                // Resolve the value expression
                self.resolve_expr(value)?;
                
                // Add defined names from the pattern
                self.resolve_pattern(pattern)?;
            },
            Statement::If { condition, then_branch, else_branch, .. } => {
                // Resolve the condition
                self.resolve_expr(condition)?;
                
                // Resolve the branches
                self.resolve_block(then_branch)?;
                self.resolve_block(else_branch)?;
            },
            Statement::Expr { expr, .. } => {
                self.resolve_expr(expr)?;
            },
            // Handle other statement types
            _ => {},
        }
        
        Ok(())
    }
    
    /// Resolve names in an expression
    fn resolve_expr(&mut self, expr: &mut Expr) -> Result<(), ModuleError> {
        match expr {
            Expr::Variable { name, .. } => {
                // Resolve the variable name
                if let Some(qualified_name) = self.name_mapping.get(name) {
                    // Replace the name with the fully qualified name
                    *name = qualified_name.clone();
                } else if !self.defined_names.contains(name) {
                    // The name is not defined locally and is not imported
                    // Check if it's defined in the current namespace
                    let current_namespace = self.namespace_stack.last()
                        .ok_or_else(|| ModuleError::Generic("No active namespace".to_string()))?;
                    
                    if let Some(namespace) = self.namespaces.get(current_namespace) {
                        if namespace.contains(name) {
                            // The name is defined in the current namespace
                            let qualified_name = format!("{}/{}", current_namespace, name);
                            *name = qualified_name;
                        } else {
                            // The name is not defined anywhere we can see
                            return Err(ModuleError::SymbolNotFound(
                                name.clone(),
                                current_namespace.clone(),
                            ));
                        }
                    }
                }
            },
            Expr::BinaryOp { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            },
            Expr::FunctionCall { function, args, named_args, .. } => {
                // Resolve the function expression
                self.resolve_expr(function)?;
                
                // Resolve the arguments
                for arg in args {
                    self.resolve_expr(arg)?;
                }
                
                // Resolve named arguments
                for (_, arg) in named_args {
                    self.resolve_expr(arg)?;
                }
            },
            Expr::Lambda { params, body, .. } => {
                // Create a new scope for the lambda
                let mut scope = self.defined_names.clone();
                
                // Add parameters to the scope
                for param in params {
                    scope.insert(param.name.clone());
                }
                
                // Save the current scope
                let old_scope = std::mem::replace(&mut self.defined_names, scope);
                
                // Resolve names in the lambda body
                self.resolve_expr(body)?;
                
                // Restore the old scope
                self.defined_names = old_scope;
            },
            Expr::UnsccopedLambda { params, body, .. } => {
                // Create a new scope for the lambda
                let mut scope = self.defined_names.clone();
                
                // Add parameters to the scope
                for param in params {
                    scope.insert(param.clone());
                }
                
                // Save the current scope
                let old_scope = std::mem::replace(&mut self.defined_names, scope);
                
                // Resolve names in the lambda body
                self.resolve_expr(body)?;
                
                // Restore the old scope
                self.defined_names = old_scope;
            },
            Expr::Block { block, .. } => {
                self.resolve_block(block)?;
            },
            // Handle other expression types
            _ => {},
        }
        
        Ok(())
    }
    
    /// Resolve names in a pattern and add defined names to the scope
    fn resolve_pattern(&mut self, pattern: &mut Pattern) -> Result<(), ModuleError> {
        match pattern {
            Pattern::Variable { name, .. } => {
                // Add the variable name to the set of defined names
                self.defined_names.insert(name.clone());
            },
            Pattern::Tuple { elements, .. } => {
                // Resolve names in tuple elements
                for element in elements {
                    self.resolve_pattern(element)?;
                }
            },
            Pattern::Constructor { fields, .. } => {
                // Resolve names in constructor fields
                for (_, field) in fields {
                    self.resolve_pattern(field)?;
                }
            },
            // Handle other pattern types
            _ => {},
        }
        
        Ok(())
    }
}