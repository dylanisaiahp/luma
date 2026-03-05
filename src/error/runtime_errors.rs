// src/error/runtime_errors.rs
use crate::error::ErrorCollector;
use crate::error::diagnostic::{Diagnostic, Span};

fn hint_for_runtime(message: &str) -> (&'static str, &'static str) {
    if message.contains("Undefined variable") {
        ("E009", "Check if the variable is defined before using it.")
    } else if message.contains("Type mismatch") {
        (
            "E010",
            "Make sure the value type matches the variable declaration.",
        )
    } else if message.contains("Cannot convert") {
        (
            "E011",
            "The value can't be converted to the expected type. Check your input.",
        )
    } else if message.contains("range()") {
        (
            "E012",
            "range() requires two integers: range(start, end) where start < end.",
        )
    } else if message.contains("expects") && message.contains("argument") {
        ("E013", "Check how many arguments this function takes.")
    } else if message.contains("Unknown function") {
        let func_name = message.strip_prefix("Unknown function: ").unwrap_or("");
        match func_name {
            "println" | "printf" | "print_ln" => (
                "E014",
                "Luma uses print() — it already adds a newline automatically.",
            ),
            "fn" | "def" | "func" => (
                "E014",
                "Luma defines functions with the return type first: int add(int x) { ... }",
            ),
            _ => (
                "E014",
                "This function doesn't exist. Check the spelling or define it first.",
            ),
        }
    } else if message.contains("condition must be a boolean") {
        (
            "E015",
            "Conditions must be true or false. Try a comparison like x > 0.",
        )
    } else {
        ("E011", "Something went wrong at runtime.")
    }
}

impl ErrorCollector {
    pub fn add_runtime_error(
        &mut self,
        message: String,
        _hint: String,
        line: usize,
        column: usize,
    ) {
        // Filter out internal control flow signals — these should never reach the user
        if message.starts_with("__return__") || message.starts_with("__break__") {
            return;
        }

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

        let (code, hint) = hint_for_runtime(&message);

        self.errors.push(Diagnostic::new_error(
            code,
            &message,
            span,
            source_line,
            hint,
        ));

        self.has_errors = true;
    }
}
