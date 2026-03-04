// src/parser/declarations.rs
use crate::ast::*;
use crate::debug::DebugLevel;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

use super::Parser;

impl Parser {
    // Parse typed function: int add(int x, int y) { ... }
    pub fn parse_typed_function(&mut self, return_type: String) -> Option<Stmt> {
        let name = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "function name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::LParen) {
            self.errors.push(e);
            return None;
        }

        // Parse parameters
        let mut params = Vec::new();
        while let Some(token) = self.current_token() {
            if token.kind == TokenKind::RParen {
                break;
            }

            let param_type = match self.current_token().map(|t| &t.kind) {
                Some(TokenKind::Int) => "int".to_string(),
                Some(TokenKind::Float) => "float".to_string(),
                Some(TokenKind::Bool) => "bool".to_string(),
                Some(TokenKind::String) => "string".to_string(),
                _ => {
                    let token = self.current_or_eof();
                    self.errors.push(ParseError::UnexpectedToken {
                        expected: "parameter type".to_string(),
                        got: token.kind,
                        line_num: token.line,
                        col_num: token.column,
                    });
                    return None;
                }
            };
            self.advance();

            let param_name = match self.current_token().map(|t| &t.kind) {
                Some(TokenKind::Identifier(name)) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => {
                    let token = self.current_or_eof();
                    self.errors.push(ParseError::UnexpectedToken {
                        expected: "parameter name".to_string(),
                        got: token.kind,
                        line_num: token.line,
                        col_num: token.column,
                    });
                    return None;
                }
            };

            params.push(Param {
                type_name: param_type,
                name: param_name,
            });

            // Consume comma if present
            if let Some(TokenKind::Comma) = self.current_token().map(|t| &t.kind) {
                self.advance();
            }
        }

        if let Err(e) = self.expect_token(TokenKind::RParen) {
            self.errors.push(e);
            return None;
        }

        if let Err(e) = self.expect_token(TokenKind::LBrace) {
            self.errors.push(e);
            return None;
        }

        let mut body = Vec::new();
        let mut last_body_pos = 0;
        while let Some(token) = self.current_token() {
            if self.position == last_body_pos {
                self.advance();
                last_body_pos = self.position;
                continue;
            }
            last_body_pos = self.position;

            match token.kind {
                TokenKind::RBrace => {
                    self.advance();
                    break;
                }
                TokenKind::Eof => {
                    self.errors.push(ParseError::UnexpectedEOF);
                    break;
                }
                _ => {
                    if let Some(stmt) = self.parse_statement() {
                        body.push(stmt);
                    } else {
                        self.advance();
                    }
                }
            }
        }

        Some(Stmt::UserFunction {
            return_type,
            name,
            params,
            body,
        })
    }

    pub fn parse_function(&mut self) -> Option<Stmt> {
        let start_pos = self.position;

        if self.expect_token(TokenKind::Void).is_err() {
            crate::debug!(DebugLevel::Basic, "Failed to parse void");
            self.position = start_pos;
            return None;
        }

        let name = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "function name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                self.synchronize();
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::LParen) {
            self.errors.push(e);
            self.synchronize();
            return None;
        }

        if let Err(e) = self.expect_token(TokenKind::RParen) {
            self.errors.push(e);
            self.synchronize();
            return None;
        }

        if let Err(e) = self.expect_token(TokenKind::LBrace) {
            self.errors.push(e);
            self.synchronize();
            return None;
        }

        let mut body = Vec::new();
        let mut last_body_pos = 0;
        while let Some(token) = self.current_token() {
            if self.position == last_body_pos {
                self.advance();
                last_body_pos = self.position;
                continue;
            }
            last_body_pos = self.position;

            match token.kind {
                TokenKind::RBrace => {
                    self.advance();
                    break;
                }
                TokenKind::Eof => {
                    self.errors.push(ParseError::UnexpectedEOF);
                    break;
                }
                _ => {
                    if let Some(stmt) = self.parse_statement() {
                        body.push(stmt);
                    } else {
                        self.advance();
                    }
                }
            }
        }

        Some(Stmt::Function { name, body })
    }

    pub fn parse_variable_declaration(&mut self) -> Option<Stmt> {
        let start_pos = self.position;
        let type_name = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Int) => "int".to_string(),
            Some(TokenKind::Float) => "float".to_string(),
            Some(TokenKind::Bool) => "bool".to_string(),
            Some(TokenKind::String) => "string".to_string(),
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "type (int/float/bool/string)".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                return None;
            }
        };
        self.advance();

        let name = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "variable name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                self.position = start_pos;
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::Equals) {
            self.errors.push(e);
            self.position = start_pos;
            return None;
        }

        let value = match self.parse_expression() {
            Ok(expr) => expr,
            Err(e) => {
                self.errors.push(e);
                self.position = start_pos;
                return None;
            }
        };

        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Semicolon) => {
                self.advance();
            }
            _ => {
                let last_token = &self.tokens[self.position - 1];
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "semicolon".to_string(),
                    got: TokenKind::Illegal("nothing".to_string()),
                    line_num: last_token.line,
                    col_num: last_token.column + 1,
                });
            }
        }

        Some(Stmt::VariableDeclaration {
            type_name,
            name,
            value,
        })
    }
}
