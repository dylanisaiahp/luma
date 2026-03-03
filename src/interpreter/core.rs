// src/interpreter/core.rs
use crate::ast::Stmt;
use crate::debug::DebugLevel;
use crate::error::diagnostic::Diagnostic;
use crate::interpreter::VarInfo;
use crate::interpreter::value::{RuntimeError, Value};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Interpreter {
    pub variables: HashMap<String, Value>,
    pub var_info: HashMap<String, VarInfo>,
    pub used_variables: HashSet<String>,
    pub warnings: Vec<Diagnostic>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            var_info: HashMap::new(),
            used_variables: HashSet::new(),
            warnings: Vec::new(),
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

    pub fn interpret(
        &mut self,
        program: &Stmt,
        source: &str,
        filename: &str,
    ) -> Result<(), RuntimeError> {
        crate::debug!(DebugLevel::Basic, "Starting interpretation");
        self.execute_statement(program)?;
        self.check_unused_variables(source, filename);
        Ok(())
    }

    pub fn execute_statement(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        crate::debug!(DebugLevel::Verbose, "Executing {:?}", stmt);
        match stmt {
            Stmt::Program(statements) => {
                for stmt in statements {
                    self.execute_statement(stmt)?;
                }
                Ok(Value::Void)
            }
            Stmt::Function { name, body } => {
                if name == "main" {
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(Value::Void)
            }
            Stmt::Print(expr) => self.execute_print(expr),
            Stmt::VariableDeclaration {
                type_name,
                name,
                value,
            } => self.execute_variable_declaration(type_name, name, value),
            Stmt::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(Value::Void)
            }
            Stmt::While { condition, body } => self.execute_while(condition, body),
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
        }
    }
}
