// src/interpreter/mod.rs
mod assign;
mod builtins;
mod control;
mod core;
mod expressions;
mod operations;
mod statements;
pub mod value;

pub use core::Interpreter;

use crate::ast::{Param, Stmt};
use crate::error::diagnostic::Span;

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub return_type: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
}
