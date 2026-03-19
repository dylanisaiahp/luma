// src/interpreter/builtins/text.rs
use crate::interpreter::value::{RuntimeError, Value};

/// Shared methods across string and char types.
pub fn text_methods(
    s: &str,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Option<Result<Value, RuntimeError>> {
    match method {
        "len" => Some(Ok(Value::Integer(s.chars().count() as i64))),
        "upper" => Some(Ok(Value::String(s.to_uppercase()))),
        "lower" => Some(Ok(Value::String(s.to_lowercase()))),
        "trim" => Some(Ok(Value::String(s.trim().to_string()))),
        "reverse" => Some(Ok(Value::String(s.chars().rev().collect()))),
        "chars" => Some(Ok(Value::List(s.chars().map(Value::Char).collect()))),
        "exists" => Some(Ok(Value::Boolean(!s.is_empty()))),
        "contains" => Some(match args.first() {
            Some(Value::String(sub)) => Ok(Value::Boolean(s.contains(sub.as_str()))),
            Some(Value::Char(c)) => Ok(Value::Boolean(s.contains(*c))),
            _ => Err(RuntimeError {
                message: "contains() takes one string or char argument".to_string(),
                line,
                column,
            }),
        }),
        "starts_with" => Some(match args.first() {
            Some(Value::String(prefix)) => Ok(Value::Boolean(s.starts_with(prefix.as_str()))),
            Some(Value::Char(c)) => Ok(Value::Boolean(s.starts_with(*c))),
            _ => Err(RuntimeError {
                message: "starts_with() takes one string or char argument".to_string(),
                line,
                column,
            }),
        }),
        "ends_with" => Some(match args.first() {
            Some(Value::String(suffix)) => Ok(Value::Boolean(s.ends_with(suffix.as_str()))),
            Some(Value::Char(c)) => Ok(Value::Boolean(s.ends_with(*c))),
            _ => Err(RuntimeError {
                message: "ends_with() takes one string or char argument".to_string(),
                line,
                column,
            }),
        }),
        "repeat" => Some(match args.first() {
            Some(Value::Integer(n)) if *n >= 0 => Ok(Value::String(s.repeat(*n as usize))),
            _ => Err(RuntimeError {
                message: "repeat() takes one non-negative integer argument".to_string(),
                line,
                column,
            }),
        }),
        "replace" => Some(match (args.first(), args.get(1)) {
            (Some(Value::String(from)), Some(Value::String(to))) => {
                Ok(Value::String(s.replace(from.as_str(), to.as_str())))
            }
            _ => Err(RuntimeError {
                message: "replace() takes two string arguments: replace(from, to)".to_string(),
                line,
                column,
            }),
        }),
        "split" => Some(match args.first() {
            Some(Value::String(delim)) => {
                let parts: Vec<Value> = s
                    .split(delim.as_str())
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Ok(Value::List(parts))
            }
            Some(Value::Char(c)) => {
                let parts: Vec<Value> = s.split(*c).map(|p| Value::String(p.to_string())).collect();
                Ok(Value::List(parts))
            }
            None => {
                let parts: Vec<Value> = s
                    .split_whitespace()
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Ok(Value::List(parts))
            }
            _ => Err(RuntimeError {
                message: "split() takes one string or char argument".to_string(),
                line,
                column,
            }),
        }),
        "first" => Some(match args.first() {
            Some(Value::String(kind)) if kind == "char" => match s.chars().next() {
                Some(c) => Ok(Value::Maybe(Some(Box::new(Value::Char(c))))),
                None => Ok(Value::Maybe(None)),
            },
            None => match s.chars().next() {
                Some(c) => Ok(Value::Maybe(Some(Box::new(Value::Char(c))))),
                None => Ok(Value::Maybe(None)),
            },
            _ => Err(RuntimeError {
                message: "first() takes 'char' as argument, or no argument".to_string(),
                line,
                column,
            }),
        }),
        "last" => Some(match args.first() {
            Some(Value::String(kind)) if kind == "char" => match s.chars().next_back() {
                Some(c) => Ok(Value::Maybe(Some(Box::new(Value::Char(c))))),
                None => Ok(Value::Maybe(None)),
            },
            None => match s.chars().next_back() {
                Some(c) => Ok(Value::Maybe(Some(Box::new(Value::Char(c))))),
                None => Ok(Value::Maybe(None)),
            },
            _ => Err(RuntimeError {
                message: "last() takes 'char' as argument, or no argument".to_string(),
                line,
                column,
            }),
        }),
        "as" => Some(match args.first() {
            Some(Value::String(target)) => match target.as_str() {
                "int" => match s.trim().parse::<i64>() {
                    Ok(n) => Ok(Value::Maybe(Some(Box::new(Value::Integer(n))))),
                    Err(_) => Ok(Value::Maybe(None)),
                },
                "float" => match s.trim().parse::<f64>() {
                    Ok(f) => Ok(Value::Maybe(Some(Box::new(Value::Float(f))))),
                    Err(_) => Ok(Value::Maybe(None)),
                },
                "bool" => match s.trim() {
                    "true" => Ok(Value::Maybe(Some(Box::new(Value::Boolean(true))))),
                    "false" => Ok(Value::Maybe(Some(Box::new(Value::Boolean(false))))),
                    _ => Ok(Value::Maybe(None)),
                },
                "char" => {
                    if s.chars().count() == 1 {
                        Ok(Value::Maybe(Some(Box::new(Value::Char(
                            s.chars().next().unwrap(),
                        )))))
                    } else {
                        Ok(Value::Maybe(None))
                    }
                }
                "string" => Ok(Value::Maybe(Some(Box::new(Value::String(s.to_string()))))),
                _ => Err(RuntimeError {
                    message: format!("as() unknown target type '{}'", target),
                    line,
                    column,
                }),
            },
            _ => Err(RuntimeError {
                message:
                    "as() takes a type argument: as(int), as(float), as(bool), as(char), as(string)"
                        .to_string(),
                line,
                column,
            }),
        }),
        _ => None,
    }
}

pub fn string_method(
    s: &str,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    if let Some(result) = text_methods(s, method, args, line, column) {
        result
    } else {
        Err(RuntimeError {
            message: format!("string has no method '{}'", method),
            line,
            column,
        })
    }
}

pub fn char_method(
    c: &char,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method {
        "exists" => Ok(Value::Boolean(true)),
        "to_string" => Ok(Value::String(c.to_string())),
        "to_int" => Ok(Value::Integer(*c as i64)),
        _ => {
            let s = c.to_string();
            if let Some(result) = text_methods(&s, method, args, line, column) {
                result
            } else {
                Err(RuntimeError {
                    message: format!("char has no method '{}'", method),
                    line,
                    column,
                })
            }
        }
    }
}
