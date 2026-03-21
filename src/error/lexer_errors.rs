// src/error/lexer_errors.rs
use crate::error::ErrorCollector;
use crate::error::diagnostic::{Diagnostic, Span};

fn hint_for_unexpected_char(ch: char) -> String {
    match ch {
        '^' => "'^' isn't used in Luma. For exponentiation, multiply manually or use a function.".to_string(),
        '@' => "'@' isn't used in Luma. Maybe a typo?".to_string(),
        '$' => "'$' isn't used in Luma. Variables don't need a prefix — just use the name directly.".to_string(),
        '!' => "Luma uses 'not' for boolean negation: 'not done' instead of '!done'. '!=' works for not-equal.".to_string(),
        '?' => "'?' isn't used in Luma. For optional values, 'maybe()' is coming soon.".to_string(),
        '\\' => "Luma strings don't use backslash escapes. Use '&{var}' for interpolation.".to_string(),
        '/' => "Luma uses '#' for comments, not '//'. Try: # this is a comment".to_string(),
        '&' => "Luma uses 'and' instead of '&&': 'if x > 0 and y > 0 {'".to_string(),
        '|' => "Luma uses 'or' instead of '||': 'if done or cancelled {'".to_string(),
        _ => format!("'{}' isn't used in Luma. Maybe a typo?", ch),
    }
}

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
                    &format!("I don't understand '{}'", ch),
                    span,
                    source_line,
                    &hint_for_unexpected_char(ch),
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
