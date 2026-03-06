// src/lexer/lexer.rs
use crate::lexer::{LexerError, Token, TokenKind};

pub struct Lexer {
    pub input: Vec<char>,
    pub position: usize,
    pub read_position: usize,
    pub ch: char,
    pub line: usize,
    pub column: usize,
    pub byte_pos: usize,
    pub token_buffer: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lexer = Lexer {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
            column: 0,
            byte_pos: 0,
            token_buffer: Vec::new(),
        };
        lexer.read_char();
        lexer
    }

    pub fn next_token(&mut self) -> Result<Token, LexerError> {
        if let Some(token) = self.token_buffer.pop() {
            return Ok(token);
        }
        self.skip_whitespace();

        let line = self.line;
        let col = self.column;
        let byte_pos = self.byte_pos;
        let ch = self.ch;

        if ch == '#' {
            self.skip_comment();
            return self.next_token();
        }

        if ch == '\0' {
            return Ok(Token {
                kind: TokenKind::Eof,
                line,
                column: col,
                byte_pos,
            });
        }

        if ch == '"' {
            let mut string_tokens = self.read_string()?;
            if string_tokens.is_empty() {
                return Err(LexerError::UnexpectedCharacter('"', line, col));
            }
            let first_token = string_tokens.remove(0);
            for token in string_tokens.into_iter().rev() {
                self.token_buffer.push(token);
            }
            return Ok(first_token);
        }

        if ch.is_alphabetic() || ch == '_' {
            let literal = self.read_identifier();
            let kind = if let Ok(keyword) = literal.parse::<crate::syntax::Keyword>() {
                TokenKind::from(keyword)
            } else {
                TokenKind::Identifier(literal)
            };
            return Ok(Token {
                kind,
                line,
                column: col,
                byte_pos,
            });
        }

        if ch.is_ascii_digit() {
            let kind = self.read_number_token(line, col)?;
            return Ok(Token {
                kind,
                line,
                column: col,
                byte_pos,
            });
        }

        let kind = self.read_symbol(ch, line, col, byte_pos)?;
        Ok(Token {
            kind,
            line,
            column: col,
            byte_pos,
        })
    }

    pub fn tokenize(&mut self) -> (Vec<Token>, Vec<LexerError>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        loop {
            match self.next_token() {
                Ok(token) => {
                    let is_eof = matches!(token.kind, TokenKind::Eof);
                    tokens.push(token);
                    if is_eof {
                        break;
                    }
                }
                Err(e) => {
                    errors.push(e);
                    // Don't push Illegal token — just skip and continue
                }
            }
        }
        (tokens, errors)
    }
}
