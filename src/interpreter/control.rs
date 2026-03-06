// src/interpreter/control.rs
use crate::ast::{Expr, MatchPattern, Stmt};
use crate::interpreter::Interpreter;
use crate::interpreter::value::{RuntimeError, Value};

impl Interpreter {
    pub fn execute_for(
        &mut self,
        var: &str,
        start: &Expr,
        end: &Expr,
        body: &[Stmt],
    ) -> Result<Value, RuntimeError> {
        let start_val = self.evaluate_expression(start)?;
        let end_val = self.evaluate_expression(end)?;

        let (start_i, end_i) = match (start_val, end_val) {
            (Value::Integer(s), Value::Integer(e)) => (s, e),
            _ => {
                return Err(RuntimeError {
                    message: "for loop range must be integers".to_string(),
                    line: start.line,
                    column: start.column,
                });
            }
        };

        for i in start_i..end_i {
            self.push_scope();
            self.declare_variable(var, Value::Integer(i));
            let mut should_break = false;
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(_) => {}
                    Err(e) if e.message == "__break__" => {
                        should_break = true;
                        break;
                    }
                    Err(e) => {
                        self.pop_scope();
                        return Err(e);
                    }
                }
            }
            self.pop_scope();
            if should_break {
                break;
            }
        }

        Ok(Value::Void)
    }

    pub fn execute_for_in(
        &mut self,
        var: &str,
        iterable: &Expr,
        body: &[Stmt],
    ) -> Result<Value, RuntimeError> {
        let list_val = self.evaluate_expression(iterable)?;

        let items = match list_val {
            Value::List(items) => items,
            _ => {
                return Err(RuntimeError {
                    message: format!("for..in expects a list, got {}", list_val.type_name()),
                    line: iterable.line,
                    column: iterable.column,
                });
            }
        };

        for item in items {
            self.push_scope();
            self.declare_variable(var, item);
            let mut should_break = false;
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(_) => {}
                    Err(e) if e.message == "__break__" => {
                        should_break = true;
                        break;
                    }
                    Err(e) => {
                        self.pop_scope();
                        return Err(e);
                    }
                }
            }
            self.pop_scope();
            if should_break {
                break;
            }
        }

        Ok(Value::Void)
    }

    pub fn execute_for_in_table(
        &mut self,
        key_var: &str,
        val_var: &str,
        iterable: &Expr,
        body: &[Stmt],
    ) -> Result<Value, RuntimeError> {
        let table_val = self.evaluate_expression(iterable)?;

        let pairs = match table_val {
            Value::Table(pairs) => pairs,
            _ => {
                return Err(RuntimeError {
                    message: format!("for..in expects a table, got {}", table_val.type_name()),
                    line: iterable.line,
                    column: iterable.column,
                });
            }
        };

        for (k, v) in pairs {
            self.push_scope();
            self.declare_variable(key_var, k);
            self.declare_variable(val_var, v);
            let mut should_break = false;
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(_) => {}
                    Err(e) if e.message == "__break__" => {
                        should_break = true;
                        break;
                    }
                    Err(e) => {
                        self.pop_scope();
                        return Err(e);
                    }
                }
            }
            self.pop_scope();
            if should_break {
                break;
            }
        }

        Ok(Value::Void)
    }

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

        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expression(arg)?);
        }

        self.push_scope();

        for (param, val) in func.params.iter().zip(arg_values) {
            self.declare_variable(&param.name, val);
        }

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

        let args_str = args
            .iter()
            .map(|_| "…".to_string())
            .collect::<Vec<_>>()
            .join(", ");
        self.debug.log_call(name, &args_str, &return_value);

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
                    let mut should_break = false;
                    for stmt in body {
                        match self.execute_statement(stmt) {
                            Ok(_) => {}
                            Err(e) if e.message == "__break__" => {
                                should_break = true;
                                break;
                            }
                            Err(e) => {
                                self.pop_scope();
                                return Err(e);
                            }
                        }
                    }
                    self.pop_scope();
                    if should_break {
                        break;
                    }
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
                        && { m >= start && m <= end }
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
