//! Network standard library module for Bend-PVM

use crate::compiler::parser::ast::*;

/// Generate AST definitions for the Network module
pub fn generate_network_ast() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location {
        line: 0,
        column: 0,
        start: 0,
        end: 0,
    };

    definitions.push(Definition::FunctionDef {
        name: "Network/get_block_number".to_string(),
        params: Vec::new(),
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_block_timestamp".to_string(),
        params: Vec::new(),
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_block_hash".to_string(),
        params: vec![Parameter {
            name: "block_number".to_string(),
            ty: Type::U24 {
                location: dummy_loc.clone(),
            },
            location: dummy_loc.clone(),
        }],
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_caller".to_string(),
        params: Vec::new(),
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_origin".to_string(),
        params: Vec::new(),
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_chain_id".to_string(),
        params: Vec::new(),
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_gas_price".to_string(),
        params: Vec::new(),
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_balance".to_string(),
        params: vec![Parameter {
            name: "address".to_string(),
            ty: Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: dummy_loc.clone(),
            },
            location: dummy_loc.clone(),
        }],
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

    definitions.push(Definition::FunctionDef {
        name: "Network/send_transaction".to_string(),
        params: vec![
            Parameter {
                name: "to".to_string(),
                ty: Type::Named {
                    name: "String".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "amount".to_string(),
                ty: Type::U24 {
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "data".to_string(),
                ty: Type::Named {
                    name: "String".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
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

    definitions.push(Definition::FunctionDef {
        name: "Network/call_contract".to_string(),
        params: vec![
            Parameter {
                name: "contract_address".to_string(),
                ty: Type::Named {
                    name: "String".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "calldata".to_string(),
                ty: Type::Named {
                    name: "String".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_contract_code".to_string(),
        params: vec![Parameter {
            name: "contract_address".to_string(),
            ty: Type::Named {
                name: "String".to_string(),
                params: Vec::new(),
                location: dummy_loc.clone(),
            },
            location: dummy_loc.clone(),
        }],
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

    definitions.push(Definition::FunctionDef {
        name: "Network/get_storage_at".to_string(),
        params: vec![
            Parameter {
                name: "contract_address".to_string(),
                ty: Type::Named {
                    name: "String".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "key".to_string(),
                ty: Type::Named {
                    name: "String".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
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

    definitions
}
