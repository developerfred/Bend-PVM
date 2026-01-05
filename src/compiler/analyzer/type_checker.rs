#![allow(clippy::only_used_in_recursion)]

use std::collections::{HashMap, HashSet};
use thiserror::Error;

use crate::compiler::analyzer::type_inference::TypeEnv;
use crate::compiler::parser::ast::*;

#[derive(Error, Debug, Clone)]
pub enum TypeError {
    #[error("Type error: {0}")]
    Generic(String),

    #[error("Undefined variable '{name}' at line {line}, column {column}")]
    UndefinedVariable {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("Type mismatch: expected {expected}, found {found} at line {line}, column {column}")]
    TypeMismatch {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },

    #[error("Undefined type '{name}' at line {line}, column {column}")]
    UndefinedType {
        name: String,
        line: usize,
        column: usize,
    },

    #[error("Undefined constructor '{name}' at line {line}, column {column}")]
    UndefinedConstructor {
        name: String,
        line: usize,
        column: usize,
    },

    #[error(
        "Incompatible types for operation: {left} {op} {right} at line {line}, column {column}"
    )]
    IncompatibleOperation {
        left: String,
        op: String,
        right: String,
        line: usize,
        column: usize,
    },
}

/// Represents a type in the type system
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Named(String, Vec<TypeInfo>),
    Function(Box<TypeInfo>, Box<TypeInfo>),
    Tuple(Vec<TypeInfo>),
    U24,
    I24,
    F24,
    Any,
    None,
    Unknown,
}

impl std::fmt::Display for TypeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeInfo::Named(name, params) => {
                if params.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}(", name)?;
                    for (i, param) in params.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", param)?;
                    }
                    write!(f, ")")
                }
            }
            TypeInfo::Function(param, result) => {
                write!(f, "{} -> {}", param, result)
            }
            TypeInfo::Tuple(elements) => {
                write!(f, "(")?;
                for (i, element) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", element)?;
                }
                write!(f, ")")
            }
            TypeInfo::U24 => write!(f, "u24"),
            TypeInfo::I24 => write!(f, "i24"),
            TypeInfo::F24 => write!(f, "f24"),
            TypeInfo::Any => write!(f, "Any"),
            TypeInfo::None => write!(f, "None"),
            TypeInfo::Unknown => write!(f, "_"),
        }
    }
}

/// Represents a symbol in the type system
#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(TypeInfo),
    Function(TypeInfo),
    Type(Vec<String>),             // Type parameters
    Constructor(String, TypeInfo), // Type name, constructor type
}

/// Environment for type checking
pub struct TypeChecker {
    /// Symbol table for variables, functions, types, and constructors
    symbols: HashMap<String, Symbol>,

    /// Type definitions
    types: HashMap<String, Vec<TypeVariant>>,

    /// Type parameters for generic types
    type_params: HashMap<String, HashSet<String>>,

    /// Check for cyclic type definitions
    visited_types: HashSet<String>,

    /// Track function return types for checking
    current_function_return_type: Option<TypeInfo>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut checker = TypeChecker {
            symbols: HashMap::new(),
            types: HashMap::new(),
            type_params: HashMap::new(),
            visited_types: HashSet::new(),
            current_function_return_type: None,
        };

        // Add built-in types and functions
        checker.add_builtin_types();

        checker
    }

    /// Add built-in types and functions to the environment
    fn add_builtin_types(&mut self) {
        // Basic types
        self.symbols.insert("u24".to_string(), Symbol::Type(vec![]));
        self.symbols.insert("i24".to_string(), Symbol::Type(vec![]));
        self.symbols.insert("f24".to_string(), Symbol::Type(vec![]));
        self.symbols.insert("Any".to_string(), Symbol::Type(vec![]));
        self.symbols
            .insert("None".to_string(), Symbol::Type(vec![]));

        // Common generic types
        self.symbols
            .insert("List".to_string(), Symbol::Type(vec!["T".to_string()]));
        self.type_params.insert(
            "List".to_string(),
            vec!["T".to_string()].into_iter().collect(),
        );

        self.symbols
            .insert("Option".to_string(), Symbol::Type(vec!["T".to_string()]));
        self.type_params.insert(
            "Option".to_string(),
            vec!["T".to_string()].into_iter().collect(),
        );

        self.symbols.insert(
            "Result".to_string(),
            Symbol::Type(vec!["T".to_string(), "E".to_string()]),
        );
        self.type_params.insert(
            "Result".to_string(),
            vec!["T".to_string(), "E".to_string()].into_iter().collect(),
        );

        self.symbols
            .insert("Tree".to_string(), Symbol::Type(vec!["T".to_string()]));
        self.type_params.insert(
            "Tree".to_string(),
            vec!["T".to_string()].into_iter().collect(),
        );

        // Some common constructors
        self.symbols.insert(
            "List/Nil".to_string(),
            Symbol::Constructor(
                "List".to_string(),
                TypeInfo::Named("List".to_string(), vec![TypeInfo::Unknown]),
            ),
        );

        self.symbols.insert(
            "List/Cons".to_string(),
            Symbol::Constructor(
                "List".to_string(),
                TypeInfo::Function(
                    Box::new(TypeInfo::Unknown), // head: T
                    Box::new(TypeInfo::Function(
                        Box::new(TypeInfo::Named("List".to_string(), vec![TypeInfo::Unknown])), // tail: List<T>
                        Box::new(TypeInfo::Named("List".to_string(), vec![TypeInfo::Unknown])), // return: List<T>
                    )),
                ),
            ),
        );

        // Add more built-in types and constructors as needed
    }

    /// Type check a program
    pub fn check_program(&mut self, program: &Program) -> Result<(), TypeError> {
        // First pass: collect all type definitions
        for definition in &program.definitions {
            match definition {
                Definition::TypeDef {
                    name,
                    type_params,
                    variants,
                    ..
                } => {
                    let params = type_params.clone();
                    self.symbols
                        .insert(name.clone(), Symbol::Type(params.clone()));
                    self.types.insert(name.clone(), variants.clone());

                    // Add type parameters
                    let param_set: HashSet<String> = params.into_iter().collect();
                    self.type_params.insert(name.clone(), param_set);

                    // Add constructors
                    for variant in variants {
                        let constructor_name = format!("{}/{}", name, variant.name);
                        let constructor_type =
                            self.variant_to_type_info(name, type_params, variant)?;
                        self.symbols.insert(
                            constructor_name,
                            Symbol::Constructor(name.clone(), constructor_type),
                        );
                    }
                }
                Definition::ObjectDef {
                    name,
                    type_params,
                    fields,
                    ..
                } => {
                    let params = type_params.clone();
                    self.symbols
                        .insert(name.clone(), Symbol::Type(params.clone()));

                    // Create a single variant for the object
                    let object_variant = TypeVariant {
                        name: name.clone(),
                        fields: fields.clone(),
                        location: definition.location().clone(),
                    };

                    self.types
                        .insert(name.clone(), vec![object_variant.clone()]);

                    // Add type parameters
                    let param_set: HashSet<String> = params.into_iter().collect();
                    self.type_params.insert(name.clone(), param_set);

                    // Add constructor
                    let constructor_name = name.clone();
                    let constructor_type =
                        self.variant_to_type_info(name, type_params, &object_variant)?;
                    self.symbols.insert(
                        constructor_name,
                        Symbol::Constructor(name.clone(), constructor_type),
                    );
                }
                _ => {}
            }
        }

        // Second pass: type check function definitions
        for definition in &program.definitions {
            if let Definition::FunctionDef {
                name,
                params,
                return_type,
                body,
                checked,
                ..
            } = definition
            {
                // Skip type checking for unchecked functions
                if let Some(false) = checked {
                    continue;
                }

                // Create a new scope for the function
                let mut checker = self.new_scope();

                // Add parameters to the scope
                let mut param_types = Vec::new();
                for param in params {
                    let param_type = checker.ast_type_to_type_info(&param.ty)?;

                    checker
                        .symbols
                        .insert(param.name.clone(), Symbol::Variable(param_type.clone()));
                    param_types.push(param_type);
                }

                // Set the current function return type
                checker.current_function_return_type = if let Some(ret_type) = return_type {
                    Some(checker.ast_type_to_type_info(ret_type)?)
                } else {
                    Some(TypeInfo::Any)
                };

                // Type check the function body
                let inferred_return_type = checker.check_block(body)?;

                // Check if the inferred return type matches the annotated return type
                if let Some(ret_type) = &checker.current_function_return_type {
                    if !checker.is_compatible(ret_type, &inferred_return_type)? {
                        return Err(TypeError::TypeMismatch {
                            expected: ret_type.to_string(),
                            found: inferred_return_type.to_string(),
                            line: body.location.line,
                            column: body.location.column,
                        });
                    }
                }

                // Construct the function type
                let function_type = if params.is_empty() {
                    inferred_return_type.clone()
                } else {
                    let mut fn_type = inferred_return_type.clone();

                    // Build the function type from right to left
                    for param_type in param_types.into_iter().rev() {
                        fn_type = TypeInfo::Function(Box::new(param_type), Box::new(fn_type));
                    }

                    fn_type
                };

                // Add the function to the symbol table
                self.symbols
                    .insert(name.clone(), Symbol::Function(function_type));
            }
        }

        Ok(())
    }

    /// Create a new scope with inherited symbols and type definitions
    fn new_scope(&self) -> TypeChecker {
        TypeChecker {
            symbols: self.symbols.clone(),
            types: self.types.clone(),
            type_params: self.type_params.clone(),
            visited_types: HashSet::new(),
            current_function_return_type: self.current_function_return_type.clone(),
        }
    }

    /// Convert a type variant to a type info
    fn variant_to_type_info(
        &self,
        type_name: &str,
        type_params: &[String],
        variant: &TypeVariant,
    ) -> Result<TypeInfo, TypeError> {
        // Create a map of type parameters to Unknown types
        let mut type_param_map = HashMap::new();
        for param in type_params {
            type_param_map.insert(param.clone(), TypeInfo::Unknown);
        }

        // Build the constructor type from right to left
        let mut constructor_type = TypeInfo::Named(
            type_name.to_string(),
            type_params.iter().map(|_| TypeInfo::Unknown).collect(),
        );

        // Add fields from right to left
        for field in variant.fields.iter().rev() {
            let field_type = if let Some(type_annotation) = &field.type_annotation {
                self.ast_type_to_type_info_with_params(type_annotation, &type_param_map)?
            } else {
                TypeInfo::Any
            };

            constructor_type = TypeInfo::Function(Box::new(field_type), Box::new(constructor_type));
        }

        Ok(constructor_type)
    }

    /// Convert AST type to TypeInfo
    fn ast_type_to_type_info(&self, ast_type: &Type) -> Result<TypeInfo, TypeError> {
        match ast_type {
            Type::Named {
                name,
                params,
                location,
            } => {
                match name.as_str() {
                    "u24" => Ok(TypeInfo::U24),
                    "i24" => Ok(TypeInfo::I24),
                    "f24" => Ok(TypeInfo::F24),
                    "Any" => Ok(TypeInfo::Any),
                    "None" => Ok(TypeInfo::None),
                    "_" => Ok(TypeInfo::Unknown),
                    _ => {
                        // Check if the type exists
                        if !self.symbols.contains_key(name) {
                            return Err(TypeError::UndefinedType {
                                name: name.clone(),
                                line: location.line,
                                column: location.column,
                            });
                        }

                        // Check if the type parameters match
                        if let Some(param_set) = self.type_params.get(name) {
                            if params.len() != param_set.len() {
                                return Err(TypeError::TypeMismatch {
                                    expected: format!(
                                        "{} with {} type parameters",
                                        name,
                                        param_set.len()
                                    ),
                                    found: format!(
                                        "{} with {} type parameters",
                                        name,
                                        params.len()
                                    ),
                                    line: location.line,
                                    column: location.column,
                                });
                            }

                            // Convert the type parameters
                            let mut param_types = Vec::new();
                            for param in params {
                                param_types.push(self.ast_type_to_type_info(param)?);
                            }

                            Ok(TypeInfo::Named(name.clone(), param_types))
                        } else {
                            // Non-generic type shouldn't have parameters
                            if !params.is_empty() {
                                return Err(TypeError::TypeMismatch {
                                    expected: format!("{} with no type parameters", name),
                                    found: format!(
                                        "{} with {} type parameters",
                                        name,
                                        params.len()
                                    ),
                                    line: location.line,
                                    column: location.column,
                                });
                            }

                            Ok(TypeInfo::Named(name.clone(), vec![]))
                        }
                    }
                }
            }
            Type::Function { param, result, .. } => {
                let param_type = self.ast_type_to_type_info(param)?;
                let result_type = self.ast_type_to_type_info(result)?;

                Ok(TypeInfo::Function(
                    Box::new(param_type),
                    Box::new(result_type),
                ))
            }
            Type::Tuple { elements, .. } => {
                let mut element_types = Vec::new();
                for element in elements {
                    element_types.push(self.ast_type_to_type_info(element)?);
                }

                Ok(TypeInfo::Tuple(element_types))
            }
            Type::Any { .. } => Ok(TypeInfo::Any),
            Type::None { .. } => Ok(TypeInfo::None),
            Type::Hole { .. } => Ok(TypeInfo::Unknown),
            Type::U24 { .. } => Ok(TypeInfo::U24),
            Type::I24 { .. } => Ok(TypeInfo::I24),
            Type::F24 { .. } => Ok(TypeInfo::F24),
            Type::Unknown { .. } => Ok(TypeInfo::Unknown),
            Type::Generic { .. } => Ok(TypeInfo::Unknown),
            Type::Constrained { .. } => Ok(TypeInfo::Unknown),
            Type::Effect { .. } => Ok(TypeInfo::Unknown),
        }
    }

    /// Convert AST type to TypeInfo with type parameters
    fn ast_type_to_type_info_with_params(
        &self,
        ast_type: &Type,
        type_param_map: &HashMap<String, TypeInfo>,
    ) -> Result<TypeInfo, TypeError> {
        match ast_type {
            Type::Named {
                name,
                params,
                location,
            } => {
                // Check if it's a type parameter
                if let Some(param_type) = type_param_map.get(name) {
                    if !params.is_empty() {
                        return Err(TypeError::TypeMismatch {
                            expected: format!("type parameter {} with no type parameters", name),
                            found: format!("{} with {} type parameters", name, params.len()),
                            line: location.line,
                            column: location.column,
                        });
                    }

                    return Ok(param_type.clone());
                }

                // Otherwise, proceed as normal
                self.ast_type_to_type_info(ast_type)
            }
            // Other cases same as ast_type_to_type_info
            _ => self.ast_type_to_type_info(ast_type),
        }
    }

    /// Type check a block
    fn check_block(&mut self, block: &Block) -> Result<TypeInfo, TypeError> {
        let mut result_type = TypeInfo::None;

        for statement in &block.statements {
            result_type = self.check_statement(statement)?;
        }

        Ok(result_type)
    }

    /// Type check a statement
    fn check_statement(&mut self, statement: &Statement) -> Result<TypeInfo, TypeError> {
        match statement {
            Statement::Return { value, .. } => {
                let value_type = self.check_expr(value)?;

                if let Some(ret_type) = &self.current_function_return_type {
                    if !self.is_compatible(ret_type, &value_type)? {
                        return Err(TypeError::TypeMismatch {
                            expected: ret_type.to_string(),
                            found: value_type.to_string(),
                            line: value.location().line,
                            column: value.location().column,
                        });
                    }
                }

                Ok(value_type)
            }
            Statement::Assignment { pattern, value, .. } => {
                let value_type = self.check_expr(value)?;
                self.check_pattern(pattern, &value_type)?;
                Ok(TypeInfo::None)
            }
            // Add type checking for other statement types
            // For brevity, we're not implementing all statement types here
            _ => Err(TypeError::Generic(
                "Type checking not implemented for this statement type yet".to_string(),
            )),
        }
    }

    /// Type check a pattern
    fn check_pattern(
        &mut self,
        pattern: &Pattern,
        expected_type: &TypeInfo,
    ) -> Result<(), TypeError> {
        match pattern {
            Pattern::Variable { name, location: _ } => {
                // Add the variable to the symbol table with the expected type
                self.symbols
                    .insert(name.clone(), Symbol::Variable(expected_type.clone()));
                Ok(())
            }
            Pattern::Tuple { elements, location } => {
                // Check if the expected type is a tuple with the same number of elements
                match expected_type {
                    TypeInfo::Tuple(element_types) => {
                        if elements.len() != element_types.len() {
                            return Err(TypeError::TypeMismatch {
                                expected: format!("tuple with {} elements", element_types.len()),
                                found: format!("tuple with {} elements", elements.len()),
                                line: location.line,
                                column: location.column,
                            });
                        }

                        // Check each element
                        for (element, element_type) in elements.iter().zip(element_types.iter()) {
                            self.check_pattern(element, element_type)?;
                        }

                        Ok(())
                    }
                    TypeInfo::Any => {
                        // Assume the tuple has the right structure
                        for element in elements {
                            self.check_pattern(element, &TypeInfo::Any)?;
                        }

                        Ok(())
                    }
                    _ => Err(TypeError::TypeMismatch {
                        expected: "tuple".to_string(),
                        found: expected_type.to_string(),
                        line: location.line,
                        column: location.column,
                    }),
                }
            }
            // Add type checking for other pattern types
            // For brevity, we're not implementing all pattern types here
            _ => Err(TypeError::Generic(
                "Type checking not implemented for this pattern type yet".to_string(),
            )),
        }
    }

    /// Type check an expression
    fn check_expr(&mut self, expr: &Expr) -> Result<TypeInfo, TypeError> {
        match expr {
            Expr::Variable { name, location } => {
                // Look up the variable in the symbol table
                if let Some(symbol) = self.symbols.get(name) {
                    match symbol {
                        Symbol::Variable(type_info) => Ok(type_info.clone()),
                        Symbol::Function(type_info) => Ok(type_info.clone()),
                        Symbol::Constructor(_, type_info) => Ok(type_info.clone()),
                        _ => Err(TypeError::UndefinedVariable {
                            name: name.clone(),
                            line: location.line,
                            column: location.column,
                        }),
                    }
                } else {
                    Err(TypeError::UndefinedVariable {
                        name: name.clone(),
                        line: location.line,
                        column: location.column,
                    })
                }
            }
            Expr::Literal { kind, location: _ } => match kind {
                LiteralKind::Uint(_) => Ok(TypeInfo::U24),
                LiteralKind::Int(_) => Ok(TypeInfo::I24),
                LiteralKind::Float(_) => Ok(TypeInfo::F24),
                LiteralKind::String(_) => Ok(TypeInfo::Named("String".to_string(), vec![])),
                LiteralKind::Char(_) => Ok(TypeInfo::U24),
                LiteralKind::Symbol(_) => Ok(TypeInfo::U24),
                LiteralKind::Bool(_) => Ok(TypeInfo::U24),
            },
            Expr::Tuple {
                elements,
                location: _,
            } => {
                let mut element_types = Vec::new();
                for element in elements {
                    element_types.push(self.check_expr(element)?);
                }

                Ok(TypeInfo::Tuple(element_types))
            }
            Expr::List {
                elements,
                location: _,
            } => {
                // Infer the element type from the first element, or use Any if empty
                let element_type = if let Some(first) = elements.first() {
                    self.check_expr(first)?
                } else {
                    TypeInfo::Any
                };

                // Check that all elements have the same type
                for element in elements {
                    let current_type = self.check_expr(element)?;
                    if !self.is_compatible(&element_type, &current_type)? {
                        return Err(TypeError::TypeMismatch {
                            expected: element_type.to_string(),
                            found: current_type.to_string(),
                            line: element.location().line,
                            column: element.location().column,
                        });
                    }
                }

                Ok(TypeInfo::Named("List".to_string(), vec![element_type]))
            }
            Expr::FunctionCall {
                function,
                args,
                named_args: _,
                location,
            } => {
                let function_type = self.check_expr(function)?;

                // Check if the function type is a function
                match function_type {
                    TypeInfo::Function(param_type, result_type) => {
                        // Check if the argument matches the parameter type
                        if args.len() != 1 {
                            return Err(TypeError::TypeMismatch {
                                expected: "function with 1 argument".to_string(),
                                found: format!("function with {} arguments", args.len()),
                                line: location.line,
                                column: location.column,
                            });
                        }

                        let arg_type = self.check_expr(&args[0])?;
                        if !self.is_compatible(&param_type, &arg_type)? {
                            return Err(TypeError::TypeMismatch {
                                expected: param_type.to_string(),
                                found: arg_type.to_string(),
                                line: args[0].location().line,
                                column: args[0].location().column,
                            });
                        }

                        Ok(*result_type)
                    }
                    TypeInfo::Any => {
                        // Any can be called with any arguments
                        Ok(TypeInfo::Any)
                    }
                    _ => Err(TypeError::TypeMismatch {
                        expected: "function".to_string(),
                        found: function_type.to_string(),
                        line: function.location().line,
                        column: function.location().column,
                    }),
                }
            }
            Expr::BinaryOp {
                left,
                operator,
                right,
                location,
            } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;

                // Check operator compatibility
                match operator {
                    BinaryOperator::Add
                    | BinaryOperator::Sub
                    | BinaryOperator::Mul
                    | BinaryOperator::Div
                    | BinaryOperator::Mod => {
                        // Numeric operations
                        if !self.is_numeric(&left_type)? || !self.is_numeric(&right_type)? {
                            return Err(TypeError::IncompatibleOperation {
                                left: left_type.to_string(),
                                op: operator.to_string(),
                                right: right_type.to_string(),
                                line: location.line,
                                column: location.column,
                            });
                        }

                        // Check numeric type compatibility
                        if !self.is_compatible(&left_type, &right_type)? {
                            return Err(TypeError::TypeMismatch {
                                expected: left_type.to_string(),
                                found: right_type.to_string(),
                                line: right.location().line,
                                column: right.location().column,
                            });
                        }

                        // Result has the same type as the operands
                        Ok(left_type)
                    }
                    BinaryOperator::Equal
                    | BinaryOperator::NotEqual
                    | BinaryOperator::Less
                    | BinaryOperator::LessEqual
                    | BinaryOperator::Greater
                    | BinaryOperator::GreaterEqual => {
                        // Comparison operations
                        if !self.is_compatible(&left_type, &right_type)? {
                            return Err(TypeError::TypeMismatch {
                                expected: left_type.to_string(),
                                found: right_type.to_string(),
                                line: right.location().line,
                                column: right.location().column,
                            });
                        }

                        // Result is a u24 (boolean)
                        Ok(TypeInfo::U24)
                    }
                    BinaryOperator::BitAnd | BinaryOperator::BitOr | BinaryOperator::BitXor => {
                        // Bitwise operations
                        if !self.is_integral(&left_type)? || !self.is_integral(&right_type)? {
                            return Err(TypeError::IncompatibleOperation {
                                left: left_type.to_string(),
                                op: operator.to_string(),
                                right: right_type.to_string(),
                                line: location.line,
                                column: location.column,
                            });
                        }

                        // Result has the same type as the operands
                        Ok(left_type)
                    }
                    BinaryOperator::Pow => {
                        // Exponentiation (only for f24)
                        if left_type != TypeInfo::F24 || right_type != TypeInfo::F24 {
                            return Err(TypeError::IncompatibleOperation {
                                left: left_type.to_string(),
                                op: "**".to_string(),
                                right: right_type.to_string(),
                                line: location.line,
                                column: location.column,
                            });
                        }

                        Ok(TypeInfo::F24)
                    }
                    BinaryOperator::BitShiftLeft | BinaryOperator::BitShiftRight => {
                        // Shift operations (only for integral types)
                        if !self.is_integral(&left_type)? || !self.is_integral(&right_type)? {
                            return Err(TypeError::IncompatibleOperation {
                                left: left_type.to_string(),
                                op: operator.to_string(),
                                right: right_type.to_string(),
                                line: location.line,
                                column: location.column,
                            });
                        }

                        // Result has the same type as the left operand
                        Ok(left_type)
                    }
                }
            }
            // Add type checking for other expression types
            // For brevity, we're not implementing all expression types here
            _ => Err(TypeError::Generic("Type checking not implemented for this expression type yet".to_string())),
        }
    }

    /// Check if a type is numeric (u24, i24, f24)
    fn is_numeric(&self, type_info: &TypeInfo) -> Result<bool, TypeError> {
        Ok(matches!(
            type_info,
            TypeInfo::U24 | TypeInfo::I24 | TypeInfo::F24 | TypeInfo::Any
        ))
    }

    /// Check if a type is integral (u24, i24)
    fn is_integral(&self, type_info: &TypeInfo) -> Result<bool, TypeError> {
        Ok(matches!(
            type_info,
            TypeInfo::U24 | TypeInfo::I24 | TypeInfo::Any
        ))
    }

    /// Check if one type is compatible with another
    fn is_compatible(&self, expected: &TypeInfo, actual: &TypeInfo) -> Result<bool, TypeError> {
        match (expected, actual) {
            (TypeInfo::Any, _) | (_, TypeInfo::Any) => Ok(true),
            (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => Ok(true),
            (TypeInfo::U24, TypeInfo::U24) => Ok(true),
            (TypeInfo::I24, TypeInfo::I24) => Ok(true),
            (TypeInfo::F24, TypeInfo::F24) => Ok(true),
            (TypeInfo::None, TypeInfo::None) => Ok(true),
            (TypeInfo::Tuple(expected_elements), TypeInfo::Tuple(actual_elements)) => {
                if expected_elements.len() != actual_elements.len() {
                    return Ok(false);
                }

                for (expected_element, actual_element) in
                    expected_elements.iter().zip(actual_elements.iter())
                {
                    if !self.is_compatible(expected_element, actual_element)? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            (
                TypeInfo::Function(expected_param, expected_result),
                TypeInfo::Function(actual_param, actual_result),
            ) => Ok(self.is_compatible(expected_param, actual_param)?
                && self.is_compatible(expected_result, actual_result)?),
            (
                TypeInfo::Named(expected_name, expected_params),
                TypeInfo::Named(actual_name, actual_params),
            ) => {
                if expected_name != actual_name {
                    return Ok(false);
                }

                if expected_params.len() != actual_params.len() {
                    return Ok(false);
                }

                for (expected_param, actual_param) in
                    expected_params.iter().zip(actual_params.iter())
                {
                    if !self.is_compatible(expected_param, actual_param)? {
                        return Ok(false);
                    }
                }

                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

// Helper trait for getting string representation of binary operators
impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperator::Add => write!(f, "+"),
            BinaryOperator::Sub => write!(f, "-"),
            BinaryOperator::Mul => write!(f, "*"),
            BinaryOperator::Div => write!(f, "/"),
            BinaryOperator::Mod => write!(f, "%"),
            BinaryOperator::Pow => write!(f, "**"),
            BinaryOperator::BitAnd => write!(f, "&"),
            BinaryOperator::BitOr => write!(f, "|"),
            BinaryOperator::BitXor => write!(f, "^"),
            BinaryOperator::Equal => write!(f, "=="),
            BinaryOperator::NotEqual => write!(f, "!="),
            BinaryOperator::Less => write!(f, "<"),
            BinaryOperator::LessEqual => write!(f, "<="),
            BinaryOperator::Greater => write!(f, ">"),
            BinaryOperator::GreaterEqual => write!(f, ">="),
            BinaryOperator::BitShiftLeft => write!(f, "<<"),
            BinaryOperator::BitShiftRight => write!(f, ">>"),
        }
    }
}
