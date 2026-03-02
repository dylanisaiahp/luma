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
                // Track that this variable was used
                self.used_variables.insert(name.clone());

                match self.variables.get(name) {
                    Some(val) => Ok(val.clone()),
                    None => Err(RuntimeError {
                        message: format!("Undefined variable: {}", name),
                        line: expr.line,
                        column: expr.column,
                    }),
                }
            }
            ExprKind::Interpolation(ident) => {
                // Track that this variable was used in interpolation
                self.used_variables.insert(ident.clone());

                match self.variables.get(ident) {
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
                let val = self.evaluate_expression(value)?;
                self.variables.insert(name.clone(), val.clone());
                Ok(val)
            }
            ExprKind::AssignOp { name, op, value } => {
                // Get current value
                let current = match self.variables.get(name) {
                    Some(val) => val.clone(),
                    None => {
                        return Err(RuntimeError {
                            message: format!("Undefined variable: {}", name),
                            line: expr.line,
                            column: expr.column,
                        });
                    }
                };

                // Evaluate the right side
                let right_val = self.evaluate_expression(value)?;

                // Perform the operation
                let result = match (current, right_val, op) {
                    // Integer operations
                    (Value::Integer(l), Value::Integer(r), crate::ast::AssignOpKind::Add) => {
                        Value::Integer(l + r)
                    }
                    (Value::Integer(l), Value::Integer(r), crate::ast::AssignOpKind::Subtract) => {
                        Value::Integer(l - r)
                    }
                    (Value::Integer(l), Value::Integer(r), crate::ast::AssignOpKind::Multiply) => {
                        Value::Integer(l * r)
                    }
                    (Value::Integer(l), Value::Integer(r), crate::ast::AssignOpKind::Divide) => {
                        if r == 0 {
                            return Err(RuntimeError {
                                message: "Division by zero".to_string(),
                                line: expr.line,
                                column: expr.column,
                            });
                        }
                        Value::Integer(l / r)
                    }

                    // Float operations
                    (Value::Float(l), Value::Float(r), crate::ast::AssignOpKind::Add) => {
                        Value::Float(l + r)
                    }
                    (Value::Float(l), Value::Float(r), crate::ast::AssignOpKind::Subtract) => {
                        Value::Float(l - r)
                    }
                    (Value::Float(l), Value::Float(r), crate::ast::AssignOpKind::Multiply) => {
                        Value::Float(l * r)
                    }
                    (Value::Float(l), Value::Float(r), crate::ast::AssignOpKind::Divide) => {
                        if r == 0.0 {
                            return Err(RuntimeError {
                                message: "Division by zero".to_string(),
                                line: expr.line,
                                column: expr.column,
                            });
                        }
                        Value::Float(l / r)
                    }

                    // Mixed integer/float
                    (Value::Integer(l), Value::Float(r), crate::ast::AssignOpKind::Add) => {
                        Value::Float(l as f64 + r)
                    }
                    (Value::Float(l), Value::Integer(r), crate::ast::AssignOpKind::Add) => {
                        Value::Float(l + r as f64)
                    }
                    (Value::Integer(l), Value::Float(r), crate::ast::AssignOpKind::Subtract) => {
                        Value::Float(l as f64 - r)
                    }
                    (Value::Float(l), Value::Integer(r), crate::ast::AssignOpKind::Subtract) => {
                        Value::Float(l - r as f64)
                    }
                    (Value::Integer(l), Value::Float(r), crate::ast::AssignOpKind::Multiply) => {
                        Value::Float(l as f64 * r)
                    }
                    (Value::Float(l), Value::Integer(r), crate::ast::AssignOpKind::Multiply) => {
                        Value::Float(l * r as f64)
                    }
                    (Value::Integer(l), Value::Float(r), crate::ast::AssignOpKind::Divide) => {
                        if r == 0.0 {
                            return Err(RuntimeError {
                                message: "Division by zero".to_string(),
                                line: expr.line,
                                column: expr.column,
                            });
                        }
                        Value::Float(l as f64 / r)
                    }
                    (Value::Float(l), Value::Integer(r), crate::ast::AssignOpKind::Divide) => {
                        if r == 0 {
                            return Err(RuntimeError {
                                message: "Division by zero".to_string(),
                                line: expr.line,
                                column: expr.column,
                            });
                        }
                        Value::Float(l / r as f64)
                    }

                    // String concatenation with +=
                    (Value::String(l), Value::String(r), crate::ast::AssignOpKind::Add) => {
                        Value::String(l + &r)
                    }
                    (Value::String(l), Value::Integer(r), crate::ast::AssignOpKind::Add) => {
                        Value::String(l + &r.to_string())
                    }
                    (Value::String(l), Value::Float(r), crate::ast::AssignOpKind::Add) => {
                        Value::String(l + &r.to_string())
                    }

                    _ => {
                        return Err(RuntimeError {
                            message: "Type mismatch in compound assignment".to_string(),
                            line: expr.line,
                            column: expr.column,
                        });
                    }
                };

                // Store the result
                self.variables.insert(name.clone(), result.clone());
                Ok(result)
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
                        // For now, just return the start value
                        // We'll make it actually generate ranges later
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
