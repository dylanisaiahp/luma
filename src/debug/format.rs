// src/debug/format.rs
use colored::Colorize;

pub fn lexer_tag() -> String {
    "[lexer]".blue().to_string()
}

pub fn parser_tag() -> String {
    "[parser]".yellow().to_string()
}

pub fn interpreter_tag() -> String {
    "[interpreter]".magenta().to_string()
}

pub fn level_tag(verbose: bool, filename: Option<&str>) -> String {
    let base = if verbose {
        "[verbose]".dimmed().to_string()
    } else {
        "[basic]".dimmed().to_string()
    };
    if let Some(name) = filename {
        format!("{} {}", base, name)
    } else {
        base
    }
}

pub fn top(tag: &str, msg: &str) -> String {
    format!("{} ╭─ {}", tag, msg)
}

pub fn mid(tag: &str, msg: &str) -> String {
    format!("{} ├─ {}", tag, msg)
}

pub fn mid_indent(tag: &str, msg: &str) -> String {
    format!("{} │  ├─ {}", tag, msg)
}

pub fn mid_indent_last(tag: &str, msg: &str) -> String {
    format!("{} │  ╰─ {}", tag, msg)
}

pub fn bot(tag: &str, msg: &str) -> String {
    format!("{} ╰─ {}", tag, msg)
}
