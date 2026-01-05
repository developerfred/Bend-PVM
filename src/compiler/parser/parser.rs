#![allow(clippy::only_used_in_recursion)]

use std::collections::HashMap;

use super::ast::*;

use crate::compiler::lexer::lexer::{BendLexer, TokenWithPosition};
use crate::compiler::lexer::token::Token;
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

    /// Check if the peek token is a colon
    fn peek_is_colon(&self) -> bool {
        matches!(self.peek_token.token, Token::Colon)
    }

    /// Check if the peek token is an equals sign
    fn peek_is_eq(&self) -> bool {
        matches!(self.peek_token.token, Token::Equal)
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

        if self.check(&Token::Semicolon) {
            self.advance();
        }

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

        if self.check(&Token::Semicolon) {
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
        let token = self.current_token.token.clone();
        match token {
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
        let type_params = if self.check(&Token::LessThan) {
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
                    // Check if we have a field name followed by colon (field: Type)
                    // or just a type directly (Type)
                    if self.check(&Token::Identifier(String::new())) && self.peek_is_colon() {
                        // Field name with type annotation: field: Type
                        let field_name_token = self.expect(Token::Identifier(String::new()))?;
                        let field_name = match &field_name_token.token {
                            Token::Identifier(s) => s.clone(),
                            _ => unreachable!(),
                        };

                        self.expect(Token::Colon)?;
                        let field_type = self.parse_type()?;

                        fields.push(Field {
                            name: field_name,
                            type_annotation: Some(field_type),
                            is_recursive: false,
                            location: Location {
                                line: self.current_token.line,
                                column: self.current_token.column,
                                start: self.current_token.start,
                                end: self.current_token.end,
                            },
                        });
                    } else {
                        // Just type without field name: Type
                        let field_type = self.parse_type()?;
                        fields.push(Field {
                            name: "_".to_string(),
                            type_annotation: Some(field_type),
                            is_recursive: false,
                            location: Location {
                                line: self.current_token.line,
                                column: self.current_token.column,
                                start: self.current_token.start,
                                end: self.current_token.end,
                            },
                        });
                    }

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
        let type_params = if self.check(&Token::LessThan) {
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

        // Consume semicolon at end of field declaration
        self.expect(Token::Semicolon)?;

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

        let token = self.current_token.token.clone();
        match token {
            Token::Identifier(name) => {
                self.advance();

                // Check for type parameters
                let params = if self.check(&Token::LessThan) {
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
        self.expect(Token::LessThan)?;
        let mut params = Vec::new();

        while !self.check(&Token::GreaterThan) {
            let param_token = self.expect(Token::Identifier(String::new()))?;
            let param = match &param_token.token {
                Token::Identifier(s) => s.clone(),
                _ => unreachable!(),
            };

            params.push(param);

            if !self.check(&Token::GreaterThan) {
                self.expect(Token::Comma)?;
            }
        }

        self.expect(Token::GreaterThan)?;
        Ok(params)
    }

    /// Parse type arguments
    fn parse_type_args(&mut self) -> Result<Vec<Type>, ParseError> {
        self.expect(Token::LessThan)?;
        let mut args = Vec::new();

        while !self.check(&Token::GreaterThan) {
            args.push(self.parse_type()?);

            if !self.check(&Token::GreaterThan) {
                self.expect(Token::Comma)?;
            }
        }

        self.expect(Token::GreaterThan)?;
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

    fn expr_to_pattern(&self, expr: Expr) -> Result<Pattern, ParseError> {
        match expr {
            Expr::Variable { name, location } => Ok(Pattern::Variable { name, location }),
            Expr::FieldAccess {
                object,
                field,
                location,
            } => {
                let parent = self.expr_to_pattern(*object)?;
                Ok(Pattern::Member {
                    parent: Box::new(parent),
                    member: field,
                    location,
                })
            }
            _ => Err(ParseError::Generic(format!(
                "Invalid assignment target: {:?}",
                expr
            ))),
        }
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.current_token.token.clone();
        match token {
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
            Token::Try => self.parse_try_catch_statement(),
            Token::Def => {
                let def = self.parse_function_def()?;
                let location = def.location().clone();
                Ok(Statement::LocalDef {
                    function_def: Box::new(def),
                    location,
                })
            }
            _ => {
                let start_token = self.current_token.clone();
                let start_line = start_token.line;
                let start_column = start_token.column;
                let start_pos = start_token.start;

                let expr = self.parse_expression()?;

                if self.check(&Token::Equal) {
                    self.advance();
                    let value = self.parse_expression()?;

                    let pattern = self.expr_to_pattern(expr)?;

                    if self.check(&Token::Semicolon) {
                        self.advance();
                    }

                    Ok(Statement::Assignment {
                        pattern,
                        value,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start: start_pos,
                            end: self.current_token.end,
                        },
                    })
                } else {
                    if self.check(&Token::Semicolon) {
                        self.advance();
                    }

                    Ok(Statement::Expr {
                        expr,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start: start_pos,
                            end: self.current_token.end,
                        },
                    })
                }
            }
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

        if self.check(&Token::Semicolon) {
            self.advance();
        }

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
        let _ty = if self.check(&Token::Colon) {
            self.advance();
            let ty = self.parse_type()?;
            eprintln!("DEBUG parse_let: parsed type annotation successfully");
            Some(ty)
        } else {
            eprintln!("DEBUG parse_let: no type annotation found");
            None
        };

        // Parse assignment
        eprintln!(
            "DEBUG parse_let: checking for = token, current token is: {:?}",
            self.current_token.token
        );
        self.expect(Token::Equal)?;
        let value = self.parse_expression()?;
        eprintln!("DEBUG parse_let: parsed value expression successfully");

        if self.check(&Token::Semicolon) {
            self.advance();
        }

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

        if self.check(&Token::Semicolon) {
            self.advance();
        }

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
        self.parse_binary_expression(0)
    }

    /// Parse a primary expression (literals, variables, etc.)
    fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        let start = self.current_token.start;
        let start_line = self.current_token.line;
        let start_column = self.current_token.column;

        let token = self.current_token.token.clone();
        match token {
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
            Token::UintLiteral(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Uint(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::IntLiteral(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Int(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::FloatLiteral(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Float(f32::from_bits(value)),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::StringLiteral(value) => {
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
            Token::CharLiteral(value) => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Char(value),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::SymbolLiteral(value) => {
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
            Token::True => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Bool(true),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::False => {
                self.advance();
                Ok(Expr::Literal {
                    kind: LiteralKind::Bool(false),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::LParen => {
                self.advance(); // consume '('

                // Check if it's a tuple or just a parenthesized expression
                let first_expr = self.parse_expression()?;

                if self.check(&Token::Comma) {
                    // It's a tuple
                    let mut elements = vec![first_expr];

                    while self.check(&Token::Comma) {
                        self.advance(); // consume ','
                        elements.push(self.parse_expression()?);
                    }

                    self.expect(Token::RParen)?;

                    Ok(Expr::Tuple {
                        elements,
                        location: Location {
                            line: start_line,
                            column: start_column,
                            start,
                            end: self.current_token.end,
                        },
                    })
                } else {
                    // It's a parenthesized expression
                    self.expect(Token::RParen)?;
                    Ok(first_expr)
                }
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
            Token::If => {
                self.advance();
                let condition = self.parse_expression()?;
                let then_branch = self.parse_expression()?;
                self.expect(Token::Else)?;
                let else_branch = self.parse_expression()?;
                Ok(Expr::If {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: Box::new(else_branch),
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::LBracket => {
                self.advance(); // consume '['
                let mut elements = Vec::new();
                if !self.check(&Token::RBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance(); // consume ','
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::Array {
                    elements,
                    location: Location {
                        line: start_line,
                        column: start_column,
                        start,
                        end: self.current_token.end,
                    },
                })
            }
            Token::Pipe => {
                self.advance(); // consume opening '|'

                // Parse lambda parameters
                let mut params = Vec::new();

                loop {
                    // Parse parameter name
                    let name_token = self.expect(Token::Identifier(String::new()))?;
                    let name = match &name_token.token {
                        Token::Identifier(s) => s.clone(),
                        _ => unreachable!(),
                    };

                    // Parse optional type annotation
                    let ty = if self.check(&Token::Colon) {
                        self.advance(); // consume ':'
                        self.parse_type()?
                    } else {
                        Type::Unknown {
                            location: Location {
                                line: name_token.line,
                                column: name_token.column,
                                start: name_token.start,
                                end: name_token.end,
                            },
                        }
                    };

                    params.push(Parameter {
                        name,
                        ty,
                        location: Location {
                            line: name_token.line,
                            column: name_token.column,
                            start: name_token.start,
                            end: self.current_token.end,
                        },
                    });

                    // Check if there are more parameters separated by comma
                    if !self.check(&Token::Comma) {
                        break;
                    }
                    self.advance(); // consume ','
                }

                // Expect closing pipe
                self.expect(Token::Pipe)?; // consume closing '|'

                // Parse lambda body
                let body = self.parse_expression()?;

                Ok(Expr::Lambda {
                    params,
                    body: Box::new(body),
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

    /// Parse a binary expression with precedence
    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<Expr, ParseError> {
        let mut left = self.parse_postfix_expression()?;

        loop {
            let operator = match self.current_token.token {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Sub,
                Token::Star => BinaryOperator::Mul,
                Token::Slash => BinaryOperator::Div,
                Token::Percent => BinaryOperator::Mod,
                Token::GreaterThan => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                Token::LessThan => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                Token::EqualEqual => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => break,
            };

            let precedence = Self::get_precedence(&operator);
            if precedence <= min_precedence {
                break;
            }

            self.advance();
            let right = self.parse_binary_expression(precedence)?;
            let location_start = left.location().start;
            left = Expr::BinaryOp {
                left: Box::new(left),
                operator,
                right: Box::new(right),
                location: Location {
                    line: self.current_token.line,
                    column: self.current_token.column,
                    start: location_start,
                    end: self.current_token.end,
                },
            };
        }

        Ok(left)
    }

    /// Parse a postfix expression (function calls, etc.)
    fn parse_postfix_expression(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_primary_expression()?;

        loop {
            if self.check(&Token::LParen) {
                // Function call
                self.advance();
                let mut args = Vec::new();
                if !self.check(&Token::RParen) {
                    loop {
                        args.push(self.parse_expression()?);
                        if !self.check(&Token::Comma) {
                            break;
                        }
                        self.advance();
                    }
                }
                let end_token = self.expect(Token::RParen)?;

                let location_start = left.location().start;
                left = Expr::FunctionCall {
                    function: Box::new(left),
                    args,
                    named_args: HashMap::new(),
                    location: Location {
                        line: self.current_token.line, // Approx
                        column: self.current_token.column,
                        start: location_start,
                        end: end_token.end,
                    },
                };
            } else if self.check(&Token::Dot) {
                // Field access (e.g., self.value)
                self.advance();
                let field_token = self.expect(Token::Identifier(String::new()))?;
                let field_name = match &field_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };

                let location_start = left.location().start;
                left = Expr::FieldAccess {
                    object: Box::new(left),
                    field: field_name,
                    location: Location {
                        line: self.current_token.line,
                        column: self.current_token.column,
                        start: location_start,
                        end: field_token.end,
                    },
                };
            } else if self.check(&Token::DoubleColon) {
                // Static access (e.g., Map::new)
                self.advance();
                let field_token = self.expect(Token::Identifier(String::new()))?;
                let field_name = match &field_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };

                // Merge into a single variable name for now (compatibility)
                if let Expr::Variable { name, location } = left {
                    let new_name = format!("{}::{}", name, field_name);
                    left = Expr::Variable {
                        name: new_name,
                        location: Location {
                            line: location.line,
                            column: location.column,
                            start: location.start,
                            end: field_token.end,
                        },
                    };
                } else {
                    return Err(ParseError::Generic(
                        "Expected identifier before ::".to_string(),
                    ));
                }
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn get_precedence(operator: &BinaryOperator) -> u8 {
        match operator {
            BinaryOperator::Equal | BinaryOperator::NotEqual => 3,
            BinaryOperator::Less
            | BinaryOperator::LessEqual
            | BinaryOperator::Greater
            | BinaryOperator::GreaterEqual => 4,
            BinaryOperator::Add | BinaryOperator::Sub => 5,
            BinaryOperator::Mul | BinaryOperator::Div | BinaryOperator::Mod => 6,
            _ => 0, // Other operators not handled yet
        }
    }

    // Placeholder implementations for other statement types
    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.expect(Token::If)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;

        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;

        self.expect(Token::Else)?;
        let else_branch = self.parse_block()?;

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: self.current_token.end,
            },
        })
    }

    fn parse_switch_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "Switch statements not implemented yet".to_string(),
        ))
    }

    fn parse_match_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.expect(Token::Match)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;

        let value = self.parse_expression()?;
        self.expect(Token::LBrace)?;

        let mut cases = Vec::new();

        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            let pattern = self.parse_pattern()?;
            self.expect(Token::FatArrow)?;

            let body = if self.check(&Token::LBrace) {
                self.parse_block()?
            } else {
                let expr = self.parse_expression()?;
                let expr_loc = expr.location().clone();
                Block {
                    statements: vec![Statement::Expr {
                        expr,
                        location: expr_loc.clone(),
                    }],
                    location: expr_loc,
                }
            };

            cases.push(MatchCase {
                pattern,
                body,
                location: Location {
                    line: self.current_token.line,
                    column: self.current_token.column,
                    start: self.current_token.start,
                    end: self.current_token.end,
                },
            });

            if self.check(&Token::Comma) {
                self.advance();
            }
        }

        let end_token = self.expect(Token::RBrace)?;

        Ok(Statement::Match {
            value,
            cases,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: end_token.end,
            },
        })
    }

    fn parse_fold_statement(&mut self) -> Result<Statement, ParseError> {
        Err(ParseError::Generic(
            "Fold statements not implemented yet".to_string(),
        ))
    }

    fn parse_bend_statement(&mut self) -> Result<Statement, ParseError> {
        let token = self.expect(Token::Bend)?;
        let start = token.start;
        let start_line = token.line;
        let start_column = token.column;

        // Expect '{' to start the block
        self.expect(Token::LBrace)?;

        let mut initial_states = Vec::new();
        let mut statements = Vec::new();

        // Parse content inside the block
        while !self.check(&Token::RBrace) && !self.check(&Token::EOF) {
            println!(
                "DEBUG: Current: {:?}, Peek: {:?}",
                self.current_token.token, self.peek_token.token
            );
            // Check for initializer syntax 1: Identifier <- Expression
            let is_arrow_init = if let Token::Identifier(_) = &self.current_token.token {
                matches!(self.peek_token.token, Token::LeftArrow)
            } else {
                false
            };

            if is_arrow_init {
                let var_token = self.expect(Token::Identifier(String::new()))?;
                let var = match &var_token.token {
                    Token::Identifier(s) => s.clone(),
                    _ => unreachable!(),
                };

                self.expect(Token::LeftArrow)?;
                let expr = self.parse_expression()?;
                initial_states.push((var, expr));

                // Optional semicolon or comma
                if self.check(&Token::Semicolon) || self.check(&Token::Comma) {
                    self.advance();
                }
            } else if self.check(&Token::Let) {
                // Check for initializer syntax 2: let x = 1;
                // We treat top-level let statements in bend as initializers
                let stmt = self.parse_let_statement()?;

                if let Statement::Use { name, value, .. } = stmt {
                    initial_states.push((name, value));
                } else {
                    // Should not happen for parse_let_statement
                    statements.push(stmt);
                }
            } else {
                // Not an initializer, must be a statement part of the body
                statements.push(self.parse_statement()?);
            }
        }

        let end_token = self.expect(Token::RBrace)?;

        let body = Block {
            statements,
            location: Location {
                line: start_line,
                column: start_column, // Using start of bend for block location context
                start: token.end,     // Start of block usually after brace, but here we approximate
                end: end_token.end,
            },
        };

        // For now, condition is always true (placeholder) as per original code
        let condition = Expr::Literal {
            kind: LiteralKind::Uint(1),
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: end_token.end,
            },
        };

        Ok(Statement::Bend {
            initial_states,
            condition,
            body,
            else_body: None,
            location: Location {
                line: start_line,
                column: start_column,
                start,
                end: end_token.end,
            },
        })
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

    fn parse_try_catch_statement(&mut self) -> Result<Statement, ParseError> {
        let _start_line = self.current_token.line;
        let _start_column = self.current_token.column;
        let start = self.current_token.start;

        // Consume 'try' token
        self.advance();

        // Parse try block
        let try_block = self.parse_block()?;

        // Parse catch blocks
        let mut catch_blocks = Vec::new();
        while self.check(&Token::Catch) {
            self.advance();

            // Parse optional error type and variable
            let error_type = if self.check(&Token::Identifier("".to_string())) {
                None
            } else {
                Some("Error".to_string()) // Default error type
            };
            let error_var = None; // For now, no error variable

            // Parse catch block body
            let catch_body = self.parse_block()?;

            let catch_location = Location {
                line: self.current_token.line,
                column: self.current_token.column,
                start: self.current_token.start,
                end: self.current_token.end,
            };

            catch_blocks.push(CatchBlock {
                error_type,
                error_var,
                body: catch_body,
                location: catch_location,
            });
        }

        let end_location = Location {
            line: self.current_token.line,
            column: self.current_token.column,
            start,
            end: self.current_token.end,
        };

        Ok(Statement::TryCatch {
            try_block,
            catch_blocks,
            location: end_location,
        })
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
        let start_location = Location {
            line: self.current_token.line,
            column: self.current_token.column,
            start: self.current_token.start,
            end: self.current_token.end,
        };

        let token = self.current_token.token.clone();
        match token {
            Token::Identifier(name) => {
                self.advance();

                if self.check(&Token::LBrace) {
                    // Constructor with named fields
                    self.advance();
                    let mut fields = HashMap::new();

                    while !self.check(&Token::RBrace) {
                        let field_name_token = self.expect(Token::Identifier(String::new()))?;
                        let field_name = match &field_name_token.token {
                            Token::Identifier(s) => s.clone(),
                            _ => unreachable!(),
                        };
                        self.expect(Token::Colon)?;
                        let field_pattern = self.parse_pattern()?;
                        fields.insert(field_name, field_pattern);

                        if !self.check(&Token::RBrace) {
                            self.expect(Token::Comma)?;
                        }
                    }

                    self.expect(Token::RBrace)?;
                    let end_location = Location {
                        line: self.current_token.line,
                        column: self.current_token.column,
                        start: self.current_token.start,
                        end: self.current_token.end,
                    };
                    let location = Location::span(&start_location, &end_location);

                    Ok(Pattern::Constructor {
                        name: name.clone(),
                        fields,
                        location,
                    })
                } else if self.check(&Token::LParen) {
                    // TupleConstructor (Identifier followed by LParen)
                    self.advance();
                    let mut args = Vec::new();

                    while !self.check(&Token::RParen) {
                        args.push(self.parse_pattern()?);
                        if !self.check(&Token::RParen) {
                            self.expect(Token::Comma)?;
                        }
                    }

                    let end_token = self.expect(Token::RParen)?;
                    let end_location = Location {
                        line: end_token.line,
                        column: end_token.column,
                        start: end_token.start,
                        end: end_token.end,
                    };
                    let location = Location::span(&start_location, &end_location);

                    Ok(Pattern::TupleConstructor {
                        name: name.clone(),
                        args,
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
            Token::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard {
                    location: start_location,
                })
            }
            Token::LeftParen => {
                self.advance();
                let mut elements = Vec::new();

                while !self.check(&Token::RightParen) {
                    let element = self.parse_pattern()?;
                    elements.push(element);

                    if !self.check(&Token::RightParen) {
                        self.expect(Token::Comma)?;
                    }
                }

                self.expect(Token::RightParen)?;
                let end_location = Location {
                    line: self.current_token.line,
                    column: self.current_token.column,
                    start: self.current_token.start,
                    end: self.current_token.end,
                };
                let location = Location::span(&start_location, &end_location);

                Ok(Pattern::Tuple { elements, location })
            }
            _ => {
                // Try to parse as a literal pattern
                let expr = self.parse_expression()?;
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

/// Parse a source string into a program
pub fn parse_from_source(source: &str) -> Result<Program, ParseError> {
    let mut parser = Parser::new(source);
    parser.parse_program()
}
