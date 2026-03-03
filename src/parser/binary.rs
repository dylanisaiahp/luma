// src/parser/binary.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_binary_expression(&mut self, min_precedence: i32) -> Result<Expr, ParseError> {
        let mut left = self.parse_call_expression()?;

        while let Some(token) = self.current_token().cloned() {
            let op = match &token.kind {
                TokenKind::Plus => crate::syntax::BinaryOp::Add,
                TokenKind::Minus => crate::syntax::BinaryOp::Subtract,
                TokenKind::Star => crate::syntax::BinaryOp::Multiply,
                TokenKind::Slash => crate::syntax::BinaryOp::Divide,
                TokenKind::Greater => crate::syntax::BinaryOp::Greater,
                TokenKind::Less => crate::syntax::BinaryOp::Less,
                TokenKind::GreaterEqual => crate::syntax::BinaryOp::GreaterEqual,
                TokenKind::LessEqual => crate::syntax::BinaryOp::LessEqual,
                TokenKind::EqualEqual => crate::syntax::BinaryOp::Equal,
                TokenKind::BangEqual => crate::syntax::BinaryOp::NotEqual,
                _ => break,
            };

            let precedence = match op {
                crate::syntax::BinaryOp::Equal | crate::syntax::BinaryOp::NotEqual => 10,
                crate::syntax::BinaryOp::Greater
                | crate::syntax::BinaryOp::Less
                | crate::syntax::BinaryOp::GreaterEqual
                | crate::syntax::BinaryOp::LessEqual => 20,
                crate::syntax::BinaryOp::Add | crate::syntax::BinaryOp::Subtract => 30,
                crate::syntax::BinaryOp::Multiply | crate::syntax::BinaryOp::Divide => 40,
            };

            if precedence < min_precedence {
                break;
            }

            self.advance();
            let right = self.parse_binary_expression(precedence + 1)?;

            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                line: token.line,
                column: token.column,
            };
        }

        Ok(left)
    }
}
