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
                        file_path: String::new(),
                        line,
                        column,
                    })?;
                Ok(Value::String(body))
            }
            Err(e) => Err(RuntimeError {
                message: format!("fetch().get() request failed: {}", e),
                file_path: String::new(),
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
                        file_path: String::new(),
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
                                file_path: String::new(),
                                line,
                                column,
                            })?;
                    Ok(Value::String(resp_body))
                }
                Err(e) => Err(RuntimeError {
                    message: format!("fetch().send() request failed: {}", e),
                    file_path: String::new(),
                    line,
                    column,
                }),
            }
        }
        _ => Err(RuntimeError {
            message: format!("fetch() has no method '{}'", method),
            file_path: String::new(),
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
                file_path: String::new(),
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
                    file_path: String::new(),
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
                        file_path: String::new(),
                        line,
                        column,
                    }),
                },
                Err(e) => Err(RuntimeError {
                    message: format!("file(\"{}\").append() failed to open: {}", path, e),
                    file_path: String::new(),
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
                        file_path: String::new(),
                        line,
                        column,
                    });
                }
            };
            let dir_path = std::path::Path::new(path);
            let entries = std::fs::read_dir(dir_path).map_err(|e| RuntimeError {
                message: format!("file(\"{}\").list() failed: {}", path, e),
                file_path: String::new(),
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
                                    file_path: String::new(),
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
            file_path: String::new(),
            line,
            column,
        }),
    }
}

pub fn json_method(
    json_str: &str,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method {
        "parse" => {
            let value: serde_json::Value =
                serde_json::from_str(json_str).map_err(|e| RuntimeError {
                    message: format!("json.parse() failed: {}", e),
                    file_path: String::new(),
                    line,
                    column,
                })?;
            let table = json_to_table(value);
            Ok(Value::Table(table))
        }
        "encode" => {
            let table = match args.first() {
                Some(Value::Table(t)) => t.clone(),
                Some(v) => {
                    return Err(RuntimeError {
                        message: format!("json.encode() requires a table, got {}", v.type_name()),
                        file_path: String::new(),
                        line,
                        column,
                    });
                }
                None => Vec::new(),
            };
            let value = table_to_json(&table);
            let encoded = serde_json::to_string_pretty(&value).map_err(|e| RuntimeError {
                message: format!("json.encode() failed: {}", e),
                file_path: String::new(),
                line,
                column,
            })?;
            Ok(Value::String(encoded))
        }
        _ => Err(RuntimeError {
            message: format!("json() has no method '{}'", method),
            file_path: String::new(),
            line,
            column,
        }),
    }
}

fn json_to_table(value: serde_json::Value) -> Vec<(Value, Value)> {
    let mut result = Vec::new();
    flatten_json("", value, &mut result);
    result
}

fn flatten_json(prefix: &str, value: serde_json::Value, result: &mut Vec<(Value, Value)>) {
    match value {
        serde_json::Value::Null => {
            result.push((
                Value::String(prefix.to_string()),
                Value::String("null".to_string()),
            ));
        }
        serde_json::Value::Bool(b) => {
            result.push((
                Value::String(prefix.to_string()),
                Value::String(b.to_string()),
            ));
        }
        serde_json::Value::Number(n) => {
            result.push((
                Value::String(prefix.to_string()),
                Value::String(n.to_string()),
            ));
        }
        serde_json::Value::String(s) => {
            result.push((Value::String(prefix.to_string()), Value::String(s)));
        }
        serde_json::Value::Array(arr) => {
            let list: Vec<Value> = arr.into_iter().map(json_value_to_string).collect();
            result.push((Value::String(prefix.to_string()), Value::List(list)));
        }
        serde_json::Value::Object(obj) => {
            for (k, v) in obj {
                let new_prefix = if prefix.is_empty() {
                    k
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_json(&new_prefix, v, result);
            }
        }
    }
}

fn json_value_to_string(value: serde_json::Value) -> Value {
    match value {
        serde_json::Value::Null => Value::String("null".to_string()),
        serde_json::Value::Bool(b) => Value::String(b.to_string()),
        serde_json::Value::Number(n) => Value::String(n.to_string()),
        serde_json::Value::String(s) => Value::String(s),
        serde_json::Value::Array(arr) => {
            Value::List(arr.into_iter().map(json_value_to_string).collect())
        }
        serde_json::Value::Object(obj) => {
            let table: Vec<(Value, Value)> = obj
                .into_iter()
                .map(|(k, v)| (Value::String(k), json_value_to_string(v)))
                .collect();
            Value::Table(table)
        }
    }
}

fn table_to_json(table: &[(Value, Value)]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in table {
        let key = match k {
            Value::String(s) => s.clone(),
            _ => continue,
        };
        let value = value_to_json_value(v);
        map.insert(key, value);
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

pub fn toml_method(
    toml_str: &str,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method {
        "parse" => {
            let trimmed = toml_str.trim_matches('\n').trim_matches('\r').trim();
            let value: toml::Value = toml::de::from_str(trimmed).map_err(|e| RuntimeError {
                message: format!("toml.parse() failed: {}", e),
                file_path: String::new(),
                line,
                column,
            })?;
            let table = toml_to_table(value);
            Ok(Value::Table(table))
        }
        "encode" => {
            let table = match args.first() {
                Some(Value::Table(t)) => t.clone(),
                Some(v) => {
                    return Err(RuntimeError {
                        message: format!("toml.encode() requires a table, got {}", v.type_name()),
                        file_path: String::new(),
                        line,
                        column,
                    });
                }
                None => Vec::new(),
            };
            let value = table_to_toml(&table);
            let encoded = toml::to_string_pretty(&value).map_err(|e| RuntimeError {
                message: format!("toml.encode() failed: {}", e),
                file_path: String::new(),
                line,
                column,
            })?;
            Ok(Value::String(encoded))
        }
        _ => Err(RuntimeError {
            message: format!("toml() has no method '{}'", method),
            file_path: String::new(),
            line,
            column,
        }),
    }
}

fn toml_to_table(value: toml::Value) -> Vec<(Value, Value)> {
    let mut result = Vec::new();
    flatten_toml("", &value, &mut result);
    result
}

fn flatten_toml(prefix: &str, value: &toml::Value, result: &mut Vec<(Value, Value)>) {
    match value {
        toml::Value::String(s) => {
            result.push((Value::String(prefix.to_string()), Value::String(s.clone())));
        }
        toml::Value::Integer(i) => {
            result.push((Value::String(prefix.to_string()), Value::Integer(*i)));
        }
        toml::Value::Float(f) => {
            result.push((Value::String(prefix.to_string()), Value::Float(*f)));
        }
        toml::Value::Boolean(b) => {
            result.push((Value::String(prefix.to_string()), Value::Boolean(*b)));
        }
        toml::Value::Datetime(dt) => {
            result.push((
                Value::String(prefix.to_string()),
                Value::String(dt.to_string()),
            ));
        }
        toml::Value::Array(arr) => {
            let list: Vec<Value> = arr.iter().map(toml_value_to_value).collect();
            result.push((Value::String(prefix.to_string()), Value::List(list)));
        }
        toml::Value::Table(table) => {
            for (k, v) in table {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten_toml(&new_prefix, v, result);
            }
        }
    }
}

fn toml_value_to_value(value: &toml::Value) -> Value {
    match value {
        toml::Value::String(s) => Value::String(s.clone()),
        toml::Value::Integer(i) => Value::Integer(*i),
        toml::Value::Float(f) => Value::Float(*f),
        toml::Value::Boolean(b) => Value::Boolean(*b),
        toml::Value::Datetime(dt) => Value::String(dt.to_string()),
        toml::Value::Array(arr) => Value::List(arr.iter().map(toml_value_to_value).collect()),
        toml::Value::Table(table) => {
            let pairs: Vec<(Value, Value)> = table
                .iter()
                .map(|(k, v)| (Value::String(k.clone()), toml_value_to_value(v)))
                .collect();
            Value::Table(pairs)
        }
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
