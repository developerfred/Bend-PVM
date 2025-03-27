use std::collections::HashMap;

use crate::compiler::lexer::{
    token::Token,
    lexer::{BendLexer, TokenWithPosition},
};
use super::ast::*;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Unexpected token {found} at line {line}, column {column}, expected {expected}")]
    UnexpectedToken {
        found: String,
        expected: String,
        line: usize,
        column: usize,
    },
    
    #[error("Unexpected end of input, expected {expected}")]
    UnexpectedEOF {
        expected: String,
    },
    
    #[error("Lexical error: {0}")]
    LexicalError(String),
    
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    
    #[error("Parse error: {0}")]
    Generic(String),
}

pub struct Parser<'a> {
    lexer: BendLexer<'a>,
    current_token: TokenWithPosition,
    peek_token: TokenWithPosition,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = BendLexer::new(source);
        let current_token = lexer.next_token();
        let peek_token = lexer.next_token();
        
        Parser {
            lexer,
            current_token,
            peek_token,
        }
    }
    
    /// Advance to the next token
    fn advance(&mut self) {
        self.current_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }
    
    /// Check if the current token matches the expected token
    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(&self.current_token.token) == std::mem::discriminant(expected)
    }
    
    /// Expect and consume a token, or return an error
    fn expect(&mut self, expected: Token) -> Result<TokenWithPosition, ParseError> {
        if self.check(&expected) {
            let token = self.current_token.clone();
            self.advance();
            Ok(token)
        } else {
            Err(ParseError::UnexpectedToken {
                found: self.current_token.token.to_string(),
                expected: expected.to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            })
        }
    }
    
    /// Parse a complete program
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;
        
        let mut imports = Vec::new();
        let mut definitions = Vec::new();
        
        // Parse imports
        while self.check(&Token::Import) || self.check(&Token::From) {
            imports.push(self.parse_import()?);
        }
        
        // Parse top-level definitions
        while !self.check(&Token::EOF) {
            definitions.push(self.parse_definition()?);
        }
        
        let end = self.current_token.end;
        
        Ok(Program {
            imports,
            definitions,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end,
            },
        })
    }
    
    /// Parse an import statement
    fn parse_import(&mut self) -> Result<Import, ParseError> {
        if self.check(&Token::From) {
            self.parse_from_import()
        } else {
            self.parse_direct_import()
        }
    }
    
    /// Parse a 'from X import Y' style import
    fn parse_from_import(&mut self) -> Result<Import, ParseError> {
        let token = self.expect(Token::From)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;
        
        // Parse the module path
        let path_token = self.expect(Token::Identifier(String::new()))?;
        let path = match &path_token.token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };
        
        // Expect 'import'
        self.expect(Token::Import)?;
        
        // Parse imported names
        let names = if self.check(&Token::Star) {
            // From X import *
            self.advance();
            vec![ImportName {
                name: "*".to_string(),
                alias: None,
                location: Location {
                    line: self.current_token.line,
                    column: self.current_token.column,
                    start: self.current_token.start,
                    end: self.current_token.end,
                },
            }]
        } else if self.check(&Token::LParen) {
            // From X import (Y, Z)
            self.advance();
            let mut names = Vec::new();
            
            loop {
                let name_token = self.expect(Token::Identifier(String::new()))?;
                let name = match &name_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                
                let mut alias = None;
                
                // Check for alias
                if self.check(&Token::As) {
                    self.advance();
                    let alias_token = self.expect(Token::Identifier(String::new()))?;
                    alias = match &alias_token.token {
                        Token::Identifier(s) => Some(s.clone()),
                        _ => unreachable!(),
                    };
                }
                
                names.push(ImportName {
                    name,
                    alias,
                    location: Location {
                        line: name_token.line,
                        column: name_token.column,
                        start: name_token.start,
                        end: self.current_token.end,
                    },
                });
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
                
                self.expect(Token::Comma)?;
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
            }
            
            names
        } else {
            // From X import Y
            let name_token = self.expect(Token::Identifier(String::new()))?;
            let name = match &name_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };
            
            let mut alias = None;
            
            // Check for alias
            if self.check(&Token::As) {
                self.advance();
                let alias_token = self.expect(Token::Identifier(String::new()))?;
                alias = match &alias_token.token {
                    Token::Identifier(s) => Some(s.clone()),
                    _ => unreachable!(),
                };
            }
            
            vec![ImportName {
                name,
                alias,
                location: Location {
                    line: name_token.line,
                    column: name_token.column,
                    start: name_token.start,
                    end: self.current_token.end,
                },
            }]
        };
        
        Ok(Import::FromImport {
            path,
            names,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }
    
    /// Parse a direct import statement
    fn parse_direct_import(&mut self) -> Result<Import, ParseError> {
        let token = self.expect(Token::Import)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;
        
        // Parse imported names
        let names = if self.check(&Token::LParen) {
            // Import (X, Y, Z)
            self.advance();
            let mut names = Vec::new();
            
            loop {
                let name_token = self.expect(Token::Identifier(String::new()))?;
                let name = match &name_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                
                names.push(name);
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
                
                self.expect(Token::Comma)?;
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
            }
            
            names
        } else {
            // Import X
            let name_token = self.expect(Token::Identifier(String::new()))?;
            let name = match &name_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };
            
            vec![name]
        };
        
        Ok(Import::DirectImport {
            names,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }
    
    /// Parse a top-level definition
    fn parse_definition(&mut self) -> Result<Definition, ParseError> {
        match self.current_token.token {
            Token::Def => self.parse_function_def(),
            Token::Type => self.parse_type_def(),
            Token::Object => self.parse_object_def(),
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token.token.to_string(),
                expected: "def, type, or object".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            }),
        }
    }
    
    /// Parse a function definition
    fn parse_function_def(&mut self) -> Result<Definition, ParseError> {
        let token = self.expect(Token::Def)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;
        
        // Check for 'checked' or 'unchecked' modifier
        let mut checked = None;
        if self.check(&Token::Identifier(String::new())) {
            match &self.current_token.token {
                Token::Identifier(s) if s == "checked" => {
                    checked = Some(true);
                    self.advance();
                }
                Token::Identifier(s) if s == "unchecked" => {
                    checked = Some(false);
                    self.advance();
                }
                _ => {}
            }
        }
        
        // Parse function name
        let name_token = self.expect(Token::Identifier(String::new()))?;
        let name = match &name_token.token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };
        
        // Parse parameters
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        
        if !self.check(&Token::RParen) {
            loop {
                let param_name_token = self.expect(Token::Identifier(String::new()))?;
                let param_name = match &param_name_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                
                // Check for type annotation
                let mut type_annotation = None;
                if self.check(&Token::Colon) {
                    self.advance();
                    type_annotation = Some(self.parse_type()?);
                }
                
                params.push(Parameter {
                    name: param_name,
                    type_annotation,
                    location: Location {
                        line: param_name_token.line,
                        column: param_name_token.column,
                        start: param_name_token.start,
                        end: self.current_token.end,
                    },
                });
                
                if self.check(&Token::RParen) {
                    break;
                }
                
                self.expect(Token::Comma)?;
                
                if self.check(&Token::RParen) {
                    break;
                }
            }
        }
        
        self.expect(Token::RParen)?;
        
        // Parse return type (optional)
        let mut return_type = None;
        if self.check(&Token::Arrow) {
            self.advance();
            return_type = Some(self.parse_type()?);
        }
        
        // Parse function body
        self.expect(Token::Colon)?;
        let body = self.parse_block()?;
        
        Ok(Definition::FunctionDef {
            name,
            params,
            return_type,
            body,
            checked,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: body.location.end,
            },
        })
    }
    
    /// Parse a type definition
    fn parse_type_def(&mut self) -> Result<Definition, ParseError> {
        let token = self.expect(Token::Type)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;
        
        // Parse type name
        let name_token = self.expect(Token::Identifier(String::new()))?;
        let name = match &name_token.token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };
        
        // Parse type parameters (optional)
        let mut type_params = Vec::new();
        
        if self.check(&Token::LParen) {
            self.advance();
            
            loop {
                let param_token = self.expect(Token::Identifier(String::new()))?;
                let param = match &param_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                
                type_params.push(param);
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
                
                self.expect(Token::Comma)?;
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
            }
        }
        
        // Expect colon after type header
        self.expect(Token::Colon)?;
        
        // Parse variants
        let mut variants = Vec::new();
        
        loop {
            let variant_token = self.expect(Token::Identifier(String::new()))?;
            let variant_name = match &variant_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };
            
            let variant_start = variant_token.start;
            let variant_line = variant_token.line;
            let variant_column = variant_token.column;
            
            // Parse fields (optional)
            let mut fields = Vec::new();
            
            if self.check(&Token::LBrace) {
                self.advance();
                
                loop {
                    let is_recursive = self.check(&Token::Tilde);
                    if is_recursive {
                        self.advance();
                    }
                    
                    let field_name_token = self.expect(Token::Identifier(String::new()))?;
                    let field_name = match &field_name_token.token {
                        Token::Identifier(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    
                    // Check for type annotation
                    let mut type_annotation = None;
                    if self.check(&Token::Colon) {
                        self.advance();
                        type_annotation = Some(self.parse_type()?);
                    }
                    
                    fields.push(Field {
                        name: field_name,
                        type_annotation,
                        is_recursive,
                        location: Location {
                            line: field_name_token.line,
                            column: field_name_token.column,
                            start: field_name_token.start,
                            end: self.current_token.end,
                        },
                    });
                    
                    if self.check(&Token::RBrace) {
                        self.advance();
                        break;
                    }
                    
                    self.expect(Token::Comma)?;
                    
                    if self.check(&Token::RBrace) {
                        self.advance();
                        break;
                    }
                }
            }
            
            variants.push(TypeVariant {
                name: variant_name,
                fields,
                location: Location {
                    line: variant_line,
                    column: variant_column,
                    start: variant_start,
                    end: self.current_token.end,
                },
            });
            
            // If the next token is not an identifier, we're done parsing variants
            if !matches!(self.current_token.token, Token::Identifier(_)) {
                break;
            }
        }
        
        Ok(Definition::TypeDef {
            name,
            type_params,
            variants,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }
    
    /// Parse an object definition
    fn parse_object_def(&mut self) -> Result<Definition, ParseError> {
        let token = self.expect(Token::Object)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;
        
        // Parse object name
        let name_token = self.expect(Token::Identifier(String::new()))?;
        let name = match &name_token.token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };
        
        // Parse type parameters (optional)
        let mut type_params = Vec::new();
        
        if self.check(&Token::LParen) {
            self.advance();
            
            loop {
                let param_token = self.expect(Token::Identifier(String::new()))?;
                let param = match &param_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                
                type_params.push(param);
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
                
                self.expect(Token::Comma)?;
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
            }
        }
        
        // Parse fields
        self.expect(Token::LBrace)?;
        
        let mut fields = Vec::new();
        
        if !self.check(&Token::RBrace) {
            loop {
                let is_recursive = self.check(&Token::Tilde);
                if is_recursive {
                    self.advance();
                }
                
                let field_name_token = self.expect(Token::Identifier(String::new()))?;
                let field_name = match &field_name_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };
                
                // Check for type annotation
                let mut type_annotation = None;
                if self.check(&Token::Colon) {
                    self.advance();
                    type_annotation = Some(self.parse_type()?);
                }
                
                fields.push(Field {
                    name: field_name,
                    type_annotation,
                    is_recursive,
                    location: Location {
                        line: field_name_token.line,
                        column: field_name_token.column,
                        start: field_name_token.start,
                        end: self.current_token.end,
                    },
                });
                
                if self.check(&Token::RBrace) {
                    break;
                }
                
                self.expect(Token::Comma)?;
                
                if self.check(&Token::RBrace) {
                    break;
                }
            }
        }
        
        self.expect(Token::RBrace)?;
        
        Ok(Definition::ObjectDef {
            name,
            type_params,
            fields,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }
    
    /// Parse a type annotation
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;
        
        if self.check(&Token::LParen) {
            // Tuple type or parenthesized type
            self.advance();
            
            let first_type = self.parse_type()?;
            
            if self.check(&Token::Comma) {
                // Tuple type
                self.advance();
                
                let mut elements = vec![first_type];
                
                loop {
                    elements.push(self.parse_type()?);
                    
                    if self.check(&Token::RParen) {
                        self.advance();
                        break;
                    }
                    
                    self.expect(Token::Comma)?;
                    
                    if self.check(&Token::RParen) {
                        self.advance();
                        break;
                    }
                }
                
                Ok(Type::Tuple {
                    elements,
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            } else {
                // Parenthesized type
                self.expect(Token::RParen)?;
                Ok(first_type)
            }
        } else if self.check(&Token::Identifier(String::new())) {
            // Named type or built-in type
            let name_token = self.expect(Token::Identifier(String::new()))?;
            let name = match &name_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };
            
            match name.as_str() {
                "Any" => Ok(Type::Any {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                }),
                "None" => Ok(Type::None {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                }),
                "_" => Ok(Type::Hole {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                }),
                "u24" => Ok(Type::U24 {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                }),
                "i24" => Ok(Type::I24 {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                }),
                "f24" => Ok(Type::F24 {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                }),
                _ => {
                    // Check for type parameters
                    let mut params = Vec::new();
                    
                    if self.check(&Token::LParen) {
                        self.advance();
                        
                        loop {
                            params.push(self.parse_type()?);
                            
                            if self.check(&Token::RParen) {
                                self.advance();
                                break;
                            }
                            
                            self.expect(Token::Comma)?;
                            
                            if self.check(&Token::RParen) {
                                self.advance();
                                break;
                            }
                        }
                    }
                    
                    let type_end = self.current_token.end;
                    
                    // Check for function type (->)
                    if self.check(&Token::Arrow) {
                        self.advance();
                        
                        let result_type = self.parse_type()?;
                        
                        Ok(Type::Function {
                            param: Box::new(Type::Named {
                                name,
                                params,
                                location: Location {
                                    line: start_line,
                                    column: start_column,
                                    start,
                                    end: type_end,
                                },
                            }),
                            result: Box::new(result_type),
                            location: Location {
                                line: start_line,
                                column: start_column,
                                start,
                                end: self.current_token.end,
                            },
                        })
                    } else {
                        Ok(Type::Named {
                            name,
                            params,
                            location: Location {
                                line: start_line,
                                column: start_column,
                                start,
                                end: type_end,
                            },
                        })
                    }
                }
            }
        } else {
            Err(ParseError::UnexpectedToken {
                found: self.current_token.token.to_string(),
                expected: "type".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            })
        }
    }
    
    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;
        
        let mut statements = Vec::new();
        
        // Parse statements until we reach the end of the block
        while !self.check(&Token::EOF) && 
              !self.check(&Token::RBrace) &&
              !self.check(&Token::Else) &&
              !self.check(&Token::Case) {
            statements.push(self.parse_statement()?);
        }
        
        Ok(Block {
            statements,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }
    
    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current_token.token {
            Token::Return => self.parse_return_statement(),
            Token::If => self.parse_if_statement(),
            Token::Switch => self.parse_switch_statement(),
            Token::Match => self.parse_match_statement(),
            Token::Fold => self.parse_fold_statement(),
            Token::Bend => self.parse_bend_statement(),
            Token::Open => self.parse_open_statement(),
            Token::With => self.parse_with_statement(),
            Token::Use => self.parse_use_statement(),
            Token::Def => {
                let def = self.parse_function_def()?;
                Ok(Statement::LocalDef {
                    function_def: Box::new(def),
                    location: def.location().clone(),
                })
            },
            _ => {
                // Try to parse as an assignment, in-place operation, or expression statement
                let start = self.current_token.start;
                let start_line = self.current_token.line;
                let start_column = self.current_token.column;
                
                let expr = self.parse_expr()?;
                
                // Check if this is an assignment or an expression statement
                if self.check(&Token::Equal) {
                    self.advance();
                    
                    let value = self.parse_expr()?;
                    
                    // Convert the expression to a pattern
                    let pattern = self.expr_to_pattern(expr.clone())?;
                    
                    Ok(Statement::Assignment {
                        pattern,
                        value,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.end,
                        },
                    })
                } else if let Token::PlusEqual | Token::MinusEqual | Token::StarEqual | 
                              Token::SlashEqual | Token::PercentEqual | Token::CaretEqual | 
                              Token::AmpersandEqual | Token::PipeEqual = self.current_token.token {
                    // In-place operation
                    let operator = match self.current_token.token {
                        Token::PlusEqual => InPlaceOperator::Add,
                        Token::MinusEqual => InPlaceOperator::Sub,
                        Token::StarEqual => InPlaceOperator::Mul,
                        Token::SlashEqual => InPlaceOperator::Div,
                        Token::PercentEqual => InPlaceOperator::Mod,
                        Token::AmpersandEqual => InPlaceOperator::BitAnd,
                        Token::PipeEqual => InPlaceOperator::BitOr,
                        Token::CaretEqual => InPlaceOperator::BitXor,
                        _ => unreachable!(),
                    };
                    
                    self.advance();
                    
                    let value = self.parse_expr()?;
                    
                    // Convert the expression to a target name
                    let target = match expr {
                        Expr::Variable { name, .. } => name,
                        _ => return Err(ParseError::InvalidPattern(
                            "In-place operations require a variable as the target".to_string()
                        )),
                    };
                    
                    Ok(Statement::InPlaceOp {
                        target,
                        operator,
                        value,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.end,
                        },
                    })
                } else {
                    // Expression statement
                    Ok(Statement::Expr {
                        expr,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.end,
                        },
                    })
                }
            }
        }
    }
    
    /// Convert an expression to a pattern for assignment
    fn expr_to_pattern(&self, expr: Expr) -> Result<Pattern, ParseError> {
        match expr {
            Expr::Variable { name, location } => {
                Ok(Pattern::Variable { name, location })
            },
            Expr::Tuple { elements, location } => {
                let mut pattern_elements = Vec::new();
                for element in elements {
                    pattern_elements.push(self.expr_to_pattern(element)?);
                }
                Ok(Pattern::Tuple { elements: pattern_elements, location })
            },
            Expr::Constructor { name, args, named_args, location } => {
                let mut fields = HashMap::new();
                
                // Convert positional args to named args based on field order (not ideal)
                for (i, arg) in args.into_iter().enumerate() {
                    let field_name = format!("_{}", i);
                    fields.insert(field_name, self.expr_to_pattern(arg)?);
                }
                
                // Add named args
                for (name, arg) in named_args {
                    fields.insert(name, self.expr_to_pattern(arg)?);
                }
                
                Ok(Pattern::Constructor { name, fields, location })
            },
            Expr::Literal { kind, location } => {
                Ok(Pattern::Literal { 
                    value: Expr::Literal { kind, location: location.clone() }, 
                    location 
                })
            },
            Expr::Eraser { location } => {
                Ok(Pattern::Wildcard { location })
            },
            _ => Err(ParseError::InvalidPattern(
                "Invalid pattern in assignment".to_string()
            )),
        }
    }
    
    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.expect(Token::Return)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;
        
        let value = self.parse_expr()?;
        
        Ok(Statement::Return {
            value,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }
    
    /// Parse expressions
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        // For now, we'll keep this simple
        self.parse_primary_expr()
    }
    
    /// Parse a primary expression (variables, literals, etc.)
    fn parse_primary_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;
        
        match &self.current_token.token {
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                
                // Check for function call, access, etc.
                if self.check(&Token::LParen) {
                    self.parse_function_call(Expr::Variable {
                        name,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.start,
                        },
                    })
                } else {
                    Ok(Expr::Variable {
                        name,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.start,
                        },
                    })
                }
            },
            Token::UintLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Uint(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            Token::IntLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Int(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            Token::FloatLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Float(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            Token::StringLiteral(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::String(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            Token::CharLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Char(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            Token::SymbolLiteral(value) => {
                let value = value.clone();
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Symbol(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            Token::LParen => {
                self.advance();
                
                // Check for empty tuple (), which is equivalent to the eraser *
                if self.check(&Token::RParen) {
                    self.advance();
                    return Ok(Expr::Eraser {
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.start,
                        },
                    });
                }
                
                let first_expr = self.parse_expr()?;
                
                if self.check(&Token::Comma) {
                    // This is a tuple
                    self.advance();
                    
                    let mut elements = vec![first_expr];
                    
                    loop {
                        elements.push(self.parse_expr()?);
                        
                        if self.check(&Token::RParen) {
                            self.advance();
                            break;
                        }
                        
                        self.expect(Token::Comma)?;
                        
                        if self.check(&Token::RParen) {
                            self.advance();
                            break;
                        }
                    }
                    
                    Ok(Expr::Tuple {
                        elements,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.start,
                        },
                    })
                } else {
                    // This is a parenthesized expression
                    self.expect(Token::RParen)?;
                    Ok(first_expr)
                }
            },
            Token::LBracket => {
                self.advance();
                
                let mut elements = Vec::new();
                
                if !self.check(&Token::RBracket) {
                    loop {
                        elements.push(self.parse_expr()?);
                        
                        if self.check(&Token::RBracket) {
                            self.advance();
                            break;
                        }
                        
                        self.expect(Token::Comma)?;
                        
                        if self.check(&Token::RBracket) {
                            self.advance();
                            break;
                        }
                    }
                } else {
                    self.advance();
                }
                
                Ok(Expr::List {
                    elements,
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            Token::Star => {
                self.advance();
                Ok(Expr::Eraser {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            Token::Lambda => {
                self.advance();
                
                // Parse parameters
                let mut params = Vec::new();
                
                if self.check(&Token::Identifier(String::new())) {
                    let param_token = self.current_token.clone();
                    self.advance();
                    
                    let param_name = match &param_token.token {
                        Token::Identifier(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    
                    params.push(Parameter {
                        name: param_name,
                        type_annotation: None,
                        location: Location {
                            line: param_token.line,
                            column: param_token.column,
                            start: param_token.start,
                            end: param_token.end,
                        },
                    });
                    
                    // Parse additional parameters
                    while self.check(&Token::Comma) {
                        self.advance();
                        
                        let param_token = self.expect(Token::Identifier(String::new()))?;
                        let param_name = match &param_token.token {
                            Token::Identifier(s) => s.clone(),
                            _ => unreachable!(),
                        };
                        
                        params.push(Parameter {
                            name: param_name,
                            type_annotation: None,
                            location: Location {
                                line: param_token.line,
                                column: param_token.column,
                                start: param_token.start,
                                end: param_token.end,
                            },
                        });
                    }
                }
                
                // Parse lambda body
                self.expect(Token::Colon)?;
                let body = self.parse_expr()?;
                
                Ok(Expr::Lambda {
                    params,
                    body: Box::new(body),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.start,
                    },
                })
            },
            // Add more cases for other expression types
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token.token.to_string(),
                expected: "expression".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            }),
        }
    }
    
    /// Parse a function call
    fn parse_function_call(&mut self, function: Expr) -> Result<Expr, ParseError> {
        let start = function.location().start;
        let start_line = function.location().line;
        let start_column = function.location().column;
        
        self.expect(Token::LParen)?;
        
        let mut args = Vec::new();
        let mut named_args = HashMap::new();
        
        if !self.check(&Token::RParen) {
            loop {
                // Check for named arguments
                if self.check(&Token::Identifier(String::new())) && 
                   matches!(self.peek_token.token, Token::Equal) {
                    let name_token = self.expect(Token::Identifier(String::new()))?;
                    let name = match &name_token.token {
                        Token::Identifier(s) => s.clone(),
                        _ => unreachable!(),
                    };
                    
                    self.expect(Token::Equal)?;
                    
                    let value = self.parse_expr()?;
                    
                    named_args.insert(name, value);
                } else {
                    args.push(self.parse_expr()?);
                }
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
                
                self.expect(Token::Comma)?;
                
                if self.check(&Token::RParen) {
                    self.advance();
                    break;
                }
            }
        } else {
            self.advance();
        }
        
        Ok(Expr::FunctionCall {
            function: Box::new(function),
            args,
            named_args,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.start,
            },
        })
    }
    
    // Add methods for parsing other statement types (if, match, fold, etc.)
    // For brevity, these are not implemented here
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        // Not implementing all statement types for brevity
        Err(ParseError::Generic("Not implemented: parse_if_statement".to_string()))
    }
    
    fn parse_switch_statement(&mut self) -> Result<Statement, ParseError> {
        // Not implementing all statement types for brevity
        Err(ParseError::Generic("Not implemented: parse_switch_statement".to_string()))
    }
    
    fn parse_match_statement(&mut self) -> Result<Statement, ParseError> {
        // Not implementing all statement types for brevity
        Err(ParseError::Generic("Not implemented: parse_match_statement".to_string()))
    }
    
    fn parse_fold_statement(&mut self) -> Result<Statement, ParseError> {
        // Not implementing all statement types for brevity
        Err(ParseError::Generic("Not implemented: parse_fold_statement".to_string()))
    }
    
    fn parse_bend_statement(&mut self) -> Result<Statement, ParseError> {
        // Not implementing all statement types for brevity
        Err(ParseError::Generic("Not implemented: parse_bend_statement".to_string()))
    }
    
    fn parse_open_statement(&mut self) -> Result<Statement, ParseError> {
        // Not implementing all statement types for brevity
        Err(ParseError::Generic("Not implemented: parse_open_statement".to_string()))
    }
    
    fn parse_with_statement(&mut self) -> Result<Statement, ParseError> {
        // Not implementing all statement types for brevity
        Err(ParseError::Generic("Not implemented: parse_with_statement".to_string()))
    }
    
    fn parse_use_statement(&mut self) -> Result<Statement, ParseError> {
        // Not implementing all statement types for brevity
        Err(ParseError::Generic("Not implemented: parse_use_statement".to_string()))
    }
}

/// Helper trait to get the location of an AST node
trait LocationProvider {
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