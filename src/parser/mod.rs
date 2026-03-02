// src/parser/mod.rs
use crate::ast::*;
use crate::debug::DebugLevel;
use crate::lexer::{Token, TokenKind};

pub mod declarations;
pub mod error;
pub mod expressions;
pub mod statements;

pub use error::ParseError;

pub struct Parser {
    pub tokens: Vec<Token>,
    pub position: usize,
    pub errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
            errors: Vec::new(),
        }
    }

    pub fn take_errors(&mut self) -> Vec<ParseError> {
        std::mem::take(&mut self.errors)
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    pub fn current_or_eof(&self) -> Token {
        self.current_token().cloned().unwrap_or(Token {
            kind: TokenKind::Eof,
            line: 0,
            column: 0,
            byte_pos: 0,
        })
    }

    pub fn advance(&mut self) -> Option<&Token> {
        let token = self.tokens.get(self.position);
        self.position += 1;
        token
    }

    pub fn expect_token(&mut self, expected: TokenKind) -> Result<(), ParseError> {
        crate::debug!(
            DebugLevel::Verbose,
            "Expecting {:?}, current: {:?}",
            expected,
            self.current_token().map(|t| &t.kind)
        );

        let current = self.current_token().cloned();
        match current {
            Some(token)
                if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) =>
            {
                self.advance();
                Ok(())
            }
            Some(token) => {
                // Don't push error here — just return it
                Err(ParseError::UnexpectedToken {
                    expected: format!("{:?}", expected),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                })
            }
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn synchronize(&mut self) {
        while let Some(token) = self.current_token() {
            match token.kind {
                TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof => {
                    break;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        crate::debug!(DebugLevel::Basic, "Starting parse");
        let mut statements = Vec::new();
        let mut last_position = usize::MAX;

        while let Some(token) = self.current_token() {
            crate::debug!(
                DebugLevel::Verbose,
                "IN LOOP - Token kind: {:?}",
                token.kind
            );
            crate::debug!(
                DebugLevel::Verbose,
                "Processing token: {:?} at position {}",
                token.kind,
                self.position
            );

            if self.position == last_position {
                crate::debug!(
                    DebugLevel::Basic,
                    "Parser stuck at position {}, token: {:?}",
                    self.position,
                    token.kind
                );
                // Skip the problematic token and continue
                self.advance();
                last_position = self.position;
                continue;
            }
            last_position = self.position;

            match token.kind {
                TokenKind::Void => {
                    crate::debug!(DebugLevel::Basic, "Found void, calling parse_function");
                    if let Some(func) = self.parse_function() {
                        statements.push(func);
                    }
                }
                TokenKind::Eof => break,
                _ => {
                    if let Some(stmt) = self.parse_statement() {
                        statements.push(stmt);
                    } else {
                        // If we can't parse a statement, skip this token
                        self.advance();
                    }
                }
            }
        }

        statements
    }
}
