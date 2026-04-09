// src/interpreter/core.rs
use crate::ast::{EnumVariant, Stmt};
use crate::debug::InterpreterDebug;
use crate::error::diagnostic::Diagnostic;
use crate::interpreter::value::{RuntimeError, Value};
use crate::interpreter::{FunctionDef, StructDef, VarInfo};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Interpreter {
    pub scopes: Vec<HashMap<String, Value>>,
    pub var_info: HashMap<String, VarInfo>,
    pub used_variables: HashSet<String>,
    pub warnings: Vec<Diagnostic>,
    pub functions: HashMap<String, FunctionDef>,
    pub struct_defs: HashMap<String, StructDef>,
    pub enum_defs: HashMap<String, Vec<EnumVariant>>,
    pub debug: InterpreterDebug,
    pub output_buffer: Vec<String>,
    pub debug_mode: bool,
    pub return_slot: Option<Value>,
    pub call_depth: usize,
    pub current_file: String,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            var_info: HashMap::new(),
            used_variables: HashSet::new(),
            warnings: Vec::new(),
            functions: HashMap::new(),
            struct_defs: HashMap::new(),
            enum_defs: HashMap::new(),
            debug: InterpreterDebug::new(),
            output_buffer: Vec::new(),
            debug_mode: false,
            return_slot: None,
            call_depth: 0,
            current_file: String::new(),
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

    pub fn set_variable(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        if let Some(info) = self.var_info.get(name)
            && !info.mutable
        {
            return Err(RuntimeError {
                message: format!(
                    "Cannot reassign '{}' — it is immutable. Use 'mutable {} ...' to allow reassignment.",
                    name, name
                ),
                file_path: String::new(),
                line: 0,
                column: 0,
            });
        }
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), value);
        }
        Ok(())
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
                    &format!(
                        "If you meant to ignore it, prefix with underscore: _{}",
                        info.name
                    ),
                );
                self.warnings.push(warning);
            }
        }
    }

    fn register_functions(&mut self, statements: &[Stmt], source_file: &str) {
        for stmt in statements {
            match stmt {
                Stmt::Function {
                    return_type,
                    name,
                    params,
                    body,
                } if name != "main" => {
                    self.functions.insert(
                        name.clone(),
                        FunctionDef {
                            return_type: return_type.clone(),
                            params: params.clone(),
                            body: body.clone(),
                            source_file: source_file.to_string(),
                        },
                    );
                }
                Stmt::StructDeclaration {
                    name,
                    fields,
                    methods,
                } => {
                    self.struct_defs.insert(
                        name.clone(),
                        StructDef {
                            fields: fields.clone(),
                            methods: methods.clone(),
                        },
                    );
                }
                Stmt::EnumDeclaration { name, variants } => {
                    self.enum_defs.insert(name.clone(), variants.clone());
                }
                _ => {}
            }
        }
    }

    /// Interpret a program with per-file statement groups for accurate error locations.
    pub fn interpret_grouped(
        &mut self,
        files: &[(String, String, Vec<Stmt>)], // (file_path, source, statements)
        entry_file: &str,
    ) -> Result<(), RuntimeError> {
        // Register all functions with their source files
        for (file_path, _source, stmts) in files {
            self.register_functions(stmts, file_path);
        }

        // Execute statements file by file, setting current_file for each
        for (file_path, _source, stmts) in files {
            self.current_file = file_path.clone();
            for stmt in stmts {
                self.execute_statement(stmt)?;
            }
        }

        // Check unused variables for entry file
        if let Some((_, source, _)) = files.iter().find(|(p, _, _)| p == entry_file) {
            self.check_unused_variables(source, entry_file);
        }

        Ok(())
    }

    pub fn execute_statement(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        let file = self.current_file.clone();
        let result = self.execute_statement_inner(stmt);
        result.map_err(|e| e.with_file(&file))
    }

    fn execute_statement_inner(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
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
            Stmt::StructDeclaration { .. } => Ok(Value::Void),
            Stmt::EnumDeclaration { .. } => Ok(Value::Void),
            Stmt::Use { .. } => Ok(Value::Void),
            Stmt::ModuleDeclaration { .. } => Ok(Value::Void),
            Stmt::Print(expr) => self.execute_print(expr),
            Stmt::VariableDeclaration {
                type_name,
                name,
                value,
                mutable,
                else_error,
            } => self.execute_variable_declaration(type_name, name, value, *mutable, else_error),
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
                    file_path: String::new(),
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
                    file_path: String::new(),
                    line: *line,
                    column: *column,
                })
            }
            Stmt::Break => Err(RuntimeError {
                message: "__break__".to_string(),
                file_path: String::new(),
                line: 0,
                column: 0,
            }),
        }
    }

    pub fn encode_return_value(&mut self, val: &Value) -> String {
        match val {
            Value::Integer(n) => format!("int:{}", n),
            Value::Float(f) => format!("float:{}", f),
            Value::Boolean(b) => format!("bool:{}", b),
            Value::String(s) => format!("string:{}", s),
            Value::Char(c) => format!("char:{}", c),
            Value::Void => "void:".to_string(),
            Value::Option(Some(inner)) => {
                let inner_encoded = self.encode_return_value(inner);
                format!("option:{}", inner_encoded)
            }
            Value::Option(None) => "option:none".to_string(),
            Value::List(_) | Value::Table(_) | Value::Struct { .. } => {
                self.return_slot = Some(val.clone());
                "__return_slot__".to_string()
            }
            Value::FetchHandle(url) => format!("fetch:{}", url),
            Value::InputHandle => "input:".to_string(),
            Value::FileHandle(path) => format!("file:{}", path),
            Value::EnumVariant { enum_name, variant } => {
                format!("enum:{}.{}", enum_name, variant)
            }
            Value::EnumVariantData {
                enum_name,
                variant,
                data,
            } => {
                let mut encoded_data: Vec<String> = Vec::new();
                for v in data {
                    encoded_data.push(self.encode_return_value(v));
                }
                format!("enum:{}.{}({})", enum_name, variant, encoded_data.join(","))
            }
        }
    }

    pub fn decode_return_value(encoded: &str, return_slot: Option<Value>) -> Value {
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
        } else if encoded == "option:none" {
            Value::Option(None)
        } else if encoded == "void:" {
            Value::Void
        } else if encoded == "__return_slot__" {
            return_slot.unwrap_or(Value::Void)
        } else if let Some(rest) = encoded.strip_prefix("option:") {
            if rest == "__return_slot__" {
                Value::Option(return_slot.map(Box::new))
            } else {
                Value::Option(Some(Box::new(Self::decode_return_value(rest, return_slot))))
            }
        } else if let Some(rest) = encoded.strip_prefix("enum:") {
            // format: EnumName.Variant or EnumName.Variant(data1,data2,...)
            if let Some((before_paren, data_str)) = rest.split_once('(') {
                // EnumVariantData
                if let Some((enum_name, variant)) = before_paren.split_once('.') {
                    let data = if let Some(inner) = data_str.strip_suffix(')') {
                        if inner.is_empty() {
                            Vec::new()
                        } else {
                            inner
                                .split(',')
                                .map(|s| Self::decode_return_value(s, None))
                                .collect()
                        }
                    } else {
                        Vec::new()
                    };
                    Value::EnumVariantData {
                        enum_name: enum_name.to_string(),
                        variant: variant.to_string(),
                        data,
                    }
                } else {
                    Value::Void
                }
            } else if let Some((enum_name, variant)) = rest.split_once('.') {
                Value::EnumVariant {
                    enum_name: enum_name.to_string(),
                    variant: variant.to_string(),
                }
            } else {
                Value::Void
            }
        } else {
            Value::Void
        }
    }
}
