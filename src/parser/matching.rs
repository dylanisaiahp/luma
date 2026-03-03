// src/parser/matching.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
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
                    arms.push(MatchArm {
                        pattern: MatchPattern::Wildcard,
                        body,
                    });
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

        Some(Stmt::Match {
            value,
            arms,
            else_arm,
        })
    }

    pub fn is_start_of_match_pattern(&self) -> bool {
        match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Number(_)) => true,
            Some(TokenKind::Underscore) => true,
            Some(TokenKind::Identifier(name)) if name == "range" => true,
            _ => false,
        }
    }
}
