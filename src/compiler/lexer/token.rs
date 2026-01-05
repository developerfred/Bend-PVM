use std::fmt;
use std::hash::Hash;

/// Represents a token in the Bend-PVM language
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    // Keywords
    Def,
    Type,
    Object,
    Return,
    If,
    Else,
    Match,
    Case,
    Fold,
    Bend,
    When,
    Fork,
    Open,
    With,
    Use,
    Lambda,
    In,
    Let,
    Switch,
    Import,
    From,
    As,
    Fn, // Alias for Def (compatibility)
    Contract,
    Interface,
    Library,
    Underscore, // For pattern matching
    True,
    False,

    // Symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    DoubleColon, // ::
    Semicolon,
    Comma,
    Dot,
    Arrow,
    FatArrow,
    LeftArrow,
    Equal,
    Tilde,
    BackTick,
    Assign,     // Alias for Equal (compatibility)
    LeftParen,  // Alias for LParen
    RightParen, // Alias for RParen
    LeftBrace,  // Alias for LBrace
    RightBrace, // Alias for RBrace

    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Ampersand,
    Pipe,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    EqualEqual,
    NotEqual,
    BangEqual, // !=
    AndAnd,    // &&
    OrOr,      // ||
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
    CaretEqual,
    AmpersandEqual,
    PipeEqual,
    Less,    // Alias for LessThan (for generics)
    Greater, // Alias for GreaterThan (for generics)

    // Type annotations
    Uint,   // For type annotations
    Int,    // For type annotations
    Float,  // For type annotations
    String, // For type annotations
    Char,   // For type annotations
    Symbol, // For type annotations
    Any,    // For type annotations
    U24,    // Alias for UintLiteral
    I24,    // Alias for IntLiteral
    F24,    // Alias for FloatLiteral

    // Literals
    Identifier(String),
    UintLiteral(u32),  // For u24
    IntLiteral(i32),   // For i24
    FloatLiteral(u32), // For f24 (stored as bits to enable Eq/Hash)
    StringLiteral(String),
    CharLiteral(char),
    SymbolLiteral(String),

    // Special
    EOF,
    Error(String),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Def => write!(f, "def"),
            Token::Type => write!(f, "type"),
            Token::Object => write!(f, "object"),
            Token::Return => write!(f, "return"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::Match => write!(f, "match"),
            Token::Case => write!(f, "case"),
            Token::Fold => write!(f, "fold"),
            Token::Bend => write!(f, "bend"),
            Token::When => write!(f, "when"),
            Token::Fork => write!(f, "fork"),
            Token::Open => write!(f, "open"),
            Token::With => write!(f, "with"),
            Token::Use => write!(f, "use"),
            Token::Lambda => write!(f, "lambda"),
            Token::In => write!(f, "in"),
            Token::Let => write!(f, "let"),
            Token::Switch => write!(f, "switch"),
            Token::Import => write!(f, "import"),
            Token::From => write!(f, "from"),
            Token::As => write!(f, "as"),
            Token::Fn => write!(f, "fn"),
            Token::Contract => write!(f, "contract"),
            Token::Interface => write!(f, "interface"),
            Token::Library => write!(f, "library"),
            Token::Underscore => write!(f, "_"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Colon => write!(f, ":"),
            Token::DoubleColon => write!(f, "::"),
            Token::Semicolon => write!(f, ";"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Arrow => write!(f, "->"),
            Token::FatArrow => write!(f, "=>"),
            Token::LeftArrow => write!(f, "<-"),
            Token::Equal => write!(f, "="),
            Token::Tilde => write!(f, "~"),
            Token::BackTick => write!(f, "`"),
            Token::Assign => write!(f, "="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Caret => write!(f, "^"),
            Token::Ampersand => write!(f, "&"),
            Token::Pipe => write!(f, "|"),
            Token::GreaterThan => write!(f, ">"),
            Token::LessThan => write!(f, "<"),
            Token::GreaterEqual => write!(f, ">="),
            Token::LessEqual => write!(f, "<="),
            Token::EqualEqual => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::BangEqual => write!(f, "!="),
            Token::AndAnd => write!(f, "&&"),
            Token::OrOr => write!(f, "||"),
            Token::PlusEqual => write!(f, "+="),
            Token::MinusEqual => write!(f, "-="),
            Token::StarEqual => write!(f, "*="),
            Token::SlashEqual => write!(f, "/="),
            Token::PercentEqual => write!(f, "%="),
            Token::CaretEqual => write!(f, "^="),
            Token::AmpersandEqual => write!(f, "&="),
            Token::PipeEqual => write!(f, "|="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::Uint => write!(f, "Uint"),
            Token::Int => write!(f, "Int"),
            Token::Float => write!(f, "Float"),
            Token::String => write!(f, "String"),
            Token::Char => write!(f, "Char"),
            Token::Symbol => write!(f, "Symbol"),
            Token::Any => write!(f, "Any"),
            Token::U24 => write!(f, "U24"),
            Token::I24 => write!(f, "I24"),
            Token::F24 => write!(f, "F24"),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::UintLiteral(n) => write!(f, "{}", n),
            Token::IntLiteral(n) => write!(f, "{}", n),
            Token::FloatLiteral(bits) => write!(f, "{}", f32::from_bits(*bits)),
            Token::StringLiteral(s) => write!(f, "\"{}\"", s),
            Token::CharLiteral(c) => write!(f, "'{}'", c),
            Token::SymbolLiteral(s) => write!(f, "`{}`", s),
            Token::EOF => write!(f, "EOF"),
            Token::Error(e) => write!(f, "Error: {}", e),
        }
    }
}
