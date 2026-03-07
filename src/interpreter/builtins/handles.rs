// src/interpreter/builtins/handles.rs
use crate::interpreter::value::{RuntimeError, Value};

pub fn fetch_method(
    url: &str,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method {
        // TODO: switch to worry(string) once worry() is implemented
        "get" => match ureq::get(url).call() {
            Ok(mut response) => {
                let body = response
                    .body_mut()
                    .read_to_string()
                    .map_err(|e| RuntimeError {
                        message: format!("fetch().get() failed to read response: {}", e),
                        line,
                        column,
                    })?;
                Ok(Value::String(body))
            }
            Err(e) => Err(RuntimeError {
                message: format!("fetch().get() request failed: {}", e),
                line,
                column,
            }),
        },
        "send" => {
            let body = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(v) => {
                    return Err(RuntimeError {
                        message: format!(
                            "fetch().send() body must be a string, got {}",
                            v.type_name()
                        ),
                        line,
                        column,
                    });
                }
                None => String::new(),
            };
            match ureq::post(url)
                .content_type("text/plain")
                .send(body.as_bytes())
            {
                Ok(mut response) => {
                    let resp_body =
                        response
                            .body_mut()
                            .read_to_string()
                            .map_err(|e| RuntimeError {
                                message: format!("fetch().send() failed to read response: {}", e),
                                line,
                                column,
                            })?;
                    Ok(Value::String(resp_body))
                }
                Err(e) => Err(RuntimeError {
                    message: format!("fetch().send() request failed: {}", e),
                    line,
                    column,
                }),
            }
        }
        _ => Err(RuntimeError {
            message: format!("fetch() has no method '{}'", method),
            line,
            column,
        }),
    }
}

pub fn input_method(
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method {
        "flag" => match args.first() {
            Some(Value::String(flag_name)) => {
                let cli_args: Vec<String> = std::env::args().skip(3).collect();
                let long = format!("--{}", flag_name);
                let short = format!("-{}", &flag_name[..1]);
                Ok(Value::Boolean(
                    cli_args.iter().any(|a| a == &long || a == &short),
                ))
            }
            _ => Err(RuntimeError {
                message: "input().flag() takes one string argument".to_string(),
                line,
                column,
            }),
        },
        "option" => match args.first() {
            Some(Value::String(opt_name)) => {
                let cli_args: Vec<String> = std::env::args().skip(3).collect();
                let long = format!("--{}", opt_name);
                let short = format!("-{}", &opt_name[..1]);
                let mut found = None;
                for (i, arg) in cli_args.iter().enumerate() {
                    if arg == &long || arg == &short {
                        found = cli_args.get(i + 1).cloned();
                        break;
                    }
                }
                Ok(Value::String(found.unwrap_or_default()))
            }
            _ => Err(RuntimeError {
                message: "input().option() takes one string argument".to_string(),
                line,
                column,
            }),
        },
        _ => Err(RuntimeError {
            message: format!("input() has no method '{}'", method),
            line,
            column,
        }),
    }
}

pub fn file_method(
    path: &str,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method {
        "read" => match std::fs::read_to_string(path) {
            Ok(contents) => Ok(Value::String(contents)),
            Err(e) => Err(RuntimeError {
                message: format!("file(\"{}\").read() failed: {}", path, e),
                line,
                column,
            }),
        },
        "write" => {
            let content = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.type_name().to_string(),
                None => String::new(),
            };
            if let Some(parent) = std::path::Path::new(path).parent()
                && !parent.as_os_str().is_empty()
            {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(path, &content) {
                Ok(_) => Ok(Value::Void),
                Err(e) => Err(RuntimeError {
                    message: format!("file(\"{}\").write() failed: {}", path, e),
                    line,
                    column,
                }),
            }
        }
        "append" => {
            use std::io::Write;
            let content = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.type_name().to_string(),
                None => String::new(),
            };
            if let Some(parent) = std::path::Path::new(path).parent()
                && !parent.as_os_str().is_empty()
            {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)
            {
                Ok(mut f) => match f.write_all(content.as_bytes()) {
                    Ok(_) => Ok(Value::Void),
                    Err(e) => Err(RuntimeError {
                        message: format!("file(\"{}\").append() failed: {}", path, e),
                        line,
                        column,
                    }),
                },
                Err(e) => Err(RuntimeError {
                    message: format!("file(\"{}\").append() failed to open: {}", path, e),
                    line,
                    column,
                }),
            }
        }
        "exists" => Ok(Value::Boolean(std::path::Path::new(path).exists())),
        _ => Err(RuntimeError {
            message: format!("file() has no method '{}'", method),
            line,
            column,
        }),
    }
}
