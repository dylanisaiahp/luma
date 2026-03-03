// src/error/collector.rs
use crate::error::diagnostic::{Diagnostic, Severity};
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

    pub(crate) fn line_col_to_byte_range(
        &self,
        line: usize,
        column: usize,
        length: usize,
    ) -> std::ops::Range<usize> {
        let mut offset = 0usize;
        for (i, l) in self.source.lines().enumerate() {
            if i + 1 == line {
                let col_start = column.saturating_sub(1);
                let start = offset + col_start;
                let end = (start + length).min(offset + l.len());
                return start..end;
            }
            offset += l.len() + 1;
        }
        0..0
    }

    pub fn print_all(&self) {
        let mut colors = ColorGenerator::new();

        for warning in &self.warnings {
            let report = self.build_report(warning, &mut colors);
            match report.eprint((self.filename.clone(), Source::from(&self.source))) {
                Ok(_) => (),
                Err(e) => eprintln!("Failed to print warning: {}", e),
            }
        }

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
