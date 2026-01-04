use std::collections::HashMap;

use super::ast::*;
use crate::compiler::lexer::{
    lexer::{BendLexer, TokenWithPosition},
    token::Token,
};
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
    UnexpectedEOF { expected: String },

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
            // From X import Y, Z
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

                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }

            names
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

    /// Parse a direct import
    fn parse_direct_import(&mut self) -> Result<Import, ParseError> {
        let token = self.expect(Token::Import)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;

        // Parse imported names
        let mut names = Vec::new();

        loop {
            let name_token = self.expect(Token::Identifier(String::new()))?;
            let name = match &name_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };

            names.push(name);

            if !self.check(&Token::Comma) {
                break;
            }
            self.advance();
        }

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
        match &self.current_token.token {
            Token::Fn => self.parse_function_def(),
            Token::Type => self.parse_type_def(),
            Token::Object => self.parse_object_def(),
            Token::Contract => self.parse_contract_def(),
            Token::Interface => self.parse_interface_def(),
            Token::Library => self.parse_library_def(),
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token.token.to_string(),
                expected: "definition keyword".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            }),
        }
    }

    /// Parse a function definition
    fn parse_function_def(&mut self) -> Result<Definition, ParseError> {
        let token = self.expect(Token::Fn)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;

        // Parse function name
        let name_token = self.expect(Token::Identifier(String::new()))?;
        let name = match &name_token.token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        // Parse parameters
        self.expect(Token::LParen)?;
        let mut params = Vec::new();

        while !self.check(&Token::RParen) {
            let param_name_token = self.expect(Token::Identifier(String::new()))?;
            let param_name = match &param_name_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };

            self.expect(Token::Colon)?;
            let param_type = self.parse_type()?;

            params.push(Parameter {
                name: param_name,
                ty: param_type,
                location: Location {
                    line: param_name_token.line,
                    column: param_name_token.column,
                    start: param_name_token.start,
                    end: self.current_token.end,
                },
            });

            if !self.check(&Token::RParen) {
                self.expect(Token::Comma)?;
            }
        }

        self.expect(Token::RParen)?;

        // Parse return type (optional)
        let return_type = if self.check(&Token::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse function body
        let body = if self.check(&Token::LBrace) {
            self.parse_block()?
        } else {
            // External function declaration
            Block {
                statements: Vec::new(),
                location: Location {
                    line: self.current_token.line,
                    column: self.current_token.column,
                    start: self.current_token.start,
                    end: self.current_token.end,
                },
            }
        };

        Ok(Definition::FunctionDef {
            name,
            params,
            return_type,
            body,
            checked: None,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
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
        let type_params = if self.check(&Token::Less) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        // Parse type body
        self.expect(Token::LBrace)?;
        let mut variants = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            let variant_name_token = self.expect(Token::Identifier(String::new()))?;
            let variant_name = match &variant_name_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };

            let mut fields = Vec::new();

            // Parse constructor parameters if present
            if self.check(&Token::LParen) {
                self.advance();
                while !self.check(&Token::RParen) {
                    let field_name = if self.check(&Token::Identifier(String::new())) {
                        let field_name_token = self.expect(Token::Identifier(String::new()))?;
                        match &field_name_token.token {
                            Token::Identifier(s) => Some(s.clone()),
                            _ => unreachable!(),
                        }
                    } else {
                        None
                    };

                    if field_name.is_some() {
                        self.expect(Token::Colon)?;
                    }

                    let field_type = self.parse_type()?;

                    fields.push(Field {
                        name: field_name.unwrap_or_else(|| "_".to_string()),
                        type_annotation: Some(field_type),
                        is_recursive: false,
                        location: Location {
                            line: self.current_token.line,
                            column: self.current_token.column,
                            start: self.current_token.start,
                            end: self.current_token.end,
                        },
                    });

                    if !self.check(&Token::RParen) {
                        self.expect(Token::Comma)?;
                    }
                }
                self.expect(Token::RParen)?;
            }

            variants.push(TypeVariant {
                name: variant_name,
                fields,
                location: Location {
                    line: variant_name_token.line,
                    column: variant_name_token.column,
                    start: variant_name_token.start,
                    end: self.current_token.end,
                },
            });

            if !self.check(&Token::RBrace) {
                self.expect(Token::Comma)?;
            }
        }

        self.expect(Token::RBrace)?;

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
        let type_params = if self.check(&Token::Less) {
            self.parse_type_params()?
        } else {
            Vec::new()
        };

        // Parse object body
        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();
        let mut functions = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            if self.check(&Token::Let) {
                fields.push(self.parse_field()?);
            } else if self.check(&Token::Fn) {
                functions.push(self.parse_function_def()?);
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token.token.to_string(),
                    expected: "field or function".to_string(),
                    line: self.current_token.line,
                    column: self.current_token.column,
                });
            }
        }

        self.expect(Token::RBrace)?;

        Ok(Definition::ObjectDef {
            name,
            type_params,
            fields,
            functions,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }

    /// Parse a field
    fn parse_field(&mut self) -> Result<Field, ParseError> {
        let token = self.expect(Token::Let)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;

        // Parse field name
        let name_token = self.expect(Token::Identifier(String::new()))?;
        let name = match &name_token.token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        // Parse type annotation
        self.expect(Token::Colon)?;
        let type_annotation = self.parse_type()?;

        // Check for recursive marker
        let is_recursive = if self.check(&Token::Tilde) {
            self.advance();
            true
        } else {
            false
        };

        Ok(Field {
            name,
            type_annotation: Some(type_annotation),
            is_recursive,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }

    /// Parse a type
    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;

        match &self.current_token.token {
            Token::Identifier(name) => {
                self.advance();

                // Check for type parameters
                let params = if self.check(&Token::Less) {
                    self.parse_type_args()?
                } else {
                    Vec::new()
                };

                // Check for function type (->)
                if self.check(&Token::Arrow) {
                    self.advance();
                    let result_type = self.parse_type()?;

                    Ok(Type::Function {
                        param: Box::new(Type::Named {
                            name: name.clone(),
                            params,
                            location: Location {
                                line: start_line,
                                column: start_column,
                                start,
                                end: self.current_token.end,
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
                        name: name.clone(),
                        params,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.end,
                        },
                    })
                }
            }
            Token::LParen => {
                // Tuple type
                self.advance();
                let mut elements = Vec::new();

                while !self.check(&Token::RParen) {
                    elements.push(self.parse_type()?);
                    if !self.check(&Token::RParen) {
                        self.expect(Token::Comma)?;
                    }
                }

                self.expect(Token::RParen)?;

                Ok(Type::Tuple {
                    elements,
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::U24 => {
                self.advance();
                Ok(Type::U24 {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::I24 => {
                self.advance();
                Ok(Type::I24 {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::F24 => {
                self.advance();
                Ok(Type::F24 {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::Any => {
                self.advance();
                Ok(Type::Any {
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token.token.to_string(),
                expected: "type".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            }),
        }
    }

    /// Parse type parameters
    fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect(Token::Less)?;
        let mut params = Vec::new();

        while !self.check(&Token::Greater) {
            let param_token = self.expect(Token::Identifier(String::new()))?;
            let param = match &param_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };

            params.push(param);

            if !self.check(&Token::Greater) {
                self.expect(Token::Comma)?;
            }
        }

        self.expect(Token::Greater)?;
        Ok(params)
    }

    /// Parse type arguments
    fn parse_type_args(&mut self) -> Result<Vec<Type>, ParseError> {
        self.expect(Token::Less)?;
        let mut args = Vec::new();

        while !self.check(&Token::Greater) {
            args.push(self.parse_type()?);

            if !self.check(&Token::Greater) {
                self.expect(Token::Comma)?;
            }
        }

        self.expect(Token::Greater)?;
        Ok(args)
    }

    /// Parse a block of statements
    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;

        self.expect(Token::LBrace)?;
        let mut statements = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            statements.push(self.parse_statement()?);
        }

        self.expect(Token::RBrace)?;

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
        match &self.current_token.token {
            Token::Return => self.parse_return_statement(),
            Token::If => self.parse_if_statement(),
            Token::Switch => self.parse_switch_statement(),
            Token::Match => self.parse_match_statement(),
            Token::Fold => self.parse_fold_statement(),
            Token::Bend => self.parse_bend_statement(),
            Token::Open => self.parse_open_statement(),
            Token::With => self.parse_with_statement(),
            Token::Use => self.parse_use_statement(),
            Token::Let => self.parse_let_statement(),
            Token::Def => {
                let def = self.parse_function_def()?;
                let location = def.location().clone();
                Ok(Statement::LocalDef {
                    function_def: Box::new(def),
                    location,
                })
            }
            _ => self.parse_expression_statement(),
        }
    }

    /// Parse a return statement
    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.expect(Token::Return)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;

        let value = if !self.check(&Token::Semicolon) && !self.check(&Token::RBrace) {
            self.parse_expression()?
        } else {
            // Default return value (None)
            Expr::Literal {
                kind: LiteralKind::Uint(0),
                location: Location {
                    line: self.current_token.line,
                    column: self.current_token.column,
                    start: self.current_token.start,
                    end: self.current_token.end,
                },
            }
        };

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

    /// Parse a let statement
    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.expect(Token::Let)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;

        // Parse pattern (for now, just variable name)
        let name_token = self.expect(Token::Identifier(String::new()))?;
        let name = match &name_token.token {
            Token::Identifier(s) => s.clone(),
            _ => unreachable!(),
        };

        // Parse optional type annotation
        let ty = if self.check(&Token::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };

        // Parse assignment
        self.expect(Token::Assign)?;
        let value = self.parse_expression()?;

        Ok(Statement::Use {
            name,
            value,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }

    /// Parse an expression statement
    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;

        let expr = self.parse_expression()?;

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

    /// Parse an expression
    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_primary_expression()
    }

    /// Parse a primary expression
    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;

        match &self.current_token.token {
            Token::Identifier(name) => {
                self.advance();
                Ok(Expr::Variable {
                    name: name.clone(),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::Uint(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Uint(*value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::Int(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Int(*value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::Float(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Float(*value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::String(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::String(value.clone()),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::Char(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Char(*value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::Symbol(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Symbol(value.clone()),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Token::LBrace => {
                let block = self.parse_block()?;
                Ok(Expr::Block {
                    block,
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token.token.to_string(),
                expected: "expression".to_string(),
                line: self.current_token.line,
                column: self.current_token.column,
            }),
        }
    }

    // Placeholder implementations for other statement types
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "If statements not implemented yet".to_string(),
        ))
    }

    fn parse_switch_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "Switch statements not implemented yet".to_string(),
        ))
    }

    fn parse_match_statement(&mut self) -> Result<Statement, ParseError> {
        // For now, just return a placeholder implementation
        // This will be expanded once the basic infrastructure is working
        Err(ParseError::Generic(
            "Match statements not implemented yet".to_string(),
        ))
    }

    fn parse_fold_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "Fold statements not implemented yet".to_string(),
        ))
    }

    fn parse_bend_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "Bend statements not implemented yet".to_string(),
        ))
    }

    fn parse_open_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "Open statements not implemented yet".to_string(),
        ))
    }

    fn parse_with_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "With statements not implemented yet".to_string(),
        ))
    }

    fn parse_use_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "Use statements not implemented yet".to_string(),
        ))
    }

    fn parse_contract_def(&mut self) -> Result<Definition, ParseError> {
        Err(ParseError::Generic(
            "Contract definitions not implemented yet".to_string(),
        ))
    }

    fn parse_interface_def(&mut self) -> Result<Definition, ParseError> {
        Err(ParseError::Generic(
            "Interface definitions not implemented yet".to_string(),
        ))
    }

    fn parse_library_def(&mut self) -> Result<Definition, ParseError> {
        Err(ParseError::Generic(
            "Library definitions not implemented yet".to_string(),
        ))
    }

    /// Parse a pattern for pattern matching
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        let start_location = self.current_token.location.clone();

        match &self.current_token.token {
            Token::Identifier(name) => {
                self.advance();
                Ok(Pattern::Variable {
                    name: name.clone(),
                    location: start_location,
                })
            }
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard {
                    location: start_location,
                })
            }
            Token::LeftParen => {
                self.advance();
                let mut elements = Vec::new();

                while !self.check_token(&Token::RightParen) {
                    let element = self.parse_pattern()?;
                    elements.push(element);

                    if !self.check_token(&Token::RightParen) {
                        self.expect_token(Token::Comma)?;
                    }
                }

                self.expect_token(Token::RightParen)?;
                let end_location = self.previous_token.location.clone();
                let location = Location::span(&start_location, &end_location);

                Ok(Pattern::Tuple { elements, location })
            }
            Token::Identifier(name) => {
                // Could be a constructor pattern
                self.advance();

                if self.check_token(&Token::LeftBrace) {
                    // Constructor with named fields
                    self.advance();
                    let mut fields = HashMap::new();

                    while !self.check_token(&Token::RightBrace) {
                        let field_name = self.expect_identifier()?;
                        self.expect_token(Token::Colon)?;
                        let field_pattern = self.parse_pattern()?;
                        fields.insert(field_name, field_pattern);

                        if !self.check_token(&Token::RightBrace) {
                            self.expect_token(Token::Comma)?;
                        }
                    }

                    self.expect_token(Token::RightBrace)?;
                    let end_location = self.previous_token.location.clone();
                    let location = Location::span(&start_location, &end_location);

                    Ok(Pattern::Constructor {
                        name: name.clone(),
                        fields,
                        location,
                    })
                } else {
                    // Simple variable pattern
                    Ok(Pattern::Variable {
                        name: name.clone(),
                        location: start_location,
                    })
                }
            }
            _ => {
                // Try to parse as a literal pattern
                let expr = self.parse_primary_expression()?;
                if let Expr::Literal { kind, location } = expr {
                    Ok(Pattern::Literal {
                        value: Expr::Literal {
                            kind,
                            location: location.clone(),
                        },
                        location,
                    })
                } else {
                    Err(ParseError::Generic(format!("Invalid pattern: {:?}", expr)))
                }
            }
        }
    }
}
