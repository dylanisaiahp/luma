// src/parser/declarations.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    // Helper: parse a simple inner type (int/float/bool/string/char or custom type)
    fn parse_inner_type(&mut self) -> Option<String> {
        match self.current_token().map(|t| t.kind.clone()) {
            Some(TokenKind::Int) => {
                self.advance();
                Some("int".to_string())
            }
            Some(TokenKind::Float) => {
                self.advance();
                Some("float".to_string())
            }
            Some(TokenKind::Bool) => {
                self.advance();
                Some("bool".to_string())
            }
            Some(TokenKind::String) => {
                self.advance();
                Some("string".to_string())
            }
            Some(TokenKind::Char) => {
                self.advance();
                Some("char".to_string())
            }
            Some(TokenKind::Identifier(name)) => {
                self.advance();
                Some(name)
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "inner type (int/float/bool/string/char or struct/enum name)"
                        .to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                None
            }
        }
    }

    // Helper: parse a full type string
    pub fn parse_type_string(&mut self) -> Option<String> {
        match self.current_token().map(|t| t.kind.clone()) {
            Some(TokenKind::Int) => {
                self.advance();
                Some("int".to_string())
            }
            Some(TokenKind::Float) => {
                self.advance();
                Some("float".to_string())
            }
            Some(TokenKind::Bool) => {
                self.advance();
                Some("bool".to_string())
            }
            Some(TokenKind::String) => {
                self.advance();
                Some("string".to_string())
            }
            Some(TokenKind::Char) => {
                self.advance();
                Some("char".to_string())
            }
            Some(TokenKind::Option) => {
                self.advance();
                if let Err(e) = self.expect_token(TokenKind::LParen) {
                    self.errors.push(e);
                    return None;
                }
                let inner = self.parse_inner_type()?;
                if let Err(e) = self.expect_token(TokenKind::RParen) {
                    self.errors.push(e);
                    return None;
                }
                Some(format!("option({})", inner))
            }
            Some(TokenKind::Worry) => {
                self.advance();
                if let Err(e) = self.expect_token(TokenKind::LParen) {
                    self.errors.push(e);
                    return None;
                }
                let inner = self.parse_inner_type()?;
                if let Err(e) = self.expect_token(TokenKind::RParen) {
                    self.errors.push(e);
                    return None;
                }
                Some(format!("worry({})", inner))
            }
            Some(TokenKind::List) => {
                self.advance();
                if let Err(e) = self.expect_token(TokenKind::LParen) {
                    self.errors.push(e);
                    return None;
                }
                let inner = self.parse_inner_type()?;
                if let Err(e) = self.expect_token(TokenKind::RParen) {
                    self.errors.push(e);
                    return None;
                }
                Some(format!("list({})", inner))
            }
            Some(TokenKind::Table) => {
                self.advance();
                if let Err(e) = self.expect_token(TokenKind::LParen) {
                    self.errors.push(e);
                    return None;
                }
                let key_type = self.parse_inner_type()?;
                if let Err(e) = self.expect_token(TokenKind::Comma) {
                    self.errors.push(e);
                    return None;
                }
                let val_type = self.parse_inner_type()?;
                if let Err(e) = self.expect_token(TokenKind::RParen) {
                    self.errors.push(e);
                    return None;
                }
                Some(format!("table({}, {})", key_type, val_type))
            }
            // Struct or enum type — identifier used as type name
            // Only consume if followed by another identifier (the variable name)
            Some(TokenKind::Identifier(name)) => {
                if matches!(
                    self.tokens.get(self.position + 1),
                    Some(t) if matches!(&t.kind, TokenKind::Identifier(_))
                ) {
                    let name = name.clone();
                    self.advance();
                    Some(name)
                } else {
                    let token = self.current_or_eof();
                    self.errors.push(ParseError::UnexpectedToken {
                        expected:
                            "type (int/float/bool/string/char/option/list/table or struct/enum name)"
                                .to_string(),
                        got: token.kind,
                        line_num: token.line,
                        col_num: token.column,
                    });
                    None
                }
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "type (int/float/bool/string/char/option/list/table)".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                None
            }
        }
    }

    /// Peek ahead to decide if the RHS is a table literal `("key": value, ...)`.
    pub fn looks_like_table_literal(&self) -> bool {
        if !matches!(
            self.tokens.get(self.position).map(|t| &t.kind),
            Some(TokenKind::LParen)
        ) {
            return false;
        }

        let mut depth = 0;
        let mut i = self.position;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return false;
                    }
                }
                TokenKind::Colon if depth == 1 => {
                    return true;
                }
                TokenKind::Eof | TokenKind::Semicolon => break,
                _ => {}
            }
            i += 1;
        }
        false
    }

    pub fn parse_function(&mut self, return_type: String) -> Option<Stmt> {
        let name = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "function name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                self.synchronize();
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::LParen) {
            self.errors.push(e);
            self.synchronize();
            return None;
        }

        let mut params = Vec::new();
        while let Some(token) = self.current_token() {
            if token.kind == TokenKind::RParen {
                break;
            }

            let param_type = self.parse_type_string()?;

            let param_name = match self.current_token().map(|t| &t.kind) {
                Some(TokenKind::Identifier(name)) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => {
                    let token = self.current_or_eof();
                    self.errors.push(ParseError::UnexpectedToken {
                        expected: "parameter name".to_string(),
                        got: token.kind,
                        line_num: token.line,
                        col_num: token.column,
                    });
                    return None;
                }
            };

            params.push(Param {
                type_name: param_type,
                name: param_name,
            });

            if let Some(TokenKind::Comma) = self.current_token().map(|t| &t.kind) {
                self.advance();
            }
        }

        if let Err(e) = self.expect_token(TokenKind::RParen) {
            self.errors.push(e);
            return None;
        }

        let body = self.parse_block()?;

        Some(Stmt::Function {
            return_type,
            name,
            params,
            body,
        })
    }

    /// Parse an enum declaration: enum Color { Red, Green, Blue }
    /// or with data: enum Result { Ok(int), Err(string) }
    pub fn parse_enum_declaration(&mut self) -> Option<Stmt> {
        // consume 'enum'
        self.advance();

        let name = match self.current_token().cloned() {
            Some(t) => match t.kind {
                TokenKind::Identifier(n) => {
                    self.advance();
                    n
                }
                _ => {
                    self.errors.push(ParseError::UnexpectedToken {
                        expected: "enum name".to_string(),
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

        if let Err(e) = self.expect_token(TokenKind::LBrace) {
            self.errors.push(e);
            return None;
        }

        let mut variants = Vec::new();

        loop {
            match self.current_token().cloned() {
                Some(t) if t.kind == TokenKind::RBrace => {
                    self.advance();
                    break;
                }
                Some(t) => match t.kind {
                    TokenKind::Identifier(variant_name) => {
                        self.advance();

                        let data_type =
                            if self.current_token().map(|t| &t.kind) == Some(&TokenKind::LParen) {
                                self.advance();
                                let inner = self.parse_inner_type()?;
                                if let Err(e) = self.expect_token(TokenKind::RParen) {
                                    self.errors.push(e);
                                    return None;
                                }
                                Some(inner)
                            } else {
                                None
                            };

                        variants.push(crate::ast::EnumVariant {
                            name: variant_name,
                            data_type,
                        });

                        if let Some(TokenKind::Comma) = self.current_token().map(|t| &t.kind) {
                            self.advance();
                        }
                    }
                    _ => {
                        self.errors.push(ParseError::UnexpectedToken {
                            expected: "variant name or '}'".to_string(),
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
            }
        }

        Some(Stmt::EnumDeclaration { name, variants })
    }

    pub fn parse_variable_declaration(&mut self) -> Option<Stmt> {
        let start_pos = self.position;

        // Check for mutable or const prefix
        let mutable = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Mutable) => {
                self.advance();
                true
            }
            Some(TokenKind::Const) => {
                // const is immutable — advance past it, mutable = false
                self.advance();
                false
            }
            _ => false,
        };

        let type_name = self.parse_type_string()?;

        let name = match self.current_token().map(|t| &t.kind) {
            Some(TokenKind::Identifier(name)) => {
                let name = name.clone();
                self.advance();
                name
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "variable name".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                self.position = start_pos;
                return None;
            }
        };

        if let Err(e) = self.expect_token(TokenKind::Equals) {
            self.errors.push(e);
            self.position = start_pos;
            return None;
        }

        let value = if type_name.starts_with("table") && self.looks_like_table_literal() {
            match self.parse_table_literal() {
                Ok(expr) => expr,
                Err(e) => {
                    self.errors.push(e);
                    self.position = start_pos;
                    return None;
                }
            }
        } else {
            match self.parse_expression() {
                Ok(expr) => {
                    if (type_name.starts_with("list") || type_name.starts_with("table"))
                        && matches!(expr.kind, ExprKind::List(ref items) if items.is_empty())
                    {
                        self.errors.push(ParseError::UnexpectedToken {
                            expected: "empty".to_string(),
                            got: TokenKind::LParen,
                            line_num: expr.line,
                            col_num: expr.column,
                        });
                        self.position = start_pos;
                        return None;
                    }
                    expr
                }
                Err(e) => {
                    self.errors.push(e);
                    self.position = start_pos;
                    return None;
                }
            }
        };

        // Check for 'else error_var { body }' before semicolon
        let else_error = if let Some(TokenKind::Else) = self.current_token().map(|t| &t.kind) {
            self.advance();
            let error_var = match self.current_token().map(|t| &t.kind) {
                Some(TokenKind::Identifier(name)) => {
                    let name = name.clone();
                    self.advance();
                    name
                }
                _ => {
                    let token = self.current_or_eof();
                    self.errors.push(ParseError::UnexpectedToken {
                        expected: "error variable name".to_string(),
                        got: token.kind,
                        line_num: token.line,
                        col_num: token.column,
                    });
                    return None;
                }
            };
            let body = self.parse_block()?;
            Some((error_var, body))
        } else {
            None
        };

        if else_error.is_none() {
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
        }

        Some(Stmt::VariableDeclaration {
            type_name,
            name,
            value,
            mutable,
            else_error,
        })
    }

    pub fn parse_table_literal(&mut self) -> Result<Expr, ParseError> {
        let line = self.current_or_eof().line;
        let col = self.current_or_eof().column;

        if let Some(TokenKind::Empty) = self.current_token().map(|t| &t.kind) {
            self.advance();
            return Ok(Expr {
                file_path: self.current_file.clone(),
                kind: ExprKind::Empty,
                line,
                column: col,
            });
        }

        self.expect_token(TokenKind::LParen)?;

        let mut pairs = Vec::new();

        loop {
            let key = self.parse_expression()?;
            self.expect_token(TokenKind::Colon)?;
            let val = self.parse_expression()?;
            pairs.push((key, val));

            match self.current_token().map(|t| t.kind.clone()) {
                Some(TokenKind::Comma) => {
                    self.advance();
                    if let Some(t) = self.current_token()
                        && t.kind == TokenKind::RParen
                    {
                        break;
                    }
                }
                _ => break,
            }
        }

        self.expect_token(TokenKind::RParen)?;

        Ok(Expr {
            file_path: self.current_file.clone(),
            kind: ExprKind::Table(pairs),
            line,
            column: col,
        })
    }
}
