pub mod loader;
pub mod namespace;
pub mod resolver;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

use self::loader::ModuleLoader;
use self::namespace::Namespace;
use self::resolver::NameResolver;
use crate::compiler::parser::ast::*;

#[derive(Error, Debug, Clone)]
pub enum ModuleError {
    #[error("Module error: {0}")]
    Generic(String),

    #[error("Module not found: {0}")]
    NotFound(String),

    #[error("Circular module dependency: {0}")]
    CircularDependency(String),

    #[error("Failed to load module: {0}")]
    LoadFailure(String),

    #[error("Symbol {0} not found in module {1}")]
    SymbolNotFound(String, String),

    #[error("Duplicate symbol: {0}")]
    DuplicateSymbol(String),

    #[error("IO error: {0}")]
    IO(String),
}

/// Represents a module in the Bend-PVM language
#[derive(Debug, Clone)]
pub struct Module {
    /// Module name
    pub name: String,

    /// Module path
    pub path: PathBuf,

    /// AST for the module
    pub ast: Program,

    /// Namespace for the module
    pub namespace: Namespace,

    /// Imported modules
    pub imports: HashMap<String, Module>,

    /// Exported symbols
    pub exports: HashMap<String, Symbol>,
}

/// Represents a symbol in a module
#[derive(Debug, Clone)]
pub enum Symbol {
    /// Function symbol
    Function {
        /// Function name
        name: String,

        /// Function definition
        definition: Box<Definition>,
    },

    /// Type symbol
    Type {
        /// Type name
        name: String,

        /// Type definition
        definition: Box<Definition>,
    },

    /// Object symbol
    Object {
        /// Object name
        name: String,

        /// Object definition
        definition: Box<Definition>,
    },

    /// Module symbol
    Module {
        /// Module name
        name: String,

        /// Module definition
        definition: Box<Definition>,
    },

    /// Value symbol
    Value {
        /// Value name
        name: String,

        /// Value expression
        expression: Box<Expr>,
    },
}

/// Module system for managing modules and namespaces
pub struct ModuleSystem {
    /// Module loader
    loader: ModuleLoader,

    /// Name resolver
    resolver: NameResolver,

    /// Loaded modules
    modules: HashMap<String, Module>,

    /// Search paths for modules
    search_paths: Vec<PathBuf>,
}

impl Default for ModuleSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleSystem {
    /// Create a new module system
    pub fn new() -> Self {
        ModuleSystem {
            loader: ModuleLoader::new(),
            resolver: NameResolver::new(),
            modules: HashMap::new(),
            search_paths: Vec::new(),
        }
    }

    /// Add a search path
    pub fn add_search_path<P: AsRef<Path>>(&mut self, path: P) {
        self.search_paths.push(path.as_ref().to_path_buf());
    }

    /// Load a module
    pub fn load_module<P: AsRef<Path>>(&mut self, path: P) -> Result<Module, ModuleError> {
        let path_buf = path.as_ref().to_path_buf();
        let module_name = path_buf
            .file_stem()
            .ok_or_else(|| ModuleError::Generic("Invalid module path".to_string()))?
            .to_string_lossy()
            .to_string();

        // Check if the module is already loaded
        if let Some(module) = self.modules.get(&module_name) {
            return Ok(module.clone());
        }

        // Load the module
        let ast = self
            .loader
            .load_module(&path_buf)
            .map_err(|e| ModuleError::LoadFailure(e.to_string()))?;

        // Create a namespace for the module
        let namespace = Namespace::new(module_name.clone());

        // Create a placeholder module to handle circular dependencies
        let placeholder_module = Module {
            name: module_name.clone(),
            path: path_buf.clone(),
            ast: Program {
                imports: Vec::new(),
                definitions: Vec::new(),
                location: Location {
                    line: 0,
                    column: 0,
                    start: 0,
                    end: 0,
                },
            },
            namespace: namespace.clone(),
            imports: HashMap::new(),
            exports: HashMap::new(),
        };

        // Add the placeholder to the loaded modules
        self.modules.insert(module_name.clone(), placeholder_module);

        // Create a new module
        let mut module = Module {
            name: module_name.clone(),
            path: path_buf,
            ast,
            namespace,
            imports: HashMap::new(),
            exports: HashMap::new(),
        };

        // Process imports
        self.process_imports(&mut module)?;

        // Process definitions
        self.process_definitions(&mut module)?;

        // Update the module in the loaded modules
        self.modules.insert(module_name, module.clone());

        Ok(module)
    }

    /// Process imports in a module
    fn process_imports(&mut self, module: &mut Module) -> Result<(), ModuleError> {
        for import in &module.ast.imports {
            match import {
                Import::FromImport {
                    path,
                    names,
                    location: _,
                } => {
                    // Resolve the module path
                    let module_path = self
                        .resolve_module_path(path)
                        .ok_or_else(|| ModuleError::NotFound(path.clone()))?;

                    // Load the imported module
                    let imported_module = self.load_module(&module_path)?;

                    // Process imported names
                    for name in names {
                        let import_name = if name.name == "*" {
                            // Import all exports
                            for export_name in imported_module.exports.keys() {
                                let alias = format!("{}/{}", imported_module.name, export_name);
                                module.namespace.add_import(
                                    export_name.clone(),
                                    alias.clone(),
                                    imported_module.name.clone(),
                                )?;
                            }
                            continue;
                        } else {
                            name.name.clone()
                        };

                        // Get the symbol from the imported module
                        let _symbol =
                            imported_module.exports.get(&import_name).ok_or_else(|| {
                                ModuleError::SymbolNotFound(
                                    import_name.clone(),
                                    imported_module.name.clone(),
                                )
                            })?;

                        // Add the import to the namespace
                        let alias = if let Some(alias) = &name.alias {
                            alias.clone()
                        } else {
                            import_name.clone()
                        };

                        module.namespace.add_import(
                            import_name,
                            alias,
                            imported_module.name.clone(),
                        )?;
                    }

                    // Add the imported module
                    module
                        .imports
                        .insert(imported_module.name.clone(), imported_module);
                }
                Import::DirectImport { names, location: _ } => {
                    for name in names {
                        // Resolve the module path
                        let module_path = self
                            .resolve_module_path(name)
                            .ok_or_else(|| ModuleError::NotFound(name.clone()))?;

                        // Load the imported module
                        let imported_module = self.load_module(&module_path)?;

                        // Add all exports as imports with qualified names
                        for export_name in imported_module.exports.keys() {
                            let alias = format!("{}/{}", imported_module.name, export_name);
                            module.namespace.add_import(
                                export_name.clone(),
                                alias,
                                imported_module.name.clone(),
                            )?;
                        }

                        // Add the imported module
                        module
                            .imports
                            .insert(imported_module.name.clone(), imported_module);
                    }
                }
            }
        }

        Ok(())
    }

    /// Process definitions in a module
    fn process_definitions(&mut self, module: &mut Module) -> Result<(), ModuleError> {
        for definition in &module.ast.definitions {
            match definition {
                Definition::FunctionDef { name, .. } => {
                    // Add the function to the namespace and exports
                    module
                        .namespace
                        .add_definition(name.clone(), definition.clone())?;

                    module.exports.insert(
                        name.clone(),
                        Symbol::Function {
                            name: name.clone(),
                            definition: Box::new(definition.clone()),
                        },
                    );
                }
                Definition::TypeDef { name, .. } => {
                    // Add the type to the namespace and exports
                    module
                        .namespace
                        .add_definition(name.clone(), definition.clone())?;

                    module.exports.insert(
                        name.clone(),
                        Symbol::Type {
                            name: name.clone(),
                            definition: Box::new(definition.clone()),
                        },
                    );
                }
                Definition::ObjectDef { name, .. } => {
                    // Add the object to the namespace and exports
                    module
                        .namespace
                        .add_definition(name.clone(), definition.clone())?;

                    module.exports.insert(
                        name.clone(),
                        Symbol::Object {
                            name: name.clone(),
                            definition: Box::new(definition.clone()),
                        },
                    );
                }
                Definition::TypeAlias { name, .. } => {
                    // Add the type alias to the namespace and exports
                    module
                        .namespace
                        .add_definition(name.clone(), definition.clone())?;

                    module.exports.insert(
                        name.clone(),
                        Symbol::Type {
                            name: name.clone(),
                            definition: Box::new(definition.clone()),
                        },
                    );
                }
                Definition::Module { name, .. } => {
                    // Add the module to the namespace and exports
                    module
                        .namespace
                        .add_definition(name.clone(), definition.clone())?;

                    module.exports.insert(
                        name.clone(),
                        Symbol::Module {
                            name: name.clone(),
                            definition: Box::new(definition.clone()),
                        },
                    );
                }
            }
        }

        Ok(())
    }

    /// Resolve a module path to a file path
    fn resolve_module_path(&self, module_name: &str) -> Option<PathBuf> {
        // First, check if the module name is a direct path
        let direct_path = PathBuf::from(module_name);
        if direct_path.exists() {
            return Some(direct_path);
        }

        // Check search paths
        for search_path in &self.search_paths {
            let mut path = search_path.clone();
            path.push(format!("{}.bend", module_name));

            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Resolve names in a module
    pub fn resolve_names(&mut self, module: &mut Module) -> Result<(), ModuleError> {
        // Create a name resolver with the module's namespace
        let mut resolver = NameResolver::new();
        resolver.push_namespace(module.namespace.clone());

        // Add imported namespaces
        for (name, imported_module) in &module.imports {
            resolver.add_import(name.clone(), imported_module.namespace.clone());
        }

        // Resolve names in the module
        resolver.resolve_program(&mut module.ast)?;

        Ok(())
    }
}
