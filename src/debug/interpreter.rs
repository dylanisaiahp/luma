// src/debug/interpreter.rs
use crate::debug::format::{self, interpreter_tag, level_tag};
use crate::interpreter::value::Value;

pub struct InterpreterDebug {
    pub events: Vec<DebugEvent>,
}

pub enum DebugEvent {
    Call {
        name: String,
        args: String,
        result: String,
    },
    MethodCall {
        object: String,
        method: String,
        result: String,
    },
    Print {
        value: String,
    },
}

impl InterpreterDebug {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn log_call(&mut self, name: &str, args: &str, result: &Value) {
        self.events.push(DebugEvent::Call {
            name: name.to_string(),
            args: args.to_string(),
            result: format_value(result),
        });
    }

    pub fn log_method_call(&mut self, object: &Value, method: &str, result: &Value) {
        self.events.push(DebugEvent::MethodCall {
            object: format_value(object),
            method: method.to_string(),
            result: format_value(result),
        });
    }

    pub fn log_print(&mut self, value: &str) {
        self.events.push(DebugEvent::Print {
            value: value.to_string(),
        });
    }

    pub fn print_debug(&self, verbose: bool) {
        let tag = interpreter_tag();
        let level = level_tag(verbose);

        println!("{}", level);
        println!("{}", format::top(&tag, "Executing main()"));

        let len = self.events.len();
        for (i, event) in self.events.iter().enumerate() {
            let is_last = i == len - 1;
            let line = match event {
                DebugEvent::Call { name, args, result } => {
                    format!("called {}({}) → {}", name, args, result)
                }
                DebugEvent::MethodCall {
                    object,
                    method,
                    result,
                } => {
                    format!("{}.{}() → {}", object, method, result)
                }
                DebugEvent::Print { value } => {
                    format!("printed: {}", value)
                }
            };
            if is_last {
                println!("{}", format::bot(&tag, &line));
            } else {
                println!("{}", format::mid(&tag, &line));
            }
        }

        if self.events.is_empty() {
            println!("{}", format::bot(&tag, "Done"));
        }
        println!();
    }
}

impl Default for InterpreterDebug {
    fn default() -> Self {
        Self::new()
    }
}

fn format_value(val: &Value) -> String {
    match val {
        Value::Integer(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{}\"", s),
        Value::Char(c) => format!("'{}'", c),
        Value::Boolean(b) => b.to_string(),
        Value::Void => "void".to_string(),
        Value::Maybe(Some(inner)) => format!("Maybe({})", format_value(inner)),
        Value::Maybe(None) => "Maybe(empty)".to_string(),
        Value::List(items) => format!("List({})", items.len()),
        Value::Table(pairs) => format!("Table({})", pairs.len()),
        Value::FetchHandle(url) => format!("fetch(\"{}\")", url),
        Value::InputHandle => "input()".to_string(),
        Value::FileHandle(path) => format!("file(\"{}\")", path),
        Value::Struct { name, fields } => format!("{}({} fields)", name, fields.len()),
    }
}
