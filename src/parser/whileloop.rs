// src/parser/whileloop.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;

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

        let body = self.parse_block()?;

        Some(Stmt::While { condition, body })
    }
}
