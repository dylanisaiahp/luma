// src/interpreter/mod.rs
mod assign;
mod builtins;
mod control;
mod core;
mod expressions;
mod operations;
mod statements;
mod structs;
pub mod value;

pub use core::Interpreter;

use crate::ast::{Param, Stmt, StructField, StructMethod};
use crate::error::diagnostic::Span;

#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub span: Span,
    pub mutable: bool,
}

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub return_type: String,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub source_file: String,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub fields: Vec<StructField>,
    pub methods: Vec<StructMethod>,
}
