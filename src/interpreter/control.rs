// src/interpreter/control.rs
use crate::ast::{MatchPattern, Stmt};
use crate::debug::DebugLevel;
use crate::interpreter::Interpreter;
use crate::interpreter::value::{RuntimeError, Value};

impl Interpreter {
    pub fn execute_while(
        &mut self,
        condition: &crate::ast::Expr,
        body: &[Stmt],
    ) -> Result<Value, RuntimeError> {
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
