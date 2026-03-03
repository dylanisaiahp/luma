// src/error/runtime_errors.rs
use crate::error::ErrorCollector;
use crate::error::diagnostic::{Diagnostic, Span};

impl ErrorCollector {
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
}
