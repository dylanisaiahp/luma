// src/lexer/mod.rs
mod comments;
mod core;
mod identifiers;
mod reader;
mod strings;
mod symbols;
mod tokens;

pub use core::Lexer;
pub use tokens::{Token, TokenKind};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character: {0} at line {1}, column {2}")]
    UnexpectedCharacter(char, usize, usize),

    #[error("Unterminated string at line {0}, column {1}")]
    UnterminatedString(usize, usize),

    #[error("Invalid number: {0} at line {1}, column {2}")]
    InvalidNumber(String, usize, usize),

    #[error(
        "Invalid interpolation syntax at line {1}, column {2}. Expected '}}' after variable name, found {0}."
    )]
    InvalidInterpolationSyntax(String, usize, usize),
}
