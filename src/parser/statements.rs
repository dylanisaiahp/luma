// src/parser/statements.rs
use crate::ast::*;
use crate::debug::DebugLevel;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

use super::Parser;

impl Parser {
    pub fn parse_statement(&mut self) -> Option<Stmt> {
        crate::debug!(
            DebugLevel::Verbose,
            "Parsing statement at pos {}",
            self.position
        );
        let start_pos = self.position;
        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Print) => self.parse_print_statement(),
            Some(TokenKind::Int)
            | Some(TokenKind::Float)
            | Some(TokenKind::Bool)
            | Some(TokenKind::String) => self.parse_variable_declaration(),
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

    fn is_start_of_match_pattern(&self) -> bool {
        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Number(_)) => true,
            Some(TokenKind::Underscore) => true,
            Some(TokenKind::Identifier(name)) if name == "range" => true,
            _ => false,
        }
    }

    pub fn parse_match_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;
        if let Err(e) = self.expect_token(TokenKind::Match) {
            self.errors.push(e);
            self.position = start_pos;
            return None;
        }

        let value = match self.parse_expression() {
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

        let mut arms = Vec::new();
        let mut else_arm = None;
        let mut last_pos = 0;

        while let Some(token) = self.current_token().cloned() {
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
                TokenKind::Else => {
                    self.advance();
                    if let Err(e) = self.expect_token(TokenKind::Colon) {
                        self.errors.push(e);
                        return None;
                    }
                    let mut body = Vec::new();
                    while let Some(t) = self.current_token().cloned() {
                        match t.kind {
                            TokenKind::RBrace => break,
                            _ => {
                                if let Some(stmt) = self.parse_statement() {
                                    body.push(stmt);
                                } else {
                                    self.advance();
                                }
                            }
                        }
                    }
                    else_arm = Some(body);
                }
                TokenKind::Underscore => {
                    let pattern = MatchPattern::Wildcard;
                    self.advance();
                    if let Err(e) = self.expect_token(TokenKind::Colon) {
                        self.errors.push(e);
                        return None;
                    }

                    let mut body = Vec::new();
                    while let Some(t) = self.current_token().cloned() {
                        if matches!(t.kind, TokenKind::RBrace | TokenKind::Else)
                            || self.is_start_of_match_pattern()
                        {
                            break;
                        }
                        if let Some(stmt) = self.parse_statement() {
                            body.push(stmt);
                        } else {
                            self.advance();
                        }
                    }
                    arms.push(MatchArm { pattern, body });
                }
                _ => {
                    let expr = match self.parse_expression() {
                        Ok(e) => e,
                        Err(e) => {
                            self.errors.push(e);
                            self.advance();
                            continue;
                        }
                    };

                    if let Err(e) = self.expect_token(TokenKind::Colon) {
                        self.errors.push(e);
                        return None;
                    }

                    let pattern = match &expr.kind {
                        ExprKind::Integer(n) => MatchPattern::Integer(*n),
                        ExprKind::Range { start, end } => {
                            if let (ExprKind::Integer(s), ExprKind::Integer(e)) =
                                (&start.kind, &end.kind)
                            {
                                MatchPattern::Range(*s, *e)
                            } else {
                                self.errors.push(ParseError::UnexpectedToken {
                                    expected: "integer range bounds".to_string(),
                                    got: token.kind,
                                    line_num: token.line,
                                    col_num: token.column,
                                });
                                return None;
                            }
                        }
                        _ => {
                            self.errors.push(ParseError::UnexpectedToken {
                                expected: "valid match pattern (integer or range())".to_string(),
                                got: token.kind,
                                line_num: token.line,
                                col_num: token.column,
                            });
                            return None;
                        }
                    };

                    let mut body = Vec::new();
                    while let Some(t) = self.current_token().cloned() {
                        if matches!(t.kind, TokenKind::RBrace | TokenKind::Else)
                            || self.is_start_of_match_pattern()
                        {
                            break;
                        }
                        if let Some(stmt) = self.parse_statement() {
                            body.push(stmt);
                        } else {
                            self.advance();
                        }
                    }
                    arms.push(MatchArm { pattern, body });
                }
            }
        }

        crate::debug!(DebugLevel::Basic, "Parsed {} arms", arms.len());
        for (i, arm) in arms.iter().enumerate() {
            crate::debug!(DebugLevel::Basic, "Arm {}: {:?}", i, arm.pattern);
        }

        Some(Stmt::Match {
            value,
            arms,
            else_arm,
        })
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

    pub fn parse_while_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;
        if let Err(e) = self.expect_token(TokenKind::While) {
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

        Some(Stmt::While { condition, body })
    }

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

    fn parse_else_chain(&mut self) -> Option<Option<Vec<Stmt>>> {
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

    pub fn parse_print_statement(&mut self) -> Option<Stmt> {
        let start_pos = self.position;
        if let Err(e) = self.expect_token(TokenKind::Print) {
            self.errors.push(e);
            return None;
        }
        if let Err(e) = self.expect_token(TokenKind::LParen) {
            self.errors.push(e);
            self.position = start_pos;
            return None;
        }

        let value = match self.parse_expression() {
            Ok(expr) => expr,
            Err(e) => {
                self.errors.push(e);
                self.position = start_pos;
                return None;
            }
        };

        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::RParen) => {
                self.advance();
            }
            Some(_) => {
                let token = self.current_token().unwrap();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: ")".to_string(),
                    got: token.kind.clone(),
                    line_num: token.line,
                    col_num: token.column,
                });
                self.synchronize();
                return None;
            }
            None => {
                self.errors.push(ParseError::UnexpectedEOF);
                return None;
            }
        }

        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Semicolon) => {
                self.advance();
            }
            _ => {
                let last_token = &self.tokens[self.position - 1];
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "semicolon".to_string(),
                    got: TokenKind::Illegal("nothing".to_string()),
                    line_num: last_token.line,
                    col_num: last_token.column + 1,
                });
            }
        }

        Some(Stmt::Print(value))
    }
}
