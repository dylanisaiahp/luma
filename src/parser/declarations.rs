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

        if let Err(e) = self.expect_token(TokenKind::LBrace) {
            self.errors.push(e);
            return None;
        }

        let mut body = Vec::new();
        let mut last_body_pos = 0;
        while let Some(token) = self.current_token() {
            if self.position == last_body_pos {
                self.advance();
                last_body_pos = self.position;
                continue;
            }
            last_body_pos = self.position;

            match token.kind {
                TokenKind::RBrace => {
                    self.advance();
                    break;
                }
                TokenKind::Eof => {
                    self.errors.push(ParseError::UnexpectedEOF);
                    break;
                }
                _ => {
                    if let Some(stmt) = self.parse_statement() {
                        body.push(stmt);
                    } else {
                        self.advance();
                    }
                }
            }
        }

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

        // Table literals need special parsing: ("key": value, ...)
        let value = if type_name.starts_with("table") {
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

        Some(Stmt::VariableDeclaration {
            type_name,
            name,
            value,
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
