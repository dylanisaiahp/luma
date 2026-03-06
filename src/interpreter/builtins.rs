// src/interpreter/builtins.rs
use crate::interpreter::value::{RuntimeError, Value};
use std::io::{self, Write};

pub fn eval_read(
    args: &[crate::ast::Expr],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError {
            message: "read() takes no arguments".to_string(),
            line,
            column,
        });
    }

    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        line,
        column,
    })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| RuntimeError {
            message: format!("Failed to read input: {}", e),
            line,
            column,
        })?;

    Ok(Value::String(input.trim().to_string()))
}

pub fn eval_int(
    arg: &crate::ast::Expr,
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    let arg_val = interpreter.evaluate_expression(arg)?;
    match arg_val {
        Value::String(s) => match s.trim().parse::<i64>() {
            Ok(n) => Ok(Value::Integer(n)),
            Err(_) => Err(RuntimeError {
                message: format!("Cannot convert '{}' to integer", s),
                line,
                column,
            }),
        },
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(f) => Ok(Value::Integer(f as i64)),
        Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
        _ => Err(RuntimeError {
            message: format!("Cannot convert {:?} to integer", arg_val),
            line,
            column,
        }),
    }
}

pub fn eval_float(
    arg: &crate::ast::Expr,
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    let arg_val = interpreter.evaluate_expression(arg)?;
    match arg_val {
        Value::String(s) => match s.trim().parse::<f64>() {
            Ok(f) => Ok(Value::Float(f)),
            Err(_) => Err(RuntimeError {
                message: format!("Cannot convert '{}' to float", s),
                line,
                column,
            }),
        },
        Value::Integer(n) => Ok(Value::Float(n as f64)),
        Value::Float(f) => Ok(Value::Float(f)),
        Value::Boolean(b) => Ok(Value::Float(if b { 1.0 } else { 0.0 })),
        _ => Err(RuntimeError {
            message: format!("Cannot convert {:?} to float", arg_val),
            line,
            column,
        }),
    }
}

pub fn eval_string(
    arg: &crate::ast::Expr,
    interpreter: &mut crate::interpreter::Interpreter,
) -> Result<Value, RuntimeError> {
    let arg_val = interpreter.evaluate_expression(arg)?;
    let s = interpreter.value_to_display_string(&arg_val);
    Ok(Value::String(s))
}

pub fn eval_random(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 2 {
        return Err(RuntimeError {
            message: "random() takes exactly two arguments (min, max)".to_string(),
            line,
            column,
        });
    }

    let min_val = interpreter.evaluate_expression(&args[0])?;
    let max_val = interpreter.evaluate_expression(&args[1])?;

    match (min_val, max_val) {
        (Value::Integer(min), Value::Integer(max)) => {
            if min >= max {
                return Err(RuntimeError {
                    message: "random(): min must be less than max".to_string(),
                    line,
                    column,
                });
            }
            use rand::RngExt;
            let mut rng = rand::rng();
            let num = rng.random_range(min..=max);
            Ok(Value::Integer(num))
        }
        _ => Err(RuntimeError {
            message: "random() arguments must be integers".to_string(),
            line,
            column,
        }),
    }
}

pub fn eval_write(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError {
            message: "write() takes exactly one argument".to_string(),
            line,
            column,
        });
    }
    let val = interpreter.evaluate_expression(&args[0])?;
    let s = interpreter.value_to_display_string(&val);
    print!("{}", s);
    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        line,
        column,
    })?;
    Ok(Value::Void)
}

pub fn eval_method(
    object: Value,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match (&object, method) {
        // int methods
        (Value::Integer(n), "abs") => Ok(Value::Integer(n.abs())),
        (Value::Integer(n), "to_float") => Ok(Value::Float(*n as f64)),
        (Value::Integer(n), "to_string") => Ok(Value::String(n.to_string())),
        (Value::Integer(n), "pow") => match args.first() {
            Some(Value::Integer(exp)) if *exp >= 0 => Ok(Value::Integer(n.pow(*exp as u32))),
            _ => Err(RuntimeError {
                message: "int.pow() takes one non-negative integer argument".to_string(),
                line,
                column,
            }),
        },

        // float methods
        (Value::Float(f), "abs") => Ok(Value::Float(f.abs())),
        (Value::Float(f), "floor") => Ok(Value::Integer(f.floor() as i64)),
        (Value::Float(f), "ceil") => Ok(Value::Integer(f.ceil() as i64)),
        (Value::Float(f), "round") => Ok(Value::Integer(f.round() as i64)),
        (Value::Float(f), "to_int") => Ok(Value::Integer(*f as i64)),
        (Value::Float(f), "to_string") => Ok(Value::String(f.to_string())),

        // string methods
        (Value::String(s), "len") => Ok(Value::Integer(s.chars().count() as i64)),
        (Value::String(s), "upper") => Ok(Value::String(s.to_uppercase())),
        (Value::String(s), "lower") => Ok(Value::String(s.to_lowercase())),
        (Value::String(s), "trim") => Ok(Value::String(s.trim().to_string())),
        (Value::String(s), "reverse") => Ok(Value::String(s.chars().rev().collect())),
        (Value::String(s), "is_empty") => Ok(Value::Boolean(s.is_empty())),
        (Value::String(s), "contains") => match args.first() {
            Some(Value::String(sub)) => Ok(Value::Boolean(s.contains(sub.as_str()))),
            _ => Err(RuntimeError {
                message: "string.contains() takes one string argument".to_string(),
                line,
                column,
            }),
        },
        (Value::String(s), "starts_with") => match args.first() {
            Some(Value::String(prefix)) => Ok(Value::Boolean(s.starts_with(prefix.as_str()))),
            _ => Err(RuntimeError {
                message: "string.starts_with() takes one string argument".to_string(),
                line,
                column,
            }),
        },
        (Value::String(s), "ends_with") => match args.first() {
            Some(Value::String(suffix)) => Ok(Value::Boolean(s.ends_with(suffix.as_str()))),
            _ => Err(RuntimeError {
                message: "string.ends_with() takes one string argument".to_string(),
                line,
                column,
            }),
        },
        (Value::String(s), "repeat") => match args.first() {
            Some(Value::Integer(n)) if *n >= 0 => Ok(Value::String(s.repeat(*n as usize))),
            _ => Err(RuntimeError {
                message: "string.repeat() takes one non-negative integer argument".to_string(),
                line,
                column,
            }),
        },

        // bool methods
        (Value::Boolean(b), "to_string") => Ok(Value::String(b.to_string())),

        // maybe methods
        (Value::Maybe(inner), "exists") => Ok(Value::Boolean(inner.is_some())),
        (Value::Maybe(Some(inner)), "or") => Ok(*inner.clone()),
        (Value::Maybe(None), "or") => match args.first() {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError {
                message: "maybe.or() requires a fallback value".to_string(),
                line,
                column,
            }),
        },
        (Value::Maybe(Some(inner)), method) => {
            eval_method(*inner.clone(), method, args, line, column)
        }
        (Value::Maybe(None), method) => Err(RuntimeError {
            message: format!(
                "Cannot call '{}' on empty maybe — use .or() to provide a fallback first",
                method
            ),
            line,
            column,
        }),

        // list methods
        (Value::List(items), "len") => Ok(Value::Integer(items.len() as i64)),
        (Value::List(items), "is_empty") => Ok(Value::Boolean(items.is_empty())),
        (Value::List(items), "get") => match args.first() {
            Some(Value::Integer(i)) => {
                let idx = *i as usize;
                match items.get(idx) {
                    Some(v) => Ok(v.clone()),
                    None => Err(RuntimeError {
                        message: format!("list index {} out of bounds (len {})", i, items.len()),
                        line,
                        column,
                    }),
                }
            }
            _ => Err(RuntimeError {
                message: "list.get() takes one integer argument".to_string(),
                line,
                column,
            }),
        },
        (Value::List(items), "contains") => {
            let target = args.first().ok_or(RuntimeError {
                message: "list.contains() takes one argument".to_string(),
                line,
                column,
            })?;
            Ok(Value::Boolean(items.contains(target)))
        }
        (Value::List(items), "where") => match args.first() {
            Some(target) => {
                let idx = items.iter().position(|v| v == target);
                match idx {
                    Some(i) => Ok(Value::Integer(i as i64)),
                    None => Ok(Value::Integer(-1)),
                }
            }
            _ => Err(RuntimeError {
                message: "list.where() takes one argument".to_string(),
                line,
                column,
            }),
        },
        (Value::List(items), "add") => {
            let val = args.first().ok_or(RuntimeError {
                message: "list.add() takes one argument".to_string(),
                line,
                column,
            })?;
            let mut new_items = items.clone();
            new_items.push(val.clone());
            Ok(Value::List(new_items))
        }
        (Value::List(items), "remove") => match args.first() {
            Some(Value::Integer(i)) => {
                let idx = *i as usize;
                if idx >= items.len() {
                    return Err(RuntimeError {
                        message: format!("list index {} out of bounds (len {})", i, items.len()),
                        line,
                        column,
                    });
                }
                let mut new_items = items.clone();
                new_items.remove(idx);
                Ok(Value::List(new_items))
            }
            _ => Err(RuntimeError {
                message: "list.remove() takes one integer index argument".to_string(),
                line,
                column,
            }),
        },
        (Value::List(items), "reverse") => {
            let mut new_items = items.clone();
            new_items.reverse();
            Ok(Value::List(new_items))
        }
        (Value::List(items), "sort") => {
            let mut new_items = items.clone();
            new_items.sort_by(|a, b| match (a, b) {
                (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                (Value::Float(x), Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::List(new_items))
        }
        (Value::List(items), "first") => match items.first() {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError {
                message: "list.first() called on empty list".to_string(),
                line,
                column,
            }),
        },
        (Value::List(items), "last") => match items.last() {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError {
                message: "list.last() called on empty list".to_string(),
                line,
                column,
            }),
        },

        // table methods
        (Value::Table(pairs), "len") => Ok(Value::Integer(pairs.len() as i64)),
        (Value::Table(pairs), "is_empty") => Ok(Value::Boolean(pairs.is_empty())),
        (Value::Table(pairs), "has") => {
            let key = args.first().ok_or(RuntimeError {
                message: "table.has() takes one argument".to_string(),
                line,
                column,
            })?;
            Ok(Value::Boolean(pairs.iter().any(|(k, _)| k == key)))
        }
        (Value::Table(pairs), "get") => {
            let key = args.first().ok_or(RuntimeError {
                message: "table.get() takes one argument".to_string(),
                line,
                column,
            })?;
            match pairs.iter().find(|(k, _)| k == key) {
                Some((_, v)) => Ok(v.clone()),
                None => Err(RuntimeError {
                    message: "table key not found".to_string(),
                    line,
                    column,
                }),
            }
        }
        (Value::Table(pairs), "set") => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "table.set() takes two arguments (key, value)".to_string(),
                    line,
                    column,
                });
            }
            let key = args[0].clone();
            let val = args[1].clone();
            let mut new_pairs = pairs.clone();
            if let Some(entry) = new_pairs.iter_mut().find(|(k, _)| k == &key) {
                entry.1 = val;
            } else {
                new_pairs.push((key, val));
            }
            Ok(Value::Table(new_pairs))
        }
        (Value::Table(pairs), "remove") => {
            let key = args.first().ok_or(RuntimeError {
                message: "table.remove() takes one argument".to_string(),
                line,
                column,
            })?;
            let mut new_pairs = pairs.clone();
            new_pairs.retain(|(k, _)| k != key);
            Ok(Value::Table(new_pairs))
        }
        (Value::Table(pairs), "keys") => {
            let keys: Vec<Value> = pairs.iter().map(|(k, _)| k.clone()).collect();
            Ok(Value::List(keys))
        }
        (Value::Table(pairs), "values") => {
            let vals: Vec<Value> = pairs.iter().map(|(_, v)| v.clone()).collect();
            Ok(Value::List(vals))
        }

        _ => Err(RuntimeError {
            message: format!("'{}' has no method '{}'", object.type_name(), method),
            line,
            column,
        }),
    }
}
