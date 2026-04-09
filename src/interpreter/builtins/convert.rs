// src/interpreter/builtins/convert.rs
use crate::interpreter::value::{RuntimeError, Value};

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
                file_path: String::new(),
                line,
                column,
            }),
        },
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(f) => Ok(Value::Integer(f as i64)),
        Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
        _ => Err(RuntimeError {
            message: format!("Cannot convert {:?} to integer", arg_val),
            file_path: String::new(),
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
                file_path: String::new(),
                line,
                column,
            }),
        },
        Value::Integer(n) => Ok(Value::Float(n as f64)),
        Value::Float(f) => Ok(Value::Float(f)),
        Value::Boolean(b) => Ok(Value::Float(if b { 1.0 } else { 0.0 })),
        _ => Err(RuntimeError {
            message: format!("Cannot convert {:?} to float", arg_val),
            file_path: String::new(),
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
            file_path: String::new(),
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
                    file_path: String::new(),
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
            file_path: String::new(),
            line,
            column,
        }),
    }
}
