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

                    // Expect method name — keywords are valid method names
                    let method = match self.current_token().cloned() {
                        Some(t) => {
                            let name = keyword_as_method_name(&t.kind);
                            if let Some(name) = name {
                                self.advance();
                                name
                            } else if let TokenKind::Identifier(name) = t.kind {
                                self.advance();
                                name
                            } else {
                                return Err(ParseError::UnexpectedToken {
                                    expected: "method name".to_string(),
                                    got: t.kind,
                                    line_num: t.line,
                                    col_num: t.column,
                                });
                            }
                        }
                        None => return Err(ParseError::UnexpectedEOF),
                    };

                    // Parse optional args — special case: .as(type) accepts type keywords
                    let args = if let Some(t) = self.current_token()
                        && t.kind == TokenKind::LParen
                    {
                        self.advance(); // consume '('
                        let mut args = Vec::new();
                        if let Some(next) = self.current_token()
                            && next.kind != TokenKind::RParen
                        {
                            // For .as(), .first(), .last() — accept bare type keywords as string args
                            if method == "as" || method == "first" || method == "last" {
                                if let Some(type_name) = type_keyword_as_string(
                                    &self
                                        .current_token()
                                        .cloned()
                                        .map(|t| t.kind)
                                        .unwrap_or(TokenKind::Eof),
                                ) {
                                    let t = self.current_token().cloned().unwrap();
                                    self.advance();
                                    args.push(Expr {
                                        kind: ExprKind::String(type_name),
                                        line: t.line,
                                        column: t.column,
                                    });
                                } else {
                                    args.push(self.parse_expression()?);
                                }
                            } else {
                                args.push(self.parse_expression()?);
                            }

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

// Allow keywords to be used as method names after a dot
fn keyword_as_method_name(kind: &TokenKind) -> Option<String> {
    match kind {
        TokenKind::Read => Some("read".to_string()),
        TokenKind::String => Some("string".to_string()),
        TokenKind::Int => Some("int".to_string()),
        TokenKind::Float => Some("float".to_string()),
        TokenKind::Bool => Some("bool".to_string()),
        TokenKind::Char => Some("char".to_string()),
        TokenKind::Word => Some("word".to_string()),
        TokenKind::In => Some("in".to_string()),
        TokenKind::Not => Some("not".to_string()),
        _ => None,
    }
}

// For .as(int), .first(char), .last(word) — type keywords as string arguments
fn type_keyword_as_string(kind: &TokenKind) -> Option<String> {
    match kind {
        TokenKind::Int => Some("int".to_string()),
        TokenKind::Float => Some("float".to_string()),
        TokenKind::Bool => Some("bool".to_string()),
        TokenKind::String => Some("string".to_string()),
        TokenKind::Char => Some("char".to_string()),
        TokenKind::Word => Some("word".to_string()),
        _ => None,
    }
}
