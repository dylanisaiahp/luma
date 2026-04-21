// src/parser/ifelse.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;

impl Parser {
    pub fn parse_if_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;
        if let Err(e) = self.expect_token(TokenKind::If) {
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

        let then_branch = self.parse_block()?;
        let else_branch = self.parse_else_chain()?;

        Some(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    pub fn parse_else_chain(&mut self) -> Option<Option<Vec<Stmt>>> {
        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Else) => {
                self.advance();
                if let Some(TokenKind::If) = self.current_token().map(|t| &t.kind) {
                    self.parse_if_statement().map(|inner_if| Some(vec![inner_if]))
                } else {
                    let else_branch = self.parse_block()?;
                    Some(Some(else_branch))
                }
            }
            _ => Some(None),
        }
    }
}
