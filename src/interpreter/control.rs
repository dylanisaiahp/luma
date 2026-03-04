// src/interpreter/control.rs
use crate::ast::{MatchPattern, Stmt};
use crate::debug::DebugLevel;
use crate::interpreter::Interpreter;
use crate::interpreter::value::{RuntimeError, Value};

impl Interpreter {
    pub fn execute_call(
        &mut self,
        name: &str,
        args: &[crate::ast::Expr],
        line: usize,
        column: usize,
    ) -> Result<Value, RuntimeError> {
        let func = match self.functions.get(name) {
            Some(f) => f.clone(),
            None => {
                return Err(RuntimeError {
                    message: format!("Unknown function: {}", name),
                    line,
                    column,
                });
            }
        };

        if args.len() != func.params.len() {
            return Err(RuntimeError {
                message: format!(
                    "{}() expects {} argument(s), got {}",
                    name,
                    func.params.len(),
                    args.len()
                ),
                line,
                column,
            });
        }

        // Evaluate arguments before pushing scope
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expression(arg)?);
        }

        // Push a fresh scope for the function
        self.push_scope();

        // Bind parameters
        for (param, val) in func.params.iter().zip(arg_values) {
            self.declare_variable(&param.name, val);
        }

        // Execute body, catching return signals
        let mut return_value = Value::Void;
        for stmt in &func.body {
            match self.execute_statement(stmt) {
                Ok(_) => {}
                Err(e) if e.message.starts_with("__return__") => {
                    let encoded = e.message.strip_prefix("__return__").unwrap_or("");
                    return_value = Interpreter::decode_return_value(encoded);
                    break;
                }
                Err(e) => {
                    self.pop_scope();
                    return Err(e);
                }
            }
        }

        self.pop_scope();
        Ok(return_value)
    }

    pub fn execute_while(
        &mut self,
        condition: &crate::ast::Expr,
        body: &[Stmt],
    ) -> Result<Value, RuntimeError> {
        loop {
            let cond_val = self.evaluate_expression(condition)?;
            match cond_val {
                Value::Boolean(true) => {
                    self.push_scope();
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                    self.pop_scope();
                }
                Value::Boolean(false) => break,
                _ => {
                    return Err(RuntimeError {
                        message: format!("While condition must be a boolean, got {:?}", cond_val),
                        line: condition.line,
                        column: condition.column,
                    });
                }
            }
        }
        Ok(Value::Void)
    }

    pub fn execute_if(
        &mut self,
        condition: &crate::ast::Expr,
        then_branch: &[Stmt],
        else_branch: &Option<Vec<Stmt>>,
    ) -> Result<Value, RuntimeError> {
        let cond_val = self.evaluate_expression(condition)?;
        match cond_val {
            Value::Boolean(true) => {
                self.push_scope();
                for stmt in then_branch {
                    self.execute_statement(stmt)?;
                }
                self.pop_scope();
            }
            Value::Boolean(false) => {
                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    for stmt in else_branch {
                        self.execute_statement(stmt)?;
                    }
                    self.pop_scope();
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

    pub fn execute_match(
        &mut self,
        value: &crate::ast::Expr,
        arms: &[crate::ast::MatchArm],
        else_arm: &Option<Vec<Stmt>>,
    ) -> Result<Value, RuntimeError> {
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
                        self.push_scope();
                        for stmt in &arm.body {
                            self.execute_statement(stmt)?;
                        }
                        self.pop_scope();
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
                        self.push_scope();
                        for stmt in &arm.body {
                            self.execute_statement(stmt)?;
                        }
                        self.pop_scope();
                        matched = true;
                        break;
                    }
                }
                MatchPattern::Wildcard => {
                    self.push_scope();
                    for stmt in &arm.body {
                        self.execute_statement(stmt)?;
                    }
                    self.pop_scope();
                    matched = true;
                    break;
                }
            }
            if matched {
                break;
            }
        }

        if !matched && let Some(else_body) = else_arm {
            self.push_scope();
            for stmt in else_body {
                self.execute_statement(stmt)?;
            }
            self.pop_scope();
        }

        Ok(Value::Void)
    }
}
