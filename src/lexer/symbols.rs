// src/lexer/symbols.rs
use crate::lexer::TokenKind;
use crate::lexer::{Lexer, LexerError};

impl Lexer {
    pub fn read_symbol(
        &mut self,
        ch: char,
        line: usize,
        col: usize,
        _byte_pos: usize,
    ) -> Result<TokenKind, LexerError> {
        match ch {
            '+' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    Ok(TokenKind::PlusEquals)
                } else {
                    Ok(TokenKind::Plus)
                }
            }
            '-' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    Ok(TokenKind::MinusEquals)
                } else {
                    Ok(TokenKind::Minus)
                }
            }
            '*' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    Ok(TokenKind::StarEquals)
                } else {
                    Ok(TokenKind::Star)
                }
            }
            '/' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    Ok(TokenKind::SlashEquals)
                } else if self.ch == '/' {
                    // "//" — common mistake from other languages
                    self.read_char();
                    Err(LexerError::UnexpectedCharacter('/', line, col))
                } else {
                    Ok(TokenKind::Slash)
                }
            }
            '>' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    Ok(TokenKind::GreaterEqual)
                } else {
                    Ok(TokenKind::Greater)
                }
            }
            '<' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    Ok(TokenKind::LessEqual)
                } else {
                    Ok(TokenKind::Less)
                }
            }
            '=' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    Ok(TokenKind::EqualEqual)
                } else {
                    Ok(TokenKind::Equals)
                }
            }
            '!' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    Ok(TokenKind::BangEqual)
                } else {
                    self.read_char();
                    Ok(TokenKind::Illegal("Unexpected character '!'".to_string()))
                }
            }
            '&' => {
                self.read_char();
                if self.ch == '&' {
                    self.read_char();
                    Ok(TokenKind::And)
                } else if self.ch == '{' {
                    let ident = self.read_interpolation()?;
                    Ok(TokenKind::Interpolation(ident))
                } else {
                    Err(LexerError::UnexpectedCharacter(self.ch, line, col))
                }
            }
            '|' => {
                self.read_char();
                if self.ch == '|' {
                    self.read_char();
                    Ok(TokenKind::Or)
                } else {
                    Err(LexerError::UnexpectedCharacter('|', line, col))
                }
            }
            '_' => {
                self.read_char();
                Ok(TokenKind::Underscore)
            }
            '.' => {
                self.read_char();
                if self.ch == '.' {
                    self.read_char();
                    Ok(TokenKind::DotDot)
                } else {
                    Ok(TokenKind::Dot)
                }
            }
            '(' => {
                self.read_char();
                Ok(TokenKind::LParen)
            }
            ')' => {
                self.read_char();
                Ok(TokenKind::RParen)
            }
            '{' => {
                self.read_char();
                Ok(TokenKind::LBrace)
            }
            '}' => {
                self.read_char();
                Ok(TokenKind::RBrace)
            }
            ';' => {
                self.read_char();
                Ok(TokenKind::Semicolon)
            }
            ',' => {
                self.read_char();
                Ok(TokenKind::Comma)
            }
            ':' => {
                self.read_char();
                Ok(TokenKind::Colon)
            }
            '%' => {
                self.read_char();
                Ok(TokenKind::Percent)
            }
            _ => {
                self.read_char();
                Ok(TokenKind::Illegal(format!("Unexpected character '{}'", ch)))
            }
        }
    }
}
