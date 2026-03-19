// src/interpreter/statements.rs
use crate::error::diagnostic::Span;
use crate::interpreter::value::{RuntimeError, Value};
use crate::interpreter::{Interpreter, VarInfo};

impl Interpreter {
    pub fn execute_print(&mut self, expr: &crate::ast::Expr) -> Result<Value, RuntimeError> {
        let val = self.evaluate_expression(expr)?;
        let output = self.value_to_display_string(&val);
        self.debug.log_print(&output);
        if self.debug_mode {
            self.output_buffer.push(output);
        } else {
            println!("{}", output);
        }
        Ok(Value::Void)
    }

    pub fn value_to_display_string(&self, val: &Value) -> String {
        match val {
            Value::String(s) => s.clone(),
            Value::Char(c) => c.to_string(),
            Value::Integer(n) => n.to_string(),
            Value::Float(f) => {
                if f.abs() > 1_000_000_000_000.0 || (f.abs() < 0.0001 && *f != 0.0) {
                    format!("{:e}", f)
                } else {
                    f.to_string()
                }
            }
            Value::Boolean(b) => b.to_string(),
            Value::Void => String::new(),
            Value::Maybe(Some(inner)) => self.value_to_display_string(inner),
            Value::Maybe(None) => String::new(),
            Value::List(items) => {
                let parts: Vec<String> = items
                    .iter()
                    .map(|v| self.value_to_display_string(v))
                    .collect();
                format!("({})", parts.join(", "))
            }
            Value::Table(pairs) => {
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "{}: {}",
                            self.value_to_display_string(k),
                            self.value_to_display_string(v)
                        )
                    })
                    .collect();
                format!("({})", parts.join(", "))
            }
            Value::FetchHandle(url) => format!("fetch(\"{}\")", url),
            Value::InputHandle => "input()".to_string(),
            Value::FileHandle(path) => format!("file(\"{}\")", path),
            Value::Struct { name, fields } => {
                let mut pairs: Vec<(&String, &Value)> = fields.iter().collect();
                pairs.sort_by_key(|(k, _)| k.as_str());
                let parts: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, self.value_to_display_string(v)))
                    .collect();
                format!("{}({})", name, parts.join(", "))
            }
        }
    }

    pub fn execute_variable_declaration(
        &mut self,
        type_name: &str,
        name: &str,
        value: &crate::ast::Expr,
        else_error: &Option<(String, Vec<crate::ast::Stmt>)>,
    ) -> Result<Value, RuntimeError> {
        let val = match self.evaluate_expression(value) {
            Ok(v) => v,
            Err(e) if e.message.starts_with("__raise__") => {
                let msg = e
                    .message
                    .strip_prefix("__raise__")
                    .unwrap_or("")
                    .to_string();
                if let Some((error_var, body)) = else_error {
                    self.push_scope();
                    self.declare_variable(error_var, Value::String(msg));
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                    self.pop_scope();
                    return Ok(Value::Void);
                } else {
                    return Err(RuntimeError {
                        message: e
                            .message
                            .strip_prefix("__raise__")
                            .unwrap_or(&e.message)
                            .to_string(),
                        line: e.line,
                        column: e.column,
                    });
                }
            }
            Err(e) => return Err(e),
        };

        let val = match (type_name, val) {
            ("int", Value::Integer(n)) => Value::Integer(n),
            ("float", Value::Float(f)) => Value::Float(f),
            ("bool", Value::Boolean(b)) => Value::Boolean(b),
            ("string", Value::String(s)) => Value::String(s),

            // char — from CharLiteral (Char variant) or single-char string
            ("char", Value::Char(c)) => Value::Char(c),
            ("char", Value::String(s)) if s.chars().count() == 1 => {
                Value::Char(s.chars().next().unwrap())
            }
            ("char", Value::String(s)) => {
                return Err(RuntimeError {
                    message: format!(
                        "Type mismatch: expected char (single character), got string of length {}",
                        s.chars().count()
                    ),
                    line: value.line,
                    column: value.column,
                });
            }

            // maybe — auto-wrap non-maybe values
            (t, Value::Maybe(inner)) if t.starts_with("maybe") => Value::Maybe(inner),
            (t, v) if t.starts_with("maybe") => Value::Maybe(Some(Box::new(v))),

            // worry — error case handled above via __raise__, success just passes value through
            (t, v) if t.starts_with("worry") => v,

            // list
            (t, Value::List(items)) if t.starts_with("list") => Value::List(items),
            (t, Value::Maybe(None)) if t.starts_with("list") => Value::List(Vec::new()),

            // table
            (t, Value::Table(pairs)) if t.starts_with("table") => Value::Table(pairs),
            (t, Value::Maybe(None)) if t.starts_with("table") => Value::Table(Vec::new()),

            // struct — type_name is the struct name e.g. "Point"
            (t, Value::Struct { name, fields }) if t == name => Value::Struct { name, fields },

            (expected, actual) => {
                return Err(RuntimeError {
                    message: format!(
                        "Type mismatch: expected {}, got {}",
                        expected,
                        actual.type_name()
                    ),
                    line: value.line,
                    column: value.column,
                });
            }
        };

        let span = Span {
            filename: String::new(),
            line: value.line,
            column: value.column,
            length: name.len(),
        };

        self.var_info.insert(
            name.to_string(),
            VarInfo {
                name: name.to_string(),
                span,
            },
        );

        self.declare_variable(name, val);
        Ok(Value::Void)
    }
}
