//! DateTime standard library module for Bend-PVM

use crate::compiler::parser::ast::*;

/// Generate AST definitions for the DateTime module
pub fn generate_datetime_ast() -> Vec<Definition> {
    let mut definitions = Vec::new();
    let dummy_loc = Location {
        line: 0,
        column: 0,
        start: 0,
        end: 0,
    };

    definitions.push(Definition::FunctionDef {
        name: "DateTime/now".to_string(),
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
        name: "DateTime/from_timestamp".to_string(),
        params: vec![Parameter {
            name: "timestamp".to_string(),
            ty: Type::U24 {
                location: dummy_loc.clone(),
            },
            location: dummy_loc.clone(),
        }],
        return_type: Some(Type::Named {
            name: "DateTime".to_string(),
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
        name: "DateTime/year".to_string(),
        params: vec![Parameter {
            name: "dt".to_string(),
            ty: Type::Named {
                name: "DateTime".to_string(),
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
        name: "DateTime/month".to_string(),
        params: vec![Parameter {
            name: "dt".to_string(),
            ty: Type::Named {
                name: "DateTime".to_string(),
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
        name: "DateTime/day".to_string(),
        params: vec![Parameter {
            name: "dt".to_string(),
            ty: Type::Named {
                name: "DateTime".to_string(),
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
        name: "DateTime/hour".to_string(),
        params: vec![Parameter {
            name: "dt".to_string(),
            ty: Type::Named {
                name: "DateTime".to_string(),
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
        name: "DateTime/minute".to_string(),
        params: vec![Parameter {
            name: "dt".to_string(),
            ty: Type::Named {
                name: "DateTime".to_string(),
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
        name: "DateTime/second".to_string(),
        params: vec![Parameter {
            name: "dt".to_string(),
            ty: Type::Named {
                name: "DateTime".to_string(),
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
        name: "DateTime/format".to_string(),
        params: vec![
            Parameter {
                name: "dt".to_string(),
                ty: Type::Named {
                    name: "DateTime".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "format".to_string(),
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
        name: "DateTime/add_days".to_string(),
        params: vec![
            Parameter {
                name: "dt".to_string(),
                ty: Type::Named {
                    name: "DateTime".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "days".to_string(),
                ty: Type::U24 {
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(Type::Named {
            name: "DateTime".to_string(),
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
        name: "DateTime/subtract_days".to_string(),
        params: vec![
            Parameter {
                name: "dt".to_string(),
                ty: Type::Named {
                    name: "DateTime".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "days".to_string(),
                ty: Type::U24 {
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
        ],
        return_type: Some(Type::Named {
            name: "DateTime".to_string(),
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
        name: "DateTime/days_between".to_string(),
        params: vec![
            Parameter {
                name: "dt1".to_string(),
                ty: Type::Named {
                    name: "DateTime".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
                location: dummy_loc.clone(),
            },
            Parameter {
                name: "dt2".to_string(),
                ty: Type::Named {
                    name: "DateTime".to_string(),
                    params: Vec::new(),
                    location: dummy_loc.clone(),
                },
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

    definitions
}
