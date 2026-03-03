// src/error/mod.rs
pub mod collector;
pub mod diagnostic;
pub mod lexer_errors;
pub mod parse_errors;
pub mod runtime_errors;

pub use collector::ErrorCollector;
