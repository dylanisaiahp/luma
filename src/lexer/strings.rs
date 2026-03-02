// src/lexer/strings.rs
use crate::lexer::LexerError;
use crate::lexer::{Lexer, Token, TokenKind};

impl Lexer {
    pub fn read_string(&mut self) -> Result<Vec<Token>, LexerError> {
        let start_line = self.line;
        let start_col = self.column;
        let start_byte = self.byte_pos;
        self.read_char(); // skip opening quote
        let mut tokens = Vec::new();
        let mut current_string = String::new();
        let mut string_start_line = start_line;
        let mut string_start_col = start_col + 1;
        let mut string_start_byte = start_byte + 1;

        while self.ch != '"' && self.ch != '\0' {
            if self.ch == '&' && self.peek_char() == '{' {
                // Push any accumulated string
                if !current_string.is_empty() {
                    tokens.push(Token {
                        kind: TokenKind::StringLiteral(current_string.clone()),
                        line: string_start_line,
                        column: string_start_col,
                        byte_pos: string_start_byte,
                    });
                    current_string.clear();
                }

                // Parse interpolation
                self.read_char(); // consume '&'
                self.read_char(); // consume '{'
                let ident_start = self.position;
                let ident_line = self.line;
                let ident_col = self.column;
                let ident_byte = self.byte_pos;

                while self.ch.is_alphabetic() || self.ch == '_' {
                    self.read_char();
                }

                if self.ch != '}' {
                    let found = if self.ch == '\0' {
                        "end of file".to_string()
                    } else {
                        format!("'{}'", self.ch)
                    };
                    return Err(LexerError::InvalidInterpolationSyntax(
                        found,
                        self.line,
                        self.column,
                    ));
                }

                let ident: String = self.input[ident_start..self.position].iter().collect();
                tokens.push(Token {
                    kind: TokenKind::Interpolation(ident),
                    line: ident_line,
                    column: ident_col,
                    byte_pos: ident_byte,
                });
                self.read_char(); // consume '}'

                // Reset string start position for next part
                string_start_line = self.line;
                string_start_col = self.column;
                string_start_byte = self.byte_pos;
            } else {
                // Regular string character
                if current_string.is_empty() {
                    string_start_line = self.line;
                    string_start_col = self.column;
                    string_start_byte = self.byte_pos;
                }

                if self.ch == '\\' {
                    match self.peek_char() {
                        'n' => {
                            current_string.push('\n');
                            self.read_char(); // consume 'n'
                            self.read_char(); // move past it
                        }
                        't' => {
                            current_string.push('\t');
                            self.read_char(); // consume 't'
                            self.read_char(); // move past it
                        }
                        '\\' => {
                            current_string.push('\\');
                            self.read_char(); // consume '\\'
                            self.read_char(); // move past it
                        }
                        '"' => {
                            current_string.push('"');
                            self.read_char(); // consume '"'
                            self.read_char(); // move past it
                        }
                        _ => {
                            current_string.push('\\');
                            self.read_char();
                        }
                    }
                } else {
                    if self.ch == '\n' {
                        self.line += 1;
                        self.column = 0;
                    }
                    current_string.push(self.ch);
                    self.read_char();
                }
            }
        }

        if self.ch == '\0' {
            return Err(LexerError::UnterminatedString(start_line, start_col));
        }

        // Push any remaining string
        if !current_string.is_empty() {
            tokens.push(Token {
                kind: TokenKind::StringLiteral(current_string),
                line: string_start_line,
                column: string_start_col,
                byte_pos: string_start_byte,
            });
        }

        self.read_char(); // skip closing quote
        Ok(tokens)
    }

    pub fn read_interpolation(&mut self) -> Result<String, LexerError> {
        self.read_char(); // skip '{'
        let position = self.position;

        while self.ch.is_alphabetic() || self.ch == '_' {
            self.read_char();
        }

        if self.ch != '}' {
            return Err(LexerError::UnexpectedCharacter(
                self.ch,
                self.line,
                self.column,
            ));
        }

        let ident = self.input[position..self.position].iter().collect();
        self.read_char(); // skip '}'
        Ok(ident)
    }
}
