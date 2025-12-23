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
    Uint(u32),    // For u24
    Int(i32),     // For i24
    Float(f32),   // For f24
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

