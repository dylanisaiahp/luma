// src/parser/expressions.rs
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

use super::Parser;

impl Parser {
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment_expression(0)
    }

    pub fn parse_assignment_expression(
        &mut self,
        _min_precedence: i32,
    ) -> Result<Expr, ParseError> {
        let left = self.parse_binary_expression(0)?;

        // Check for assignment operators
        if let Some(token) = self.current_token().cloned() {
            let (op_kind, _is_compound) = match token.kind {
                TokenKind::Equals => (None, false),
                TokenKind::PlusEquals => (Some(crate::ast::AssignOpKind::Add), true),
                TokenKind::MinusEquals => (Some(crate::ast::AssignOpKind::Subtract), true),
                TokenKind::StarEquals => (Some(crate::ast::AssignOpKind::Multiply), true),
                TokenKind::SlashEquals => (Some(crate::ast::AssignOpKind::Divide), true),
                _ => (None, false),
            };

            if let Some(op) = op_kind {
                // Only allow assignment if left is an identifier
                match &left.kind {
                    ExprKind::Identifier(name) => {
                        self.advance(); // consume the operator
                        let value = self.parse_assignment_expression(0)?;

                        // Create compound assignment node
                        return Ok(Expr {
                            kind: ExprKind::AssignOp {
                                name: name.clone(),
                                op,
                                value: Box::new(value),
                            },
                            line: token.line,
                            column: token.column,
                        });
                    }
                    _ => {
                        // Invalid assignment target
                        let err = ParseError::UnexpectedToken {
                            expected: "valid assignment target".to_string(),
                            got: token.kind,
                            line_num: token.line,
                            col_num: token.column,
                        };
                        self.errors.push(err);
                        self.synchronize();
                        return Ok(left);
                    }
                }
            } else if token.kind == TokenKind::Equals {
                // Simple assignment
                match &left.kind {
                    ExprKind::Identifier(name) => {
                        self.advance(); // consume '='
                        let value = self.parse_assignment_expression(0)?;
                        return Ok(Expr {
                            kind: ExprKind::Assign {
                                name: name.clone(),
                                value: Box::new(value),
                            },
                            line: token.line,
                            column: token.column,
                        });
                    }
                    _ => {
                        // Invalid assignment target
                        let err = ParseError::UnexpectedToken {
                            expected: "valid assignment target".to_string(),
                            got: token.kind,
                            line_num: token.line,
                            col_num: token.column,
                        };
                        self.errors.push(err);
                        self.synchronize();
                        return Ok(left);
                    }
                }
            }
        }

        Ok(left)
    }

    pub fn parse_binary_expression(&mut self, min_precedence: i32) -> Result<Expr, ParseError> {
        let mut left = self.parse_call_expression()?;

        while let Some(token) = self.current_token().cloned() {
            let op = match &token.kind {
                TokenKind::Plus => crate::syntax::BinaryOp::Add,
                TokenKind::Minus => crate::syntax::BinaryOp::Subtract,
                TokenKind::Star => crate::syntax::BinaryOp::Multiply,
                TokenKind::Slash => crate::syntax::BinaryOp::Divide,
                TokenKind::Greater => crate::syntax::BinaryOp::Greater,
                TokenKind::Less => crate::syntax::BinaryOp::Less,
                TokenKind::GreaterEqual => crate::syntax::BinaryOp::GreaterEqual,
                TokenKind::LessEqual => crate::syntax::BinaryOp::LessEqual,
                TokenKind::EqualEqual => crate::syntax::BinaryOp::Equal,
                TokenKind::BangEqual => crate::syntax::BinaryOp::NotEqual,
                _ => break,
            };

            let precedence = match op {
                crate::syntax::BinaryOp::Equal | crate::syntax::BinaryOp::NotEqual => 10,
                crate::syntax::BinaryOp::Greater
                | crate::syntax::BinaryOp::Less
                | crate::syntax::BinaryOp::GreaterEqual
                | crate::syntax::BinaryOp::LessEqual => 20,
                crate::syntax::BinaryOp::Add | crate::syntax::BinaryOp::Subtract => 30,
                crate::syntax::BinaryOp::Multiply | crate::syntax::BinaryOp::Divide => 40,
            };

            if precedence < min_precedence {
                break;
            }

            self.advance();
            let right = self.parse_binary_expression(precedence + 1)?;

            left = Expr {
                kind: ExprKind::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                line: token.line,
                column: token.column,
            };
        }

        Ok(left)
    }

    pub fn parse_call_expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary_expression()?;

        // Check for function call
        while let Some(token) = self.current_token().cloned() {
            if token.kind == TokenKind::LParen {
                self.advance(); // consume '('

                let mut args = Vec::new();

                // Parse arguments if not immediate closing paren
                if let Some(next) = self.current_token()
                    && next.kind != TokenKind::RParen
                {
                    args.push(self.parse_expression()?);

                    // Parse remaining arguments
                    while let Some(t) = self.current_token().cloned() {
                        if t.kind == TokenKind::Comma {
                            self.advance(); // consume ','
                            args.push(self.parse_expression()?);
                        } else {
                            break;
                        }
                    }
                }

                self.expect_token(TokenKind::RParen)?;

                // Check for special built-in functions that we want as expressions
                match expr.kind {
                    ExprKind::Identifier(name) => {
                        if name == "range" && args.len() == 2 {
                            // Special case: range() is an expression, not a function call
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
            } else {
                break;
            }
        }

        Ok(expr)
    }

    pub fn parse_primary_expression(&mut self) -> Result<Expr, ParseError> {
        match self.current_token().cloned() {
            Some(token) => {
                let line = token.line;
                let col = token.column;

                match token.kind {
                    TokenKind::LParen => {
                        self.advance();
                        let expr = self.parse_expression()?;
                        self.expect_token(TokenKind::RParen)?;
                        Ok(expr)
                    }
                    TokenKind::StringLiteral(_) | TokenKind::Interpolation(_) => {
                        self.parse_string_sequence()
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
                    TokenKind::Read | TokenKind::Int | TokenKind::Float | TokenKind::String => {
                        let name = match token.kind {
                            TokenKind::Read => "read".to_string(),
                            TokenKind::Int => "int".to_string(),
                            TokenKind::Float => "float".to_string(),
                            TokenKind::String => "string".to_string(),
                            _ => unreachable!(),
                        };
                        self.advance();
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

        // Keep consuming string literals and interpolations
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

        // If only one part, return it directly
        if parts.len() == 1 {
            return Ok(parts.remove(0));
        }

        // Combine multiple parts using binary operations
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
