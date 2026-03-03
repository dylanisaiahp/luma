// src/parser/whileloop.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_while_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;
        if let Err(e) = self.expect_token(TokenKind::While) {
            self.errors.push(e);
            self.position = start_pos;
            return None;
        }

        let condition = match self.parse_expression() {
            Ok(expr) => expr,
            Err(e) => {
                self.errors.push(e);
                self.position = start_pos;
                self.synchronize();
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::LBrace) {
            self.errors.push(e);
            self.position = start_pos;
            self.synchronize();
            return None;
        }

        let mut body = Vec::new();
        let mut last_pos = 0;
        while let Some(token) = self.current_token() {
            if self.position == last_pos {
                self.advance();
                last_pos = self.position;
                continue;
            }
            last_pos = self.position;

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

        Some(Stmt::While { condition, body })
    }
}
