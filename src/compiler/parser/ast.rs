use std::collections::HashMap;

/// Represents a source location for AST nodes
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

/// Represents a complete Bend-PVM program
#[derive(Debug, Clone)]
pub struct Program {
    pub imports: Vec<Import>,
    pub definitions: Vec<Definition>,
    pub location: Location,
}

/// Represents an import statement
#[derive(Debug, Clone)]
pub enum Import {
    FromImport {
        path: String,
        names: Vec<ImportName>,
        location: Location,
    },
    DirectImport {
        names: Vec<String>,
        location: Location,
    },
}

/// Represents an imported name, optionally aliased
#[derive(Debug, Clone)]
pub struct ImportName {
    pub name: String,
    pub alias: Option<String>,
    pub location: Location,
}

/// Represents a top-level definition
#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    FunctionDef {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: Block,
        checked: Option<bool>, // None = default, Some(true) = checked, Some(false) = unchecked
        location: Location,
    },
    TypeDef {
        name: String,
        type_params: Vec<String>,
        variants: Vec<TypeVariant>,
        location: Location,
    },
    ObjectDef {
        name: String,
        type_params: Vec<String>,
        fields: Vec<Field>,
        location: Location,
    },
    TypeAlias {
        name: String,
        type_params: Vec<String>,
        target_type: Type,
        location: Location,
    },
    Module {
        name: String,
        definitions: Vec<Definition>,
        exports: Vec<String>,
        location: Location,
    },
}

/// Represents a parameter in a function definition
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
    pub location: Location,
}

/// Represents a function or constructor parameter field
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub is_recursive: bool, // Marked with ~
    pub location: Location,
}

/// Represents a variant in a type definition
#[derive(Debug, Clone, PartialEq)]
pub struct TypeVariant {
    pub name: String,
    pub fields: Vec<Field>,
    pub location: Location,
}

/// Represents a type in the Bend-PVM language
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named {
        name: String,
        params: Vec<Type>,
        location: Location,
    },
    Function {
        param: Box<Type>,
        result: Box<Type>,
        location: Location,
    },
    Tuple {
        elements: Vec<Type>,
        location: Location,
    },
    Any {
        location: Location,
    },
    None {
        location: Location,
    },
    Hole {
        location: Location,
    },
    U24 {
        location: Location,
    },
    I24 {
        location: Location,
    },
    F24 {
        location: Location,
    },
    Unknown {
        location: Location,
    },
    Generic {
        name: String,
        bounds: Vec<TypeBound>,
        location: Location,
    },
    Constrained {
        base: Box<Type>,
        bounds: Vec<TypeBound>,
        location: Location,
    },
    Effect {
        input: Box<Type>,
        output: Box<Type>,
        location: Location,
    },
}

/// Represents a type bound for generics
#[derive(Debug, Clone, PartialEq)]
pub struct TypeBound {
    pub trait_name: String,
    pub args: Vec<Type>,
    location: Location,
}

/// Represents a block of statements
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub location: Location,
}

/// Represents a statement in the Bend-PVM language
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Assignment {
        pattern: Pattern,
        value: Expr,
        location: Location,
    },
    Use {
        name: String,
        value: Expr,
        location: Location,
    },
    InPlaceOp {
        target: String,
        operator: InPlaceOperator,
        value: Expr,
        location: Location,
    },
    Return {
        value: Expr,
        location: Location,
    },
    If {
        condition: Expr,
        then_branch: Block,
        else_branch: Block,
        location: Location,
    },
    Switch {
        value: Expr,
        cases: Vec<SwitchCase>,
        location: Location,
    },
    Match {
        value: Expr,
        cases: Vec<MatchCase>,
        location: Location,
    },
    Fold {
        value: Expr,
        cases: Vec<MatchCase>,
        location: Location,
    },
    Bend {
        initial_states: Vec<(String, Expr)>,
        condition: Expr,
        body: Block,
        else_body: Option<Block>,
        location: Location,
    },
    Open {
        type_name: String,
        value: Expr,
        location: Location,
    },
    With {
        monad_type: String,
        body: Block,
        location: Location,
    },
    LocalDef {
        function_def: Box<Definition>,
        location: Location,
    },
    Expr {
        expr: Expr,
        location: Location,
    },
}

/// Represents an in-place operation like +=, -=, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum InPlaceOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Map,
}

/// Represents a switch case
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub value: Option<u32>, // None means default case (_)
    pub body: Block,
    pub location: Location,
}

/// Represents a match case
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    pub pattern: Pattern,
    pub body: Block,
    pub location: Location,
}

/// Represents a pattern in pattern matching
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Variable {
        name: String,
        location: Location,
    },
    Tuple {
        elements: Vec<Pattern>,
        location: Location,
    },
    Constructor {
        name: String,
        fields: HashMap<String, Pattern>, // For object/type constructors
        location: Location,
    },
    Literal {
        value: Expr, // Only for literals
        location: Location,
    },
    Wildcard {
        location: Location,
    },
}

/// Represents an expression in the Bend-PVM language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Variable {
        name: String,
        location: Location,
    },
    Literal {
        kind: LiteralKind,
        location: Location,
    },
    Tuple {
        elements: Vec<Expr>,
        location: Location,
    },
    List {
        elements: Vec<Expr>,
        location: Location,
    },
    Constructor {
        name: String,
        args: Vec<Expr>,
        named_args: HashMap<String, Expr>,
        location: Location,
    },
    FunctionCall {
        function: Box<Expr>,
        args: Vec<Expr>,
        named_args: HashMap<String, Expr>,
        location: Location,
    },
    Lambda {
        params: Vec<Parameter>,
        body: Box<Expr>,
        location: Location,
    },
    UnsccopedLambda {
        params: Vec<String>,
        body: Box<Expr>,
        location: Location,
    },
    BinaryOp {
        left: Box<Expr>,
        operator: BinaryOperator,
        right: Box<Expr>,
        location: Location,
    },
    Superposition {
        elements: Vec<Expr>,
        location: Location,
    },
    MapAccess {
        map: Box<Expr>,
        key: Box<Expr>,
        location: Location,
    },
    TreeLeaf {
        value: Box<Expr>,
        location: Location,
    },
    TreeNode {
        left: Box<Expr>,
        right: Box<Expr>,
        location: Location,
    },
    Block {
        block: Block,
        location: Location,
    },
    Eraser {
        location: Location,
    },
}

/// Represents a literal value
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    Uint(u32),  // For u24
    Int(i32),   // For i24
    Float(f32), // For f24
    String(String),
    Char(char),
    Symbol(String),
}

/// Represents a binary operator
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    BitAnd,
    BitOr,
    BitXor,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BitShiftLeft,
    BitShiftRight,
}

/// Helper trait to get the location of an AST node
pub trait LocationProvider {
    fn location(&self) -> &Location;
}

impl LocationProvider for Definition {
    fn location(&self) -> &Location {
        match self {
            Definition::FunctionDef { location, .. } => location,
            Definition::TypeDef { location, .. } => location,
            Definition::ObjectDef { location, .. } => location,
            Definition::TypeAlias { location, .. } => location,
            Definition::Module { location, .. } => location,
        }
    }
}

impl LocationProvider for Expr {
    fn location(&self) -> &Location {
        match self {
            Expr::Variable { location, .. } => location,
            Expr::Literal { location, .. } => location,
            Expr::Tuple { location, .. } => location,
            Expr::List { location, .. } => location,
            Expr::Constructor { location, .. } => location,
            Expr::FunctionCall { location, .. } => location,
            Expr::Lambda { location, .. } => location,
            Expr::UnsccopedLambda { location, .. } => location,
            Expr::BinaryOp { location, .. } => location,
            Expr::Superposition { location, .. } => location,
            Expr::MapAccess { location, .. } => location,
            Expr::TreeLeaf { location, .. } => location,
            Expr::TreeNode { location, .. } => location,
            Expr::Block { location, .. } => location,
            Expr::Eraser { location } => location,
        }
    }
}

impl LocationProvider for Statement {
    fn location(&self) -> &Location {
        match self {
            Statement::Assignment { location, .. } => location,
            Statement::Use { location, .. } => location,
            Statement::InPlaceOp { location, .. } => location,
            Statement::Return { location, .. } => location,
            Statement::If { location, .. } => location,
            Statement::Switch { location, .. } => location,
            Statement::Match { location, .. } => location,
            Statement::Fold { location, .. } => location,
            Statement::Bend { location, .. } => location,
            Statement::Open { location, .. } => location,
            Statement::With { location, .. } => location,
            Statement::LocalDef { location, .. } => location,
            Statement::Expr { location, .. } => location,
        }
    }
}

// Implement LocationProvider for Box<T> where T implements LocationProvider
impl<T: LocationProvider> LocationProvider for Box<T> {
    fn location(&self) -> &Location {
        self.as_ref().location()
    }
}

// Implement LocationProvider for &T where T implements LocationProvider
impl<T: LocationProvider> LocationProvider for &T {
    fn location(&self) -> &Location {
        (*self).location()
    }
}

// Implement LocationProvider for std::rc::Rc<T> where T implements LocationProvider
impl<T: LocationProvider> LocationProvider for std::rc::Rc<T> {
    fn location(&self) -> &Location {
        self.as_ref().location()
    }
}

/// AST visitor trait for traversing the AST
pub trait AstVisitor {
    type Output;
    
    fn visit_program(&mut self, program: &Program) -> Self::Output;
    fn visit_definition(&mut self, definition: &Definition) -> Self::Output;
    fn visit_statement(&mut self, statement: &Statement) -> Self::Output;
    fn visit_expression(&mut self, expression: &Expr) -> Self::Output;
    fn visit_pattern(&mut self, pattern: &Pattern) -> Self::Output;
    fn visit_type(&mut self, type_: &Type) -> Self::Output;
}

/// Basic AST visitor implementation
pub struct BasicAstVisitor;

impl BasicAstVisitor {
    pub fn new() -> Self {
        BasicAstVisitor
    }
}

impl AstVisitor for BasicAstVisitor {
    type Output = ();
    
    fn visit_program(&mut self, _program: &Program) {}
    fn visit_definition(&mut self, _definition: &Definition) {}
    fn visit_statement(&mut self, _statement: &Statement) {}
    fn visit_expression(&mut self, _expression: &Expr) {}
    fn visit_pattern(&mut self, _pattern: &Pattern) {}
    fn visit_type(&mut self, _type_: &Type) {}
}

/// AST validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum AstValidationError {
    DuplicateDefinition { name: String, location: Location },
    UndefinedVariable { name: String, location: Location },
    TypeMismatch { expected: Type, found: Type, location: Location },
    PatternMismatch { pattern: Pattern, value: Expr, location: Location },
    InvalidRecursion { location: Location },
    DuplicateField { name: String, location: Location },
    MissingField { name: String, location: Location },
}

/// AST validator
pub struct AstValidator;

impl AstValidator {
    pub fn new() -> Self {
        AstValidator
    }

    pub fn validate(&self, program: &Program) -> Result<(), Vec<AstValidationError>> {
        let mut errors = Vec::new();
        self.validate_program(program, &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_program(&self, program: &Program, errors: &mut Vec<AstValidationError>) {
        let mut definitions = std::collections::HashSet::new();
        
        for def in &program.definitions {
            self.validate_definition(def, &mut definitions, errors);
        }
    }

    fn validate_definition(&self, def: &Definition, _definitions: &mut std::collections::HashSet<String>, errors: &mut Vec<AstValidationError>) {
        match def {
            Definition::FunctionDef { name, body, location, .. } => {
                self.validate_block(body, errors);
            }
            Definition::TypeDef { variants, location, .. } => {
                let mut variant_names = std::collections::HashSet::new();
                for variant in variants {
                    if !variant_names.insert(variant.name.clone()) {
                        errors.push(AstValidationError::DuplicateDefinition {
                            name: variant.name.clone(),
                            location: variant.location.clone(),
                        });
                    }
                }
            }
            Definition::ObjectDef { fields, location, .. } => {
                let mut field_names = std::collections::HashSet::new();
                for field in fields {
                    if !field_names.insert(field.name.clone()) {
                        errors.push(AstValidationError::DuplicateField {
                            name: field.name.clone(),
                            location: field.location.clone(),
                        });
                    }
                }
            }
            Definition::TypeAlias { target_type, location, .. } => {
                self.validate_type(target_type, errors);
            }
            Definition::Module { definitions, location, .. } => {
                let mut def_names = std::collections::HashSet::new();
                for def in definitions {
                    let def_name = match def {
                        Definition::FunctionDef { name, .. } => name.clone(),
                        Definition::TypeDef { name, .. } => name.clone(),
                        Definition::ObjectDef { name, .. } => name.clone(),
                        Definition::TypeAlias { name, .. } => name.clone(),
                        Definition::Module { .. } => continue,
                    };
                    if !def_names.insert(def_name.clone()) {
                        errors.push(AstValidationError::DuplicateDefinition {
                            name: def_name,
                            location: def.location().clone(),
                        });
                    }
                }
            }
        }
    }

    fn validate_block(&self, block: &Block, errors: &mut Vec<AstValidationError>) {
        for stmt in &block.statements {
            self.validate_statement(stmt, errors);
        }
    }

    fn validate_statement(&self, stmt: &Statement, errors: &mut Vec<AstValidationError>) {
        match stmt {
            Statement::Assignment { pattern: _, value, location: _ } => {
                self.validate_expression(value, errors);
            }
            Statement::Use { value, location: _, .. } => {
                self.validate_expression(value, errors);
            }
            Statement::Return { value, location: _ } => {
                self.validate_expression(value, errors);
            }
            Statement::If { condition, then_branch, else_branch, location: _ } => {
                self.validate_expression(condition, errors);
                self.validate_block(then_branch, errors);
                self.validate_block(else_branch, errors);
            }
            Statement::Match { value, cases, location: _ } => {
                self.validate_expression(value, errors);
                for case in cases {
                    self.validate_block(&case.body, errors);
                }
            }
            Statement::Bend { initial_states, condition, body, else_body, location: _ } => {
                for (_, expr) in initial_states {
                    self.validate_expression(expr, errors);
                }
                self.validate_expression(condition, errors);
                self.validate_block(body, errors);
                if let Some(else_b) = else_body {
                    self.validate_block(else_b, errors);
                }
            }
            Statement::With { body, location: _, .. } => {
                self.validate_block(body, errors);
            }
            Statement::LocalDef { function_def, location: _ } => {
                self.validate_definition(function_def, &mut std::collections::HashSet::new(), errors);
            }
            Statement::Expr { expr, location: _ } => {
                self.validate_expression(expr, errors);
            }
            _ => {}
        }
    }

    fn validate_expression(&self, expr: &Expr, errors: &mut Vec<AstValidationError>) {
        match expr {
            Expr::FunctionCall { function, args, location: _, .. } => {
                self.validate_expression(function, errors);
                for arg in args {
                    self.validate_expression(arg, errors);
                }
            }
            Expr::BinaryOp { left, right, location: _, .. } => {
                self.validate_expression(left, errors);
                self.validate_expression(right, errors);
            }
            Expr::Lambda { body, location: _, .. } => {
                self.validate_expression(body, errors);
            }
            _ => {}
        }
    }

    fn validate_type(&self, type_: &Type, errors: &mut Vec<AstValidationError>) {
        match type_ {
            Type::Named { params, location: _, .. } => {
                for param in params {
                    self.validate_type(param, errors);
                }
            }
            Type::Function { param, result, location: _ } => {
                self.validate_type(param, errors);
                self.validate_type(result, errors);
            }
            Type::Tuple { elements, location: _ } => {
                for elem in elements {
                    self.validate_type(elem, errors);
                }
            }
            _ => {}
        }
    }
}
}

/// AST Visitor trait for traversing the AST
pub trait AstVisitor<T> {
    fn visit_program(&mut self, program: &Program) -> T;
    fn visit_definition(&mut self, definition: &Definition) -> T;
    fn visit_statement(&mut self, statement: &Statement) -> T;
    fn visit_expression(&mut self, expression: &Expr) -> T;
    fn visit_pattern(&mut self, pattern: &Pattern) -> T;
    fn visit_type(&mut self, type_: &Type) -> T;
}

/// Default AST visitor implementation
pub struct DefaultAstVisitor;

impl DefaultAstVisitor {
    pub fn new() -> Self {
        DefaultAstVisitor
    }
}

impl<T> AstVisitor<T> for DefaultAstVisitor
where
    T: Default + Clone,
{
    fn visit_program(&mut self, program: &Program) -> T {
        let mut result = T::default();
        for import in &program.imports {
            self.visit_import(import, &mut result);
        }
        for definition in &program.definitions {
            result = self.visit_definition(definition);
        }
        result
    }

    fn visit_definition(&mut self, definition: &Definition) -> T {
        match definition {
            Definition::FunctionDef {
                name,
                params,
                return_type,
                body,
                checked,
                location,
            } => self.visit_function_def(
                name,
                params,
                return_type.as_ref(),
                body,
                *checked,
                location,
            ),
            Definition::TypeDef {
                name,
                type_params,
                variants,
                location,
            } => self.visit_type_def(name, type_params, variants, location),
            Definition::ObjectDef {
                name,
                type_params,
                fields,
                location,
            } => self.visit_object_def(name, type_params, fields, location),
            Definition::TypeAlias {
                name,
                type_params,
                target_type,
                location,
            } => self.visit_type_alias(name, type_params, target_type, location),
            Definition::Module {
                name,
                definitions,
                exports,
                location,
            } => self.visit_module(name, definitions, exports, location),
        }
    }

    fn visit_function_def(
        &mut self,
        _name: &str,
        _params: &[Parameter],
        _return_type: Option<&Type>,
        _body: &Block,
        _checked: Option<bool>,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_type_def(
        &mut self,
        _name: &str,
        _type_params: &[String],
        _variants: &[TypeVariant],
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_object_def(
        &mut self,
        _name: &str,
        _type_params: &[String],
        _fields: &[Field],
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_type_alias(
        &mut self,
        _name: &str,
        _type_params: &[String],
        _target_type: &Type,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_module(
        &mut self,
        _name: &str,
        _definitions: &[Definition],
        _exports: &[String],
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_import(&mut self, _import: &Import, _result: &mut T) {}

    fn visit_statement(&mut self, statement: &Statement) -> T {
        match statement {
            Statement::Assignment {
                pattern,
                value,
                location,
            } => self.visit_assignment(pattern, value, location),
            Statement::Use {
                name,
                value,
                location,
            } => self.visit_use(name, value, location),
            Statement::InPlaceOp {
                target,
                operator,
                value,
                location,
            } => self.visit_inplace_op(target, operator, value, location),
            Statement::Return { value, location } => self.visit_return(value, location),
            Statement::If {
                condition,
                then_branch,
                else_branch,
                location,
            } => self.visit_if(condition, then_branch, else_branch, location),
            Statement::Switch {
                value,
                cases,
                location,
            } => self.visit_switch(value, cases, location),
            Statement::Match {
                value,
                cases,
                location,
            } => self.visit_match(value, cases, location),
            Statement::Fold {
                value,
                cases,
                location,
            } => self.visit_fold(value, cases, location),
            Statement::Bend {
                initial_states,
                condition,
                body,
                else_body,
                location,
            } => self.visit_bend(
                initial_states,
                condition,
                body,
                else_body.as_ref(),
                location,
            ),
            Statement::Open {
                type_name,
                value,
                location,
            } => self.visit_open(type_name, value, location),
            Statement::With {
                monad_type,
                body,
                location,
            } => self.visit_with(monad_type, body, location),
            Statement::LocalDef {
                function_def,
                location,
            } => self.visit_local_def(function_def, location),
            Statement::Expr { expr, location } => self.visit_statement_expr(expr, location),
        }
    }

    fn visit_assignment(&mut self, _pattern: &Pattern, _value: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_use(&mut self, _name: &str, _value: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_inplace_op(
        &mut self,
        _target: &str,
        _operator: &InPlaceOperator,
        _value: &Expr,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_return(&mut self, _value: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_if(
        &mut self,
        _condition: &Expr,
        _then_branch: &Block,
        _else_branch: &Block,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_switch(&mut self, _value: &Expr, _cases: &[SwitchCase], _location: &Location) -> T {
        T::default()
    }

    fn visit_match(&mut self, _value: &Expr, _cases: &[MatchCase], _location: &Location) -> T {
        T::default()
    }

    fn visit_fold(&mut self, _value: &Expr, _cases: &[MatchCase], _location: &Location) -> T {
        T::default()
    }

    fn visit_bend(
        &mut self,
        _initial_states: &[(String, Expr)],
        _condition: &Expr,
        _body: &Block,
        _else_body: Option<&Block>,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_open(&mut self, _type_name: &str, _value: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_with(&mut self, _monad_type: &str, _body: &Block, _location: &Location) -> T {
        T::default()
    }

    fn visit_local_def(&mut self, _function_def: &Definition, _location: &Location) -> T {
        T::default()
    }

    fn visit_statement_expr(&mut self, _expr: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_expression(&mut self, expression: &Expr) -> T {
        match expression {
            Expr::Variable { name, location } => self.visit_variable(name, location),
            Expr::Literal { kind, location } => self.visit_literal(kind, location),
            Expr::Tuple { elements, location } => self.visit_tuple(elements, location),
            Expr::List { elements, location } => self.visit_list(elements, location),
            Expr::Constructor {
                name,
                args,
                named_args,
                location,
            } => self.visit_constructor(name, args, named_args, location),
            Expr::FunctionCall {
                function,
                args,
                named_args,
                location,
            } => self.visit_function_call(function, args, named_args, location),
            Expr::Lambda {
                params,
                body,
                location,
            } => self.visit_lambda(params, body, location),
            Expr::UnsccopedLambda {
                params,
                body,
                location,
            } => self.visit_unsccoped_lambda(params, body, location),
            Expr::BinaryOp {
                left,
                operator,
                right,
                location,
            } => self.visit_binary_op(left, operator, right, location),
            Expr::Superposition { elements, location } => {
                self.visit_superposition(elements, location)
            }
            Expr::MapAccess { map, key, location } => self.visit_map_access(map, key, location),
            Expr::TreeLeaf { value, location } => self.visit_tree_leaf(value, location),
            Expr::TreeNode {
                left,
                right,
                location,
            } => self.visit_tree_node(left, right, location),
            Expr::Block { block, location } => self.visit_block_expr(block, location),
            Expr::Eraser { location } => self.visit_eraser(location),
        }
    }

    fn visit_variable(&mut self, _name: &str, _location: &Location) -> T {
        T::default()
    }

    fn visit_literal(&mut self, _kind: &LiteralKind, _location: &Location) -> T {
        T::default()
    }

    fn visit_tuple(&mut self, _elements: &[Expr], _location: &Location) -> T {
        T::default()
    }

    fn visit_list(&mut self, _elements: &[Expr], _location: &Location) -> T {
        T::default()
    }

    fn visit_constructor(
        &mut self,
        _name: &str,
        _args: &[Expr],
        _named_args: &HashMap<String, Expr>,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_function_call(
        &mut self,
        _function: &Expr,
        _args: &[Expr],
        _named_args: &HashMap<String, Expr>,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_lambda(&mut self, _params: &[Parameter], _body: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_unsccoped_lambda(
        &mut self,
        _params: &[String],
        _body: &Expr,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_binary_op(
        &mut self,
        _left: &Expr,
        _operator: &BinaryOperator,
        _right: &Expr,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_superposition(&mut self, _elements: &[Expr], _location: &Location) -> T {
        T::default()
    }

    fn visit_map_access(&mut self, _map: &Expr, _key: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_tree_leaf(&mut self, _value: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_tree_node(&mut self, _left: &Expr, _right: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_block_expr(&mut self, _block: &Block, _location: &Location) -> T {
        T::default()
    }

    fn visit_eraser(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_pattern(&mut self, pattern: &Pattern) -> T {
        match pattern {
            Pattern::Variable { name, location } => self.visit_pattern_variable(name, location),
            Pattern::Tuple { elements, location } => self.visit_pattern_tuple(elements, location),
            Pattern::Constructor {
                name,
                fields,
                location,
            } => self.visit_pattern_constructor(name, fields, location),
            Pattern::Literal { value, location } => self.visit_pattern_literal(value, location),
            Pattern::Wildcard { location } => self.visit_pattern_wildcard(location),
        }
    }

    fn visit_pattern_variable(&mut self, _name: &str, _location: &Location) -> T {
        T::default()
    }

    fn visit_pattern_tuple(&mut self, _elements: &[Pattern], _location: &Location) -> T {
        T::default()
    }

    fn visit_pattern_constructor(
        &mut self,
        _name: &str,
        _fields: &HashMap<String, Pattern>,
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_pattern_literal(&mut self, _value: &Expr, _location: &Location) -> T {
        T::default()
    }

    fn visit_pattern_wildcard(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_type(&mut self, type_: &Type) -> T {
        match type_ {
            Type::Named {
                name,
                params,
                location,
            } => self.visit_type_named(name, params, location),
            Type::Function {
                param,
                result,
                location,
            } => self.visit_type_function(param, result, location),
            Type::Tuple { elements, location } => self.visit_type_tuple(elements, location),
            Type::Any { location } => self.visit_type_any(location),
            Type::None { location } => self.visit_type_none(location),
            Type::Hole { location } => self.visit_type_hole(location),
            Type::U24 { location } => self.visit_type_u24(location),
            Type::I24 { location } => self.visit_type_i24(location),
            Type::F24 { location } => self.visit_type_f24(location),
            Type::Unknown { location } => self.visit_type_unknown(location),
            Type::Generic {
                name,
                bounds,
                location,
            } => self.visit_type_generic(name, bounds, location),
            Type::Constrained {
                base,
                bounds,
                location,
            } => self.visit_type_constrained(base, bounds, location),
            Type::Effect {
                input,
                output,
                location,
            } => self.visit_type_effect(input, output, location),
        }
    }

    fn visit_type_named(&mut self, _name: &str, _params: &[Type], _location: &Location) -> T {
        T::default()
    }

    fn visit_type_function(&mut self, _param: &Type, _result: &Type, _location: &Location) -> T {
        T::default()
    }

    fn visit_type_tuple(&mut self, _elements: &[Type], _location: &Location) -> T {
        T::default()
    }

    fn visit_type_any(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_type_none(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_type_hole(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_type_u24(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_type_i24(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_type_f24(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_type_unknown(&mut self, _location: &Location) -> T {
        T::default()
    }

    fn visit_type_generic(
        &mut self,
        _name: &str,
        _bounds: &[TypeBound],
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_type_constrained(
        &mut self,
        _base: &Type,
        _bounds: &[TypeBound],
        _location: &Location,
    ) -> T {
        T::default()
    }

    fn visit_type_effect(&mut self, _input: &Type, _output: &Type, _location: &Location) -> T {
        T::default()
    }
}

/// AST validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum AstValidationError {
    DuplicateDefinition {
        name: String,
        location: Location,
    },
    UndefinedVariable {
        name: String,
        location: Location,
    },
    TypeMismatch {
        expected: Type,
        found: Type,
        location: Location,
    },
    PatternMismatch {
        pattern: Pattern,
        value: Expr,
        location: Location,
    },
    InvalidRecursion {
        location: Location,
    },
    DuplicateField {
        name: String,
        location: Location,
    },
    MissingField {
        name: String,
        location: Location,
    },
}

/// AST validator
pub struct AstValidator;

impl AstValidator {
    pub fn new() -> Self {
        AstValidator
    }

    pub fn validate(&self, program: &Program) -> Result<(), Vec<AstValidationError>> {
        let mut errors = Vec::new();
        self.validate_program(program, &mut errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_program(&self, program: &Program, errors: &mut Vec<AstValidationError>) {
        let mut definitions = std::collections::HashSet::new();

        for def in &program.definitions {
            self.validate_definition(def, &mut definitions, errors);
        }
    }

    fn validate_definition(
        &self,
        def: &Definition,
        _definitions: &mut std::collections::HashSet<String>,
        errors: &mut Vec<AstValidationError>,
    ) {
        match def {
            Definition::FunctionDef {
                name,
                params,
                return_type,
                body,
                location,
                ..
            } => {
                if name.starts_with('_') {
                    // Private function, ok
                }
                self.validate_block(body, errors);
            }
            Definition::TypeDef {
                name,
                variants,
                location,
                ..
            } => {
                let mut variant_names = std::collections::HashSet::new();
                for variant in variants {
                    if !variant_names.insert(variant.name.clone()) {
                        errors.push(AstValidationError::DuplicateDefinition {
                            name: variant.name.clone(),
                            location: variant.location.clone(),
                        });
                    }
                }
            }
            Definition::ObjectDef {
                name,
                fields,
                location,
                ..
            } => {
                let mut field_names = std::collections::HashSet::new();
                for field in fields {
                    if !field_names.insert(field.name.clone()) {
                        errors.push(AstValidationError::DuplicateField {
                            name: field.name.clone(),
                            location: field.location.clone(),
                        });
                    }
                }
            }
            Definition::TypeAlias {
                name,
                target_type,
                location,
                ..
            } => {
                self.validate_type(target_type, errors);
            }
            Definition::Module {
                name,
                definitions,
                exports,
                location,
                ..
            } => {
                let mut def_names = std::collections::HashSet::new();
                for def in definitions {
                    let def_name = match def {
                        Definition::FunctionDef { name, .. } => name.clone(),
                        Definition::TypeDef { name, .. } => name.clone(),
                        Definition::ObjectDef { name, .. } => name.clone(),
                        Definition::TypeAlias { name, .. } => name.clone(),
                        Definition::Module { .. } => continue,
                    };
                    if !def_names.insert(def_name.clone()) {
                        errors.push(AstValidationError::DuplicateDefinition {
                            name: def_name,
                            location: def.location().clone(),
                        });
                    }
                }
            }
        }
    }

    fn validate_block(&self, block: &Block, errors: &mut Vec<AstValidationError>) {
        for stmt in &block.statements {
            self.validate_statement(stmt, errors);
        }
    }

    fn validate_statement(&self, stmt: &Statement, errors: &mut Vec<AstValidationError>) {
        match stmt {
            Statement::Assignment {
                pattern,
                value,
                location,
            } => {
                self.validate_expression(value, errors);
            }
            Statement::Use {
                value, location, ..
            } => {
                self.validate_expression(value, errors);
            }
            Statement::Return {
                value, location, ..
            } => {
                self.validate_expression(value, errors);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                location,
            } => {
                self.validate_expression(condition, errors);
                self.validate_block(then_branch, errors);
                self.validate_block(else_branch, errors);
            }
            Statement::Match {
                value,
                cases,
                location,
            } => {
                self.validate_expression(value, errors);
                for case in cases {
                    self.validate_block(&case.body, errors);
                }
            }
            Statement::Bend {
                initial_states,
                condition,
                body,
                else_body,
                location,
            } => {
                for (_, expr) in initial_states {
                    self.validate_expression(expr, errors);
                }
                self.validate_expression(condition, errors);
                self.validate_block(body, errors);
                if let Some(else_b) = else_body {
                    self.validate_block(else_b, errors);
                }
            }
            Statement::With { body, location, .. } => {
                self.validate_block(body, errors);
            }
            Statement::LocalDef {
                function_def,
                location,
            } => {
                self.validate_definition(
                    function_def,
                    &mut std::collections::HashSet::new(),
                    errors,
                );
            }
            Statement::Expr { expr, location } => {
                self.validate_expression(expr, errors);
            }
            _ => {}
        }
    }

    fn validate_expression(&self, expr: &Expr, errors: &mut Vec<AstValidationError>) {
        match expr {
            Expr::FunctionCall {
                function,
                args,
                location,
                ..
            } => {
                self.validate_expression(function, errors);
                for arg in args {
                    self.validate_expression(arg, errors);
                }
            }
            Expr::BinaryOp {
                left,
                right,
                location,
                ..
            } => {
                self.validate_expression(left, errors);
                self.validate_expression(right, errors);
            }
            Expr::Lambda { body, location, .. } => {
                self.validate_expression(body, errors);
            }
            _ => {}
        }
    }

    fn validate_type(&self, type_: &Type, errors: &mut Vec<AstValidationError>) {
        match type_ {
            Type::Named {
                params, location, ..
            } => {
                for param in params {
                    self.validate_type(param, errors);
                }
            }
            Type::Function {
                param,
                result,
                location,
            } => {
                self.validate_type(param, errors);
                self.validate_type(result, errors);
            }
            Type::Tuple { elements, location } => {
                for elem in elements {
                    self.validate_type(elem, errors);
                }
            }
            _ => {}
        }
    }
}
