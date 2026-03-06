// src/parser/statements.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;
        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Print) => self.parse_print_statement(),
            Some(TokenKind::Return) => self.parse_return_statement(),
            Some(TokenKind::Break) => {
                self.advance();
                // consume optional semicolon
                if let Some(TokenKind::Semicolon) = self.current_token().map(|t| &t.kind) {
                    self.advance();
                }
                Some(Stmt::Break)
            }
            Some(TokenKind::For) => self.parse_for_statement(),
            Some(TokenKind::Int)
            | Some(TokenKind::Float)
            | Some(TokenKind::Bool)
            | Some(TokenKind::String)
            | Some(TokenKind::Maybe)
            | Some(TokenKind::List)
            | Some(TokenKind::Table) => self.parse_variable_declaration(),
            Some(TokenKind::If) => self.parse_if_statement(),
            Some(TokenKind::While) => self.parse_while_statement(),
            Some(TokenKind::Match) => self.parse_match_statement(),
            Some(TokenKind::Number(_))
            | Some(TokenKind::Identifier(_))
            | Some(TokenKind::True)
            | Some(TokenKind::False)
            | Some(TokenKind::LParen) => match self.parse_expression_statement() {
                Ok(stmt) => Some(stmt),
                Err(_) => {
                    self.position = start_pos;
                    self.synchronize();
                    None
                }
            },
            _ => {
                let token = self.current_token().cloned();
                if let Some(t) = token {
                    self.errors.push(ParseError::UnexpectedToken {
                        expected: "statement".to_string(),
                        got: t.kind,
                        line_num: t.line,
                        col_num: t.column,
                    });
                }
                self.synchronize();
                None
            }
        }
    }

    pub fn parse_return_statement(&mut self) -> Option<Stmt> {
        self.advance();

        if let Some(TokenKind::Semicolon) = self.current_token().map(|t| &t.kind) {
            self.advance();
            return Some(Stmt::Return(None));
        }

        let expr = match self.parse_expression() {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if let Some(TokenKind::Semicolon) = self.current_token().map(|t| &t.kind) {
            self.advance();
        }

        Some(Stmt::Return(Some(expr)))
    }

    pub fn parse_expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.parse_expression()?;
        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Semicolon) => {
                self.advance();
                Ok(Stmt::Expression(expr))
            }
            _ => {
                let last_token = &self.tokens[self.position - 1];
                Err(ParseError::UnexpectedToken {
                    expected: "semicolon".to_string(),
                    got: TokenKind::Illegal("nothing".to_string()),
                    line_num: last_token.line,
                    col_num: last_token.column + 1,
                })
            }
        }
    }
}
