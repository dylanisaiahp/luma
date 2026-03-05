// src/parser/call.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_call_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expression()?;

        loop {
            match self.current_token().map(|t| t.kind.clone()) {
                Some(TokenKind::LParen) => {
                    let token = self.current_token().cloned().unwrap();
                    self.advance();

                    let mut args = Vec::new();
                    if let Some(next) = self.current_token()
                        && next.kind != TokenKind::RParen
                    {
                        args.push(self.parse_expression()?);
                        while let Some(t) = self.current_token().cloned() {
                            if t.kind == TokenKind::Comma {
                                self.advance();
                                args.push(self.parse_expression()?);
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect_token(TokenKind::RParen)?;

                    match expr.kind {
                        ExprKind::Identifier(name) => {
                            if name == "range" && args.len() == 2 {
                                expr = Expr {
                                    kind: ExprKind::Range {
                                        start: Box::new(args[0].clone()),
                                        end: Box::new(args[1].clone()),
                                    },
                                    line: token.line,
                                    column: token.column,
                                };
                            } else {
                                expr = Expr {
                                    kind: ExprKind::Call { name, args },
                                    line: token.line,
                                    column: token.column,
                                };
                            }
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                expected: "function name".to_string(),
                                got: token.kind,
                                line_num: token.line,
                                col_num: token.column,
                            });
                        }
                    }
                }
                Some(TokenKind::Dot) => {
                    let dot_token = self.current_token().cloned().unwrap();
                    self.advance(); // consume '.'

                    // Expect method name
                    let method = match self.current_token().cloned() {
                        Some(t) => match t.kind {
                            TokenKind::Identifier(name) => {
                                self.advance();
                                name
                            }
                            _ => {
                                return Err(ParseError::UnexpectedToken {
                                    expected: "method name".to_string(),
                                    got: t.kind,
                                    line_num: t.line,
                                    col_num: t.column,
                                });
                            }
                        },
                        None => return Err(ParseError::UnexpectedEOF),
                    };

                    // Parse optional args
                    let args = if let Some(t) = self.current_token()
                        && t.kind == TokenKind::LParen
                    {
                        self.advance(); // consume '('
                        let mut args = Vec::new();
                        if let Some(next) = self.current_token()
                            && next.kind != TokenKind::RParen
                        {
                            args.push(self.parse_expression()?);
                            while let Some(t) = self.current_token().cloned() {
                                if t.kind == TokenKind::Comma {
                                    self.advance();
                                    args.push(self.parse_expression()?);
                                } else {
                                    break;
                                }
                            }
                        }
                        self.expect_token(TokenKind::RParen)?;
                        args
                    } else {
                        Vec::new()
                    };

                    expr = Expr {
                        kind: ExprKind::MethodCall {
                            object: Box::new(expr),
                            method,
                            args,
                        },
                        line: dot_token.line,
                        column: dot_token.column,
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }
}
