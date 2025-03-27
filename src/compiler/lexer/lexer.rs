use logos::{Logos, Lexer};
use std::collections::HashMap;
use super::token::Token;

/// Define tokens using the Logos derive macro for efficient lexing
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+", error = LexError)]
enum LogosToken {
    #[regex("[A-Za-z][A-Za-z0-9_./]*")]
    Identifier,

    // Keywords are handled in the callback for Identifier
    
    #[regex("0|[1-9][0-9]*")]
    UintLiteral,
    
    #[regex("[+-][0-9]+")]
    IntLiteral,
    
    #[regex("[+-]?[0-9]+\\.[0-9]+")]
    FloatLiteral,
    
    #[regex("\"[^\"]*\"")]
    StringLiteral,
    
    #[regex("'[^']'")]
    CharLiteral,
    
    #[regex("`[^`]*`")]
    SymbolLiteral,
    
    #[token("(")]
    LParen,
    
    #[token(")")]
    RParen,
    
    #[token("{")]
    LBrace,
    
    #[token("}")]
    RBrace,
    
    #[token("[")]
    LBracket,
    
    #[token("]")]
    RBracket,
    
    #[token(":")]
    Colon,
    
    #[token(";")]
    Semicolon,
    
    #[token(",")]
    Comma,
    
    #[token(".")]
    Dot,
    
    #[token("->")]
    Arrow,
    
    #[token("=>")]
    FatArrow,
    
    #[token("=")]
    Equal,
    
    #[token("~")]
    Tilde,
    
    #[token("+")]
    Plus,
    
    #[token("-")]
    Minus,
    
    #[token("*")]
    Star,
    
    #[token("/")]
    Slash,
    
    #[token("%")]
    Percent,
    
    #[token("^")]
    Caret,
    
    #[token("&")]
    Ampersand,
    
    #[token("|")]
    Pipe,
    
    #[token(">")]
    GreaterThan,
    
    #[token("<")]
    LessThan,
    
    #[token(">=")]
    GreaterEqual,
    
    #[token("<=")]
    LessEqual,
    
    #[token("==")]
    EqualEqual,
    
    #[token("!=")]
    NotEqual,
    
    #[token("+=")]
    PlusEqual,
    
    #[token("-=")]
    MinusEqual,
    
    #[token("*=")]
    StarEqual,
    
    #[token("/=")]
    SlashEqual,
    
    #[token("%=")]
    PercentEqual,
    
    #[token("^=")]
    CaretEqual,
    
    #[token("&=")]
    AmpersandEqual,
    
    #[token("|=")]
    PipeEqual,
    
    // Comments
    #[regex("#\\{[^}]*\\}#", logos::skip)]
    MultiLineComment,
    
    #[regex("#[^\\n]*", logos::skip)]
    SingleLineComment,
    
    #[error]
    LexError,
}

/// Represents a token with its position in the source code
#[derive(Debug, Clone)]
pub struct TokenWithPosition {
    pub token: Token,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// The main lexer for the Bend-PVM language
pub struct BendLexer<'a> {
    logos_lexer: Lexer<'a, LogosToken>,
    /// Mapping from identifier strings to keyword tokens
    keywords: HashMap<&'static str, Token>,
    /// Current line number (1-based)
    line: usize,
    /// Current column number (1-based)
    column: usize,
    /// Source code for error reporting
    source: &'a str,
}

impl<'a> BendLexer<'a> {
    /// Create a new lexer for the given source code
    pub fn new(source: &'a str) -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("def", Token::Def);
        keywords.insert("type", Token::Type);
        keywords.insert("object", Token::Object);
        keywords.insert("return", Token::Return);
        keywords.insert("if", Token::If);
        keywords.insert("else", Token::Else);
        keywords.insert("match", Token::Match);
        keywords.insert("case", Token::Case);
        keywords.insert("fold", Token::Fold);
        keywords.insert("bend", Token::Bend);
        keywords.insert("when", Token::When);
        keywords.insert("fork", Token::Fork);
        keywords.insert("open", Token::Open);
        keywords.insert("with", Token::With);
        keywords.insert("use", Token::Use);
        keywords.insert("lambda", Token::Lambda);
        keywords.insert("in", Token::In);
        keywords.insert("let", Token::Let);
        keywords.insert("switch", Token::Switch);
        keywords.insert("import", Token::Import);
        keywords.insert("from", Token::From);
        keywords.insert("as", Token::As);
        
        BendLexer {
            logos_lexer: LogosToken::lexer(source),
            keywords,
            line: 1,
            column: 1,
            source,
        }
    }
    
    /// Get the next token from the source
    pub fn next_token(&mut self) -> TokenWithPosition {
        let start_pos = self.logos_lexer.span().start;
        let start_column = self.column;
        let start_line = self.line;
        
        let token = match self.logos_lexer.next() {
            Some(Ok(logos_token)) => {
                let span = self.logos_lexer.span();
                let text = &self.source[span.clone()];
                
                match logos_token {
                    LogosToken::Identifier => {
                        // Check if it's a keyword
                        if let Some(keyword) = self.keywords.get(text) {
                            keyword.clone()
                        } else {
                            Token::Identifier(text.to_string())
                        }
                    }
                    LogosToken::UintLiteral => {
                        if let Ok(value) = text.parse::<u32>() {
                            if value > 0xFFFFFF {
                                Token::Error(format!("Unsigned integer literal exceeds u24 maximum value: {}", value))
                            } else {
                                Token::UintLiteral(value)
                            }
                        } else {
                            Token::Error(format!("Invalid unsigned integer literal: {}", text))
                        }
                    }
                    LogosToken::IntLiteral => {
                        if let Ok(value) = text.parse::<i32>() {
                            if value > 0x7FFFFF || value < -0x800000 {
                                Token::Error(format!("Signed integer literal exceeds i24 range: {}", value))
                            } else {
                                Token::IntLiteral(value)
                            }
                        } else {
                            Token::Error(format!("Invalid signed integer literal: {}", text))
                        }
                    }
                    LogosToken::FloatLiteral => {
                        if let Ok(value) = text.parse::<f32>() {
                            Token::FloatLiteral(value)
                        } else {
                            Token::Error(format!("Invalid float literal: {}", text))
                        }
                    }
                    LogosToken::StringLiteral => {
                        // Remove the quotes
                        let value = &text[1..text.len() - 1];
                        Token::StringLiteral(value.to_string())
                    }
                    LogosToken::CharLiteral => {
                        // Remove the quotes and parse as char
                        let char_str = &text[1..text.len() - 1];
                        let c = char_str.chars().next().unwrap_or('\0');
                        Token::CharLiteral(c)
                    }
                    LogosToken::SymbolLiteral => {
                        // Remove backticks
                        let value = &text[1..text.len() - 1];
                        Token::SymbolLiteral(value.to_string())
                    }
                    LogosToken::LParen => Token::LParen,
                    LogosToken::RParen => Token::RParen,
                    LogosToken::LBrace => Token::LBrace,
                    LogosToken::RBrace => Token::RBrace,
                    LogosToken::LBracket => Token::LBracket,
                    LogosToken::RBracket => Token::RBracket,
                    LogosToken::Colon => Token::Colon,
                    LogosToken::Semicolon => Token::Semicolon,
                    LogosToken::Comma => Token::Comma,
                    LogosToken::Dot => Token::Dot,
                    LogosToken::Arrow => Token::Arrow,
                    LogosToken::FatArrow => Token::FatArrow,
                    LogosToken::Equal => Token::Equal,
                    LogosToken::Tilde => Token::Tilde,
                    LogosToken::Plus => Token::Plus,
                    LogosToken::Minus => Token::Minus,
                    LogosToken::Star => Token::Star,
                    LogosToken::Slash => Token::Slash,
                    LogosToken::Percent => Token::Percent,
                    LogosToken::Caret => Token::Caret,
                    LogosToken::Ampersand => Token::Ampersand,
                    LogosToken::Pipe => Token::Pipe,
                    LogosToken::GreaterThan => Token::GreaterThan,
                    LogosToken::LessThan => Token::LessThan,
                    LogosToken::GreaterEqual => Token::GreaterEqual,
                    LogosToken::LessEqual => Token::LessEqual,
                    LogosToken::EqualEqual => Token::EqualEqual,
                    LogosToken::NotEqual => Token::NotEqual,
                    LogosToken::PlusEqual => Token::PlusEqual,
                    LogosToken::MinusEqual => Token::MinusEqual,
                    LogosToken::StarEqual => Token::StarEqual,
                    LogosToken::SlashEqual => Token::SlashEqual,
                    LogosToken::PercentEqual => Token::PercentEqual,
                    LogosToken::CaretEqual => Token::CaretEqual,
                    LogosToken::AmpersandEqual => Token::AmpersandEqual,
                    LogosToken::PipeEqual => Token::PipeEqual,
                    LogosToken::LexError => Token::Error(format!("Invalid token: {}", text)),
                    _ => Token::Error(format!("Unexpected token: {}", text)),
                }
            }
            Some(Err(_)) => {
                let span = self.logos_lexer.span();
                let text = &self.source[span.clone()];
                Token::Error(format!("Lexer error at position {}: {}", span.start, text))
            }
            None => Token::EOF,
        };
        
        // Update line and column count for next token
        for c in self.source[start_pos..self.logos_lexer.span().start].chars() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        
        TokenWithPosition {
            token,
            start: start_pos,
            end: self.logos_lexer.span().end,
            line: start_line,
            column: start_column,
        }
    }
}