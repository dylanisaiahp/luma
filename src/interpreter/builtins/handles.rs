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
    _args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    // Get raw CLI args (skip: program name, "run", file path)
    let raw: Vec<String> = std::env::args().skip(3).collect();

    match method {
        // Positional args — everything that doesn't start with -
        "args" => {
            let positional: Vec<Value> = raw
                .iter()
                .filter(|a| !a.starts_with('-'))
                .map(|a| Value::String(a.clone()))
                .collect();
            Ok(Value::List(positional))
        }
        // Flags — args starting with - or --, strip the dashes, no value after
        "flags" => {
            let mut flags: Vec<Value> = Vec::new();
            let mut i = 0;
            while i < raw.len() {
                let arg = &raw[i];
                if arg.starts_with("--") {
                    // Check if next arg is a value (not a flag)
                    let next_is_value =
                        raw.get(i + 1).map(|n| !n.starts_with('-')).unwrap_or(false);
                    if !next_is_value {
                        flags.push(Value::String(arg.trim_start_matches('-').to_string()));
                    }
                    i += if next_is_value { 2 } else { 1 };
                } else if arg.starts_with('-') {
                    let next_is_value =
                        raw.get(i + 1).map(|n| !n.starts_with('-')).unwrap_or(false);
                    if !next_is_value {
                        flags.push(Value::String(arg.trim_start_matches('-').to_string()));
                    }
                    i += if next_is_value { 2 } else { 1 };
                } else {
                    i += 1;
                }
            }
            Ok(Value::List(flags))
        }
        // Options — flags that have a value after them, returned as table(string, string)
        "options" => {
            let mut opts: Vec<(Value, Value)> = Vec::new();
            let mut i = 0;
            while i < raw.len() {
                let arg = &raw[i];
                if arg.starts_with('-')
                    && let Some(next) = raw.get(i + 1)
                    && !next.starts_with('-')
                {
                    let key = arg.trim_start_matches('-').to_string();
                    opts.push((Value::String(key), Value::String(next.clone())));
                    i += 2;
                    continue;
                }
                i += 1;
            }
            Ok(Value::Table(opts))
        }
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
        "list" => {
            let filter = match args.first() {
                Some(Value::String(s)) => s.clone(),
                None => String::new(),
                _ => {
                    return Err(RuntimeError {
                        message: "file.list() takes an optional string filter".to_string(),
                        line,
                        column,
                    });
                }
            };
            let dir_path = std::path::Path::new(path);
            let entries = std::fs::read_dir(dir_path).map_err(|e| RuntimeError {
                message: format!("file(\"{}\").list() failed: {}", path, e),
                line,
                column,
            })?;
            let mut results: Vec<Value> = Vec::new();
            for entry in entries.flatten() {
                let entry_path = entry.path();
                let name = entry_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                let is_dir = entry_path.is_dir();
                let matches = match filter.as_str() {
                    "" => true,
                    "." => !is_dir, // all files
                    "/" => is_dir,  // all dirs
                    ext if ext.starts_with('.') => {
                        // by extension
                        !is_dir && name.ends_with(ext)
                    }
                    subdir if subdir.starts_with('/') && subdir.ends_with('/') => {
                        // /dirname/ — list contents of subdirectory
                        let sub = subdir.trim_matches('/');
                        let sub_path = dir_path.join(sub);
                        if sub_path.is_dir() {
                            let sub_entries =
                                std::fs::read_dir(&sub_path).map_err(|e| RuntimeError {
                                    message: format!(
                                        "file(\"{}\").list() failed on subdir: {}",
                                        path, e
                                    ),
                                    line,
                                    column,
                                })?;
                            for sub_entry in sub_entries.flatten() {
                                let sub_name = sub_entry
                                    .path()
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .unwrap_or("")
                                    .to_string();
                                results.push(Value::String(sub_name));
                            }
                        }
                        false // already handled above
                    }
                    _ => false,
                };
                if matches {
                    results.push(Value::String(name));
                }
            }
            Ok(Value::List(results))
        }
        _ => Err(RuntimeError {
            message: format!("file() has no method '{}'", method),
            line,
            column,
        }),
    }
}
