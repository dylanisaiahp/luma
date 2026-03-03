// src/error/parse_errors.rs
use crate::error::ErrorCollector;
use crate::error::diagnostic::{Diagnostic, Span};

impl ErrorCollector {
    pub fn add_parse_error(&mut self, error: crate::parser::ParseError) {
        match error {
            crate::parser::ParseError::UnexpectedToken {
                expected,
                got: _,
                line_num,
                col_num: _,
            } => {
                let source_line = self
                    .source
                    .lines()
                    .nth(line_num - 1)
                    .unwrap_or("")
                    .to_string();

                let mut insertion_col = source_line.len() + 1;

                if let Some(comment_index) = source_line.find('#') {
                    insertion_col = comment_index + 1;
                }

                let trimmed = source_line[..insertion_col - 1].trim_end();
                insertion_col = trimmed.len() + 1;

                let span = Span {
                    filename: self.filename.clone(),
                    line: line_num,
                    column: insertion_col,
                    length: 1,
                };

                self.errors.push(Diagnostic::new_error(
                    "E001",
                    &format!("Missing {}", expected),
                    span,
                    source_line,
                    &format!(
                        "Suggestion: Add a {} at the end of this statement.",
                        expected
                    ),
                ));
            }
            crate::parser::ParseError::ExpectedExpression(token_kind, line_num, col_num) => {
                let source_line = self
                    .source
                    .lines()
                    .nth(line_num - 1)
                    .unwrap_or("")
                    .to_string();

                let span = Span {
                    filename: self.filename.clone(),
                    line: line_num,
                    column: col_num,
                    length: 1,
                };

                self.errors.push(Diagnostic::new_error(
                    "E003",
                    "I was expecting an expression here",
                    span,
                    source_line,
                    &format!(
                        "I found {:?} here, but I was expecting a value like 42 or \"hello\".",
                        token_kind
                    ),
                ));
            }
            crate::parser::ParseError::UnexpectedEOF => {
                let span = Span {
                    filename: self.filename.clone(),
                    line: 0,
                    column: 0,
                    length: 0,
                };

                self.errors.push(Diagnostic::new_error(
                    "E004",
                    "The file ended unexpectedly",
                    span,
                    "".to_string(),
                    "Maybe you forgot a closing brace or semicolon?",
                ));
            }
        }

        self.has_errors = true;
    }
}
