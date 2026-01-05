use std::collections::{BTreeSet, HashMap};
use thiserror::Error;

use crate::compiler::parser::ast::*;

/// Type inference and checking errors
#[derive(Error, Debug, Clone)]
pub enum TypeError {
    #[error("Type error: {0}")]
    Generic(String),

    #[error("Undefined variable '{name}'")]
    UndefinedVariable { name: String },

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },
}

/// Represents a type in the type system
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InferType {
    Named(String, Vec<InferType>),
    Function(Box<InferType>, Box<InferType>),
    Tuple(Vec<InferType>),
    U24,
    I24,
    F24,
    Any,
    None,
    Variable(String),
    Generic { name: String, bounds: Vec<String> },
}

/// A type schema (generic type)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeSchema {
    pub type_vars: BTreeSet<String>,
    pub type_: InferType,
}

/// Represents a symbol in the symbol table
#[derive(Debug, Clone)]
pub enum Symbol {
    Variable(InferType),
    Function(InferType),
    Type(TypeSchema),
    Constructor { type_name: String, type_: InferType },
    Module(InferType),
}

/// Type environment for inference
#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub symbols: HashMap<String, Symbol>,
    pub type_defs: HashMap<String, TypeDefInfo>,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = TypeEnv {
            symbols: HashMap::new(),
            type_defs: HashMap::new(),
        };
        env.add_builtin_types();
        env
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    fn add_builtin_types(&mut self) {
        self.symbols.insert(
            "u24".to_string(),
            Symbol::Type(TypeSchema {
                type_vars: BTreeSet::new(),
                type_: InferType::U24,
            }),
        );
        self.symbols.insert(
            "i24".to_string(),
            Symbol::Type(TypeSchema {
                type_vars: BTreeSet::new(),
                type_: InferType::I24,
            }),
        );
        self.symbols.insert(
            "f24".to_string(),
            Symbol::Type(TypeSchema {
                type_vars: BTreeSet::new(),
                type_: InferType::F24,
            }),
        );
        self.symbols.insert(
            "String".to_string(),
            Symbol::Type(TypeSchema {
                type_vars: BTreeSet::new(),
                type_: InferType::Named("String".to_string(), vec![]),
            }),
        );
        self.symbols.insert(
            "Bool".to_string(),
            Symbol::Type(TypeSchema {
                type_vars: BTreeSet::new(),
                type_: InferType::Named("Bool".to_string(), vec![]),
            }),
        );

        // Option type
        self.type_defs.insert(
            "Option".to_string(),
            TypeDefInfo {
                name: "Option".to_string(),
                type_params: vec!["T".to_string()],
                variants: vec![
                    VariantInfo {
                        name: "None".to_string(),
                        fields: vec![],
                    },
                    VariantInfo {
                        name: "Some".to_string(),
                        fields: vec![InferType::Variable("T".to_string())],
                    },
                ],
            },
        );

        // Result type
        self.type_defs.insert(
            "Result".to_string(),
            TypeDefInfo {
                name: "Result".to_string(),
                type_params: vec!["T".to_string(), "E".to_string()],
                variants: vec![
                    VariantInfo {
                        name: "Ok".to_string(),
                        fields: vec![InferType::Variable("T".to_string())],
                    },
                    VariantInfo {
                        name: "Err".to_string(),
                        fields: vec![InferType::Variable("E".to_string())],
                    },
                ],
            },
        );

        // List type
        self.type_defs.insert(
            "List".to_string(),
            TypeDefInfo {
                name: "List".to_string(),
                type_params: vec!["T".to_string()],
                variants: vec![
                    VariantInfo {
                        name: "Nil".to_string(),
                        fields: vec![],
                    },
                    VariantInfo {
                        name: "Cons".to_string(),
                        fields: vec![
                            InferType::Variable("T".to_string()),
                            InferType::Named(
                                "List".to_string(),
                                vec![InferType::Variable("T".to_string())],
                            ),
                        ],
                    },
                ],
            },
        );
    }
}

#[derive(Debug, Clone)]
pub struct TypeDefInfo {
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<VariantInfo>,
}

#[derive(Debug, Clone)]
pub struct VariantInfo {
    pub name: String,
    pub fields: Vec<InferType>,
}

/// Constraint solver for type inference
#[derive(Debug, Clone, Default)]
pub struct ConstraintSolver {
    pub substitutions: HashMap<String, InferType>,
}

impl ConstraintSolver {
    pub fn unify(&mut self, t1: &InferType, t2: &InferType) -> Result<(), TypeError> {
        let t1 = self.apply_subst(t1);
        let t2 = self.apply_subst(t2);

        match (&t1, &t2) {
            (InferType::Variable(name), _) if !name.starts_with('_') => {
                if let InferType::Variable(name2) = &t2 {
                    if name == name2 {
                        return Ok(());
                    }
                }
                self.substitutions.insert(name.clone(), t2);
                Ok(())
            }
            (_, InferType::Variable(name)) if !name.starts_with('_') => {
                self.substitutions.insert(name.clone(), t1);
                Ok(())
            }
            (InferType::Named(n1, p1), InferType::Named(n2, p2)) if n1 == n2 => {
                for (a, b) in p1.iter().zip(p2.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            }
            (InferType::Function(p1, r1), InferType::Function(p2, r2)) => {
                self.unify(p1.as_ref(), p2.as_ref())?;
                self.unify(r1.as_ref(), r2.as_ref())?;
                Ok(())
            }
            (InferType::Tuple(e1), InferType::Tuple(e2)) if e1.len() == e2.len() => {
                for (a, b) in e1.iter().zip(e2.iter()) {
                    self.unify(a, b)?;
                }
                Ok(())
            }
            _ if t1 == t2 => Ok(()),
            _ => Err(TypeError::TypeMismatch {
                expected: t1.to_string(),
                found: t2.to_string(),
            }),
        }
    }

    pub fn apply_subst(&self, t: &InferType) -> InferType {
        match t {
            InferType::Variable(name) => self
                .substitutions
                .get(name)
                .cloned()
                .unwrap_or_else(|| t.clone()),
            InferType::Named(name, params) => InferType::Named(
                name.clone(),
                params.iter().map(|p| self.apply_subst(p)).collect(),
            ),
            InferType::Function(param, result) => InferType::Function(
                Box::new(self.apply_subst(param)),
                Box::new(self.apply_subst(result)),
            ),
            InferType::Tuple(elements) => {
                InferType::Tuple(elements.iter().map(|e| self.apply_subst(e)).collect())
            }
            _ => t.clone(),
        }
    }
}

impl InferType {
    pub fn fresh_var(prefix: &str) -> Self {
        static mut COUNTER: usize = 0;
        let id = unsafe {
            COUNTER += 1;
            COUNTER
        };
        InferType::Variable(format!("{}_{}", prefix, id))
    }

    pub fn free_vars(&self) -> BTreeSet<String> {
        let mut vars = BTreeSet::new();
        self.collect_free_vars(&mut vars);
        vars
    }

    fn collect_free_vars(&self, vars: &mut BTreeSet<String>) {
        match self {
            InferType::Named(_, params) => params.iter().for_each(|p| p.collect_free_vars(vars)),
            InferType::Function(param, result) => {
                param.collect_free_vars(vars);
                result.collect_free_vars(vars);
            }
            InferType::Tuple(elements) => elements.iter().for_each(|e| e.collect_free_vars(vars)),
            InferType::Variable(name) => {
                vars.insert(name.clone());
            }
            _ => {}
        }
    }
}

impl std::fmt::Display for InferType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InferType::Named(name, params) => {
                if params.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}(", name)?;
                    for (i, p) in params.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", p)?;
                    }
                    write!(f, ")")
                }
            }
            InferType::Function(param, result) => write!(f, "{} -> {}", param, result),
            InferType::Tuple(elements) => {
                write!(f, "(")?;
                for (i, e) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", e)?;
                }
                write!(f, ")")
            }
            InferType::U24 => write!(f, "u24"),
            InferType::I24 => write!(f, "i24"),
            InferType::F24 => write!(f, "f24"),
            InferType::Any => write!(f, "Any"),
            InferType::None => write!(f, "None"),
            InferType::Variable(name) => write!(f, "{}", name),
            InferType::Generic { name, bounds } => {
                if bounds.is_empty() {
                    write!(f, "{}", name)
                } else {
                    write!(f, "{}: {:?}", name, bounds)
                }
            }
        }
    }
}

impl Default for InferType {
    fn default() -> Self {
        InferType::Variable("_".to_string())
    }
}

pub struct TypeInferrer {
    pub env: TypeEnv,
    pub solver: ConstraintSolver,
    pub var_counter: usize,
}

impl TypeInferrer {
    pub fn new() -> Self {
        TypeInferrer {
            env: TypeEnv::new(),
            solver: ConstraintSolver::default(),
            var_counter: 0,
        }
    }

    fn fresh_var(&mut self, prefix: &str) -> InferType {
        self.var_counter += 1;
        InferType::Variable(format!("{}_{}", prefix, self.var_counter))
    }

    pub fn check_program(&mut self, program: &Program) -> Result<InferType, TypeError> {
        for def in &program.definitions {
            self.check_definition(def)?;
        }
        Ok(InferType::None)
    }

    fn check_definition(&mut self, def: &Definition) -> Result<InferType, TypeError> {
        match def {
            Definition::FunctionDef {
                name,
                params,
                return_type,
                body,
                ..
            } => {
                let mut param_types = Vec::new();
                for param in params {
                    let param_type = self.infer_from_ast_type(&param.ty)?;
                    param_types.push(param_type.clone());
                    self.env
                        .symbols
                        .insert(param.name.clone(), Symbol::Variable(param_type));
                }

                let return_type = return_type
                    .as_ref()
                    .map(|rt| self.infer_from_ast_type(rt))
                    .unwrap_or(Ok(InferType::None))?;

                let fn_type = param_types
                    .into_iter()
                    .rev()
                    .fold(return_type.clone(), |acc, param| {
                        InferType::Function(Box::new(param), Box::new(acc))
                    });

                self.env
                    .symbols
                    .insert(name.clone(), Symbol::Function(fn_type));
                let body_type = self.check_block(body)?;
                self.solver.unify(&body_type, &return_type)?;
                Ok(InferType::None)
            }
            Definition::TypeDef {
                name, type_params, ..
            } => {
                let schema = TypeSchema {
                    type_vars: type_params.iter().cloned().collect(),
                    type_: InferType::Named(name.clone(), vec![]),
                };
                self.env.symbols.insert(name.clone(), Symbol::Type(schema));
                Ok(InferType::None)
            }
            Definition::ObjectDef {
                name, type_params, ..
            } => {
                let schema = TypeSchema {
                    type_vars: type_params.iter().cloned().collect(),
                    type_: InferType::Named(name.clone(), vec![]),
                };
                self.env.symbols.insert(name.clone(), Symbol::Type(schema));
                Ok(InferType::None)
            }
            Definition::TypeAlias {
                name, target_type, ..
            } => {
                let target = self.infer_from_ast_type(target_type)?;
                let schema = TypeSchema {
                    type_vars: BTreeSet::new(),
                    type_: target,
                };
                self.env.symbols.insert(name.clone(), Symbol::Type(schema));
                Ok(InferType::None)
            }
            Definition::Module { name, .. } => {
                self.env.symbols.insert(
                    name.clone(),
                    Symbol::Module(InferType::Named(name.clone(), vec![])),
                );
                Ok(InferType::None)
            }
        }
    }

    fn check_block(&mut self, block: &Block) -> Result<InferType, TypeError> {
        let mut result = InferType::None;
        for stmt in &block.statements {
            result = self.check_statement(stmt)?;
        }
        Ok(result)
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<InferType, TypeError> {
        match stmt {
            Statement::Return { value, .. } => self.check_expr(value),
            Statement::Assignment { pattern, value, .. } => {
                let value_type = self.check_expr(value)?;
                self.check_pattern(pattern, &value_type)?;
                Ok(InferType::None)
            }
            Statement::Use { name, value, .. } => {
                let value_type = self.check_expr(value)?;
                self.env
                    .symbols
                    .insert(name.clone(), Symbol::Variable(value_type.clone()));
                Ok(value_type)
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let _cond_type = self.check_expr(condition)?;
                let then_type = self.check_block(then_branch)?;
                let else_type = self.check_block(else_branch)?;
                self.solver.unify(&then_type, &else_type)?;
                Ok(then_type)
            }
            Statement::Match { value, cases, .. } => {
                let _value_type = self.check_expr(value)?;
                let mut result_type = None;
                for case in cases {
                    let case_type = self.check_block(&case.body)?;
                    if let Some(rt) = result_type.clone() {
                        self.solver.unify(&rt, &case_type)?;
                    } else {
                        result_type = Some(case_type);
                    }
                }
                Ok(result_type.unwrap_or(InferType::None))
            }
            Statement::Expr { expr, .. } => self.check_expr(expr),
            Statement::LocalDef { function_def, .. } => self.check_definition(function_def),
            _ => Ok(InferType::None),
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<InferType, TypeError> {
        match expr {
            Expr::Variable { name, .. } => {
                if let Some(symbol) = self.env.symbols.get(name) {
                    match symbol {
                        Symbol::Variable(t) | Symbol::Function(t) => Ok(self.solver.apply_subst(t)),
                        Symbol::Constructor { type_, .. } => Ok(self.solver.apply_subst(type_)),
                        Symbol::Type(schema) => Ok(self.solver.apply_subst(&schema.type_)),
                        Symbol::Module(t) => Ok(self.solver.apply_subst(t)),
                    }
                } else {
                    Err(TypeError::UndefinedVariable { name: name.clone() })
                }
            }
            Expr::Literal { kind, .. } => match kind {
                LiteralKind::Uint(_) => Ok(InferType::U24),
                LiteralKind::Int(_) => Ok(InferType::I24),
                LiteralKind::Float(_) => Ok(InferType::F24),
                LiteralKind::String(_) => Ok(InferType::Named("String".to_string(), vec![])),
                LiteralKind::Char(_) => Ok(InferType::U24),
                LiteralKind::Symbol(_) => Ok(InferType::U24),
                LiteralKind::Bool(_) => Ok(InferType::U24),
            },
            Expr::Tuple { elements, .. } => {
                let types: Result<Vec<_>, _> =
                    elements.iter().map(|e| self.check_expr(e)).collect();
                Ok(InferType::Tuple(types?))
            }
            Expr::List { elements, .. } => {
                let element_type = if let Some(first) = elements.first() {
                    self.check_expr(first)?
                } else {
                    InferType::Any
                };
                Ok(InferType::Named("List".to_string(), vec![element_type]))
            }
            Expr::Constructor { name, args, .. } => {
                let type_name = self
                    .env
                    .type_defs
                    .values()
                    .find(|d| d.variants.iter().any(|v| v.name == *name))
                    .map(|d| d.name.clone());

                if let Some(type_name) = type_name {
                    let _arg_types: Result<Vec<_>, _> =
                        args.iter().map(|e| self.check_expr(e)).collect();
                    Ok(InferType::Named(type_name, vec![]))
                } else {
                    Err(TypeError::Generic(format!("Unknown constructor: {}", name)))
                }
            }
            Expr::FunctionCall { function, args, .. } => {
                let fn_type = self.check_expr(function)?;
                match fn_type {
                    InferType::Function(param_type, result_type) => {
                        if !args.is_empty() {
                            let arg_type = self.check_expr(&args[0])?;
                            self.solver.unify(&param_type, &arg_type)?;
                        }
                        Ok(*result_type)
                    }
                    InferType::Any => Ok(InferType::Any),
                    _ => Err(TypeError::Generic("Cannot call non-function".to_string())),
                }
            }
            Expr::BinaryOp { left, right, .. } => {
                let left_type = self.check_expr(left)?;
                let right_type = self.check_expr(right)?;
                self.solver.unify(&left_type, &right_type)?;
                Ok(left_type)
            }
            Expr::Lambda { params, body, .. } => {
                let mut param_types = Vec::new();
                for param in params {
                    let param_type = self.infer_from_ast_type(&param.ty)?;
                    param_types.push(param_type.clone());
                    self.env
                        .symbols
                        .insert(param.name.clone(), Symbol::Variable(param_type));
                }
                let body_type = self.check_expr(body)?;
                let fn_type = param_types.into_iter().rev().fold(body_type, |acc, param| {
                    InferType::Function(Box::new(param), Box::new(acc))
                });
                Ok(fn_type)
            }
            Expr::Block { block, .. } => self.check_block(block),
            _ => Ok(InferType::Any),
        }
    }

    fn check_pattern(
        &mut self,
        pattern: &Pattern,
        expected_type: &InferType,
    ) -> Result<(), TypeError> {
        match pattern {
            Pattern::Variable { name, .. } => {
                self.env
                    .symbols
                    .insert(name.clone(), Symbol::Variable(expected_type.clone()));
                Ok(())
            }
            Pattern::Tuple { elements, .. } => {
                if let InferType::Tuple(element_types) = expected_type {
                    if elements.len() == element_types.len() {
                        for (element, element_type) in elements.iter().zip(element_types.iter()) {
                            self.check_pattern(element, element_type)?;
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn infer_from_ast_type(&self, ast_type: &Type) -> Result<InferType, TypeError> {
        match ast_type {
            Type::Named { name, params, .. } => {
                let param_types = params
                    .iter()
                    .map(|p| self.infer_from_ast_type(p))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(InferType::Named(name.clone(), param_types))
            }
            Type::Function { param, result, .. } => {
                let param_type = self.infer_from_ast_type(param)?;
                let result_type = self.infer_from_ast_type(result)?;
                Ok(InferType::Function(
                    Box::new(param_type),
                    Box::new(result_type),
                ))
            }
            Type::Tuple { elements, .. } => {
                let element_types = elements
                    .iter()
                    .map(|e| self.infer_from_ast_type(e))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(InferType::Tuple(element_types))
            }
            Type::U24 { .. } => Ok(InferType::U24),
            Type::I24 { .. } => Ok(InferType::I24),
            Type::F24 { .. } => Ok(InferType::F24),
            Type::Any { .. } => Ok(InferType::Any),
            Type::None { .. } => Ok(InferType::None),
            Type::Hole { .. } => Ok(InferType::Variable("_".to_string())),
            Type::Unknown { .. } => Ok(InferType::Variable("_".to_string())),
            Type::Generic { name, bounds, .. } => Ok(InferType::Generic {
                name: name.clone(),
                bounds: bounds.iter().map(|b| b.trait_name.clone()).collect(),
            }),
            Type::Constrained { bounds, .. } => Ok(InferType::Generic {
                name: "unknown".to_string(),
                bounds: bounds.iter().map(|b| b.trait_name.clone()).collect(),
            }),
            Type::Effect { input, output, .. } => {
                let input_type = self.infer_from_ast_type(input)?;
                let output_type = self.infer_from_ast_type(output)?;
                Ok(InferType::Function(
                    Box::new(input_type),
                    Box::new(output_type),
                ))
            }
        }
    }
}

impl Default for TypeInferrer {
    fn default() -> Self {
        Self::new()
    }
}

pub fn type_check_program(program: &Program) -> Result<InferType, TypeError> {
    let mut inferrer = TypeInferrer::new();
    inferrer.check_program(program)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_inference_basic() {
        let mut inferrer = TypeInferrer::new();

        let program = Program {
            imports: vec![],
            definitions: vec![Definition::FunctionDef {
                name: "test".to_string(),
                params: vec![],
                return_type: Some(Type::U24 {
                    location: Location::default(),
                }),
                body: Block {
                    statements: vec![Statement::Expr {
                        expr: Expr::Literal {
                            kind: LiteralKind::Uint(42),
                            location: Location::default(),
                        },
                        location: Location::default(),
                    }],
                    location: Location::default(),
                },
                checked: Some(true),
                location: Location::default(),
            }],
            location: Location::default(),
        };

        let result = inferrer.check_program(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_builtin_types() {
        let env = TypeEnv::new();
        assert!(env.lookup("u24").is_some());
        assert!(env.lookup("String").is_some());
    }

    #[test]
    fn test_type_display() {
        let t = InferType::Named("Option".to_string(), vec![InferType::U24]);
        assert_eq!(t.to_string(), "Option(u24)");
    }
}
