// src/interpreter/builtins/io.rs
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
    print!("{}", s);
    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        file_path: String::new(),
        line,
        column,
    })?;
    Ok(Value::Void)
}

pub fn eval_input(
    args: &[crate::ast::Expr],
    interpreter: &mut crate::interpreter::Interpreter,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if args.is_empty() {
        return Ok(Value::InputHandle);
    }

    if args.len() == 1 {
        let val = interpreter.evaluate_expression(&args[0])?;
        let index = match val {
            Value::Integer(n) if n >= 0 => n as usize,
            _ => {
                return Err(RuntimeError {
                    message: "input() index must be a non-negative integer".to_string(),
                    file_path: String::new(),
                    line,
                    column,
                });
            }
        };
        let user_args: Vec<String> = std::env::args().skip(3).collect();
        return Ok(Value::String(
            user_args.get(index).cloned().unwrap_or_default(),
        ));
    }

    Err(RuntimeError {
        message: "input() takes 0 or 1 arguments".to_string(),
        file_path: String::new(),
        line,
        column,
    })
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
