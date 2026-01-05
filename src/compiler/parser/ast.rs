use std::collections::HashMap;

/// Represents a source location for AST nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct Location {
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

impl Location {
    /// Create a new location with the same start and end position
    pub fn new(line: usize, column: usize, start: usize, end: usize) -> Self {
        Location {
            line,
            column,
            start,
            end,
        }
    }

    /// Create a location that spans from the start of one location to the end of another
    pub fn span(start: &Location, end: &Location) -> Self {
        Location {
            line: start.line,
            column: start.column,
            start: start.start,
            end: end.end,
        }
    }
}

/// Represents a complete Bend-PVM program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub imports: Vec<Import>,
    pub definitions: Vec<Definition>,
    pub location: Location,
}

/// Represents an import statement
#[derive(Debug, Clone, PartialEq)]
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
#[derive(Debug, Clone, PartialEq)]
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
        functions: Vec<Definition>,
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    TupleConstructor {
        name: String,
        args: Vec<Pattern>,
        location: Location,
    },
    Literal {
        value: Expr, // Only for literals
        location: Location,
    },
    Member {
        parent: Box<Pattern>,
        member: String,
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
    Array {
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
    FieldAccess {
        object: Box<Expr>,
        field: String,
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
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
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
    Bool(bool),
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
            Expr::Array { location, .. } => location,
            Expr::Constructor { location, .. } => location,
            Expr::FunctionCall { location, .. } => location,
            Expr::Lambda { location, .. } => location,
            Expr::UnsccopedLambda { location, .. } => location,
            Expr::BinaryOp { location, .. } => location,
            Expr::FieldAccess { location, .. } => location,
            Expr::Superposition { location, .. } => location,
            Expr::MapAccess { location, .. } => location,
            Expr::TreeLeaf { location, .. } => location,
            Expr::TreeNode { location, .. } => location,
            Expr::If { location, .. } => location,
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

impl Default for BasicAstVisitor {
    fn default() -> Self {
        Self::new()
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
            Definition::FunctionDef { body, .. } => {
                self.validate_block(body, errors);
            }
            Definition::TypeDef { variants, .. } => {
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
            Definition::ObjectDef { fields, .. } => {
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
            Definition::TypeAlias { target_type, .. } => {
                self.validate_type(target_type, errors);
            }
            Definition::Module { definitions, .. } => {
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

        impl Default for AstValidator {
            fn default() -> Self {
                Self::new()
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
                pattern: _,
                value,
                location: _,
            } => {
                self.validate_expression(value, errors);
            }
            Statement::Use {
                value, location: _, ..
            } => {
                self.validate_expression(value, errors);
            }
            Statement::Return { value, location: _ } => {
                self.validate_expression(value, errors);
            }
            Statement::If {
                condition,
                then_branch,
                else_branch,
                location: _,
            } => {
                self.validate_expression(condition, errors);
                self.validate_block(then_branch, errors);
                self.validate_block(else_branch, errors);
            }
            Statement::Match {
                value,
                cases,
                location: _,
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
                location: _,
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
            Statement::With {
                body, location: _, ..
            } => {
                self.validate_block(body, errors);
            }
            Statement::LocalDef {
                function_def,
                location: _,
            } => {
                self.validate_definition(
                    function_def,
                    &mut std::collections::HashSet::new(),
                    errors,
                );
            }
            Statement::Expr { expr, location: _ } => {
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
                location: _,
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
                location: _,
                ..
            } => {
                self.validate_expression(left, errors);
                self.validate_expression(right, errors);
            }
            Expr::Lambda {
                body, location: _, ..
            } => {
                self.validate_expression(body, errors);
            }
            _ => {}
        }
    }

    fn validate_type(&self, type_: &Type, errors: &mut Vec<AstValidationError>) {
        match type_ {
            Type::Named {
                params,
                location: _,
                ..
            } => {
                for param in params {
                    self.validate_type(param, errors);
                }
            }
            Type::Function {
                param,
                result,
                location: _,
            } => {
                self.validate_type(param, errors);
                self.validate_type(result, errors);
            }
            Type::Tuple {
                elements,
                location: _,
            } => {
                for elem in elements {
                    self.validate_type(elem, errors);
                }
            }
            _ => {}
        }
    }
}
