// src/error/parse_errors.rs
use crate::error::ErrorCollector;
use crate::error::diagnostic::{Diagnostic, Span};
use crate::lexer::TokenKind;

fn suggest_from_token(token: &TokenKind) -> Option<String> {
    match token {
        TokenKind::Identifier(name) => match name.as_str() {
            "println" | "printf" | "print_ln" =>
                Some("Luma uses print() — it already adds a newline automatically.".to_string()),
            "fn" | "def" | "func" | "function" | "fun" =>
                Some("Luma defines functions with the return type first:\n           int add(int x, int y) { return x + y; }\n           void greet(string name) { print(name); }".to_string()),
            "null" | "nil" | "undefined" | "none" | "None" | "NULL" =>
                Some("Luma uses 'empty' for absent values and 'maybe(T)' for optional types.".to_string()),
            "elif" =>
                Some("Luma uses 'else if' instead of 'elif'.".to_string()),
            "import" | "include" | "require" =>
                Some("Luma uses 'use' for imports: 'use math;'  (full import system coming soon)".to_string()),
            "var" | "let" | "const" =>
                Some("Luma uses explicit types for variables: 'int x = 5;' or 'string name = \"Luma\";'".to_string()),
            "puts" | "echo" | "console" =>
                Some("Luma uses print() to output values: 'print(\"hello\");'".to_string()),
            "true" | "false" => None,
            _ => None,
        },
        TokenKind::EqualEqual => Some("Did you mean '=' for assignment? '==' is for comparison.".to_string()),
        _ => None,
    }
}

impl ErrorCollector {
    pub fn add_parse_error(&mut self, error: crate::parser::ParseError) {
        match error {
            crate::parser::ParseError::UnexpectedToken {
                expected,
                got,
                line_num,
                col_num,
            } => {
                let source_line = self
                    .source
                    .lines()
                    .nth(line_num - 1)
                    .unwrap_or("")
                    .to_string();

                if let Some(suggestion) = suggest_from_token(&got) {
                    let span = Span {
                        filename: self.filename.clone(),
                        line: line_num,
                        column: col_num,
                        length: 1,
                    };
                    self.errors.push(Diagnostic::new_error(
                        "E002",
                        &format!("I don't know what '{}' means here", got),
                        span,
                        source_line,
                        &suggestion,
                    ));
                } else {
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

                    if expected == "statement" {
                        self.has_errors = true;
                        return;
                    }

                    // Special hint for () used instead of empty
                    let hint = if expected == "empty" {
                        "Use 'empty' to initialize an empty list or table: 'list(int) x = empty;'"
                            .to_string()
                    } else {
                        format!(
                            "Suggestion: Add a {} at the end of this statement.",
                            expected
                        )
                    };

                    self.errors.push(Diagnostic::new_error(
                        "E001",
                        &format!("Missing {}", expected),
                        span,
                        source_line,
                        &hint,
                    ));
                }
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

                let hint = if let Some(suggestion) = suggest_from_token(&token_kind) {
                    suggestion
                } else {
                    format!(
                        "I found '{}' here, but I was expecting a value like 42 or \"hello\".",
                        token_kind
                    )
                };

                self.errors.push(Diagnostic::new_error(
                    "E003",
                    "I was expecting an expression here",
                    span,
                    source_line,
                    &hint,
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
