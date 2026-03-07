// src/parser/forloop.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_for_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;

        if let Err(e) = self.expect_token(TokenKind::For) {
            self.errors.push(e);
            self.position = start_pos;
            return None;
        }

        // Check for table iteration: for (key, value) in ...
        if let Some(t) = self.current_token()
            && t.kind == TokenKind::LParen
        {
            return self.parse_for_in_table();
        }

        // Parse loop variable
        let var = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "loop variable name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::In) {
            self.errors.push(e);
            return None;
        }

        // Check what follows 'in' — range() or a list variable
        match self.current_token().map(|t| t.kind.clone()) {
            Some(TokenKind::Identifier(ref name)) if name == "range" => self.parse_for_range(var),
            _ => self.parse_for_in_list(var),
        }
    }

    fn parse_for_range(&mut self, var: String) -> Option<Stmt> {
        // consume 'range'
        self.advance();

        if let Err(e) = self.expect_token(TokenKind::LParen) {
            self.errors.push(e);
            return None;
        }

        let start = match self.parse_expression() {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::Comma) {
            self.errors.push(e);
            return None;
        }

        let end = match self.parse_expression() {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::RParen) {
            self.errors.push(e);
            return None;
        }

        let body = self.parse_block()?;

        Some(Stmt::For {
            var,
            start,
            end,
            body,
        })
    }

    fn parse_for_in_list(&mut self, var: String) -> Option<Stmt> {
        let iterable = match self.parse_expression() {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        let body = self.parse_block()?;

        Some(Stmt::ForIn {
            var,
            iterable,
            body,
        })
    }

    fn parse_for_in_table(&mut self) -> Option<Stmt> {
        // consume '('
        self.advance();

        let key_var = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "key variable name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::Comma) {
            self.errors.push(e);
            return None;
        }

        let val_var = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "value variable name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::RParen) {
            self.errors.push(e);
            return None;
        }

        if let Err(e) = self.expect_token(TokenKind::In) {
            self.errors.push(e);
            return None;
        }

        let iterable = match self.parse_expression() {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        let body = self.parse_block()?;

        Some(Stmt::ForInTable {
            key_var,
            val_var,
            iterable,
            body,
        })
    }
}
