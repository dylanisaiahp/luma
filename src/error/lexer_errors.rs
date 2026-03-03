// src/error/lexer_errors.rs
use crate::error::ErrorCollector;
use crate::error::diagnostic::{Diagnostic, Span};

impl ErrorCollector {
    pub fn add_lexer_error(&mut self, error: crate::lexer::LexerError) {
        match error {
            crate::lexer::LexerError::UnexpectedCharacter(ch, line, col) => {
                let source_line = self.source.lines().nth(line - 1).unwrap_or("").to_string();
                let span = Span {
                    filename: self.filename.clone(),
                    line,
                    column: col,
                    length: 1,
                };

                self.errors.push(Diagnostic::new_error(
                    "E005",
                    "I found a character I don't understand",
                    span,
                    source_line,
                    &format!("The character '{}' isn't used in Luma. Maybe a typo?", ch),
                ));
            }
            crate::lexer::LexerError::UnterminatedString(line, col) => {
                let source_line = self.source.lines().nth(line - 1).unwrap_or("").to_string();
                let span = Span {
                    filename: self.filename.clone(),
                    line,
                    column: col,
                    length: 1,
                };

                self.errors.push(Diagnostic::new_error(
                    "E006",
                    "This string never ends",
                    span,
                    source_line,
                    "Did you forget to close the string with \"?",
                ));
            }
            crate::lexer::LexerError::InvalidNumber(num, line, col) => {
                let source_line = self.source.lines().nth(line - 1).unwrap_or("").to_string();
                let span = Span {
                    filename: self.filename.clone(),
                    line,
                    column: col,
                    length: num.len(),
                };

                self.errors.push(Diagnostic::new_error(
                    "E007",
                    "I can't understand this number",
                    span,
                    source_line,
                    &format!("'{}' isn't a valid number. Try something like 42.", num),
                ));
            }
            crate::lexer::LexerError::InvalidInterpolationSyntax(found, line, col) => {
                let source_line = self.source.lines().nth(line - 1).unwrap_or("").to_string();
                let span = Span {
                    filename: self.filename.clone(),
                    line,
                    column: col,
                    length: found.len(),
                };

                self.errors.push(Diagnostic::new_error(
                    "E008",
                    "Invalid interpolation syntax",
                    span,
                    source_line,
                    &format!("Expected '}}' after variable name, found '{}'.", found),
                ));
            }
        }

        self.has_errors = true;
    }
}
