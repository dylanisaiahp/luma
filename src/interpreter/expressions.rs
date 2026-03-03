// src/interpreter/expressions.rs
use crate::ast::*;
use crate::interpreter::Interpreter;
use crate::interpreter::builtins;
use crate::interpreter::operations;
use crate::interpreter::value::{RuntimeError, Value};

impl Interpreter {
    pub fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        crate::debug!(
            crate::debug::DebugLevel::Verbose,
            "Evaluating {:?}",
            expr.kind
        );
        match &expr.kind {
            ExprKind::Integer(n) => Ok(Value::Integer(*n)),
            ExprKind::Float(f) => Ok(Value::Float(*f)),
            ExprKind::String(s) => Ok(Value::String(s.clone())),
            ExprKind::Boolean(b) => Ok(Value::Boolean(*b)),
            ExprKind::Identifier(name) => {
                self.used_variables.insert(name.clone());
                match self.get_variable(name) {
                    Some(val) => Ok(val.clone()),
                    None => Err(RuntimeError {
                        message: format!("Undefined variable: {}", name),
                        line: expr.line,
                        column: expr.column,
                    }),
                }
            }
            ExprKind::Interpolation(ident) => {
                self.used_variables.insert(ident.clone());
                match self.get_variable(ident) {
                    Some(val) => Ok(Value::String(match val {
                        Value::Integer(n) => n.to_string(),
                        Value::Float(f) => f.to_string(),
                        Value::Boolean(b) => b.to_string(),
                        Value::Void => "void".to_string(),
                        Value::String(s) => s.clone(),
                    })),
                    None => Err(RuntimeError {
                        message: format!("Undefined variable in interpolation: {}", ident),
                        line: expr.line,
                        column: expr.column,
                    }),
                }
            }
            ExprKind::Assign { name, value } => {
                self.evaluate_assign(name, value, expr.line, expr.column)
            }
            ExprKind::AssignOp { name, op, value } => {
                self.evaluate_assign_op(name, op, value, expr.line, expr.column)
            }
            ExprKind::Call { name, args } => match name.as_str() {
                "read" => builtins::eval_read(args, expr.line, expr.column),
                "int" => {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: "int() takes exactly one argument".to_string(),
                            line: expr.line,
                            column: expr.column,
                        });
                    }
                    builtins::eval_int(&args[0], self, expr.line, expr.column)
                }
                "float" => {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: "float() takes exactly one argument".to_string(),
                            line: expr.line,
                            column: expr.column,
                        });
                    }
                    builtins::eval_float(&args[0], self, expr.line, expr.column)
                }
                "string" => {
                    if args.len() != 1 {
                        return Err(RuntimeError {
                            message: "string() takes exactly one argument".to_string(),
                            line: expr.line,
                            column: expr.column,
                        });
                    }
                    builtins::eval_string(&args[0], self)
                }
                "random" => builtins::eval_random(args, self, expr.line, expr.column),
                _ => Err(RuntimeError {
                    message: format!("Unknown function: {}", name),
                    line: expr.line,
                    column: expr.column,
                }),
            },
            ExprKind::Range { start, end } => {
                let start_val = self.evaluate_expression(start)?;
                let end_val = self.evaluate_expression(end)?;
                match (start_val, end_val) {
                    (Value::Integer(s), Value::Integer(e)) => {
                        if s >= e {
                            return Err(RuntimeError {
                                message: "range(): start must be less than end".to_string(),
                                line: expr.line,
                                column: expr.column,
                            });
                        }
                        Ok(Value::Integer(s))
                    }
                    _ => Err(RuntimeError {
                        message: "range() arguments must be integers".to_string(),
                        line: expr.line,
                        column: expr.column,
                    }),
                }
            }
            ExprKind::BinaryOp { left, op, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                operations::evaluate_binary_op(left_val, right_val, op, expr.line, expr.column)
            }
        }
    }
}
