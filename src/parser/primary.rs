// src/parser/primary.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        match self.current_token().cloned() {
            Some(token) => {
                let line = token.line;
                let col = token.column;

                match token.kind {
                    TokenKind::Not => {
                        self.advance();
                        let operand = self.parse_primary_expression()?;
                        Ok(Expr {
                            kind: ExprKind::Not(Box::new(operand)),
                            line,
                            column: col,
                        })
                    }
                    TokenKind::Minus => {
                        self.advance();
                        let operand = self.parse_primary_expression()?;
                        match operand.kind {
                            ExprKind::Integer(n) => Ok(Expr {
                                kind: ExprKind::Integer(-n),
                                line,
                                column: col,
                            }),
                            ExprKind::Float(f) => Ok(Expr {
                                kind: ExprKind::Float(-f),
                                line,
                                column: col,
                            }),
                            _ => Ok(Expr {
                                kind: ExprKind::BinaryOp {
                                    left: Box::new(Expr {
                                        kind: ExprKind::Integer(0),
                                        line,
                                        column: col,
                                    }),
                                    op: crate::syntax::BinaryOp::Subtract,
                                    right: Box::new(operand),
                                },
                                line,
                                column: col,
                            }),
                        }
                    }
                    TokenKind::LParen => {
                        self.advance(); // consume '('

                        // Empty parens () — only valid as a list literal in explicit list context.
                        // For list/table variable declarations, `empty` must be used instead.
                        // () is still valid as an empty list expression in other contexts (e.g. function args).
                        if let Some(t) = self.current_token()
                            && t.kind == TokenKind::RParen
                        {
                            self.advance();
                            return Ok(Expr {
                                kind: ExprKind::List(Vec::new()),
                                line,
                                column: col,
                            });
                        }

                        let first = self.parse_expression()?;

                        if let Some(t) = self.current_token()
                            && t.kind == TokenKind::Comma
                        {
                            let mut items = vec![first];
                            while let Some(t) = self.current_token() {
                                if t.kind != TokenKind::Comma {
                                    break;
                                }
                                self.advance();
                                if let Some(t) = self.current_token()
                                    && t.kind == TokenKind::RParen
                                {
                                    break;
                                }
                                items.push(self.parse_expression()?);
                            }
                            self.expect_token(TokenKind::RParen)?;
                            return Ok(Expr {
                                kind: ExprKind::List(items),
                                line,
                                column: col,
                            });
                        }

                        self.expect_token(TokenKind::RParen)?;
                        Ok(first)
                    }
                    TokenKind::StringLiteral(_) | TokenKind::Interpolation(_) => {
                        self.parse_string_sequence()
                    }
                    // Single-quoted literal — always a char (word type removed)
                    TokenKind::CharLiteral(s) => {
                        self.advance();
                        Ok(Expr {
                            kind: ExprKind::Char(s),
                            line,
                            column: col,
                        })
                    }
                    TokenKind::Number(n) => {
                        self.advance();
                        Ok(Expr {
                            kind: ExprKind::Integer(n),
                            line,
                            column: col,
                        })
                    }
                    TokenKind::FloatLiteral(f) => {
                        self.advance();
                        Ok(Expr {
                            kind: ExprKind::Float(f),
                            line,
                            column: col,
                        })
                    }
                    TokenKind::True => {
                        self.advance();
                        Ok(Expr {
                            kind: ExprKind::Boolean(true),
                            line,
                            column: col,
                        })
                    }
                    TokenKind::False => {
                        self.advance();
                        Ok(Expr {
                            kind: ExprKind::Boolean(false),
                            line,
                            column: col,
                        })
                    }
                    TokenKind::Empty => {
                        self.advance();
                        Ok(Expr {
                            kind: ExprKind::Empty,
                            line,
                            column: col,
                        })
                    }
                    TokenKind::Some => {
                        self.advance();
                        self.expect_token(TokenKind::LParen)?;
                        let inner = self.parse_expression()?;
                        self.expect_token(TokenKind::RParen)?;
                        Ok(Expr {
                            kind: ExprKind::Some(Box::new(inner)),
                            line,
                            column: col,
                        })
                    }
                    TokenKind::None => {
                        self.advance();
                        Ok(Expr {
                            kind: ExprKind::None,
                            line,
                            column: col,
                        })
                    }
                    TokenKind::Read
                    | TokenKind::Int
                    | TokenKind::Float
                    | TokenKind::String
                    | TokenKind::Char
                    | TokenKind::Env
                    | TokenKind::Home => {
                        let name = match token.kind {
                            TokenKind::Read => "read".to_string(),
                            TokenKind::Int => "int".to_string(),
                            TokenKind::Float => "float".to_string(),
                            TokenKind::String => "string".to_string(),
                            TokenKind::Char => "char".to_string(),
                            TokenKind::Env => "env".to_string(),
                            TokenKind::Home => "home".to_string(),
                            _ => {
                                return Err(ParseError::UnexpectedToken {
                                    expected: "type keyword".to_string(),
                                    got: self.current_or_eof().kind,
                                    line_num: line,
                                    col_num: col,
                                })
                            }
                        };
                        self.advance();

                        // Check for type constant: int.max, int.min, float.max, float.min
                        if (name == "int" || name == "float")
                            && self.current_token().map(|t| &t.kind) == Some(&TokenKind::Dot)
                            && let Some(next) = self.tokens.get(self.position + 1)
                            && let TokenKind::Identifier(ref ident) = next.kind
                            && (ident == "max" || ident == "min")
                        {
                            self.advance(); // consume '.'
                            let ident = match &self.tokens[self.position].kind {
                                TokenKind::Identifier(s) => s.clone(),
                                _ => {
                                    return Err(ParseError::UnexpectedToken {
                                        expected: "type constant identifier".to_string(),
                                        got: self.current_or_eof().kind,
                                        line_num: line,
                                        col_num: col,
                                    })
                                }
                            };
                            self.advance(); // consume 'max'/'min'
                            return Ok(Expr {
                                kind: ExprKind::TypeConstant {
                                    type_name: name,
                                    constant: ident,
                                },
                                line,
                                column: col,
                            });
                        }

                        if self.current_token().map(|t| &t.kind) == Some(&TokenKind::LParen) {
                            self.advance(); // consume '('
                            let mut args = Vec::new();
                            while let Some(t) = self.current_token() {
                                if t.kind == TokenKind::RParen {
                                    break;
                                }
                                args.push(self.parse_expression()?);
                                if let Some(t) = self.current_token()
                                    && t.kind == TokenKind::Comma
                                {
                                    self.advance();
                                }
                            }
                            self.expect_token(TokenKind::RParen)?;
                            return Ok(Expr {
                                kind: ExprKind::Call { name, args },
                                line,
                                column: col,
                            });
                        }

                        Ok(Expr {
                            kind: ExprKind::Identifier(name),
                            line,
                            column: col,
                        })
                    }
                    TokenKind::Identifier(name) => {
                        self.advance();
                        Ok(Expr {
                            kind: ExprKind::Identifier(name),
                            line,
                            column: col,
                        })
                    }
                    _ => Err(ParseError::ExpectedExpression(
                        token.kind,
                        token.line,
                        token.column,
                    )),
                }
            }
            None => {
                let token = self.current_or_eof();
                Err(ParseError::ExpectedExpression(
                    token.kind,
                    token.line,
                    token.column,
                ))
            }
        }
    }

    pub fn parse_string_sequence(&mut self) -> Result<Expr, ParseError> {
        let mut parts = Vec::new();
        let start_line = self.current_token().unwrap().line;
        let start_col = self.current_token().unwrap().column;

        while let Some(token) = self.current_token().cloned() {
            match token.kind {
                TokenKind::StringLiteral(s) => {
                    self.advance();
                    parts.push(Expr {
                        kind: ExprKind::String(s),
                        line: token.line,
                        column: token.column,
                    });
                }
                TokenKind::Interpolation(ident) => {
                    self.advance();
                    parts.push(Expr {
                        kind: ExprKind::Interpolation(ident),
                        line: token.line,
                        column: token.column,
                    });
                }
                _ => break,
            }
        }

        if parts.is_empty() {
            return Ok(Expr {
                kind: ExprKind::String(String::new()),
                line: start_line,
                column: start_col,
            });
        }

        if parts.len() == 1 {
            return Ok(parts.remove(0));
        }

        let mut result = parts.remove(0);
        for part in parts {
            result = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(result),
                    op: crate::syntax::BinaryOp::Add,
                    right: Box::new(part),
                },
                line: start_line,
                column: start_col,
            };
        }

        Ok(result)
    }
}
