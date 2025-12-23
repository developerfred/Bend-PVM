use std::collections::HashMap;
use crate::compiler::parser::ast::*;
use crate::compiler::module::ModuleError;

/// Standard library core module
pub struct StdlibCore {
    /// Available standard library modules
    modules: HashMap<String, StdlibModule>,
}

/// Standard library module
pub struct StdlibModule {
    /// Module name
    pub name: String,
    
    /// Module definitions
    pub definitions: Vec<Definition>,
}

impl StdlibCore {
    /// Create a new standard library core
    pub fn new() -> Self {
        let mut core = StdlibCore {
            modules: HashMap::new(),
        };
        
        // Register standard library modules
        core.register_module(StdlibModule::string());
        core.register_module(StdlibModule::math());
        core.register_module(StdlibModule::io());
        core.register_module(StdlibModule::list());
        core.register_module(StdlibModule::option());
        core.register_module(StdlibModule::result());
        core.register_module(StdlibModule::crypto());
        
        core
    }
    
    /// Register a standard library module
    fn register_module(&mut self, module: StdlibModule) {
        self.modules.insert(module.name.clone(), module);
    }
    
    /// Get a standard library module by name
    pub fn get_module(&self, name: &str) -> Option<&StdlibModule> {
        self.modules.get(name)
    }
    
    /// Get a list of available standard library modules
    pub fn available_modules(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }
    
    /// Load a standard library module as a program
    pub fn load_module(&self, name: &str) -> Result<Program, ModuleError> {
        let module = self.get_module(name)
            .ok_or_else(|| ModuleError::NotFound(name.to_string()))?;
        
        Ok(Program {
            imports: Vec::new(),
            definitions: module.definitions.clone(),
            location: Location {
                line: 0,
                column: 0,
                start: 0,
                end: 0,
            },
        })
    }
}

impl StdlibModule {
    /// Create the String module
    pub fn string() -> Self {
        let mut definitions = Vec::new();
        let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
        
        // String concatenation
        definitions.push(Definition::FunctionDef {
            name: "String/concat".to_string(),
            params: vec![
                Parameter {
                    name: "a".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: dummy_loc.clone(),
                    }),
                    location: dummy_loc.clone(),
                },
                Parameter {
                    name: "b".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: dummy_loc.clone(),
                    }),
                    location: dummy_loc.clone(),
                },
            ],
            return_type: Some(Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: dummy_loc.clone(),
            }),
            body: Block {
                statements: Vec::new(),
                location: dummy_loc.clone(),
            },
            checked: Some(true),
            location: dummy_loc.clone(),
        });
        
        // String length
        definitions.push(Definition::FunctionDef {
            name: "String/length".to_string(),
            params: vec![
                Parameter {
                    name: "s".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: dummy_loc.clone(),
                    }),
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
        
        StdlibModule {
            name: "String".to_string(),
            definitions,
        }
    }
    
    /// Create the Math module
    pub fn math() -> Self {
        let mut definitions = Vec::new();
        let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };

        // Math/PI constant
        definitions.push(Definition::FunctionDef {
            name: "Math/PI".to_string(),
            params: Vec::new(),
            return_type: Some(Type::F24 {
                location: dummy_loc.clone(),
            }),
            body: Block {
                statements: vec![
                    Statement::Return {
                        value: Expr::Literal {
                            kind: LiteralKind::Float(std::f32::consts::PI),
                            location: dummy_loc.clone(),
                        },
                        location: dummy_loc.clone(),
                    },
                ],
                location: dummy_loc.clone(),
            },
            checked: Some(true),
            location: dummy_loc.clone(),
        });
        
        // Math/sin
        definitions.push(Definition::FunctionDef {
            name: "Math/sin".to_string(),
            params: vec![
                Parameter {
                    name: "x".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: dummy_loc.clone(),
                    }),
                    location: dummy_loc.clone(),
                },
            ],
            return_type: Some(Type::F24 {
                location: dummy_loc.clone(),
            }),
            body: Block {
                statements: Vec::new(),
                location: dummy_loc.clone(),
            },
            checked: Some(true),
            location: dummy_loc.clone(),
        });

        StdlibModule {
            name: "Math".to_string(),
            definitions,
        }
    }
    
    /// Create the IO module for blockchain operations
    pub fn io() -> Self {
        let mut definitions = Vec::new();
        let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
        
        // IO/storage_get
        definitions.push(Definition::FunctionDef {
            name: "IO/storage_get".to_string(),
            params: vec![
                Parameter {
                    name: "key".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: dummy_loc.clone(),
                    }),
                    location: dummy_loc.clone(),
                },
            ],
            return_type: Some(Type::Named {
                name: "Result".to_string(),
                params: vec![
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: dummy_loc.clone(),
                    },
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: dummy_loc.clone(),
                    },
                ],
                location: dummy_loc.clone(),
            }),
            body: Block {
                statements: Vec::new(),
                location: dummy_loc.clone(),
            },
            checked: Some(true),
            location: dummy_loc.clone(),
        });
        
        StdlibModule {
            name: "IO".to_string(),
            definitions,
        }
    }
    
    /// Create the List module
    pub fn list() -> Self {
        let mut definitions = Vec::new();
        let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
        
        // List type definition
        definitions.push(Definition::TypeDef {
            name: "List".to_string(),
            type_params: vec!["T".to_string()],
            variants: vec![
                TypeVariant {
                    name: "Nil".to_string(),
                    fields: Vec::new(),
                    location: dummy_loc.clone(),
                },
                TypeVariant {
                    name: "Cons".to_string(),
                    fields: vec![
                        Field {
                            name: "head".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "T".to_string(),
                                params: Vec::new(),
                                location: dummy_loc.clone(),
                            }),
                            is_recursive: false,
                            location: dummy_loc.clone(),
                        },
                        Field {
                            name: "tail".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "List".to_string(),
                                params: vec![Type::Named {
                                    name: "T".to_string(),
                                    params: Vec::new(),
                                    location: dummy_loc.clone(),
                                }],
                                location: dummy_loc.clone(),
                            }),
                            is_recursive: true,
                            location: dummy_loc.clone(),
                        },
                    ],
                    location: dummy_loc.clone(),
                },
            ],
            location: dummy_loc.clone(),
        });
        
        StdlibModule {
            name: "List".to_string(),
            definitions,
        }
    }
    
    /// Create the Option module
    pub fn option() -> Self {
        let mut definitions = Vec::new();
        let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
        
        // Option type definition
        definitions.push(Definition::TypeDef {
            name: "Option".to_string(),
            type_params: vec!["T".to_string()],
            variants: vec![
                TypeVariant {
                    name: "None".to_string(),
                    fields: Vec::new(),
                    location: dummy_loc.clone(),
                },
                TypeVariant {
                    name: "Some".to_string(),
                    fields: vec![
                        Field {
                            name: "value".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "T".to_string(),
                                params: Vec::new(),
                                location: dummy_loc.clone(),
                            }),
                            is_recursive: false,
                            location: dummy_loc.clone(),
                        },
                    ],
                    location: dummy_loc.clone(),
                },
            ],
            location: dummy_loc.clone(),
        });
        
        StdlibModule {
            name: "Option".to_string(),
            definitions,
        }
    }
    
    /// Create the Result module
    pub fn result() -> Self {
        let mut definitions = Vec::new();
        let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
        
        // Result type definition
        definitions.push(Definition::TypeDef {
            name: "Result".to_string(),
            type_params: vec!["T".to_string(), "E".to_string()],
            variants: vec![
                TypeVariant {
                    name: "Ok".to_string(),
                    fields: vec![
                        Field {
                            name: "value".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "T".to_string(),
                                params: Vec::new(),
                                location: dummy_loc.clone(),
                            }),
                            is_recursive: false,
                            location: dummy_loc.clone(),
                        },
                    ],
                    location: dummy_loc.clone(),
                },
                TypeVariant {
                    name: "Err".to_string(),
                    fields: vec![
                        Field {
                            name: "error".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "E".to_string(),
                                params: Vec::new(),
                                location: dummy_loc.clone(),
                            }),
                            is_recursive: false,
                            location: dummy_loc.clone(),
                        },
                    ],
                    location: dummy_loc.clone(),
                },
            ],
            location: dummy_loc.clone(),
        });
        
        StdlibModule {
            name: "Result".to_string(),
            definitions,
        }
    }
    
    /// Create the Crypto module
    pub fn crypto() -> Self {
        let mut definitions = Vec::new();
        let dummy_loc = Location { line: 0, column: 0, start: 0, end: 0 };
        
        // Crypto/keccak256
        definitions.push(Definition::FunctionDef {
            name: "Crypto/keccak256".to_string(),
            params: vec![
                Parameter {
                    name: "data".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: dummy_loc.clone(),
                    }),
                    location: dummy_loc.clone(),
                },
            ],
            return_type: Some(Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: dummy_loc.clone(),
            }),
            body: Block {
                statements: Vec::new(),
                location: dummy_loc.clone(),
            },
            checked: Some(true),
            location: dummy_loc.clone(),
        });
        
        StdlibModule {
            name: "Crypto".to_string(),
            definitions,
        }
    }
}
