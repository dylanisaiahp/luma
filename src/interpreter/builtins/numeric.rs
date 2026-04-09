// src/interpreter/builtins/numeric.rs
use crate::interpreter::value::{RuntimeError, Value};

pub fn int_method(
    n: &i64,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Option<Result<Value, RuntimeError>> {
    match method {
        "abs" => Some(Ok(Value::Integer(n.abs()))),
        "to_float" => Some(Ok(Value::Float(*n as f64))),
        "to_string" => Some(Ok(Value::String(n.to_string()))),
        "exists" => Some(Ok(Value::Boolean(*n != 0))),
        "pow" => Some(match args.first() {
            Some(Value::Integer(exp)) if *exp >= 0 => Ok(Value::Integer(n.pow(*exp as u32))),
            _ => Err(RuntimeError {
                message: "int.pow() takes one non-negative integer argument".to_string(),
                file_path: String::new(),
                line,
                column,
            }),
        }),
        _ => None,
    }
}

pub fn float_method(
    f: &f64,
    method: &str,
    _args: &[Value],
    _line: usize,
    _column: usize,
) -> Option<Result<Value, RuntimeError>> {
    match method {
        "abs" => Some(Ok(Value::Float(f.abs()))),
        "floor" => Some(Ok(Value::Integer(f.floor() as i64))),
        "ceil" => Some(Ok(Value::Integer(f.ceil() as i64))),
        "round" => Some(Ok(Value::Integer(f.round() as i64))),
        "to_int" => Some(Ok(Value::Integer(*f as i64))),
        "to_string" => Some(Ok(Value::String(f.to_string()))),
        "exists" => Some(Ok(Value::Boolean(*f != 0.0))),
        _ => None,
    }
}
