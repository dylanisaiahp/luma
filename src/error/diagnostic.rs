// src/error/diagnostic.rs
use colored::*;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub filename: String,
    pub line: usize,
    pub column: usize,
    pub length: usize,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub span: Span,
    pub source_line: String,
    pub hint: String,
}

impl Diagnostic {
    pub fn new_error(
        code: &str,
        message: &str,
        span: Span,
        source_line: String,
        hint: &str,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code: code.to_string(),
            message: message.to_string(),
            span,
            source_line,
            hint: hint.to_string(),
        }
    }

    pub fn new_warning(
        code: &str,
        message: &str,
        span: Span,
        source_line: String,
        hint: &str,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            code: code.to_string(),
            message: message.to_string(),
            span,
            source_line,
            hint: hint.to_string(),
        }
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.severity {
            Severity::Error => "[!]".red().bold(),
            Severity::Warning => "[?]".yellow().bold(),
        };

        writeln!(f, "{} {} ({})", prefix, self.message.yellow(), self.code)?;
        writeln!(
            f,
            "    --> {}:{}:{}",
            self.span.filename.cyan(),
            self.span.line,
            self.span.column
        )?;

        writeln!(f)?;

        if !self.source_line.is_empty() {
            writeln!(f, "   {}", self.source_line)?;

            if self.span.column > 0 && self.span.length > 0 {
                let pointer = format!("{:>width$}", "^".red().bold(), width = self.span.column + 3);
                let underline = "~".repeat(self.span.length - 1).red().bold();
                writeln!(f, "{}{}", pointer, underline)?;
            }
            writeln!(f)?;
        }

        writeln!(f, "   {}", self.hint)?;

        Ok(())
    }
}
