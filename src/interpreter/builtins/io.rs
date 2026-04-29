// src/interpreter/builtins/io.rs
use crate::interpreter::value::{RuntimeError, Value};
use std::io::{self, Read, Write};

pub fn eval_read(
    args: &[crate::ast::Expr],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if !args.is_empty() {
        return Err(RuntimeError {
            message: "read() takes no arguments".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }

    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        file_path: String::new(),
        line,
        column,
    })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| RuntimeError {
            message: format!("Failed to read input: {}", e),
            file_path: String::new(),
            line,
            column,
        })?;

    Ok(Value::String(input.trim().to_string()))
}

pub fn eval_read_n(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError {
            message: "read_n() takes exactly one argument (number of bytes)".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }

    let n_val = interpreter.evaluate_expression(&args[0])?;
    let n = match n_val {
        Value::Integer(n) => n as usize,
        _ => {
            return Err(RuntimeError {
                message: "read_n() argument must be an integer".to_string(),
                file_path: String::new(),
                line,
                column,
            });
        }
    };

    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        file_path: String::new(),
        line,
        column,
    })?;

    if n == 0 {
        return Ok(Value::String(String::new()));
    }

    let mut buffer = vec![0u8; n];
    let mut stdin = io::stdin();
    let mut total_read = 0;

    while total_read < n {
        match stdin.read(&mut buffer[total_read..]) {
            Ok(0) => break, // EOF
            Ok(bytes) => total_read += bytes,
            Err(e) => {
                return Err(RuntimeError {
                    message: format!("Failed to read input: {}", e),
                    file_path: String::new(),
                    line,
                    column,
                });
            }
        }
    }

    buffer.truncate(total_read);
    Ok(Value::String(String::from_utf8_lossy(&buffer).to_string()))
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
            file_path: String::new(),
            line,
            column,
        });
    }
    let val = interpreter.evaluate_expression(&args[0])?;
    let s = interpreter.value_to_display_string(&val);
    let unescaped = s
        .replace("\\r\\n", "\r\n")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\t", "\t");
    print!("{}", unescaped);
    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        file_path: String::new(),
        line,
        column,
    })?;
    Ok(Value::Void)
}

pub fn eval_args(
    _args: &[crate::ast::Expr],
    _interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if !_args.is_empty() {
        return Err(RuntimeError {
            message: "args() takes no arguments".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }
    let user_args: Vec<Value> = std::env::args().skip(3).map(Value::String).collect();
    Ok(Value::List(user_args))
}

pub fn eval_fetch(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError {
            message: "fetch() takes exactly one argument (url)".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }

    let url_val = interpreter.evaluate_expression(&args[0])?;
    match url_val {
        Value::String(url) => Ok(Value::FetchHandle(url)),
        _ => Err(RuntimeError {
            message: "fetch() argument must be a string URL".to_string(),
            file_path: String::new(),
            line,
            column,
        }),
    }
}

pub fn eval_file(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError {
            message: "file() takes exactly one argument (path)".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }
    let path_val = interpreter.evaluate_expression(&args[0])?;
    match path_val {
        Value::String(path) => Ok(Value::FileHandle(path)),
        _ => Err(RuntimeError {
            message: "file() argument must be a string path".to_string(),
            file_path: String::new(),
            line,
            column,
        }),
    }
}

pub fn eval_json(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError {
            message: "json() takes exactly one argument".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }
    let val = interpreter.evaluate_expression(&args[0])?;
    match val {
        Value::String(s) => Ok(Value::JsonHandle(s)),
        Value::Table(pairs) => {
            let json_str =
                serde_json::to_string_pretty(&table_to_json(&pairs)).map_err(|e| RuntimeError {
                    message: format!("json() failed to encode table: {}", e),
                    file_path: String::new(),
                    line,
                    column,
                })?;
            Ok(Value::String(json_str))
        }
        _ => Err(RuntimeError {
            message: "json() argument must be a string or table".to_string(),
            file_path: String::new(),
            line,
            column,
        }),
    }
}

fn table_to_json(table: &[(Value, Value)]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in table {
        if let Value::String(key) = k {
            map.insert(key.clone(), value_to_json_value(v));
        }
    }
    serde_json::Value::Object(map)
}

fn value_to_json_value(value: &Value) -> serde_json::Value {
    match value {
        Value::Integer(n) => serde_json::Value::Number(serde_json::Number::from(*n)),
        Value::Float(f) => serde_json::Number::from_f64(*f)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Char(c) => serde_json::Value::String(c.to_string()),
        Value::Boolean(b) => serde_json::Value::Bool(*b),
        Value::Void => serde_json::Value::Null,
        Value::Option(None) => serde_json::Value::Null,
        Value::Option(Some(v)) => value_to_json_value(v),
        Value::List(arr) => serde_json::Value::Array(arr.iter().map(value_to_json_value).collect()),
        Value::Table(pairs) => {
            let mut map = serde_json::Map::new();
            for (k, v) in pairs {
                if let Value::String(key) = k {
                    map.insert(key.clone(), value_to_json_value(v));
                }
            }
            serde_json::Value::Object(map)
        }
        Value::FetchHandle(_) | Value::FileHandle(_) => {
            serde_json::Value::String("<handle>".to_string())
        }
        Value::JsonHandle(_) | Value::TomlHandle(_) => {
            serde_json::Value::String("<handle>".to_string())
        }
        Value::Struct { .. } => serde_json::Value::String("<struct>".to_string()),
        Value::EnumVariant { .. } => serde_json::Value::String("<enum>".to_string()),
        Value::EnumVariantData { .. } => serde_json::Value::String("<enum>".to_string()),
    }
}

pub fn eval_toml(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError {
            message: "toml() takes exactly one argument".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }
    let val = interpreter.evaluate_expression(&args[0])?;
    match val {
        Value::String(s) => Ok(Value::TomlHandle(s)),
        Value::Table(pairs) => {
            let toml_str =
                toml::to_string_pretty(&table_to_toml(&pairs)).map_err(|e| RuntimeError {
                    message: format!("toml() failed to encode table: {}", e),
                    file_path: String::new(),
                    line,
                    column,
                })?;
            Ok(Value::String(toml_str))
        }
        _ => Err(RuntimeError {
            message: "toml() argument must be a string or table".to_string(),
            file_path: String::new(),
            line,
            column,
        }),
    }
}

fn table_to_toml(table: &[(Value, Value)]) -> toml::Value {
    let mut map = toml::map::Map::new();
    for (k, v) in table {
        if let Value::String(key) = k {
            map.insert(key.clone(), value_to_toml_value(v));
        }
    }
    toml::Value::Table(map)
}

fn value_to_toml_value(value: &Value) -> toml::Value {
    match value {
        Value::Integer(n) => toml::Value::Integer(*n),
        Value::Float(f) => toml::Value::Float(*f),
        Value::String(s) => toml::Value::String(s.clone()),
        Value::Char(c) => toml::Value::String(c.to_string()),
        Value::Boolean(b) => toml::Value::Boolean(*b),
        Value::Void => toml::Value::String("void".to_string()),
        Value::Option(None) => toml::Value::String("null".to_string()),
        Value::Option(Some(v)) => value_to_toml_value(v),
        Value::List(arr) => toml::Value::Array(arr.iter().map(value_to_toml_value).collect()),
        Value::Table(pairs) => {
            let mut map = toml::map::Map::new();
            for (k, v) in pairs {
                if let Value::String(key) = k {
                    map.insert(key.clone(), value_to_toml_value(v));
                }
            }
            toml::Value::Table(map)
        }
        Value::FetchHandle(_) | Value::FileHandle(_) => toml::Value::String("<handle>".to_string()),
        Value::JsonHandle(_) | Value::TomlHandle(_) => toml::Value::String("<handle>".to_string()),
        Value::Struct { .. } => toml::Value::String("<struct>".to_string()),
        Value::EnumVariant { .. } => toml::Value::String("<enum>".to_string()),
        Value::EnumVariantData { .. } => toml::Value::String("<enum>".to_string()),
    }
}

pub fn eval_env(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.len() != 1 {
        return Err(RuntimeError {
            message: "env() takes exactly one argument (key)".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }

    let key_val = interpreter.evaluate_expression(&args[0])?;
    match key_val {
        Value::String(key) => Ok(Value::String(std::env::var(&key).unwrap_or_default())),
        _ => Err(RuntimeError {
            message: "env() argument must be a string".to_string(),
            file_path: String::new(),
            line,
            column,
        }),
    }
}

pub fn eval_home(
    _args: &[crate::ast::Expr],
    _interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if !_args.is_empty() {
        return Err(RuntimeError {
            message: "home() takes no arguments".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }

    Ok(Value::String(std::env::var("HOME").unwrap_or_default()))
}

pub fn eval_run(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<crate::interpreter::value::Value, crate::interpreter::value::RuntimeError> {
    use crate::interpreter::value::{RuntimeError, Value};

    if args.is_empty() {
        return Err(RuntimeError {
            message: "run() requires at least one argument".to_string(),
            file_path: String::new(),
            line,
            column,
        });
    }

    // Evaluate all args as strings and join into a command
    let mut parts: Vec<String> = Vec::new();
    for arg in args {
        let val = interpreter.evaluate_expression(arg)?;
        match val {
            Value::String(s) => {
                // Split on whitespace so run("git status") works naturally
                parts.extend(s.split_whitespace().map(|s| s.to_string()));
            }
            other => parts.push(interpreter.value_to_display_string(&other)),
        }
    }

    let (cmd, cmd_args) = match parts.split_first() {
        Some((c, a)) => (c.clone(), a.to_vec()),
        None => {
            return Err(RuntimeError {
                message: "run() command cannot be empty".to_string(),
                file_path: String::new(),
                line,
                column,
            });
        }
    };

    match std::process::Command::new(&cmd).args(&cmd_args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

            if output.status.success() {
                Ok(Value::String(stdout))
            } else {
                let msg = if stderr.is_empty() { stdout } else { stderr };
                Err(RuntimeError {
                    message: format!("__raise__{}", msg),
                    file_path: String::new(),
                    line,
                    column,
                })
            }
        }
        Err(e) => Err(RuntimeError {
            message: format!("__raise__run(\"{}\") failed: {}", cmd, e),
            file_path: String::new(),
            line,
            column,
        }),
    }
}
