// src/interpreter/builtins/mod.rs
mod collections;
mod convert;
mod handles;
mod io;
mod numeric;
mod text;

pub use collections::{list_method, table_method};
pub use convert::{eval_float, eval_int, eval_random, eval_string};
pub use handles::{fetch_method, file_method, input_method, json_method, toml_method};
pub use io::{
    eval_env, eval_fetch, eval_file, eval_home, eval_input, eval_json, eval_read, eval_read_n,
    eval_run, eval_toml, eval_write,
};
pub use numeric::{float_method, int_method};
pub use text::{char_method, string_method};

use crate::interpreter::value::{RuntimeError, Value};

pub fn eval_method(
    object: Value,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match &object {
        Value::Integer(n) => match int_method(n, method, args, line, column) {
            Some(result) => result,
            None => Err(RuntimeError {
                message: format!("int has no method '{}'", method),
                file_path: String::new(),
                line,
                column,
            }),
        },
        Value::Float(f) => match float_method(f, method, args, line, column) {
            Some(result) => result,
            None => Err(RuntimeError {
                message: format!("float has no method '{}'", method),
                file_path: String::new(),
                line,
                column,
            }),
        },
        Value::String(s) => string_method(s, method, args, line, column),
        Value::Char(c) => char_method(c, method, args, line, column),
        Value::Boolean(b) => match method {
            "to_string" => Ok(Value::String(b.to_string())),
            "exists" => Ok(Value::Boolean(*b)),
            _ => Err(RuntimeError {
                message: format!("bool has no method '{}'", method),
                file_path: String::new(),
                line,
                column,
            }),
        },
        Value::Option(inner) => match method {
            "exists" => Ok(Value::Boolean(inner.is_some())),
            "or" => match inner {
                Some(inner) => {
                    let result = (*inner.clone()).clone();
                    Ok(result)
                }
                None => match args.first() {
                    Some(v) => Ok(v.clone()),
                    None => Err(RuntimeError {
                        message: "option.or() requires a fallback value".to_string(),
                        file_path: String::new(),
                        line,
                        column,
                    }),
                },
            },
            _ => match inner {
                Some(inner) => eval_method(*inner.clone(), method, args, line, column),
                None => Err(RuntimeError {
                    message: format!(
                        "Cannot call '{}' on empty option — use .or() to provide a fallback first",
                        method
                    ),
                    file_path: String::new(),
                    line,
                    column,
                }),
            },
        },
        Value::List(items) => list_method(items, method, args, line, column),
        Value::Table(pairs) => table_method(pairs, method, args, line, column),
        Value::FetchHandle(url) => fetch_method(url, method, args, line, column),
        Value::InputHandle => input_method(method, args, line, column),
        Value::FileHandle(path) => file_method(path, method, args, line, column),
        Value::JsonHandle(json_str) => json_method(json_str, method, args, line, column),
        Value::TomlHandle(toml_str) => toml_method(toml_str, method, args, line, column),
        Value::Struct { name, .. } => Err(RuntimeError {
            message: format!(
                "struct '{}' has no builtin methods — use defined methods instead",
                name
            ),
            file_path: String::new(),
            line,
            column,
        }),
        Value::EnumVariant { enum_name, .. } => Err(RuntimeError {
            message: format!("enum '{}' has no builtin methods", enum_name),
            file_path: String::new(),
            line,
            column,
        }),
        Value::EnumVariantData { enum_name, .. } => Err(RuntimeError {
            message: format!("enum '{}' has no builtin methods", enum_name),
            file_path: String::new(),
            line,
            column,
        }),
        Value::Void => Err(RuntimeError {
            message: format!("void has no method '{}'", method),
            file_path: String::new(),
            line,
            column,
        }),
    }
}
