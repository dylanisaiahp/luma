// src/parser/ifelse.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

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

        if let Err(e) = self.expect_token(TokenKind::LBrace) {
            self.errors.push(e);
            self.position = start_pos;
            self.synchronize();
            return None;
        }

        let mut then_branch = Vec::new();
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
                        then_branch.push(stmt);
                    } else {
                        self.advance();
                    }
                }
            }
        }

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
                    if let Some(inner_if) = self.parse_if_statement() {
                        Some(Some(vec![inner_if]))
                    } else {
                        None
                    }
                } else {
                    if let Err(e) = self.expect_token(TokenKind::LBrace) {
                        self.errors.push(e);
                        return Some(None);
                    }
                    let mut else_branch = Vec::new();
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
                                    else_branch.push(stmt);
                                } else {
                                    self.advance();
                                }
                            }
                        }
                    }
                    Some(Some(else_branch))
                }
            }
            _ => Some(None),
        }
    }
}
