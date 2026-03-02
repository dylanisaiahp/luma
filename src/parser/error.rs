// src/parser/error.rs
use crate::lexer::TokenKind;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error(
        "Unexpected token: expected {expected}, got {got:?} at line {line_num}, column {col_num}"
    )]
    UnexpectedToken {
        expected: String,
        got: TokenKind,
        line_num: usize,
        col_num: usize,
    },

    #[error("Expected expression, got {0:?} at line {1}, column {2}")]
    ExpectedExpression(TokenKind, usize, usize),

    #[error("Unexpected end of file")]
    UnexpectedEOF,
}
