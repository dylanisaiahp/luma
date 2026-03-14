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
            let mut return_err = None;
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(_) => {}
                    Err(e) if e.message == "__break__" => {
                        should_break = true;
                        break;
                    }
                    Err(e) => {
                        return_err = Some(e);
                        break;
                    }
                }
            }
            self.pop_scope();
            if let Some(e) = return_err {
                return Err(e);
            }
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
            let mut return_err = None;
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(_) => {}
                    Err(e) if e.message == "__break__" => {
                        should_break = true;
                        break;
                    }
                    Err(e) => {
                        return_err = Some(e);
                        break;
                    }
                }
            }
            self.pop_scope();
            if let Some(e) = return_err {
                return Err(e);
            }
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
            let mut return_err = None;
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(_) => {}
                    Err(e) if e.message == "__break__" => {
                        should_break = true;
                        break;
                    }
                    Err(e) => {
                        return_err = Some(e);
                        break;
                    }
                }
            }
            self.pop_scope();
            if let Some(e) = return_err {
                return Err(e);
            }
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

        // Save and restore return_slot so nested calls don't clobber each other.
        // e.g. fib(n-1) + fib(n-2): the second call must not wipe the first call's slot.
        let saved_return_slot = self.return_slot.take();

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
                    return_value = if encoded == "__return_slot__" {
                        self.return_slot.take().unwrap_or(Value::Void)
                    } else {
                        Interpreter::decode_return_value(encoded)
                    };
                    break;
                }
                Err(e) => {
                    self.pop_scope();
                    self.return_slot = saved_return_slot;
                    return Err(e);
                }
            }
        }

        self.pop_scope();

        // Restore the caller's return_slot now that this call is done
        if self.return_slot.is_none() {
            self.return_slot = saved_return_slot;
        }

        if func.return_type == "void" && return_value != Value::Void {
            return Err(RuntimeError {
                message: format!(
                    "void function '{}' must not return a value — did you mean to declare a return type?",
                    name
                ),
                line,
                column,
            });
        }

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
                    let mut return_err = None;
                    for stmt in body {
                        match self.execute_statement(stmt) {
                            Ok(_) => {}
                            Err(e) if e.message == "__break__" => {
                                should_break = true;
                                break;
                            }
                            Err(e) => {
                                return_err = Some(e);
                                break;
                            }
                        }
                    }
                    self.pop_scope();
                    if let Some(e) = return_err {
                        return Err(e);
                    }
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
                let mut result = Ok(Value::Void);
                for stmt in then_branch {
                    match self.execute_statement(stmt) {
                        Ok(_) => {}
                        Err(e) => {
                            result = Err(e);
                            break;
                        }
                    }
                }
                self.pop_scope();
                result
            }
            Value::Boolean(false) => {
                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    let mut result = Ok(Value::Void);
                    for stmt in else_branch {
                        match self.execute_statement(stmt) {
                            Ok(_) => {}
                            Err(e) => {
                                result = Err(e);
                                break;
                            }
                        }
                    }
                    self.pop_scope();
                    result
                } else {
                    Ok(Value::Void)
                }
            }
            _ => Err(RuntimeError {
                message: format!("If condition must be a boolean, got {:?}", cond_val),
                line: condition.line,
                column: condition.column,
            }),
        }
    }

    pub fn execute_match(
        &mut self,
        value: &crate::ast::Expr,
        arms: &[crate::ast::MatchArm],
        else_arm: &Option<Vec<Stmt>>,
    ) -> Result<Value, RuntimeError> {
        let match_val = self.evaluate_expression(value)?;

        // List match — iterate each element, fire every arm that matches.
        // Wildcard fires once only if nothing matched at all.
        if let Value::List(items) = &match_val {
            let items = items.clone();
            let mut any_matched = false;
            let mut unmatched: Vec<String> = Vec::new();

            for item in &items {
                let mut item_matched = false;
                for arm in arms {
                    if matches!(arm.pattern, MatchPattern::Wildcard) {
                        continue; // skip wildcard on per-item pass
                    }
                    if self.pattern_matches(&arm.pattern, item) {
                        self.push_scope();
                        for stmt in &arm.body {
                            self.execute_statement(stmt)?;
                        }
                        self.pop_scope();
                        item_matched = true;
                        any_matched = true;
                    }
                }
                if !item_matched {
                    unmatched.push(self.value_to_display_string(item));
                }
            }

            // Wildcard / else — fires once if nothing matched at all
            if !any_matched {
                if let Some(else_body) = else_arm {
                    self.push_scope();
                    for stmt in else_body {
                        self.execute_statement(stmt)?;
                    }
                    self.pop_scope();
                } else {
                    // Check for explicit wildcard arm
                    for arm in arms {
                        if matches!(arm.pattern, MatchPattern::Wildcard) {
                            self.push_scope();
                            for stmt in &arm.body {
                                self.execute_statement(stmt)?;
                            }
                            self.pop_scope();
                            break;
                        }
                    }
                }
            }

            // Always hint about unmatched items regardless of whether something else matched
            if !unmatched.is_empty() {
                eprintln!(
                    "[?] {} item(s) did not match any pattern: {}",
                    unmatched.len(),
                    unmatched
                        .iter()
                        .map(|s| format!("\"{}\"", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }

            return Ok(Value::Void);
        }

        // Scalar match — original behaviour, first match wins
        let mut matched = false;
        for arm in arms {
            let is_match = self.pattern_matches(&arm.pattern, &match_val);
            if is_match {
                self.push_scope();
                for stmt in &arm.body {
                    self.execute_statement(stmt)?;
                }
                self.pop_scope();
                matched = true;
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

    fn pattern_matches(&self, pattern: &MatchPattern, value: &Value) -> bool {
        match pattern {
            MatchPattern::Integer(n) => matches!(value, Value::Integer(m) if m == n),
            MatchPattern::Range(start, end) => {
                matches!(value, Value::Integer(m) if m >= start && m <= end)
            }
            MatchPattern::Wildcard => true,
            MatchPattern::String(s) => matches!(value, Value::String(m) if m == s),
            MatchPattern::Set(patterns) => patterns.iter().any(|p| self.pattern_matches(p, value)),
        }
    }
}
