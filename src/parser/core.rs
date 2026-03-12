// src/parser/core.rs
use crate::ast::*;
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

    pub fn parse_block(&mut self) -> Option<Vec<Stmt>> {
        if let Err(e) = self.expect_token(TokenKind::LBrace) {
            self.errors.push(e);
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

        Some(body)
    }

    pub fn parse_program(&mut self) -> Vec<Stmt> {
        let mut statements = Vec::new();
        let mut last_position = usize::MAX;

        while let Some(token) = self.current_token() {
            if self.position == last_position {
                self.advance();
                last_position = self.position;
                continue;
            }
            last_position = self.position;

            // Check for maybe(type) function_name() pattern
            // Tokens: maybe ( type ) identifier (
            //           +0   +1  +2  +3    +4    +5
            let is_maybe_function = if token.kind == TokenKind::Maybe {
                matches!(self.tokens.get(self.position + 1), Some(t) if t.kind == TokenKind::LParen)
                    && matches!(self.tokens.get(self.position + 3), Some(t) if t.kind == TokenKind::RParen)
                    && matches!(self.tokens.get(self.position + 4), Some(t) if matches!(t.kind, TokenKind::Identifier(_)))
                    && matches!(self.tokens.get(self.position + 5), Some(t) if t.kind == TokenKind::LParen)
            } else {
                false
            };

            let is_worry_function = if token.kind == TokenKind::Worry {
                matches!(self.tokens.get(self.position + 1), Some(t) if t.kind == TokenKind::LParen)
                    && matches!(self.tokens.get(self.position + 3), Some(t) if t.kind == TokenKind::RParen)
                    && matches!(self.tokens.get(self.position + 4), Some(t) if matches!(t.kind, TokenKind::Identifier(_)))
                    && matches!(self.tokens.get(self.position + 5), Some(t) if t.kind == TokenKind::LParen)
            } else {
                false
            };

            // Check for list(type) function_name() pattern
            // Tokens: list ( type ) identifier (
            //          +0   +1  +2  +3    +4    +5
            let is_list_function = if token.kind == TokenKind::List {
                matches!(self.tokens.get(self.position + 1), Some(t) if t.kind == TokenKind::LParen)
                    && matches!(self.tokens.get(self.position + 3), Some(t) if t.kind == TokenKind::RParen)
                    && matches!(self.tokens.get(self.position + 4), Some(t) if matches!(t.kind, TokenKind::Identifier(_)))
                    && matches!(self.tokens.get(self.position + 5), Some(t) if t.kind == TokenKind::LParen)
            } else {
                false
            };

            // Check for table(key, val) function_name() pattern
            // Tokens: table ( key , val ) identifier (
            //           +0   +1  +2  +3  +4  +5   +6    +7
            let is_table_function = if token.kind == TokenKind::Table {
                matches!(self.tokens.get(self.position + 1), Some(t) if t.kind == TokenKind::LParen)
                    && matches!(self.tokens.get(self.position + 3), Some(t) if t.kind == TokenKind::Comma)
                    && matches!(self.tokens.get(self.position + 5), Some(t) if t.kind == TokenKind::RParen)
                    && matches!(self.tokens.get(self.position + 6), Some(t) if matches!(t.kind, TokenKind::Identifier(_)))
                    && matches!(self.tokens.get(self.position + 7), Some(t) if t.kind == TokenKind::LParen)
            } else {
                false
            };

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
                TokenKind::Maybe if is_maybe_function => {
                    // maybe(int) function_name() { ... }
                    self.advance(); // consume 'maybe'
                    if let Err(e) = self.expect_token(TokenKind::LParen) {
                        self.errors.push(e);
                        continue;
                    }
                    let inner = match self.current_token().map(|t| &t.kind) {
                        Some(TokenKind::Int) => "int".to_string(),
                        Some(TokenKind::Float) => "float".to_string(),
                        Some(TokenKind::Bool) => "bool".to_string(),
                        Some(TokenKind::String) => "string".to_string(),
                        _ => {
                            let token = self.current_or_eof();
                            self.errors.push(ParseError::UnexpectedToken {
                                expected: "type inside maybe()".to_string(),
                                got: token.kind,
                                line_num: token.line,
                                col_num: token.column,
                            });
                            continue;
                        }
                    };
                    self.advance(); // consume inner type
                    if let Err(e) = self.expect_token(TokenKind::RParen) {
                        self.errors.push(e);
                        continue;
                    }
                    let return_type = format!("maybe({})", inner);
                    if let Some(func) = self.parse_function(return_type) {
                        statements.push(func);
                    }
                }
                TokenKind::Worry if is_worry_function => {
                    // worry(int) function_name() { ... }
                    self.advance(); // consume 'worry'
                    if let Err(e) = self.expect_token(TokenKind::LParen) {
                        self.errors.push(e);
                        continue;
                    }
                    let inner = match self.current_token().map(|t| &t.kind) {
                        Some(TokenKind::Int) => "int".to_string(),
                        Some(TokenKind::Float) => "float".to_string(),
                        Some(TokenKind::Bool) => "bool".to_string(),
                        Some(TokenKind::String) => "string".to_string(),
                        _ => {
                            let token = self.current_or_eof();
                            self.errors.push(ParseError::UnexpectedToken {
                                expected: "type inside worry()".to_string(),
                                got: token.kind,
                                line_num: token.line,
                                col_num: token.column,
                            });
                            continue;
                        }
                    };
                    self.advance(); // consume inner type
                    if let Err(e) = self.expect_token(TokenKind::RParen) {
                        self.errors.push(e);
                        continue;
                    }
                    let return_type = format!("worry({})", inner);
                    if let Some(func) = self.parse_function(return_type) {
                        statements.push(func);
                    }
                }
                TokenKind::List if is_list_function => {
                    // list(int) function_name() { ... }
                    self.advance(); // consume 'list'
                    if let Err(e) = self.expect_token(TokenKind::LParen) {
                        self.errors.push(e);
                        continue;
                    }
                    let inner = match self.current_token().map(|t| &t.kind) {
                        Some(TokenKind::Int) => "int".to_string(),
                        Some(TokenKind::Float) => "float".to_string(),
                        Some(TokenKind::Bool) => "bool".to_string(),
                        Some(TokenKind::String) => "string".to_string(),
                        Some(TokenKind::Char) => "char".to_string(),
                        Some(TokenKind::Word) => "word".to_string(),
                        _ => {
                            let token = self.current_or_eof();
                            self.errors.push(ParseError::UnexpectedToken {
                                expected: "type inside list()".to_string(),
                                got: token.kind,
                                line_num: token.line,
                                col_num: token.column,
                            });
                            continue;
                        }
                    };
                    self.advance(); // consume inner type
                    if let Err(e) = self.expect_token(TokenKind::RParen) {
                        self.errors.push(e);
                        continue;
                    }
                    let return_type = format!("list({})", inner);
                    if let Some(func) = self.parse_function(return_type) {
                        statements.push(func);
                    }
                }
                TokenKind::Table if is_table_function => {
                    // table(string, int) function_name() { ... }
                    self.advance(); // consume 'table'
                    if let Err(e) = self.expect_token(TokenKind::LParen) {
                        self.errors.push(e);
                        continue;
                    }
                    let key_type = match self.current_token().map(|t| &t.kind) {
                        Some(TokenKind::Int) => "int".to_string(),
                        Some(TokenKind::Float) => "float".to_string(),
                        Some(TokenKind::Bool) => "bool".to_string(),
                        Some(TokenKind::String) => "string".to_string(),
                        Some(TokenKind::Char) => "char".to_string(),
                        Some(TokenKind::Word) => "word".to_string(),
                        _ => {
                            let token = self.current_or_eof();
                            self.errors.push(ParseError::UnexpectedToken {
                                expected: "key type inside table()".to_string(),
                                got: token.kind,
                                line_num: token.line,
                                col_num: token.column,
                            });
                            continue;
                        }
                    };
                    self.advance(); // consume key type
                    if let Err(e) = self.expect_token(TokenKind::Comma) {
                        self.errors.push(e);
                        continue;
                    }
                    let val_type = match self.current_token().map(|t| &t.kind) {
                        Some(TokenKind::Int) => "int".to_string(),
                        Some(TokenKind::Float) => "float".to_string(),
                        Some(TokenKind::Bool) => "bool".to_string(),
                        Some(TokenKind::String) => "string".to_string(),
                        Some(TokenKind::Char) => "char".to_string(),
                        Some(TokenKind::Word) => "word".to_string(),
                        _ => {
                            let token = self.current_or_eof();
                            self.errors.push(ParseError::UnexpectedToken {
                                expected: "value type inside table()".to_string(),
                                got: token.kind,
                                line_num: token.line,
                                col_num: token.column,
                            });
                            continue;
                        }
                    };
                    self.advance(); // consume val type
                    if let Err(e) = self.expect_token(TokenKind::RParen) {
                        self.errors.push(e);
                        continue;
                    }
                    let return_type = format!("table({}, {})", key_type, val_type);
                    if let Some(func) = self.parse_function(return_type) {
                        statements.push(func);
                    }
                }
                TokenKind::Struct => {
                    if let Some(stmt) = self.parse_struct_declaration() {
                        statements.push(stmt);
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
