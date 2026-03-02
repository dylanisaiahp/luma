// src/interpreter/mod.rs
use crate::ast::{MatchPattern, Stmt};
use crate::debug::DebugLevel;
use crate::error::diagnostic::{Diagnostic, Span};
use std::collections::{HashMap, HashSet};

pub mod builtins;
pub mod expressions;
pub mod operations;
pub mod value;

pub use value::{RuntimeError, Value};

// Store variable info with its span for better warnings
#[derive(Debug, Clone)]
pub struct VarInfo {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Default)]
pub struct Interpreter {
    pub variables: HashMap<String, Value>,
    pub var_info: HashMap<String, VarInfo>, // store declaration info
    pub used_variables: HashSet<String>,    // track which variables were actually used
    pub warnings: Vec<Diagnostic>,          // collect warnings
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

    fn check_unused_variables(&mut self, source: &str, filename: &str) {
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

        // After execution, check for unused variables
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
            Stmt::Print(expr) => {
                let val = self.evaluate_expression(expr)?;
                match val {
                    Value::String(s) => println!("{}", s),
                    Value::Integer(i) => println!("{}", i),
                    Value::Float(f) => println!("{}", f),
                    Value::Boolean(b) => println!("{}", b),
                    Value::Void => println!(),
                }
                Ok(Value::Void)
            }
            Stmt::VariableDeclaration {
                type_name,
                name,
                value,
            } => {
                let val = self.evaluate_expression(value)?;

                match (type_name.as_str(), &val) {
                    ("int", Value::Integer(_)) => (),
                    ("float", Value::Float(_)) => (),
                    ("bool", Value::Boolean(_)) => (),
                    ("string", Value::String(_)) => (),
                    (expected, actual) => {
                        return Err(RuntimeError {
                            message: format!(
                                "Type mismatch: expected {}, got {:?}",
                                expected, actual
                            ),
                            line: value.line,
                            column: value.column,
                        });
                    }
                }

                // Store variable info for later warnings
                let span = Span {
                    filename: String::new(), // Will be filled later
                    line: value.line,
                    column: value.column,
                    length: name.len(),
                };

                self.var_info.insert(
                    name.clone(),
                    VarInfo {
                        name: name.clone(),
                        span,
                    },
                );

                self.variables.insert(name.clone(), val);
                Ok(Value::Void)
            }
            Stmt::Expression(expr) => {
                self.evaluate_expression(expr)?;
                Ok(Value::Void)
            }
            Stmt::While { condition, body } => {
                loop {
                    let cond_val = self.evaluate_expression(condition)?;
                    match cond_val {
                        Value::Boolean(true) => {
                            for stmt in body {
                                self.execute_statement(stmt)?;
                            }
                        }
                        Value::Boolean(false) => break,
                        _ => {
                            return Err(RuntimeError {
                                message: format!(
                                    "While condition must be a boolean, got {:?}",
                                    cond_val
                                ),
                                line: condition.line,
                                column: condition.column,
                            });
                        }
                    }
                }
                Ok(Value::Void)
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.evaluate_expression(condition)?;

                match cond_val {
                    Value::Boolean(true) => {
                        for stmt in then_branch {
                            self.execute_statement(stmt)?;
                        }
                    }
                    Value::Boolean(false) => {
                        if let Some(else_branch) = else_branch {
                            for stmt in else_branch {
                                self.execute_statement(stmt)?;
                            }
                        }
                    }
                    _ => {
                        return Err(RuntimeError {
                            message: format!("If condition must be a boolean, got {:?}", cond_val),
                            line: condition.line,
                            column: condition.column,
                        });
                    }
                }
                Ok(Value::Void)
            }
            Stmt::Match {
                value,
                arms,
                else_arm,
            } => {
                let match_val = self.evaluate_expression(value)?;
                let mut matched = false;

                for arm in arms {
                    crate::debug!(
                        DebugLevel::Basic,
                        "Checking arm with pattern: {:?}",
                        arm.pattern
                    );
                    match &arm.pattern {
                        MatchPattern::Integer(n) => {
                            if let Value::Integer(m) = &match_val
                                && m == n
                            {
                                for stmt in &arm.body {
                                    self.execute_statement(stmt)?;
                                }
                                matched = true;
                                break;
                            }
                        }
                        MatchPattern::Range(start, end) => {
                            if let Value::Integer(m) = &match_val
                                && {
                                    crate::debug!(
                                        DebugLevel::Basic,
                                        "Comparing {} between {} and {}",
                                        m,
                                        start,
                                        end
                                    );
                                    m >= start && m <= end
                                }
                            {
                                for stmt in &arm.body {
                                    self.execute_statement(stmt)?;
                                }
                                matched = true;
                                break;
                            }
                        }
                        MatchPattern::Wildcard => {
                            for stmt in &arm.body {
                                self.execute_statement(stmt)?;
                            }
                            matched = true;
                            break;
                        }
                    }
                    if matched {
                        crate::debug!(DebugLevel::Basic, "Match found, breaking");
                        break;
                    }
                    crate::debug!(DebugLevel::Basic, "No match, continuing to next arm");
                }

                if !matched && let Some(else_body) = else_arm {
                    for stmt in else_body {
                        self.execute_statement(stmt)?;
                    }
                }

                Ok(Value::Void)
            }
        }
    }
}
