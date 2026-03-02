// src/error/collector.rs
use crate::error::diagnostic::{Diagnostic, Severity, Span};

use ariadne::{ColorGenerator, Label, Report, ReportKind, Source};

#[derive(Debug)]
pub struct ErrorCollector {
    pub errors: Vec<Diagnostic>,
    pub warnings: Vec<Diagnostic>,
    pub source: String,
    pub filename: String,
    pub has_errors: bool,
}

impl ErrorCollector {
    pub fn new(source: &str, filename: &str) -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            source: source.to_string(),
            filename: filename.to_string(),
            has_errors: false,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.has_errors
    }

    pub fn add_warning(&mut self, diagnostic: Diagnostic) {
        self.warnings.push(diagnostic);
    }

    fn line_col_to_byte_range(
        &self,
        line: usize,
        column: usize,
        length: usize,
    ) -> std::ops::Range<usize> {
        let mut offset = 0usize;
        for (i, l) in self.source.lines().enumerate() {
            if i + 1 == line {
                let col_start = column.saturating_sub(1); // column is 1-based
                let start = offset + col_start;
                let end = (start + length).min(offset + l.len());
                return start..end;
            }
            offset += l.len() + 1;
        }
        0..0
    }

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

                // Find where to insert the semicolon
                let mut insertion_col = source_line.len() + 1; // default to end of line (1-based)

                // If there's a comment, insert before it
                if let Some(comment_index) = source_line.find('#') {
                    insertion_col = comment_index + 1; // 1-based
                }

                // Trim trailing whitespace before comment/end
                let trimmed = source_line[..insertion_col - 1].trim_end();
                insertion_col = trimmed.len() + 1;

                let column = insertion_col;
                let length = 1;
                let hint = format!(
                    "Suggestion: Add a {} at the end of this statement.",
                    expected
                );
                let code = "E001".to_string();

                let span = Span {
                    filename: self.filename.clone(),
                    line: line_num,
                    column,
                    length,
                };

                self.errors.push(Diagnostic::new_error(
                    &code,
                    &format!("Missing {}", expected),
                    span,
                    source_line,
                    &hint,
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

    pub fn add_runtime_error(&mut self, message: String, hint: String, line: usize, column: usize) {
        let source_line = if line > 0 {
            self.source.lines().nth(line - 1).unwrap_or("").to_string()
        } else {
            "".to_string()
        };

        let span = Span {
            filename: self.filename.clone(),
            line,
            column,
            length: 1,
        };

        let hint = if message.contains("Undefined variable") {
            "Check if the variable is defined before using it.".to_string()
        } else if message.contains("Type mismatch") {
            "Make sure the value type matches the variable declaration.".to_string()
        } else {
            hint
        };

        let code = if message.contains("Undefined variable") {
            "E009"
        } else if message.contains("Type mismatch") {
            "E010"
        } else {
            "E011"
        };

        self.errors.push(Diagnostic::new_error(
            code,
            &message,
            span,
            source_line,
            &hint,
        ));

        self.has_errors = true;
    }

    pub fn print_all(&self) {
        let mut colors = ColorGenerator::new();

        // Print warnings first (less severe)
        for warning in &self.warnings {
            let report = self.build_report(warning, &mut colors);
            match report.eprint((self.filename.clone(), Source::from(&self.source))) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to print warning: {}", e),
            }
        }

        // Then errors
        for error in &self.errors {
            let report = self.build_report(error, &mut colors);
            match report.eprint((self.filename.clone(), Source::from(&self.source))) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to print error: {}", e),
            }
        }

        if self.errors.len() > 1 {
            eprintln!("\n[!] Found {} errors", self.errors.len());
        }
        if !self.warnings.is_empty() {
            eprintln!("\n[?] Found {} warnings", self.warnings.len());
        }
    }

    fn build_report(
        &self,
        diagnostic: &Diagnostic,
        colors: &mut ColorGenerator,
    ) -> Report<'_, (String, std::ops::Range<usize>)> {
        let kind = match diagnostic.severity {
            Severity::Error => ReportKind::Error,
            Severity::Warning => ReportKind::Warning,
        };

        // Use the length you carefully calculated in add_parse_error
        let range = self.line_col_to_byte_range(
            diagnostic.span.line,
            diagnostic.span.column,
            diagnostic.span.length,
        );

        Report::build(kind, (diagnostic.span.filename.clone(), range.clone()))
            .with_message(&diagnostic.message)
            .with_code(&diagnostic.code)
            .with_label(
                Label::new((diagnostic.span.filename.clone(), range))
                    .with_message(&diagnostic.hint)
                    .with_color(colors.next()),
            )
            .finish()
    }
}
