// src/parser/assignment.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_assignment_expression(
        &mut self,
        _min_precedence: i32,
    ) -> Result<Expr, ParseError> {
        let left = self.parse_binary_expression(0)?;

        if let Some(token) = self.current_token().cloned() {
            let op_kind = match token.kind {
                TokenKind::PlusEquals => Some(AssignOpKind::Add),
                TokenKind::MinusEquals => Some(AssignOpKind::Subtract),
                TokenKind::StarEquals => Some(AssignOpKind::Multiply),
                TokenKind::SlashEquals => Some(AssignOpKind::Divide),
                _ => None,
            };

            if let Some(op) = op_kind {
                match &left.kind {
                    ExprKind::Identifier(name) => {
                        self.advance();
                        let value = self.parse_assignment_expression(0)?;
                        return Ok(Expr {
                            file_path: self.current_file.clone(),
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
                match &left.kind {
                    ExprKind::Identifier(name) => {
                        self.advance();
                        let value = self.parse_assignment_expression(0)?;
                        return Ok(Expr {
                            file_path: self.current_file.clone(),
                            kind: ExprKind::Assign {
                                name: name.clone(),
                                value: Box::new(value),
                            },
                            line: token.line,
                            column: token.column,
                        });
                    }
                    _ => {
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
}
