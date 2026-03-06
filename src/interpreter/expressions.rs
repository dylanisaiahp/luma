// src/interpreter/expressions.rs
use crate::ast::*;
use crate::interpreter::Interpreter;
use crate::interpreter::builtins;
use crate::interpreter::operations;
use crate::interpreter::value::{RuntimeError, Value};

impl Interpreter {
    pub fn evaluate_expression(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match &expr.kind {
            ExprKind::Integer(n) => Ok(Value::Integer(*n)),
            ExprKind::Float(f) => Ok(Value::Float(*f)),
            ExprKind::String(s) => Ok(Value::String(s.clone())),
            ExprKind::Boolean(b) => Ok(Value::Boolean(*b)),
            ExprKind::Empty => Ok(Value::Maybe(None)),
            ExprKind::List(items) => {
                let mut vals = Vec::new();
                for item in items {
                    vals.push(self.evaluate_expression(item)?);
                }
                Ok(Value::List(vals))
            }
            ExprKind::Table(pairs) => {
                let mut evaluated = Vec::new();
                for (k, v) in pairs {
                    let key = self.evaluate_expression(k)?;
                    let val = self.evaluate_expression(v)?;
                    evaluated.push((key, val));
                }
                Ok(Value::Table(evaluated))
            }
            ExprKind::TypeConstant {
                type_name,
                constant,
            } => match (type_name.as_str(), constant.as_str()) {
                ("int", "max") => Ok(Value::Integer(i64::MAX)),
                ("int", "min") => Ok(Value::Integer(i64::MIN)),
                ("float", "max") => Ok(Value::Float(f64::MAX)),
                ("float", "min") => Ok(Value::Float(f64::MIN)),
                _ => Err(RuntimeError {
                    message: format!("'{}' has no constant '{}'", type_name, constant),
                    line: expr.line,
                    column: expr.column,
                }),
            },
            ExprKind::Not(operand) => {
                let val = self.evaluate_expression(operand)?;
                match val {
                    Value::Boolean(b) => Ok(Value::Boolean(!b)),
                    _ => Err(RuntimeError {
                        message: format!("'not' requires a boolean, got {:?}", val),
                        line: expr.line,
                        column: expr.column,
                    }),
                }
            }
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
                    Some(val) => {
                        let s = self.value_to_display_string(val);
                        Ok(Value::String(s))
                    }
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
                "write" => builtins::eval_write(args, self, expr.line, expr.column),
                "random" => builtins::eval_random(args, self, expr.line, expr.column),
                _ => {
                    if self.functions.contains_key(name.as_str()) {
                        let args_cloned = args.clone();
                        self.execute_call(name, &args_cloned, expr.line, expr.column)
                    } else {
                        Err(RuntimeError {
                            message: format!("Unknown function: {}", name),
                            line: expr.line,
                            column: expr.column,
                        })
                    }
                }
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
            ExprKind::MethodCall {
                object,
                method,
                args,
            } => {
                let object_val = self.evaluate_expression(object)?;
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.evaluate_expression(arg)?);
                }
                builtins::eval_method(object_val, method, &arg_vals, expr.line, expr.column)
            }
        }
    }
}
