//! # Solidity AST Representation
//!
//! This module provides AST structures for representing Solidity contracts,
//! enabling parsing and analysis during migration.

use std::collections::HashMap;

/// Represents a Solidity source location
#[derive(Debug, Clone, PartialEq)]
pub struct SolLocation {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

/// Represents a Solidity version pragma
#[derive(Debug, Clone)]
pub struct VersionPragma {
    pub operator: String,
    pub version: String,
}

/// Main AST node for a Solidity source file
#[derive(Debug, Clone)]
pub struct SoliditySource {
    pub version_pragma: Option<VersionPragma>,
    pub imports: Vec<ImportDirective>,
    pub contracts: Vec<ContractDefinition>,
    pub interfaces: Vec<InterfaceDefinition>,
    pub libraries: Vec<LibraryDefinition>,
    pub enums: Vec<EnumDefinition>,
    pub structs: Vec<StructDefinition>,
    pub location: SolLocation,
}

/// Import directive
#[derive(Debug, Clone)]
pub enum ImportDirective {
    DirectImport {
        path: String,
        location: SolLocation,
    },
    NamedImport {
        path: String,
        symbols: Vec<(String, Option<String>)>,
        location: SolLocation,
    },
    SelectiveImport {
        path: String,
        items: Vec<ImportItem>,
        location: SolLocation,
    },
}

#[derive(Debug, Clone)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
    pub location: SolLocation,
}

/// Contract definition (main source unit)
#[derive(Debug, Clone)]
pub struct ContractDefinition {
    pub name: String,
    pub kind: ContractKind,
    pub base_contracts: Vec<BaseContract>,
    pub state_variables: Vec<StateVariable>,
    pub functions: Vec<FunctionDefinition>,
    pub modifiers: Vec<ModifierDefinition>,
    pub events: Vec<EventDefinition>,
    pub errors: Vec<ErrorDefinition>,
    pub structs: Vec<StructDefinition>,
    pub enums: Vec<EnumDefinition>,
    pub type_definitions: Vec<TypeDefinition>,
    pub is_abstract: bool,
    pub location: SolLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ContractKind {
    Contract,
    Interface,
    Library,
}

#[derive(Debug, Clone)]
pub struct BaseContract {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub location: SolLocation,
}

/// State variable declaration
#[derive(Debug, Clone)]
pub struct StateVariable {
    pub name: String,
    pub type_name: TypeName,
    pub visibility: Visibility,
    pub mutability: Mutability,
    pub value: Option<Expression>,
    pub is_constant: bool,
    pub overrides: Option<Vec<String>>,
    pub location: SolLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    Private,
    Internal,
    External,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mutability {
    Mutable,
    Constant,
    Immutable,
}

/// Function definition
#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<VariableDeclaration>,
    pub return_parameters: Vec<VariableDeclaration>,
    pub body: Option<Block>,
    pub visibility: Visibility,
    pub state_mutability: StateMutability,
    pub virtual_flag: bool,
    pub override_specifiers: Vec<String>,
    pub modifiers: Vec<ModifierInvocation>,
    pub is_constructor: bool,
    pub is_fallback: bool,
    pub is_receive: bool,
    pub location: SolLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateMutability {
    Pure,
    View,
    Payable,
    NonPayable,
}

#[derive(Debug, Clone)]
pub struct ModifierInvocation {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub location: SolLocation,
}

/// Modifier definition
#[derive(Debug, Clone)]
pub struct ModifierDefinition {
    pub name: String,
    pub parameters: Vec<VariableDeclaration>,
    pub body: Block,
    pub visibility: Visibility,
    pub is_virtual: bool,
    pub override_specifiers: Vec<String>,
    pub location: SolLocation,
}

/// Event definition
#[derive(Debug, Clone)]
pub struct EventDefinition {
    pub name: String,
    pub parameters: Vec<VariableDeclaration>,
    pub anonymous: bool,
    pub location: SolLocation,
}

/// Error definition
#[derive(Debug, Clone)]
pub struct ErrorDefinition {
    pub name: String,
    pub parameters: Vec<VariableDeclaration>,
    pub location: SolLocation,
}

/// Variable declaration
#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: Option<String>,
    pub type_name: TypeName,
    pub storage_location: StorageLocation,
    pub location: SolLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StorageLocation {
    Memory,
    Storage,
    Calldata,
    Default,
}

/// Type names in Solidity
#[derive(Debug, Clone)]
pub enum TypeName {
    Elementary(ElementaryTypeName),
    UserDefined(UserDefinedTypeName),
    Array(ArrayTypeName),
    Mapping(MappingTypeName),
    Function(FunctionTypeName),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ElementaryTypeName {
    pub name: String,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct UserDefinedTypeName {
    pub name: String,
    pub type_arguments: Vec<TypeName>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct ArrayTypeName {
    pub base_type: Box<TypeName>,
    pub length: Option<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct MappingTypeName {
    pub key_type: Box<TypeName>,
    pub value_type: Box<TypeName>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct FunctionTypeName {
    pub parameter_types: Vec<TypeName>,
    pub return_types: Vec<TypeName>,
    pub visibility: Visibility,
    pub state_mutability: StateMutability,
    pub location: SolLocation,
}

/// Expression types
#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(Identifier),
    Literal(Literal),
    BinaryOperation(BinaryOperation),
    UnaryOperation(UnaryOperation),
    Assignment(Assignment),
    FunctionCall(FunctionCall),
    MemberAccess(MemberAccess),
    IndexAccess(IndexAccess),
    Conditional(Conditional),
    Tuple(TupleExpression),
    TypeConversion(TypeConversion),
    NewExpression(NewExpression),
    ArrayLiteral(ArrayLiteral),
    StructLiteral(StructLiteral),
    Location(SolLocation),
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub name: String,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub value: Option<String>,
    pub subdenomination: Option<SubDenomination>,
    pub type_name: Option<String>,
    pub location: SolLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SubDenomination {
    Wei,
    Ether,
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Years,
}

#[derive(Debug, Clone)]
pub struct BinaryOperation {
    pub operator: BinaryOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub location: SolLocation,
}

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
    BitShiftLeft,
    BitShiftRight,
    LogicalAnd,
    LogicalOr,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Debug, Clone)]
pub struct UnaryOperation {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub is_prefix: bool,
    pub location: SolLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Neg,
    BitNot,
    Inc,
    Dec,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub operator: AssignmentOperator,
    pub left: Box<Expression>,
    pub right: Box<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    BitShiftLeftAssign,
    BitShiftRightAssign,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub expression: Box<Expression>,
    pub arguments: Vec<Expression>,
    pub names: Vec<String>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct MemberAccess {
    pub expression: Box<Expression>,
    pub member_name: String,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct IndexAccess {
    pub base: Box<Expression>,
    pub index: Box<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: Box<Expression>,
    pub true_expression: Box<Expression>,
    pub false_expression: Box<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct TupleExpression {
    pub elements: Vec<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct TypeConversion {
    pub type_name: TypeName,
    pub expression: Box<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct NewExpression {
    pub type_name: TypeName,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub elements: Vec<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct StructLiteral {
    pub type_name: String,
    pub arguments: Vec<Expression>,
    pub location: SolLocation,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub location: SolLocation,
}

/// Statement types
#[derive(Debug, Clone)]
pub enum Statement {
    Block(Block),
    VariableDeclaration(VariableDeclarationStatement),
    Assignment(AssignmentStatement),
    Expression(ExpressionStatement),
    If(IfStatement),
    For(ForStatement),
    While(WhileStatement),
    DoWhile(DoWhileStatement),
    Continue(ContinueStatement),
    Break(BreakStatement),
    Return(ReturnStatement),
    Emit(EmitStatement),
    Revert(RevertStatement),
    Assembly(InlineAssembly),
    Unchecked(UncheckedBlock),
    Placeholder(PlaceholderStatement),
    Location(SolLocation),
}

#[derive(Debug, Clone)]
pub struct VariableDeclarationStatement {
    pub declarations: Vec<VariableDeclaration>,
    pub initial_value: Option<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct AssignmentStatement {
    pub assignment: Assignment,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub expression: Expression,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub true_body: Box<Statement>,
    pub false_body: Option<Box<Statement>>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct ForStatement {
    pub initialization: Option<Box<Statement>>,
    pub condition: Option<Expression>,
    pub iteration: Option<Box<Statement>>,
    pub body: Box<Statement>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub condition: Expression,
    pub body: Box<Statement>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct DoWhileStatement {
    pub body: Box<Statement>,
    pub condition: Expression,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct ContinueStatement {
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct BreakStatement {
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expression: Option<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct EmitStatement {
    pub event: Expression,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct RevertStatement {
    pub error_call: Option<Expression>,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct InlineAssembly {
    pub operations: String,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct UncheckedBlock {
    pub block: Block,
    pub location: SolLocation,
}

#[derive(Debug, Clone)]
pub struct PlaceholderStatement {
    pub location: SolLocation,
}

/// Struct definition
#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: String,
    pub members: Vec<VariableDeclaration>,
    pub location: SolLocation,
}

/// Enum definition
#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: String,
    pub values: Vec<String>,
    pub location: SolLocation,
}

/// Type definition
#[derive(Debug, Clone)]
pub struct TypeDefinition {
    pub name: String,
    pub underlying_type: TypeName,
    pub location: SolLocation,
}

/// Interface definition
#[derive(Debug, Clone)]
pub struct InterfaceDefinition {
    pub name: String,
    pub base_contracts: Vec<BaseContract>,
    pub functions: Vec<FunctionDefinition>,
    pub events: Vec<EventDefinition>,
    pub structs: Vec<StructDefinition>,
    pub enums: Vec<EnumDefinition>,
    pub location: SolLocation,
}

/// Library definition
#[derive(Debug, Clone)]
pub struct LibraryDefinition {
    pub name: String,
    pub functions: Vec<FunctionDefinition>,
    pub structs: Vec<StructDefinition>,
    pub enums: Vec<EnumDefinition>,
    pub location: SolLocation,
}
