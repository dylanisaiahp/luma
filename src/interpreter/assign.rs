// src/interpreter/assign.rs
use crate::ast::AssignOpKind;
use crate::interpreter::Interpreter;
use crate::interpreter::value::{RuntimeError, Value};

impl Interpreter {
    pub fn evaluate_assign(
        &mut self,
        name: &str,
        value: &crate::ast::Expr,
        _line: usize,
        _column: usize,
    ) -> Result<Value, RuntimeError> {
        let val = self.evaluate_expression(value)?;
        self.variables.insert(name.to_string(), val.clone());
        Ok(val)
    }

    pub fn evaluate_assign_op(
        &mut self,
        name: &str,
        op: &AssignOpKind,
        value: &crate::ast::Expr,
        line: usize,
        column: usize,
    ) -> Result<Value, RuntimeError> {
        let current = match self.variables.get(name) {
            Some(val) => val.clone(),
            None => {
                return Err(RuntimeError {
                    message: format!("Undefined variable: {}", name),
                    line,
                    column,
                });
            }
        };

        let right_val = self.evaluate_expression(value)?;

        let result = match (current, right_val, op) {
            // Integer operations
            (Value::Integer(l), Value::Integer(r), AssignOpKind::Add) => Value::Integer(l + r),
            (Value::Integer(l), Value::Integer(r), AssignOpKind::Subtract) => Value::Integer(l - r),
            (Value::Integer(l), Value::Integer(r), AssignOpKind::Multiply) => Value::Integer(l * r),
            (Value::Integer(l), Value::Integer(r), AssignOpKind::Divide) => {
                if r == 0 {
                    return Err(RuntimeError {
                        message: "Division by zero".to_string(),
                        line,
                        column,
                    });
                }
                Value::Integer(l / r)
            }

            // Float operations
            (Value::Float(l), Value::Float(r), AssignOpKind::Add) => Value::Float(l + r),
            (Value::Float(l), Value::Float(r), AssignOpKind::Subtract) => Value::Float(l - r),
            (Value::Float(l), Value::Float(r), AssignOpKind::Multiply) => Value::Float(l * r),
            (Value::Float(l), Value::Float(r), AssignOpKind::Divide) => {
                if r == 0.0 {
                    return Err(RuntimeError {
                        message: "Division by zero".to_string(),
                        line,
                        column,
                    });
                }
                Value::Float(l / r)
            }

            // Mixed integer/float
            (Value::Integer(l), Value::Float(r), AssignOpKind::Add) => Value::Float(l as f64 + r),
            (Value::Float(l), Value::Integer(r), AssignOpKind::Add) => Value::Float(l + r as f64),
            (Value::Integer(l), Value::Float(r), AssignOpKind::Subtract) => {
                Value::Float(l as f64 - r)
            }
            (Value::Float(l), Value::Integer(r), AssignOpKind::Subtract) => {
                Value::Float(l - r as f64)
            }
            (Value::Integer(l), Value::Float(r), AssignOpKind::Multiply) => {
                Value::Float(l as f64 * r)
            }
            (Value::Float(l), Value::Integer(r), AssignOpKind::Multiply) => {
                Value::Float(l * r as f64)
            }
            (Value::Integer(l), Value::Float(r), AssignOpKind::Divide) => {
                if r == 0.0 {
                    return Err(RuntimeError {
                        message: "Division by zero".to_string(),
                        line,
                        column,
                    });
                }
                Value::Float(l as f64 / r)
            }
            (Value::Float(l), Value::Integer(r), AssignOpKind::Divide) => {
                if r == 0 {
                    return Err(RuntimeError {
                        message: "Division by zero".to_string(),
                        line,
                        column,
                    });
                }
                Value::Float(l / r as f64)
            }

            // String concatenation
            (Value::String(l), Value::String(r), AssignOpKind::Add) => Value::String(l + &r),
            (Value::String(l), Value::Integer(r), AssignOpKind::Add) => {
                Value::String(l + &r.to_string())
            }
            (Value::String(l), Value::Float(r), AssignOpKind::Add) => {
                Value::String(l + &r.to_string())
            }

            _ => {
                return Err(RuntimeError {
                    message: "Type mismatch in compound assignment".to_string(),
                    line,
                    column,
                });
            }
        };

        self.variables.insert(name.to_string(), result.clone());
        Ok(result)
    }
}
