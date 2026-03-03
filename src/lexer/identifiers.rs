// src/lexer/identifiers.rs
use crate::lexer::TokenKind;
use crate::lexer::{Lexer, LexerError};

impl Lexer {
    pub fn read_identifier(&mut self) -> String {
        let position = self.position;
        while self.ch.is_alphabetic() || self.ch == '_' {
            self.read_char();
        }
        self.input[position..self.position].iter().collect()
    }

    pub fn read_number_token(&mut self, line: usize, col: usize) -> Result<TokenKind, LexerError> {
        let position = self.position;

        while self.ch.is_ascii_digit() {
            self.read_char();
        }

        if self.ch == '.' && self.peek_char() == '.' {
            let num_str: String = self.input[position..self.position].iter().collect();
            match num_str.parse::<i64>() {
                Ok(num) => Ok(TokenKind::Number(num)),
                Err(_) => Err(LexerError::InvalidNumber(num_str, line, col)),
            }
        } else if self.ch == '.' {
            self.read_char();
            while self.ch.is_ascii_digit() {
                self.read_char();
            }
            let num_str: String = self.input[position..self.position].iter().collect();
            match num_str.parse::<f64>() {
                Ok(num) => Ok(TokenKind::FloatLiteral(num)),
                Err(_) => Err(LexerError::InvalidNumber(num_str, line, col)),
            }
        } else {
            let num_str: String = self.input[position..self.position].iter().collect();
            match num_str.parse::<i64>() {
                Ok(num) => Ok(TokenKind::Number(num)),
                Err(_) => Err(LexerError::InvalidNumber(num_str, line, col)),
            }
        }
    }
}
