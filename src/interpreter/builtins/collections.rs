// src/interpreter/builtins/collections.rs
use crate::interpreter::value::{RuntimeError, Value};

pub fn list_method(
    items: &[Value],
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method {
        "len" => Ok(Value::Integer(items.len() as i64)),
        "exists" => Ok(Value::Boolean(!items.is_empty())),
        "get" => match args.first() {
            Some(Value::Integer(i)) => {
                let idx = *i as usize;
                match items.get(idx) {
                    Some(v) => Ok(v.clone()),
                    None => Err(RuntimeError {
                        message: format!("list index {} out of bounds (len {})", i, items.len()),
                        line,
                        column,
                    }),
                }
            }
            _ => Err(RuntimeError {
                message: "list.get() takes one integer argument".to_string(),
                line,
                column,
            }),
        },
        "contains" => {
            let target = args.first().ok_or(RuntimeError {
                message: "list.contains() takes one argument".to_string(),
                line,
                column,
            })?;
            Ok(Value::Boolean(items.contains(target)))
        }
        "where" => match args.first() {
            Some(target) => {
                let idx = items.iter().position(|v| v == target);
                Ok(Value::Integer(idx.map(|i| i as i64).unwrap_or(-1)))
            }
            _ => Err(RuntimeError {
                message: "list.where() takes one argument".to_string(),
                line,
                column,
            }),
        },
        "add" => {
            let val = args.first().ok_or(RuntimeError {
                message: "list.add() takes one argument".to_string(),
                line,
                column,
            })?;
            let mut new_items = items.to_vec();
            new_items.push(val.clone());
            Ok(Value::List(new_items))
        }
        "remove" => match args.first() {
            Some(Value::Integer(i)) => {
                let idx = *i as usize;
                if idx >= items.len() {
                    return Err(RuntimeError {
                        message: format!("list index {} out of bounds (len {})", i, items.len()),
                        line,
                        column,
                    });
                }
                let mut new_items = items.to_vec();
                new_items.remove(idx);
                Ok(Value::List(new_items))
            }
            _ => Err(RuntimeError {
                message: "list.remove() takes one integer index argument".to_string(),
                line,
                column,
            }),
        },
        "reverse" => {
            let mut new_items = items.to_vec();
            new_items.reverse();
            Ok(Value::List(new_items))
        }
        "sort" => {
            let mut new_items = items.to_vec();
            new_items.sort_by(|a, b| match (a, b) {
                (Value::Integer(x), Value::Integer(y)) => x.cmp(y),
                (Value::Float(x), Value::Float(y)) => {
                    x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                }
                (Value::String(x), Value::String(y)) => x.cmp(y),
                _ => std::cmp::Ordering::Equal,
            });
            Ok(Value::List(new_items))
        }
        "first" => match items.first() {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError {
                message: "list.first() called on empty list".to_string(),
                line,
                column,
            }),
        },
        "last" => match items.last() {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError {
                message: "list.last() called on empty list".to_string(),
                line,
                column,
            }),
        },
        "merge" => {
            let glue = match args.first() {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Char(c)) => c.to_string(),
                None => String::new(),
                _ => {
                    return Err(RuntimeError {
                        message: "list.merge() takes a string separator argument".to_string(),
                        line,
                        column,
                    });
                }
            };
            let parts: Vec<String> = items
                .iter()
                .map(|v| match v {
                    Value::String(s) => s.clone(),
                    Value::Char(c) => c.to_string(),
                    Value::Integer(n) => n.to_string(),
                    Value::Float(f) => f.to_string(),
                    Value::Boolean(b) => b.to_string(),
                    _ => String::new(),
                })
                .collect();
            Ok(Value::String(parts.join(&glue)))
        }
        _ => Err(RuntimeError {
            message: format!("list has no method '{}'", method),
            line,
            column,
        }),
    }
}

pub fn table_method(
    pairs: &[(Value, Value)],
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match method {
        "len" => Ok(Value::Integer(pairs.len() as i64)),
        "exists" => Ok(Value::Boolean(!pairs.is_empty())),
        "has" => {
            let key = args.first().ok_or(RuntimeError {
                message: "table.has() takes one argument".to_string(),
                line,
                column,
            })?;
            Ok(Value::Boolean(pairs.iter().any(|(k, _)| k == key)))
        }
        "get" => {
            let key = args.first().ok_or(RuntimeError {
                message: "table.get() takes one argument".to_string(),
                line,
                column,
            })?;
            match pairs.iter().find(|(k, _)| k == key) {
                Some((_, v)) => Ok(v.clone()),
                None => Err(RuntimeError {
                    message: "table key not found".to_string(),
                    line,
                    column,
                }),
            }
        }
        "set" => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "table.set() takes two arguments (key, value)".to_string(),
                    line,
                    column,
                });
            }
            let key = args[0].clone();
            let val = args[1].clone();
            let mut new_pairs = pairs.to_vec();
            if let Some(entry) = new_pairs.iter_mut().find(|(k, _)| k == &key) {
                entry.1 = val;
            } else {
                new_pairs.push((key, val));
            }
            Ok(Value::Table(new_pairs))
        }
        "remove" => {
            let key = args.first().ok_or(RuntimeError {
                message: "table.remove() takes one argument".to_string(),
                line,
                column,
            })?;
            let mut new_pairs = pairs.to_vec();
            new_pairs.retain(|(k, _)| k != key);
            Ok(Value::Table(new_pairs))
        }
        "keys" => Ok(Value::List(pairs.iter().map(|(k, _)| k.clone()).collect())),
        "values" => Ok(Value::List(pairs.iter().map(|(_, v)| v.clone()).collect())),
        _ => Err(RuntimeError {
            message: format!("table has no method '{}'", method),
            line,
            column,
        }),
    }
}
