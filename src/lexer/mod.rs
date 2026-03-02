// src/lexer/mod.rs
use crate::debug::DebugLevel;
use thiserror::Error;

mod comments;
mod reader;
mod strings;
mod tokens;

pub use tokens::{Token, TokenKind};

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character: {0} at line {1}, column {2}")]
    UnexpectedCharacter(char, usize, usize),

    #[error("Unterminated string at line {0}, column {1}")]
    UnterminatedString(usize, usize),

    #[error("Invalid number: {0} at line {1}, column {2}")]
    InvalidNumber(String, usize, usize),

    #[error(
        "Invalid interpolation syntax at line {1}, column {2}. Expected '}}' after variable name, found {0}."
    )]
    InvalidInterpolationSyntax(String, usize, usize),
}

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
        // Check if we have buffered tokens
        if let Some(token) = self.token_buffer.pop() {
            return Ok(token);
        }

        crate::debug!(
            DebugLevel::Trace,
            "Lexer at line {}, col {}, byte {}",
            self.line,
            self.column,
            self.byte_pos
        );
        self.skip_whitespace();

        let line = self.line;
        let col = self.column;
        let byte_pos = self.byte_pos;
        let ch = self.ch;

        // Handle comments
        if ch == '#' {
            self.skip_comment();
            return self.next_token();
        }

        let kind = match ch {
            '\0' => TokenKind::Eof,

            // Single-character operators (with compound assignment support)
            '+' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    TokenKind::PlusEquals
                } else {
                    TokenKind::Plus
                }
            }
            '-' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    TokenKind::MinusEquals
                } else {
                    TokenKind::Minus
                }
            }
            '*' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    TokenKind::StarEquals
                } else {
                    TokenKind::Star
                }
            }
            '/' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    TokenKind::SlashEquals
                } else {
                    TokenKind::Slash
                }
            }
            '>' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '<' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '=' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    TokenKind::EqualEqual
                } else {
                    TokenKind::Equals
                }
            }
            '!' => {
                self.read_char();
                if self.ch == '=' {
                    self.read_char();
                    TokenKind::BangEqual
                } else {
                    self.read_char();
                    return Ok(Token {
                        kind: TokenKind::Illegal("Unexpected character '!'".to_string()),
                        line,
                        column: col,
                        byte_pos,
                    });
                }
            }
            '_' => {
                self.read_char();
                TokenKind::Underscore
            }

            '.' => {
                self.read_char();
                if self.ch == '.' {
                    self.read_char();
                    TokenKind::DotDot
                } else {
                    return Err(LexerError::UnexpectedCharacter('.', line, col));
                }
            }

            // Symbols
            '(' => {
                self.read_char();
                TokenKind::LParen
            }
            ')' => {
                self.read_char();
                TokenKind::RParen
            }
            '{' => {
                self.read_char();
                TokenKind::LBrace
            }
            '}' => {
                self.read_char();
                TokenKind::RBrace
            }
            ';' => {
                self.read_char();
                TokenKind::Semicolon
            }
            ',' => {
                self.read_char();
                TokenKind::Comma
            }
            ':' => {
                self.read_char();
                TokenKind::Colon
            }

            // Interpolation start
            '&' => {
                self.read_char();
                if self.ch == '{' {
                    let ident = self.read_interpolation()?;
                    TokenKind::Interpolation(ident)
                } else {
                    return Err(LexerError::UnexpectedCharacter(self.ch, line, col));
                }
            }

            // String literal
            '"' => {
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

            // Numbers and identifiers
            _ => {
                if ch.is_alphabetic() || ch == '_' {
                    let literal = self.read_identifier();
                    if let Ok(keyword) = literal.parse::<crate::syntax::Keyword>() {
                        TokenKind::from(keyword)
                    } else {
                        TokenKind::Identifier(literal)
                    }
                } else if ch.is_ascii_digit() {
                    self.read_number_token(line, col)?
                } else {
                    self.read_char();
                    return Ok(Token {
                        kind: TokenKind::Illegal(format!("Unexpected character '{}'", ch)),
                        line,
                        column: col,
                        byte_pos,
                    });
                }
            }
        };

        Ok(Token {
            kind,
            line,
            column: col,
            byte_pos,
        })
    }

    // Helper for reading numbers (handles both ints and floats)
    fn read_number_token(&mut self, line: usize, col: usize) -> Result<TokenKind, LexerError> {
        let position = self.position;

        while self.ch.is_ascii_digit() {
            self.read_char();
        }

        // Check if this is the start of a range operator
        if self.ch == '.' && self.peek_char() == '.' {
            // Just a number, not a float
            let num_str: String = self.input[position..self.position].iter().collect();
            match num_str.parse::<i64>() {
                Ok(num) => Ok(TokenKind::Number(num)),
                Err(_) => Err(LexerError::InvalidNumber(num_str, line, col)),
            }
        } else if self.ch == '.' {
            self.read_char(); // consume '.'
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

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        crate::debug!(DebugLevel::Basic, "Starting tokenization");
        let mut tokens = Vec::new();
        let mut count = 0;
        loop {
            match self.next_token() {
                Ok(token) => {
                    count += 1;
                    crate::debug!(
                        DebugLevel::Verbose,
                        "Token {}: {:?} at line {}, col {}",
                        count,
                        token.kind,
                        token.line,
                        token.column
                    );
                    let is_eof = matches!(token.kind, TokenKind::Eof);
                    tokens.push(token);
                    if is_eof {
                        break;
                    }
                }
                Err(e) => {
                    tokens.push(Token {
                        kind: TokenKind::Illegal(format!("{}", e)),
                        line: self.line,
                        column: self.column,
                        byte_pos: self.byte_pos,
                    });
                    self.read_char();
                }
            }
        }
        crate::debug!(
            DebugLevel::Basic,
            "Tokenization complete, {} tokens",
            tokens.len()
        );
        Ok(tokens)
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while self.ch.is_alphabetic() || self.ch == '_' {
            self.read_char();
        }
        self.input[position..self.position].iter().collect()
    }
}
