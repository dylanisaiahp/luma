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
                // Set pattern: ("hello", "hi", 1):
                TokenKind::LParen => {
                    self.advance(); // consume '('
                    let mut patterns = Vec::new();

                    loop {
                        let pat = match self.current_token().cloned() {
                            Some(t) => match t.kind {
                                TokenKind::StringLiteral(s) => {
                                    self.advance();
                                    MatchPattern::String(s)
                                }
                                TokenKind::Number(n) => {
                                    self.advance();
                                    MatchPattern::Integer(n)
                                }
                                TokenKind::RParen => break,
                                _ => {
                                    self.errors.push(ParseError::UnexpectedToken {
                                        expected: "string or integer in set pattern".to_string(),
                                        got: t.kind,
                                        line_num: t.line,
                                        col_num: t.column,
                                    });
                                    return None;
                                }
                            },
                            None => {
                                self.errors.push(ParseError::UnexpectedEOF);
                                return None;
                            }
                        };
                        patterns.push(pat);

                        match self.current_token().map(|t| t.kind.clone()) {
                            Some(TokenKind::Comma) => {
                                self.advance();
                            }
                            Some(TokenKind::RParen) => break,
                            _ => break,
                        }
                    }

                    if let Err(e) = self.expect_token(TokenKind::RParen) {
                        self.errors.push(e);
                        return None;
                    }
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
                        pattern: MatchPattern::Set(patterns),
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
                        ExprKind::String(s) => MatchPattern::String(s.clone()),
                        ExprKind::Call { name, args } if name == "range" && args.len() == 2 => {
                            if let (ExprKind::Integer(s), ExprKind::Integer(e)) =
                                (&args[0].kind, &args[1].kind)
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
                                expected:
                                    "valid match pattern (integer, string, range(), set, or else)"
                                        .to_string(),
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
            Some(TokenKind::StringLiteral(_)) => true,
            Some(TokenKind::LParen) => true,
            Some(TokenKind::Identifier(name)) if name == "range" => true,
            _ => false,
        }
    }
}
