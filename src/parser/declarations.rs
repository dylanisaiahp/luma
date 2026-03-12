// src/parser/declarations.rs
use super::Parser;
use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::error::ParseError;

impl Parser {
    // Helper: parse a simple inner type (int/float/bool/string/char/word)
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
            Some(TokenKind::Word) => {
                self.advance();
                Some("word".to_string())
            }
            _ => {
                let token = self.current_or_eof();
                self.errors.push(ParseError::UnexpectedToken {
                    expected: "inner type (int/float/bool/string/char/word)".to_string(),
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
            Some(TokenKind::Word) => {
                self.advance();
                Some("word".to_string())
            }
            Some(TokenKind::Maybe) => {
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
                Some(format!("maybe({})", inner))
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
            // Struct type — identifier used as type name (e.g. Point, Person)
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
                            "type (int/float/bool/string/char/word/maybe/list/table or struct name)"
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
                    expected: "type (int/float/bool/string/char/word/maybe/list/table)".to_string(),
                    got: token.kind,
                    line_num: token.line,
                    col_num: token.column,
                });
                None
            }
        }
    }

    /// Peek ahead from the current position to decide if this looks like a
    /// table literal `("key": value, ...)` vs a function call `foo()` or
    /// a list literal `(1, 2, 3)`.
    ///
    /// Rules:
    /// - If current token is NOT `(`, it's not a table literal.
    /// - If current token is `(` followed immediately by `)`, it's an empty
    ///   literal — treat as table literal only if the declaration type is table.
    /// - If current token is `(` and we find a `:` before the matching `)`,
    ///   it's a table literal.
    /// - Otherwise (no `:` found), it's a list literal or function call.
    pub fn looks_like_table_literal(&self) -> bool {
        // Must start with `(`
        if !matches!(
            self.tokens.get(self.position).map(|t| &t.kind),
            Some(TokenKind::LParen)
        ) {
            return false;
        }

        // Scan ahead counting paren depth until we find a `:` or close the
        // outer `(`. We don't want to cross into nested function calls.
        let mut depth = 0;
        let mut i = self.position;
        while i < self.tokens.len() {
            match &self.tokens[i].kind {
                TokenKind::LParen => depth += 1,
                TokenKind::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        // Closed without finding `:` at depth 1 — not a table literal.
                        return false;
                    }
                }
                TokenKind::Colon if depth == 1 => {
                    // Found a `:` at the top level of the parens — table literal.
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

    pub fn parse_variable_declaration(&mut self) -> Option<Stmt> {
        let start_pos = self.position;

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

        // For table-typed variables, we need to decide whether the RHS is:
        //   a) a table literal: ("key": val, ...)  → use parse_table_literal
        //   b) a function call or expression: foo() → use parse_expression
        //
        // We use looks_like_table_literal() to distinguish. For non-table
        // types (list, etc.) we always use parse_expression.
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
                Ok(expr) => expr,
                Err(e) => {
                    self.errors.push(e);
                    self.position = start_pos;
                    return None;
                }
            }
        };

        // Check for 'else error_var { body }' before semicolon
        let else_error = if let Some(TokenKind::Else) = self.current_token().map(|t| &t.kind) {
            self.advance(); // consume 'else'
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
            else_error,
        })
    }

    // Parse table literal: ("key": value, "key2": value2) or empty
    pub fn parse_table_literal(&mut self) -> Result<Expr, ParseError> {
        let line = self.current_or_eof().line;
        let col = self.current_or_eof().column;

        // Handle `empty`
        if let Some(TokenKind::Empty) = self.current_token().map(|t| &t.kind) {
            self.advance();
            return Ok(Expr {
                kind: ExprKind::Empty,
                line,
                column: col,
            });
        }

        self.expect_token(TokenKind::LParen)?;

        let mut pairs = Vec::new();

        // Empty table literal ()
        if let Some(t) = self.current_token()
            && t.kind == TokenKind::RParen
        {
            self.advance();
            return Ok(Expr {
                kind: ExprKind::Table(pairs),
                line,
                column: col,
            });
        }

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
            kind: ExprKind::Table(pairs),
            line,
            column: col,
        })
    }
}
