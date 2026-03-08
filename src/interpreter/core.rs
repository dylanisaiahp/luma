// src/interpreter/core.rs
use crate::ast::Stmt;
use crate::debug::InterpreterDebug;
use crate::error::diagnostic::Diagnostic;
use crate::interpreter::value::{RuntimeError, Value};
use crate::interpreter::{FunctionDef, VarInfo};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Interpreter {
    pub scopes: Vec<HashMap<String, Value>>,
    pub var_info: HashMap<String, VarInfo>,
    pub used_variables: HashSet<String>,
    pub warnings: Vec<Diagnostic>,
    pub functions: HashMap<String, FunctionDef>,
    pub debug: InterpreterDebug,
    pub output_buffer: Vec<String>,
    pub debug_mode: bool,
    /// Holds a List or Table value being returned from a function.
    /// The encode/decode string protocol can't round-trip complex values,
    /// so we stash them here and use the sentinel "__return_slot__" instead.
    pub return_slot: Option<Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            var_info: HashMap::new(),
            used_variables: HashSet::new(),
            warnings: Vec::new(),
            functions: HashMap::new(),
            debug: InterpreterDebug::new(),
            output_buffer: Vec::new(),
            debug_mode: false,
            return_slot: None,
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }
    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn get_variable(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val);
            }
        }
        None
    }

    pub fn set_variable(&mut self, name: &str, value: Value) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
        }
    }

    pub fn declare_variable(&mut self, name: &str, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
        }
    }

    pub fn take_warnings(&mut self) -> Vec<Diagnostic> {
        std::mem::take(&mut self.warnings)
    }

    pub(crate) fn check_unused_variables(&mut self, source: &str, filename: &str) {
        for info in self.var_info.values() {
            if !self.used_variables.contains(&info.name) {
                let source_line = if info.span.line > 0 {
                    source
                        .lines()
                        .nth(info.span.line - 1)
                        .unwrap_or("")
                        .to_string()
                } else {
                    "".to_string()
                };

                let mut span = info.span.clone();
                span.filename = filename.to_string();

                let warning = Diagnostic::new_warning(
                    "W001",
                    &format!("Unused variable: '{}'", info.name),
                    span,
                    source_line,
                    "If you meant to ignore it, prefix with underscore: _name",
                );
                self.warnings.push(warning);
            }
        }
    }

    fn register_functions(&mut self, statements: &[Stmt]) {
        for stmt in statements {
            if let Stmt::Function {
                return_type,
                name,
                params,
                body,
            } = stmt
                && name != "main"
            {
                self.functions.insert(
                    name.clone(),
                    FunctionDef {
                        return_type: return_type.clone(),
                        params: params.clone(),
                        body: body.clone(),
                    },
                );
            }
        }
    }

    pub fn interpret(
        &mut self,
        program: &Stmt,
        source: &str,
        filename: &str,
    ) -> Result<(), RuntimeError> {
        if let Stmt::Program(stmts) = program {
            self.register_functions(stmts);
        }
        self.execute_statement(program)?;
        self.check_unused_variables(source, filename);
        Ok(())
    }

    pub fn execute_statement(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::Program(statements) => {
                for stmt in statements {
                    self.execute_statement(stmt)?;
                }
                Ok(Value::Void)
            }
            Stmt::Function { name, body, .. } => {
                if name == "main" {
                    self.push_scope();
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                    self.pop_scope();
                }
                Ok(Value::Void)
            }
            Stmt::Use { module } => {
                eprintln!(
                    "[!] 'use {}' — imports are not yet supported. Coming soon!",
                    module
                );
                Ok(Value::Void)
            }
            Stmt::Print(expr) => self.execute_print(expr),
            Stmt::VariableDeclaration {
                type_name,
                name,
                value,
                else_error,
            } => self.execute_variable_declaration(type_name, name, value, else_error),
            Stmt::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(Value::Void)
            }
            Stmt::Return(expr) => {
                let val = match expr {
                    Some(e) => self.evaluate_expression(e)?,
                    None => Value::Void,
                };
                let encoded = self.encode_return_value(&val);
                Err(RuntimeError {
                    message: format!("__return__{}", encoded),
                    line: 0,
                    column: 0,
                })
            }
            Stmt::While { condition, body } => self.execute_while(condition, body),
            Stmt::For {
                var,
                start,
                end,
                body,
            } => self.execute_for(var, start, end, body),
            Stmt::ForIn {
                var,
                iterable,
                body,
            } => self.execute_for_in(var, iterable, body),
            Stmt::ForInTable {
                key_var,
                val_var,
                iterable,
                body,
            } => self.execute_for_in_table(key_var, val_var, iterable, body),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.execute_if(condition, then_branch, else_branch),
            Stmt::Match {
                value,
                arms,
                else_arm,
            } => self.execute_match(value, arms, else_arm),
            Stmt::Raise {
                message,
                line,
                column,
            } => {
                let msg = match self.evaluate_expression(message)? {
                    Value::String(s) => s,
                    other => self.value_to_display_string(&other).to_string(),
                };
                Err(RuntimeError {
                    message: format!("__raise__{}", msg),
                    line: *line,
                    column: *column,
                })
            }
            Stmt::Break => Err(RuntimeError {
                message: "__break__".to_string(),
                line: 0,
                column: 0,
            }),
        }
    }

    /// Encode a return value as a string sentinel for the error-propagation
    /// return mechanism. List and Table values cannot be serialized safely,
    /// so they are stashed in `self.return_slot` and a sentinel is returned.
    pub fn encode_return_value(&mut self, val: &Value) -> String {
        match val {
            Value::Integer(n) => format!("int:{}", n),
            Value::Float(f) => format!("float:{}", f),
            Value::Boolean(b) => format!("bool:{}", b),
            Value::String(s) => format!("string:{}", s),
            Value::Char(c) => format!("char:{}", c),
            Value::Word(w) => format!("word:{}", w),
            Value::Void => "void:".to_string(),
            Value::Maybe(Some(inner)) => {
                let inner_encoded = self.encode_return_value(inner);
                format!("maybe:{}", inner_encoded)
            }
            Value::Maybe(None) => "maybe:empty".to_string(),
            // List and Table cannot be round-tripped through a string.
            // Stash the value in the slot and use a sentinel instead.
            Value::List(_) | Value::Table(_) => {
                self.return_slot = Some(val.clone());
                "__return_slot__".to_string()
            }
            Value::FetchHandle(url) => format!("fetch:{}", url),
            Value::InputHandle => "input:".to_string(),
            Value::FileHandle(path) => format!("file:{}", path),
        }
    }

    pub fn decode_return_value(encoded: &str) -> Value {
        if let Some(rest) = encoded.strip_prefix("int:") {
            Value::Integer(rest.parse().unwrap_or(0))
        } else if let Some(rest) = encoded.strip_prefix("float:") {
            Value::Float(rest.parse().unwrap_or(0.0))
        } else if let Some(rest) = encoded.strip_prefix("bool:") {
            Value::Boolean(rest == "true")
        } else if let Some(rest) = encoded.strip_prefix("string:") {
            Value::String(rest.to_string())
        } else if let Some(rest) = encoded.strip_prefix("char:") {
            Value::Char(rest.chars().next().unwrap_or('\0'))
        } else if let Some(rest) = encoded.strip_prefix("word:") {
            Value::Word(rest.to_string())
        } else if encoded == "maybe:empty" {
            Value::Maybe(None)
        } else if let Some(rest) = encoded.strip_prefix("maybe:") {
            Value::Maybe(Some(Box::new(Self::decode_return_value(rest))))
        } else {
            Value::Void
        }
    }
}
