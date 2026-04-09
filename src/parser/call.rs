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

                    // Detect struct instantiation: Identifier(fieldName: value, ...)
                    let is_struct_instantiate = matches!(&expr.kind, ExprKind::Identifier(_))
                        && matches!(self.tokens.get(self.position + 1), Some(t) if matches!(&t.kind, TokenKind::Identifier(_)))
                        && matches!(self.tokens.get(self.position + 2), Some(t) if t.kind == TokenKind::Colon);

                    if is_struct_instantiate {
                        let struct_name = match &expr.kind {
                            ExprKind::Identifier(n) => n.clone(),
                            _ => panic!("struct instantiation requires identifier"),
                        };
                        self.advance(); // consume '('
                        let mut fields = Vec::new();

                        while let Some(t) = self.current_token() {
                            if t.kind == TokenKind::RParen {
                                break;
                            }
                            let field_name = match self.current_token().cloned() {
                                Some(t) => match t.kind {
                                    TokenKind::Identifier(n) => {
                                        self.advance();
                                        n
                                    }
                                    _ => {
                                        return Err(ParseError::UnexpectedToken {
                                            expected: "field name".to_string(),
                                            got: t.kind,
                                            line_num: t.line,
                                            col_num: t.column,
                                        });
                                    }
                                },
                                None => return Err(ParseError::UnexpectedEOF),
                            };
                            self.expect_token(TokenKind::Colon)?;
                            let value = self.parse_expression()?;
                            fields.push((field_name, value));

                            if let Some(t) = self.current_token()
                                && t.kind == TokenKind::Comma
                            {
                                self.advance();
                            }
                        }
                        self.expect_token(TokenKind::RParen)?;

                        expr = Expr {
                            kind: ExprKind::StructInstantiate {
                                name: struct_name,
                                fields,
                            },
                            line: token.line,
                            column: token.column,
                        };
                    } else {
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

                        match expr.kind {
                            ExprKind::Identifier(name) => {
                                expr = Expr {
                                    kind: ExprKind::Call { name, args },
                                    line: token.line,
                                    column: token.column,
                                };
                            }
                            ExprKind::EnumVariant { enum_name, variant } => {
                                expr = Expr {
                                    kind: ExprKind::EnumVariantData {
                                        enum_name,
                                        variant,
                                        data: args,
                                    },
                                    line: token.line,
                                    column: token.column,
                                };
                            }
                            _ => {
                                return Err(ParseError::UnexpectedToken {
                                    expected: "function name or enum variant".to_string(),
                                    got: token.kind,
                                    line_num: token.line,
                                    col_num: token.column,
                                });
                            }
                        }
                    }
                }
                Some(TokenKind::Dot) => {
                    let dot_token = self.current_token().cloned().unwrap();
                    self.advance(); // consume '.'

                    let member = match self.current_token().cloned() {
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
                                    expected: "field or method name".to_string(),
                                    got: t.kind,
                                    line_num: t.line,
                                    col_num: t.column,
                                });
                            }
                        }
                        None => return Err(ParseError::UnexpectedEOF),
                    };

                    if let Some(t) = self.current_token()
                        && t.kind == TokenKind::LParen
                    {
                        self.advance(); // consume '('
                        let mut args = Vec::new();
                        if let Some(next) = self.current_token()
                            && next.kind != TokenKind::RParen
                        {
                            // For .as(), .first(), .last() — accept bare type keywords as string args
                            if member == "as" || member == "first" || member == "last" {
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

                        // Check if this is an enum variant construction (EnumName.Variant(...))
                        // TODO: This heuristic uses uppercase-first-char to detect enums, but struct names
                        // also start with uppercase. Post-self-hosting, replace with proper type-system
                        // resolution that checks the actual type context.
                        let is_enum_variant = matches!(&expr.kind, ExprKind::Identifier(_))
                            && member
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false);

                        if is_enum_variant {
                            let enum_name = match &expr.kind {
                                ExprKind::Identifier(n) => n.clone(),
                                _ => panic!("enum variant with data requires identifier"),
                            };
                            expr = Expr {
                                kind: ExprKind::EnumVariantData {
                                    enum_name,
                                    variant: member,
                                    data: args,
                                },
                                line: dot_token.line,
                                column: dot_token.column,
                            };
                        } else {
                            expr = Expr {
                                kind: ExprKind::MethodCall {
                                    object: Box::new(expr),
                                    method: member,
                                    args,
                                },
                                line: dot_token.line,
                                column: dot_token.column,
                            };
                        }
                    } else {
                        // Check if this is an enum variant access (EnumName.Variant without parens)
                        let is_enum_variant = matches!(&expr.kind, ExprKind::Identifier(_))
                            && member
                                .chars()
                                .next()
                                .map(|c| c.is_uppercase())
                                .unwrap_or(false);

                        if is_enum_variant {
                            let enum_name = match &expr.kind {
                                ExprKind::Identifier(n) => n.clone(),
                                _ => panic!("enum variant requires identifier"),
                            };
                            expr = Expr {
                                kind: ExprKind::EnumVariant {
                                    enum_name,
                                    variant: member,
                                },
                                line: dot_token.line,
                                column: dot_token.column,
                            };
                        } else {
                            expr = Expr {
                                kind: ExprKind::FieldAccess {
                                    object: Box::new(expr),
                                    field: member,
                                },
                                line: dot_token.line,
                                column: dot_token.column,
                            };
                        }
                    }
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
        TokenKind::In => Some("in".to_string()),
        TokenKind::Not => Some("not".to_string()),
        TokenKind::Or => Some("or".to_string()),
        TokenKind::And => Some("and".to_string()),
        TokenKind::List => Some("list".to_string()),
        TokenKind::Table => Some("table".to_string()),
        _ => None,
    }
}

// For .as(int), .first(char) — type keywords as string arguments
fn type_keyword_as_string(kind: &TokenKind) -> Option<String> {
    match kind {
        TokenKind::Int => Some("int".to_string()),
        TokenKind::Float => Some("float".to_string()),
        TokenKind::Bool => Some("bool".to_string()),
        TokenKind::String => Some("string".to_string()),
        TokenKind::Char => Some("char".to_string()),
        _ => None,
    }
}
