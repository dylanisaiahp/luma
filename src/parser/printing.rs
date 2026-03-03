// src/parser/printing.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_print_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;
        if let Err(e) = self.expect_token(TokenKind::Print) {
            self.errors.push(e);
            return None;
        }
        if let Err(e) = self.expect_token(TokenKind::LParen) {
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
            Some(TokenKind::RParen) => {
                self.advance();
            }
            Some(_) => {
                let token = self.current_token().unwrap();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: ")".to_string(),
                    got: token.kind.clone(),
                    line_num: token.line,
                    col_num: token.column,
                });
                self.synchronize();
                return None;
            }
            None => {
                self.errors.push(ParseError::UnexpectedEOF);
                return None;
            }
        }

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

        Some(Stmt::Print(value))
    }
}
