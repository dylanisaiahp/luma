// src/parser/mod.rs
mod assignment;
mod binary;
mod call;
mod core;
mod declarations;
mod error;
mod expressions;
mod forloop;
mod ifelse;
mod matching;
mod primary;
mod printing;
mod statements;
mod whileloop;

pub use core::Parser;
pub use error::ParseError;
