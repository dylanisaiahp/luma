// src/parser/forloop.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_for_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;

        if let Err(e) = self.expect_token(TokenKind::For) {
            self.errors.push(e);
            self.position = start_pos;
            return None;
        }

        // Parse loop variable name
        let var = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "loop variable name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                return None;
            }
        };

        // Expect 'in'
        if let Err(e) = self.expect_token(TokenKind::In) {
            self.errors.push(e);
            return None;
        }

        // Expect 'range('
        let range_token = self.current_or_eof();
        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) if name == "range" => {
                self.advance();
            }
            _ => {
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "range()".to_string(),
                    got: range_token.kind,
                    line_num: range_token.line,
                    col_num: range_token.column,
                });
                return None;
            }
        }

        if let Err(e) = self.expect_token(TokenKind::LParen) {
            self.errors.push(e);
            return None;
        }

        let start = match self.parse_expression() {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::Comma) {
            self.errors.push(e);
            return None;
        }

        let end = match self.parse_expression() {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

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

        Some(Stmt::For {
            var,
            start,
            end,
            body,
        })
    }
}
