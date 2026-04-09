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
                if let Some(TokenKind::Semicolon) = self.current_token().map(|t| &t.kind) {
                    self.advance();
                }
                Some(Stmt::Break)
            }
            Some(TokenKind::For) => self.parse_for_statement(),
            Some(TokenKind::Struct) => self.parse_struct_declaration(),
            Some(TokenKind::Enum) => self.parse_enum_declaration(),
            // mutable/const prefix — route to variable declaration
            Some(TokenKind::Mutable) | Some(TokenKind::Const) => self.parse_variable_declaration(),
            Some(TokenKind::Int)
            | Some(TokenKind::Float)
            | Some(TokenKind::Bool)
            | Some(TokenKind::String)
            | Some(TokenKind::Char)
            | Some(TokenKind::Option)
            | Some(TokenKind::Worry)
            | Some(TokenKind::List)
            | Some(TokenKind::Table) => self.parse_variable_declaration(),
            // Struct/enum variable declaration: Point p = ...
            // Detect: Identifier Identifier = pattern
            Some(TokenKind::Identifier(_))
                if matches!(
                    self.tokens.get(self.position + 1),
                    Some(t) if matches!(&t.kind, TokenKind::Identifier(_))
                ) && matches!(
                    self.tokens.get(self.position + 2),
                    Some(t) if t.kind == TokenKind::Equals
                ) =>
            {
                self.parse_variable_declaration()
            }
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
            Some(TokenKind::Raise) => self.parse_raise_statement(),
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

    pub fn parse_raise_statement(&mut self) -> Option<Stmt> {
        let token = self.current_or_eof();
        let line = token.line;
        let column = token.column;
        self.advance(); // consume 'raise'

        let message = match self.parse_expression() {
            Ok(e) => e,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        if let Some(TokenKind::Semicolon) = self.current_token().map(|t| &t.kind) {
            self.advance();
        }

        Some(Stmt::Raise {
            message,
            line,
            column,
        })
    }

    pub fn parse_return_statement(&mut self) -> Option<Stmt> {
        self.advance(); // consume 'return'

        if let Some(TokenKind::Semicolon) = self.current_token().map(|t| &t.kind) {
            self.advance();
            return Some(Stmt::Return(None));
        }

        let expr = if self.looks_like_table_literal() {
            match self.parse_table_literal() {
                Ok(e) => e,
                Err(e) => {
                    self.errors.push(e);
                    return None;
                }
            }
        } else {
            match self.parse_expression() {
                Ok(e) => e,
                Err(e) => {
                    self.errors.push(e);
                    return None;
                }
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
