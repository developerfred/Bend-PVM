use super::token::Token;
use logos::{Lexer, Logos};
use std::collections::HashMap;

/// Define tokens using the Logos derive macro for efficient lexing
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+")]
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
        let mut start_pos = self.logos_lexer.span().end;

        let token_result = self.logos_lexer.next();

        // The logos lexer skip patterns don't update our line/column count.
        // We need to check the text between the end of the last token and the start of this one.
        let skipped_text = &self.source[start_pos..self.logos_lexer.span().start];
        for c in skipped_text.chars() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        let actual_start_pos = self.logos_lexer.span().start;
        let start_column = self.column;
        let start_line = self.line;

        let token = match token_result {
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
                                Token::Error(format!(
                                    "Unsigned integer literal exceeds u24 maximum value: {}",
                                    value
                                ))
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
                                Token::Error(format!(
                                    "Signed integer literal exceeds i24 range: {}",
                                    value
                                ))
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

        // Update line and column count for next token (the actual token text)
        for c in self.source[self.logos_lexer.span().clone()].chars() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }

        TokenWithPosition {
            token,
            start: actual_start_pos,
            end: self.logos_lexer.span().end,
            line: start_line,
            column: start_column,
        }
    }

    /// Helper method to collect all tokens from source
    #[cfg(test)]
    pub fn collect_all_tokens(&mut self) -> Vec<TokenWithPosition> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            tokens.push(token.clone());
            if let Token::EOF = token.token {
                break;
            }
        }
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::super::token::Token;
    use super::*;

    #[test]
    fn test_keywords() {
        let keywords = vec![
            ("def", Token::Def),
            ("type", Token::Type),
            ("return", Token::Return),
            ("if", Token::If),
            ("else", Token::Else),
            ("match", Token::Match),
            ("case", Token::Case),
            ("with", Token::With),
            ("use", Token::Use),
        ];

        for (text, expected) in keywords {
            let mut lexer = BendLexer::new(text);
            let token = lexer.next_token();
            assert_eq!(token.token, expected, "Failed for keyword: {}", text);
        }
    }

    #[test]
    fn test_identifiers() {
        let identifiers = vec![
            "variable",
            "my_function",
            "CamelCase",
            "snake_case",
            "x",
            "a123",
            "test_123",
        ];

        for ident in identifiers {
            let mut lexer = BendLexer::new(ident);
            let token = lexer.next_token();
            assert_eq!(token.token, Token::Identifier(ident.to_string()));
        }
    }

    #[test]
    fn test_unsigned_integers() {
        let test_cases = vec![
            ("0", Token::UintLiteral(0)),
            ("42", Token::UintLiteral(42)),
            ("123456", Token::UintLiteral(123456)),
            ("16777215", Token::UintLiteral(16777215)), // 2^24 - 1 (u24 max)
        ];

        for (text, expected) in test_cases {
            let mut lexer = BendLexer::new(text);
            let token = lexer.next_token();
            assert_eq!(token.token, expected, "Failed for: {}", text);
        }
    }

    #[test]
    fn test_unsigned_integer_overflow() {
        let mut lexer = BendLexer::new("16777216"); // 2^24 (too big for u24)
        let token = lexer.next_token();
        match token.token {
            Token::Error(msg) => assert!(msg.contains("exceeds u24 maximum")),
            _ => panic!("Expected error for overflow"),
        }
    }

    #[test]
    fn test_signed_integers() {
        let test_cases = vec![
            ("+42", Token::IntLiteral(42)),
            ("-123", Token::IntLiteral(-123)),
            ("+0", Token::IntLiteral(0)),
            ("-8388608", Token::IntLiteral(-8388608)), // -2^23 (i24 min)
            ("+8388607", Token::IntLiteral(8388607)),  // 2^23 - 1 (i24 max)
        ];

        for (text, expected) in test_cases {
            let mut lexer = BendLexer::new(text);
            let token = lexer.next_token();
            assert_eq!(token.token, expected, "Failed for: {}", text);
        }
    }

    #[test]
    fn test_signed_integer_overflow() {
        let test_cases = vec!["-8388609", "+8388608"]; // Outside i24 range

        for text in test_cases {
            let mut lexer = BendLexer::new(text);
            let token = lexer.next_token();
            match token.token {
                Token::Error(msg) => assert!(msg.contains("exceeds i24 range")),
                _ => panic!("Expected error for overflow: {}", text),
            }
        }
    }

    #[test]
    fn test_float_literals() {
        let test_cases = vec![
            ("3.14", Token::FloatLiteral(3.14)),
            ("-2.5", Token::FloatLiteral(-2.5)),
            ("+0.0", Token::FloatLiteral(0.0)),
            ("123.456", Token::FloatLiteral(123.456)),
        ];

        for (text, expected) in test_cases {
            let mut lexer = BendLexer::new(text);
            let token = lexer.next_token();
            match (&token.token, &expected) {
                (Token::FloatLiteral(actual), Token::FloatLiteral(expected_val)) => {
                    assert!(
                        (actual - expected_val).abs() < 0.001,
                        "Failed for: {}",
                        text
                    );
                }
                _ => panic!("Expected FloatLiteral for: {}", text),
            }
        }
    }

    #[test]
    fn test_string_literals() {
        let test_cases = vec![
            ("\"hello\"", Token::StringLiteral("hello".to_string())),
            ("\"world\"", Token::StringLiteral("world".to_string())),
            ("\"\"", Token::StringLiteral("".to_string())),
            (
                "\"hello world\"",
                Token::StringLiteral("hello world".to_string()),
            ),
        ];

        for (text, expected) in test_cases {
            let mut lexer = BendLexer::new(text);
            let token = lexer.next_token();
            assert_eq!(token.token, expected, "Failed for: {}", text);
        }
    }

    #[test]
    fn test_operators_and_symbols() {
        let test_cases = vec![
            ("+", Token::Plus),
            ("-", Token::Minus),
            ("*", Token::Star),
            ("/", Token::Slash),
            ("=", Token::Equal),
            ("==", Token::EqualEqual),
            ("!=", Token::NotEqual),
            ("<", Token::LessThan),
            (">", Token::GreaterThan),
            ("<=", Token::LessEqual),
            (">=", Token::GreaterEqual),
            ("->", Token::Arrow),
            ("=>", Token::FatArrow),
            ("(", Token::LParen),
            (")", Token::RParen),
            ("{", Token::LBrace),
            ("}", Token::RBrace),
            ("[", Token::LBracket),
            ("]", Token::RBracket),
            (":", Token::Colon),
            (";", Token::Semicolon),
            (",", Token::Comma),
            (".", Token::Dot),
        ];

        for (text, expected) in test_cases {
            let mut lexer = BendLexer::new(text);
            let token = lexer.next_token();
            assert_eq!(token.token, expected, "Failed for operator: {}", text);
        }
    }

    #[test]
    fn test_comments() {
        let mut lexer = BendLexer::new("# This is a comment\ndef test");
        let tokens = lexer.collect_all_tokens();

        // Should skip comment and return def token
        assert_eq!(tokens[0].token, Token::Def);
        assert_eq!(tokens[1].token, Token::Identifier("test".to_string()));
    }

    #[test]
    fn test_multiline_comments() {
        let mut lexer = BendLexer::new("#{\nThis is a\nmultiline comment\n}#\ndef test");
        let tokens = lexer.collect_all_tokens();

        // Should skip multiline comment and return def token
        assert_eq!(tokens[0].token, Token::Def);
        assert_eq!(tokens[1].token, Token::Identifier("test".to_string()));
    }

    #[test]
    fn test_position_tracking() {
        let mut lexer = BendLexer::new("def\ntest");
        let token1 = lexer.next_token();
        let token2 = lexer.next_token();

        assert_eq!(token1.line, 1);
        assert_eq!(token1.column, 1);
        assert_eq!(token2.line, 2);
        assert_eq!(token2.column, 1);
    }

    #[test]
    fn test_function_definition() {
        let source = "def add(a: u24, b: u24) -> u24:\n    return a + b";
        let mut lexer = BendLexer::new(source);
        let tokens = lexer.collect_all_tokens();

        let expected_tokens = vec![
            Token::Def,
            Token::Identifier("add".to_string()),
            Token::LParen,
            Token::Identifier("a".to_string()),
            Token::Colon,
            Token::Identifier("u24".to_string()),
            Token::Comma,
            Token::Identifier("b".to_string()),
            Token::Colon,
            Token::Identifier("u24".to_string()),
            Token::RParen,
            Token::Arrow,
            Token::Identifier("u24".to_string()),
            Token::Colon,
            Token::Return,
            Token::Identifier("a".to_string()),
            Token::Plus,
            Token::Identifier("b".to_string()),
            Token::EOF,
        ];

        for (i, expected) in expected_tokens.iter().enumerate() {
            assert_eq!(&tokens[i].token, expected, "Token {} mismatch", i);
        }
    }

    #[test]
    fn test_error_handling() {
        let mut lexer = BendLexer::new("@invalid");
        let token = lexer.next_token();

        match token.token {
            Token::Error(_) => (), // Expected error
            _ => panic!("Expected error token for invalid input"),
        }
    }
}
