// src/interpreter/statements.rs
use crate::error::diagnostic::Span;
use crate::interpreter::value::{RuntimeError, Value};
use crate::interpreter::{Interpreter, VarInfo};

impl Interpreter {
    pub fn execute_print(&mut self, expr: &crate::ast::Expr) -> Result<Value, RuntimeError> {
        let val = self.evaluate_expression(expr)?;
        match val {
            Value::String(s) => println!("{}", s),
            Value::Integer(i) => println!("{}", i),
            Value::Float(f) => {
                if f.abs() > 1_000_000_000_000.0 || (f.abs() < 0.0001 && f != 0.0) {
                    println!("{:e}", f)
                } else {
                    println!("{}", f)
                }
            }
            Value::Boolean(b) => println!("{}", b),
            Value::Void => println!(),
        }
        Ok(Value::Void)
    }

    pub fn execute_variable_declaration(
        &mut self,
        type_name: &str,
        name: &str,
        value: &crate::ast::Expr,
    ) -> Result<Value, RuntimeError> {
        let val = self.evaluate_expression(value)?;

        match (type_name, &val) {
            ("int", Value::Integer(_)) => (),
            ("float", Value::Float(_)) => (),
            ("bool", Value::Boolean(_)) => (),
            ("string", Value::String(_)) => (),
            (expected, actual) => {
                return Err(RuntimeError {
                    message: format!("Type mismatch: expected {}, got {:?}", expected, actual),
                    line: value.line,
                    column: value.column,
                });
            }
        }

        let span = Span {
            filename: String::new(),
            line: value.line,
            column: value.column,
            length: name.len(),
        };

        self.var_info.insert(
            name.to_string(),
            VarInfo {
                name: name.to_string(),
                span,
            },
        );

        self.declare_variable(name, val);
        Ok(Value::Void)
    }
}
