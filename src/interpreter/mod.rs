// src/interpreter/mod.rs
mod assign;
mod builtins;
mod control;
mod core;
mod expressions;
mod operations;
mod statements;
mod value;

pub use core::Interpreter;

use crate::error::diagnostic::Span;

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub span: Span,
}
