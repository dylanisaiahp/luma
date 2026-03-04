// src/parser/core.rs
use crate::ast::*;
use crate::debug::DebugLevel;
use crate::lexer::{Token, TokenKind};
use crate::parser::error::ParseError;

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
            Some(token) => Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                got: token.kind,
                line_num: token.line,
                col_num: token.column,
            }),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    pub fn synchronize(&mut self) {
        while let Some(token) = self.current_token() {
            match token.kind {
                TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn is_typed_function(&self) -> bool {
        if let Some(next) = self.tokens.get(self.position + 1)
            && let TokenKind::Identifier(_) = &next.kind
            && let Some(after) = self.tokens.get(self.position + 2)
        {
            return after.kind == TokenKind::LParen;
        }
        false
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

            if self.position == last_position {
                crate::debug!(
                    DebugLevel::Basic,
                    "Parser stuck at position {}, token: {:?}",
                    self.position,
                    token.kind
                );
                self.advance();
                last_position = self.position;
                continue;
            }
            last_position = self.position;

            match token.kind {
                TokenKind::Void if self.is_typed_function() => {
                    self.advance(); // consume 'void'
                    if let Some(func) = self.parse_function("void".to_string()) {
                        statements.push(func);
                    }
                }
                TokenKind::Int | TokenKind::Float | TokenKind::Bool | TokenKind::String
                    if self.is_typed_function() =>
                {
                    let return_type = match self.current_token().map(|t| &t.kind) {
                        Some(TokenKind::Int) => "int".to_string(),
                        Some(TokenKind::Float) => "float".to_string(),
                        Some(TokenKind::Bool) => "bool".to_string(),
                        Some(TokenKind::String) => "string".to_string(),
                        _ => unreachable!(),
                    };
                    self.advance();
                    if let Some(func) = self.parse_function(return_type) {
                        statements.push(func);
                    }
                }
                TokenKind::Use => {
                    self.advance(); // consume 'use'
                    let module = match self.current_token().map(|t| &t.kind) {
                        Some(TokenKind::Identifier(name)) => {
                            let name = name.clone();
                            self.advance();
                            name
                        }
                        _ => {
                            let token = self.current_or_eof();
                            self.errors.push(ParseError::UnexpectedToken {
                                expected: "module name".to_string(),
                                got: token.kind,
                                line_num: token.line,
                                col_num: token.column,
                            });
                            continue;
                        }
                    };
                    // consume optional semicolon
                    if let Some(TokenKind::Semicolon) = self.current_token().map(|t| &t.kind) {
                        self.advance();
                    }
                    statements.push(Stmt::Use { module });
                }
                TokenKind::Eof => break,
                _ => {
                    if let Some(stmt) = self.parse_statement() {
                        statements.push(stmt);
                    } else {
                        self.advance();
                    }
                }
            }
        }

        statements
    }
}
