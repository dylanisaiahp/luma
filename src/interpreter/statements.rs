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
            Value::Maybe(Some(inner)) => match inner.as_ref() {
                Value::String(s) => println!("{}", s),
                Value::Integer(n) => println!("{}", n),
                Value::Float(f) => {
                    if f.abs() > 1_000_000_000_000.0 || (f.abs() < 0.0001 && *f != 0.0) {
                        println!("{:e}", f)
                    } else {
                        println!("{}", f)
                    }
                }
                Value::Boolean(b) => println!("{}", b),
                _ => println!("empty"),
            },
            Value::Maybe(None) => println!("empty"),
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
            (t, Value::Maybe(_)) if t.starts_with("maybe") => (),
            // Auto-wrap plain values into Maybe when declared as maybe type
            (t, _) if t.starts_with("maybe") => {
                let wrapped = Value::Maybe(Some(Box::new(val.clone())));
                self.declare_variable(name, wrapped);
                // store span info
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
                return Ok(Value::Void);
            }
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
