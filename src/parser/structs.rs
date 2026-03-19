// src/parser/structs.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_struct_declaration(&mut self) -> Option<Stmt> {
        let start_pos = self.position;

        if let Err(e) = self.expect_token(TokenKind::Struct) {
            self.errors.push(e);
            return None;
        }

        let name = match self.current_token().cloned() {
            Some(t) => match t.kind {
                TokenKind::Identifier(n) => {
                    self.advance();
                    n
                }
                _ => {
                    self.errors.push(ParseError::UnexpectedToken {
                        expected: "struct name".to_string(),
                        got: t.kind,
                        line_num: t.line,
                        col_num: t.column,
                    });
                    self.position = start_pos;
                    return None;
                }
            },
            None => {
                self.errors.push(ParseError::UnexpectedEOF);
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::LBrace) {
            self.errors.push(e);
            self.position = start_pos;
            return None;
        }

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        loop {
            match self.current_token().cloned() {
                Some(t) if t.kind == TokenKind::RBrace => {
                    self.advance();
                    break;
                }
                None => {
                    self.errors.push(ParseError::UnexpectedEOF);
                    return None;
                }
                _ => {
                    let type_name = match self.parse_type_string() {
                        Some(t) => t,
                        None => {
                            self.synchronize();
                            continue;
                        }
                    };

                    let member_name = match self.current_token().cloned() {
                        Some(t) => match t.kind {
                            TokenKind::Identifier(n) => {
                                self.advance();
                                n
                            }
                            _ => {
                                self.errors.push(ParseError::UnexpectedToken {
                                    expected: "field or method name".to_string(),
                                    got: t.kind,
                                    line_num: t.line,
                                    col_num: t.column,
                                });
                                self.synchronize();
                                continue;
                            }
                        },
                        None => {
                            self.errors.push(ParseError::UnexpectedEOF);
                            return None;
                        }
                    };

                    if let Some(t) = self.current_token()
                        && t.kind == TokenKind::LParen
                    {
                        // Method
                        self.advance(); // consume '('
                        let mut params = Vec::new();

                        while let Some(t) = self.current_token() {
                            if t.kind == TokenKind::RParen {
                                break;
                            }
                            let param_type = match self.parse_type_string() {
                                Some(t) => t,
                                None => break,
                            };
                            let param_name = match self.current_token().cloned() {
                                Some(t) => match t.kind {
                                    TokenKind::Identifier(n) => {
                                        self.advance();
                                        n
                                    }
                                    _ => break,
                                },
                                None => break,
                            };
                            params.push(Param {
                                type_name: param_type,
                                name: param_name,
                            });
                            if let Some(t) = self.current_token()
                                && t.kind == TokenKind::Comma
                            {
                                self.advance();
                            }
                        }

                        if let Err(e) = self.expect_token(TokenKind::RParen) {
                            self.errors.push(e);
                            return None;
                        }

                        let body = self.parse_block()?;

                        methods.push(StructMethod {
                            return_type: type_name,
                            name: member_name,
                            params,
                            body,
                        });
                    } else {
                        // Field
                        if let Err(e) = self.expect_token(TokenKind::Semicolon) {
                            self.errors.push(e);
                            return None;
                        }
                        fields.push(StructField {
                            type_name,
                            name: member_name,
                        });
                    }
                }
            }
        }

        Some(Stmt::StructDeclaration {
            name,
            fields,
            methods,
        })
    }
}
