// src/parser/expressions.rs
use super::Parser;
use crate::ast::*;
use crate::parser::error::ParseError;

impl Parser {
    pub fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        self.parse_assignment_expression(0)
    }
}
