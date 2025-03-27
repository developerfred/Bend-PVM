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
        
        // String concatenation
        definitions.push(Definition::FunctionDef {
            name: "String/concat".to_string(),
            params: vec![
                Parameter {
                    name: "a".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "b".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
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
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::U24 {
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // String slice
        definitions.push(Definition::FunctionDef {
            name: "String/slice".to_string(),
            params: vec![
                Parameter {
                    name: "s".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "start".to_string(),
                    type_annotation: Some(Type::U24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "end".to_string(),
                    type_annotation: Some(Type::U24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // String/to_bytes
        definitions.push(Definition::FunctionDef {
            name: "String/to_bytes".to_string(),
            params: vec![
                Parameter {
                    name: "s".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "List".to_string(),
                params: vec![Type::U24 {
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                }],
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // String/from_bytes
        definitions.push(Definition::FunctionDef {
            name: "String/from_bytes".to_string(),
            params: vec![
                Parameter {
                    name: "bytes".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "List".to_string(),
                        params: vec![Type::U24 {
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }],
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        StdlibModule {
            name: "String".to_string(),
            definitions,
        }
    }
    
    /// Create the Math module
    pub fn math() -> Self {
        let mut definitions = Vec::new();
        
        // Math/min
        definitions.push(Definition::FunctionDef {
            name: "Math/min".to_string(),
            params: vec![
                Parameter {
                    name: "a".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "b".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::F24 {
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // Math/max
        definitions.push(Definition::FunctionDef {
            name: "Math/max".to_string(),
            params: vec![
                Parameter {
                    name: "a".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "b".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::F24 {
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // Math/abs
        definitions.push(Definition::FunctionDef {
            name: "Math/abs".to_string(),
            params: vec![
                Parameter {
                    name: "x".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::F24 {
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // Math/sqrt
        definitions.push(Definition::FunctionDef {
            name: "Math/sqrt".to_string(),
            params: vec![
                Parameter {
                    name: "x".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::F24 {
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // Math/pow
        definitions.push(Definition::FunctionDef {
            name: "Math/pow".to_string(),
            params: vec![
                Parameter {
                    name: "x".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "y".to_string(),
                    type_annotation: Some(Type::F24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::F24 {
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        StdlibModule {
            name: "Math".to_string(),
            definitions,
        }
    }
    
    /// Create the IO module for blockchain operations
    pub fn io() -> Self {
        let mut definitions = Vec::new();
        
        // IO/storage_get
        definitions.push(Definition::FunctionDef {
            name: "IO/storage_get".to_string(),
            params: vec![
                Parameter {
                    name: "key".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "Result".to_string(),
                params: vec![
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                ],
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // IO/storage_set
        definitions.push(Definition::FunctionDef {
            name: "IO/storage_set".to_string(),
            params: vec![
                Parameter {
                    name: "key".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "value".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "Result".to_string(),
                params: vec![
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                ],
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // IO/emit_event
        definitions.push(Definition::FunctionDef {
            name: "IO/emit_event".to_string(),
            params: vec![
                Parameter {
                    name: "name".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "data".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "List".to_string(),
                        params: vec![Type::Named {
                            name: "String".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }],
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "Result".to_string(),
                params: vec![
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                ],
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // IO/call
        definitions.push(Definition::FunctionDef {
            name: "IO/call".to_string(),
            params: vec![
                Parameter {
                    name: "address".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "value".to_string(),
                    type_annotation: Some(Type::U24 {
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "data".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "Result".to_string(),
                params: vec![
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                    Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                ],
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        StdlibModule {
            name: "IO".to_string(),
            definitions,
        }
    }
    
    /// Create the List module
    pub fn list() -> Self {
        let mut definitions = Vec::new();
        
        // List type definition
        definitions.push(Definition::TypeDef {
            name: "List".to_string(),
            type_params: vec!["T".to_string()],
            variants: vec![
                TypeVariant {
                    name: "Nil".to_string(),
                    fields: Vec::new(),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                TypeVariant {
                    name: "Cons".to_string(),
                    fields: vec![
                        Field {
                            name: "head".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "T".to_string(),
                                params: Vec::new(),
                                location: Location { line: 0, column: 0, start: 0, end: 0 },
                            }),
                            is_recursive: false,
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        },
                        Field {
                            name: "tail".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "List".to_string(),
                                params: vec![Type::Named {
                                    name: "T".to_string(),
                                    params: Vec::new(),
                                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                                }],
                                location: Location { line: 0, column: 0, start: 0, end: 0 },
                            }),
                            is_recursive: true,
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        },
                    ],
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // List/length
        definitions.push(Definition::FunctionDef {
            name: "List/length".to_string(),
            params: vec![
                Parameter {
                    name: "list".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "List".to_string(),
                        params: vec![Type::Named {
                            name: "T".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }],
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::U24 {
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // List/map
        definitions.push(Definition::FunctionDef {
            name: "List/map".to_string(),
            params: vec![
                Parameter {
                    name: "list".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "List".to_string(),
                        params: vec![Type::Named {
                            name: "T".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }],
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "f".to_string(),
                    type_annotation: Some(Type::Function {
                        param: Box::new(Type::Named {
                            name: "T".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }),
                        result: Box::new(Type::Named {
                            name: "U".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "List".to_string(),
                params: vec![Type::Named {
                    name: "U".to_string(),
                    params: Vec::new(),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                }],
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        StdlibModule {
            name: "List".to_string(),
            definitions,
        }
    }
    
    /// Create the Option module
    pub fn option() -> Self {
        let mut definitions = Vec::new();
        
        // Option type definition
        definitions.push(Definition::TypeDef {
            name: "Option".to_string(),
            type_params: vec!["T".to_string()],
            variants: vec![
                TypeVariant {
                    name: "None".to_string(),
                    fields: Vec::new(),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                TypeVariant {
                    name: "Some".to_string(),
                    fields: vec![
                        Field {
                            name: "value".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "T".to_string(),
                                params: Vec::new(),
                                location: Location { line: 0, column: 0, start: 0, end: 0 },
                            }),
                            is_recursive: false,
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        },
                    ],
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // Option/map
        definitions.push(Definition::FunctionDef {
            name: "Option/map".to_string(),
            params: vec![
                Parameter {
                    name: "opt".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "Option".to_string(),
                        params: vec![Type::Named {
                            name: "T".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }],
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "f".to_string(),
                    type_annotation: Some(Type::Function {
                        param: Box::new(Type::Named {
                            name: "T".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }),
                        result: Box::new(Type::Named {
                            name: "U".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "Option".to_string(),
                params: vec![Type::Named {
                    name: "U".to_string(),
                    params: Vec::new(),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                }],
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        StdlibModule {
            name: "Option".to_string(),
            definitions,
        }
    }
    
    /// Create the Result module
    pub fn result() -> Self {
        let mut definitions = Vec::new();
        
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
                                location: Location { line: 0, column: 0, start: 0, end: 0 },
                            }),
                            is_recursive: false,
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        },
                    ],
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                TypeVariant {
                    name: "Err".to_string(),
                    fields: vec![
                        Field {
                            name: "error".to_string(),
                            type_annotation: Some(Type::Named {
                                name: "E".to_string(),
                                params: Vec::new(),
                                location: Location { line: 0, column: 0, start: 0, end: 0 },
                            }),
                            is_recursive: false,
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        },
                    ],
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // Result/map
        definitions.push(Definition::FunctionDef {
            name: "Result/map".to_string(),
            params: vec![
                Parameter {
                    name: "result".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "Result".to_string(),
                        params: vec![
                            Type::Named {
                                name: "T".to_string(),
                                params: Vec::new(),
                                location: Location { line: 0, column: 0, start: 0, end: 0 },
                            },
                            Type::Named {
                                name: "E".to_string(),
                                params: Vec::new(),
                                location: Location { line: 0, column: 0, start: 0, end: 0 },
                            },
                        ],
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "f".to_string(),
                    type_annotation: Some(Type::Function {
                        param: Box::new(Type::Named {
                            name: "T".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }),
                        result: Box::new(Type::Named {
                            name: "U".to_string(),
                            params: Vec::new(),
                            location: Location { line: 0, column: 0, start: 0, end: 0 },
                        }),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "Result".to_string(),
                params: vec![
                    Type::Named {
                        name: "U".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                    Type::Named {
                        name: "E".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    },
                ],
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        StdlibModule {
            name: "Result".to_string(),
            definitions,
        }
    }
    
    /// Create the Crypto module
    pub fn crypto() -> Self {
        let mut definitions = Vec::new();
        
        // Crypto/keccak256
        definitions.push(Definition::FunctionDef {
            name: "Crypto/keccak256".to_string(),
            params: vec![
                Parameter {
                    name: "data".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // Crypto/sha256
        definitions.push(Definition::FunctionDef {
            name: "Crypto/sha256".to_string(),
            params: vec![
                Parameter {
                    name: "data".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        // Crypto/verify_signature
        definitions.push(Definition::FunctionDef {
            name: "Crypto/verify_signature".to_string(),
            params: vec![
                Parameter {
                    name: "message".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "signature".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
                Parameter {
                    name: "public_key".to_string(),
                    type_annotation: Some(Type::Named {
                        name: "String".to_string(),
                        params: Vec::new(),
                        location: Location { line: 0, column: 0, start: 0, end: 0 },
                    }),
                    location: Location { line: 0, column: 0, start: 0, end: 0 },
                },
            ],
            return_type: Some(Type::U24 {
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            }),
            body: Block {
                statements: Vec::new(), // Built-in, no body needed
                location: Location { line: 0, column: 0, start: 0, end: 0 },
            },
            checked: Some(true),
            location: Location { line: 0, column: 0, start: 0, end: 0 },
        });
        
        StdlibModule {
            name: "Crypto".to_string(),
            definitions,
        }
    }
}