// src/interpreter/builtins.rs
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
            line,
            column,
        });
    }

    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        line,
        column,
    })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| RuntimeError {
            message: format!("Failed to read input: {}", e),
            line,
            column,
        })?;

    Ok(Value::String(input.trim().to_string()))
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
            line,
            column,
        });
    }

    let url_val = interpreter.evaluate_expression(&args[0])?;
    match url_val {
        Value::String(url) => Ok(Value::FetchHandle(url)),
        _ => Err(RuntimeError {
            message: "fetch() argument must be a string URL".to_string(),
            line,
            column,
        }),
    }
}

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
                line,
                column,
            }),
        },
        Value::Integer(n) => Ok(Value::Integer(n)),
        Value::Float(f) => Ok(Value::Integer(f as i64)),
        Value::Boolean(b) => Ok(Value::Integer(if b { 1 } else { 0 })),
        _ => Err(RuntimeError {
            message: format!("Cannot convert {:?} to integer", arg_val),
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
                line,
                column,
            }),
        },
        Value::Integer(n) => Ok(Value::Float(n as f64)),
        Value::Float(f) => Ok(Value::Float(f)),
        Value::Boolean(b) => Ok(Value::Float(if b { 1.0 } else { 0.0 })),
        _ => Err(RuntimeError {
            message: format!("Cannot convert {:?} to float", arg_val),
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
            line,
            column,
        }),
    }
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
            line,
            column,
        });
    }
    let val = interpreter.evaluate_expression(&args[0])?;
    let s = interpreter.value_to_display_string(&val);
    print!("{}", s);
    io::stdout().flush().map_err(|e| RuntimeError {
        message: format!("Failed to flush stdout: {}", e),
        line,
        column,
    })?;
    Ok(Value::Void)
}

// ─── Shared text methods (string, char, word) ────────────────────────────────

fn text_methods(
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
        "exists" => Some(Ok(Value::Boolean(!s.is_empty()))),
        "contains" => Some(match args.first() {
            Some(Value::String(sub)) => Ok(Value::Boolean(s.contains(sub.as_str()))),
            Some(Value::Char(c)) => Ok(Value::Boolean(s.contains(*c))),
            Some(Value::Word(w)) => Ok(Value::Boolean(s.contains(w.as_str()))),
            _ => Err(RuntimeError {
                message: "contains() takes one string/char/word argument".to_string(),
                line,
                column,
            }),
        }),
        "starts_with" => Some(match args.first() {
            Some(Value::String(prefix)) => Ok(Value::Boolean(s.starts_with(prefix.as_str()))),
            Some(Value::Char(c)) => Ok(Value::Boolean(s.starts_with(*c))),
            Some(Value::Word(w)) => Ok(Value::Boolean(s.starts_with(w.as_str()))),
            _ => Err(RuntimeError {
                message: "starts_with() takes one string/char/word argument".to_string(),
                line,
                column,
            }),
        }),
        "ends_with" => Some(match args.first() {
            Some(Value::String(suffix)) => Ok(Value::Boolean(s.ends_with(suffix.as_str()))),
            Some(Value::Char(c)) => Ok(Value::Boolean(s.ends_with(*c))),
            Some(Value::Word(w)) => Ok(Value::Boolean(s.ends_with(w.as_str()))),
            _ => Err(RuntimeError {
                message: "ends_with() takes one string/char/word argument".to_string(),
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
        // .replace("old", "new") → string
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
        // .split(delimiter) → list(string)
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
                // split() with no args splits on whitespace
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
        // .first("char") / .first("word") → maybe(string)
        "first" => Some(match args.first() {
            Some(Value::String(kind)) if kind == "char" => match s.chars().next() {
                Some(c) => Ok(Value::Maybe(Some(Box::new(Value::Char(c))))),
                None => Ok(Value::Maybe(None)),
            },
            Some(Value::String(kind)) if kind == "word" => match s.split_whitespace().next() {
                Some(w) => Ok(Value::Maybe(Some(Box::new(Value::Word(w.to_string()))))),
                None => Ok(Value::Maybe(None)),
            },
            None => {
                // .first() with no arg defaults to first char
                match s.chars().next() {
                    Some(c) => Ok(Value::Maybe(Some(Box::new(Value::Char(c))))),
                    None => Ok(Value::Maybe(None)),
                }
            }
            _ => Err(RuntimeError {
                message: "string.first() takes 'char' or 'word' as argument".to_string(),
                line,
                column,
            }),
        }),
        // .last("char") / .last("word") → maybe(string)
        "last" => Some(match args.first() {
            Some(Value::String(kind)) if kind == "char" => match s.chars().next_back() {
                Some(c) => Ok(Value::Maybe(Some(Box::new(Value::Char(c))))),
                None => Ok(Value::Maybe(None)),
            },
            Some(Value::String(kind)) if kind == "word" => match s.split_whitespace().next_back() {
                Some(w) => Ok(Value::Maybe(Some(Box::new(Value::Word(w.to_string()))))),
                None => Ok(Value::Maybe(None)),
            },
            None => {
                // .last() with no arg defaults to last char
                match s.chars().next_back() {
                    Some(c) => Ok(Value::Maybe(Some(Box::new(Value::Char(c))))),
                    None => Ok(Value::Maybe(None)),
                }
            }
            _ => Err(RuntimeError {
                message: "string.last() takes 'char' or 'word' as argument".to_string(),
                line,
                column,
            }),
        }),
        // .as(int) / .as(float) / .as(bool) / .as(char) / .as(word) → maybe(T)
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
                "word" => {
                    if s.contains(char::is_whitespace) {
                        Ok(Value::Maybe(None))
                    } else {
                        Ok(Value::Maybe(Some(Box::new(Value::Word(s.to_string())))))
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
                    "as() takes a type argument: as(int), as(float), as(bool), as(char), as(word)"
                        .to_string(),
                line,
                column,
            }),
        }),
        _ => None,
    }
}

pub fn eval_method(
    object: Value,
    method: &str,
    args: &[Value],
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match (&object, method) {
        // ── int methods ───────────────────────────────────────────────────────
        (Value::Integer(n), "abs") => Ok(Value::Integer(n.abs())),
        (Value::Integer(n), "to_float") => Ok(Value::Float(*n as f64)),
        (Value::Integer(n), "to_string") => Ok(Value::String(n.to_string())),
        (Value::Integer(n), "exists") => Ok(Value::Boolean(*n != 0)),
        (Value::Integer(n), "pow") => match args.first() {
            Some(Value::Integer(exp)) if *exp >= 0 => Ok(Value::Integer(n.pow(*exp as u32))),
            _ => Err(RuntimeError {
                message: "int.pow() takes one non-negative integer argument".to_string(),
                line,
                column,
            }),
        },

        // ── float methods ─────────────────────────────────────────────────────
        (Value::Float(f), "abs") => Ok(Value::Float(f.abs())),
        (Value::Float(f), "floor") => Ok(Value::Integer(f.floor() as i64)),
        (Value::Float(f), "ceil") => Ok(Value::Integer(f.ceil() as i64)),
        (Value::Float(f), "round") => Ok(Value::Integer(f.round() as i64)),
        (Value::Float(f), "to_int") => Ok(Value::Integer(*f as i64)),
        (Value::Float(f), "to_string") => Ok(Value::String(f.to_string())),
        (Value::Float(f), "exists") => Ok(Value::Boolean(*f != 0.0)),

        // ── string methods — shared via text_methods ──────────────────────────
        (Value::String(s), method) => {
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

        // ── char methods ──────────────────────────────────────────────────────
        (Value::Char(_), "exists") => Ok(Value::Boolean(true)), // a char always exists
        (Value::Char(c), "to_string") => Ok(Value::String(c.to_string())),
        (Value::Char(c), "to_int") => Ok(Value::Integer(*c as i64)),
        (Value::Char(c), method) => {
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

        // ── word methods ──────────────────────────────────────────────────────
        (Value::Word(w), "exists") => Ok(Value::Boolean(!w.is_empty())),
        (Value::Word(w), "to_string") => Ok(Value::String(w.clone())),
        (Value::Word(w), method) => {
            let s = w.clone();
            if let Some(result) = text_methods(&s, method, args, line, column) {
                result
            } else {
                Err(RuntimeError {
                    message: format!("word has no method '{}'", method),
                    line,
                    column,
                })
            }
        }

        // ── bool methods ──────────────────────────────────────────────────────
        (Value::Boolean(b), "to_string") => Ok(Value::String(b.to_string())),
        (Value::Boolean(b), "exists") => Ok(Value::Boolean(*b)),

        // ── maybe methods ─────────────────────────────────────────────────────
        (Value::Maybe(inner), "exists") => Ok(Value::Boolean(inner.is_some())),
        (Value::Maybe(Some(inner)), "or") => Ok(*inner.clone()),
        (Value::Maybe(None), "or") => match args.first() {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError {
                message: "maybe.or() requires a fallback value".to_string(),
                line,
                column,
            }),
        },
        (Value::Maybe(Some(inner)), method) => {
            eval_method(*inner.clone(), method, args, line, column)
        }
        (Value::Maybe(None), method) => Err(RuntimeError {
            message: format!(
                "Cannot call '{}' on empty maybe — use .or() to provide a fallback first",
                method
            ),
            line,
            column,
        }),

        // ── list methods ──────────────────────────────────────────────────────
        (Value::List(items), "len") => Ok(Value::Integer(items.len() as i64)),
        (Value::List(items), "exists") => Ok(Value::Boolean(!items.is_empty())),
        (Value::List(items), "get") => match args.first() {
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
        (Value::List(items), "contains") => {
            let target = args.first().ok_or(RuntimeError {
                message: "list.contains() takes one argument".to_string(),
                line,
                column,
            })?;
            Ok(Value::Boolean(items.contains(target)))
        }
        (Value::List(items), "where") => match args.first() {
            Some(target) => {
                let idx = items.iter().position(|v| v == target);
                match idx {
                    Some(i) => Ok(Value::Integer(i as i64)),
                    None => Ok(Value::Integer(-1)),
                }
            }
            _ => Err(RuntimeError {
                message: "list.where() takes one argument".to_string(),
                line,
                column,
            }),
        },
        (Value::List(items), "add") => {
            let val = args.first().ok_or(RuntimeError {
                message: "list.add() takes one argument".to_string(),
                line,
                column,
            })?;
            let mut new_items = items.clone();
            new_items.push(val.clone());
            Ok(Value::List(new_items))
        }
        (Value::List(items), "remove") => match args.first() {
            Some(Value::Integer(i)) => {
                let idx = *i as usize;
                if idx >= items.len() {
                    return Err(RuntimeError {
                        message: format!("list index {} out of bounds (len {})", i, items.len()),
                        line,
                        column,
                    });
                }
                let mut new_items = items.clone();
                new_items.remove(idx);
                Ok(Value::List(new_items))
            }
            _ => Err(RuntimeError {
                message: "list.remove() takes one integer index argument".to_string(),
                line,
                column,
            }),
        },
        (Value::List(items), "reverse") => {
            let mut new_items = items.clone();
            new_items.reverse();
            Ok(Value::List(new_items))
        }
        (Value::List(items), "sort") => {
            let mut new_items = items.clone();
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
        (Value::List(items), "first") => match items.first() {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError {
                message: "list.first() called on empty list".to_string(),
                line,
                column,
            }),
        },
        (Value::List(items), "last") => match items.last() {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError {
                message: "list.last() called on empty list".to_string(),
                line,
                column,
            }),
        },

        // ── table methods ─────────────────────────────────────────────────────
        (Value::Table(pairs), "len") => Ok(Value::Integer(pairs.len() as i64)),
        (Value::Table(pairs), "exists") => Ok(Value::Boolean(!pairs.is_empty())),
        (Value::Table(pairs), "has") => {
            let key = args.first().ok_or(RuntimeError {
                message: "table.has() takes one argument".to_string(),
                line,
                column,
            })?;
            Ok(Value::Boolean(pairs.iter().any(|(k, _)| k == key)))
        }
        (Value::Table(pairs), "get") => {
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
        (Value::Table(pairs), "set") => {
            if args.len() != 2 {
                return Err(RuntimeError {
                    message: "table.set() takes two arguments (key, value)".to_string(),
                    line,
                    column,
                });
            }
            let key = args[0].clone();
            let val = args[1].clone();
            let mut new_pairs = pairs.clone();
            if let Some(entry) = new_pairs.iter_mut().find(|(k, _)| k == &key) {
                entry.1 = val;
            } else {
                new_pairs.push((key, val));
            }
            Ok(Value::Table(new_pairs))
        }
        (Value::Table(pairs), "remove") => {
            let key = args.first().ok_or(RuntimeError {
                message: "table.remove() takes one argument".to_string(),
                line,
                column,
            })?;
            let mut new_pairs = pairs.clone();
            new_pairs.retain(|(k, _)| k != key);
            Ok(Value::Table(new_pairs))
        }
        (Value::Table(pairs), "keys") => {
            let keys: Vec<Value> = pairs.iter().map(|(k, _)| k.clone()).collect();
            Ok(Value::List(keys))
        }
        (Value::Table(pairs), "values") => {
            let vals: Vec<Value> = pairs.iter().map(|(_, v)| v.clone()).collect();
            Ok(Value::List(vals))
        }

        // ── fetch handle methods ───────────────────────────────────────────────
        // TODO: switch to worry(string) once worry() is implemented
        (Value::FetchHandle(url), "get") => match ureq::get(url).call() {
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
        (Value::FetchHandle(url), "send") => {
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

        // ── input handle methods ───────────────────────────────────────────────
        (Value::InputHandle, "flag") => match args.first() {
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
        (Value::InputHandle, "option") => match args.first() {
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

        // ── file handle methods ────────────────────────────────────────────────
        (Value::FileHandle(path), "read") => match std::fs::read_to_string(path) {
            Ok(contents) => Ok(Value::String(contents)),
            Err(e) => Err(RuntimeError {
                message: format!("file(\"{}\").read() failed: {}", path, e),
                line,
                column,
            }),
        },
        (Value::FileHandle(path), "write") => {
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
        (Value::FileHandle(path), "append") => {
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
        (Value::FileHandle(path), "exists") => {
            Ok(Value::Boolean(std::path::Path::new(path).exists()))
        }

        _ => Err(RuntimeError {
            message: format!("'{}' has no method '{}'", object.type_name(), method),
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
            line,
            column,
        });
    }
    let path_val = interpreter.evaluate_expression(&args[0])?;
    match path_val {
        Value::String(path) => Ok(Value::FileHandle(path)),
        _ => Err(RuntimeError {
            message: "file() argument must be a string path".to_string(),
            line,
            column,
        }),
    }
}
