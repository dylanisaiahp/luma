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
    match arg_val {
        Value::String(s) => Ok(Value::String(s)),
        Value::Integer(n) => Ok(Value::String(n.to_string())),
        Value::Float(f) => Ok(Value::String(f.to_string())),
        Value::Boolean(b) => Ok(Value::String(b.to_string())),
        Value::Void => Ok(Value::String("void".to_string())),
    }
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
    let s = match &val {
        Value::String(s) => s.clone(),
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Boolean(b) => b.to_string(),
        Value::Void => "void".to_string(),
    };
    print!("{}", s);
    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        line,
        column,
    })?;
    Ok(Value::Void)
}
