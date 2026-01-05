use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::compiler::module::ModuleError;
use crate::compiler::parser::ast::*;
use crate::compiler::parser::parser::Parser;

/// Module loader for loading modules from files
pub struct ModuleLoader {
    /// Set of modules currently being loaded (to detect circular dependencies)
    loading: HashSet<PathBuf>,

    /// Set of loaded modules
    loaded: HashSet<PathBuf>,
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl ModuleLoader {
    /// Create a new module loader
    pub fn new() -> Self {
        ModuleLoader {
            loading: HashSet::new(),
            loaded: HashSet::new(),
        }
    }

    /// Load a module from a file
    pub fn load_module<P: AsRef<Path>>(&mut self, path: P) -> Result<Program, ModuleError> {
        let path_buf = path.as_ref().to_path_buf();

        // Check if the module is already loaded
        if self.loaded.contains(&path_buf) {
            // Read and parse the file again
            // In a real implementation, we might want to cache the parsed AST
            return self.parse_file(&path_buf);
        }

        // Check for circular dependencies
        if self.loading.contains(&path_buf) {
            return Err(ModuleError::CircularDependency(
                path_buf.to_string_lossy().to_string(),
            ));
        }

        // Mark the module as being loaded
        self.loading.insert(path_buf.clone());

        // Load the module
        let result = self.parse_file(&path_buf);

        // Mark the module as loaded
        self.loading.remove(&path_buf);
        self.loaded.insert(path_buf);

        result
    }

    /// Parse a file into a program
    fn parse_file(&self, path: &PathBuf) -> Result<Program, ModuleError> {
        // Read the file
        let source = fs::read_to_string(path).map_err(|e| ModuleError::IO(e.to_string()))?;

        // Parse the source
        let mut parser = Parser::new(&source);
        let program = parser
            .parse_program()
            .map_err(|e| ModuleError::Generic(e.to_string()))?;

        Ok(program)
    }
}

/// Extension trait for Parser to load modules
trait ParserExt {
    /// Load a module from a file
    fn load_module<P: AsRef<Path>>(&mut self, path: P) -> Result<Program, ModuleError>;
}

impl ParserExt for Parser<'_> {
    fn load_module<P: AsRef<Path>>(&mut self, path: P) -> Result<Program, ModuleError> {
        // Create a module loader
        let mut loader = ModuleLoader::new();

        // Load the module
        loader.load_module(path)
    }
}
