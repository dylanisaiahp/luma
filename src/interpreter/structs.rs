// src/interpreter/structs.rs
use crate::ast::StructMethod;
use crate::interpreter::Interpreter;
use crate::interpreter::value::{RuntimeError, Value};
use std::collections::HashMap;

impl Interpreter {
    /// Evaluate a StructInstantiate expression: Point(x: 1, y: 2)
    pub fn evaluate_struct_instantiate(
        &mut self,
        name: &str,
        field_exprs: &[(String, crate::ast::Expr)],
        line: usize,
        column: usize,
    ) -> Result<Value, RuntimeError> {
        let def = match self.struct_defs.get(name).cloned() {
            Some(d) => d,
            None => {
                return Err(RuntimeError {
                    message: format!("Unknown struct '{}'", name),
                    line,
                    column,
                });
            }
        };

        // Evaluate all provided field values
        let mut fields: HashMap<String, Value> = HashMap::new();
        for (fname, fexpr) in field_exprs {
            let val = self.evaluate_expression(fexpr)?;
            fields.insert(fname.clone(), val);
        }

        // Verify all declared fields are provided, no extras
        for sf in &def.fields {
            if !fields.contains_key(&sf.name) {
                return Err(RuntimeError {
                    message: format!("Missing field '{}' in '{}' instantiation", sf.name, name),
                    line,
                    column,
                });
            }
        }

        for fname in fields.keys() {
            let declared = def.fields.iter().any(|f| &f.name == fname);
            if !declared {
                return Err(RuntimeError {
                    message: format!("Unknown field '{}' in struct '{}'", fname, name),
                    line,
                    column,
                });
            }
        }

        Ok(Value::Struct {
            name: name.to_string(),
            fields,
        })
    }

    /// Evaluate a field access: p.x
    pub fn evaluate_field_access(
        &mut self,
        object_expr: &crate::ast::Expr,
        field: &str,
        line: usize,
        column: usize,
    ) -> Result<Value, RuntimeError> {
        let obj = self.evaluate_expression(object_expr)?;
        match obj {
            Value::Struct { ref fields, .. } => {
                fields.get(field).cloned().ok_or_else(|| RuntimeError {
                    message: format!("Struct has no field '{}'", field),
                    line,
                    column,
                })
            }
            _ => Err(RuntimeError {
                message: format!("Cannot access field '{}' on non-struct value", field),
                line,
                column,
            }),
        }
    }

    /// Call a struct method: p.sum()
    pub fn evaluate_struct_method_call(
        &mut self,
        object_expr: &crate::ast::Expr,
        method: &str,
        arg_exprs: &[crate::ast::Expr],
        line: usize,
        column: usize,
    ) -> Result<Value, RuntimeError> {
        let obj = self.evaluate_expression(object_expr)?;

        let (struct_name, fields) = match obj {
            Value::Struct {
                ref name,
                ref fields,
            } => (name.clone(), fields.clone()),
            _ => {
                return Err(RuntimeError {
                    message: format!("Cannot call method '{}' on non-struct value", method),
                    line,
                    column,
                });
            }
        };

        let def = match self.struct_defs.get(&struct_name).cloned() {
            Some(d) => d,
            None => {
                return Err(RuntimeError {
                    message: format!("Unknown struct '{}'", struct_name),
                    line,
                    column,
                });
            }
        };

        let method_def: StructMethod = match def.methods.iter().find(|m| m.name == method) {
            Some(m) => m.clone(),
            None => {
                return Err(RuntimeError {
                    message: format!("Struct '{}' has no method '{}'", struct_name, method),
                    line,
                    column,
                });
            }
        };

        // Evaluate args
        let mut arg_values = Vec::new();
        for a in arg_exprs {
            arg_values.push(self.evaluate_expression(a)?);
        }

        if arg_values.len() != method_def.params.len() {
            return Err(RuntimeError {
                message: format!(
                    "{}.{}() expects {} argument(s), got {}",
                    struct_name,
                    method,
                    method_def.params.len(),
                    arg_values.len()
                ),
                line,
                column,
            });
        }

        self.push_scope();

        // Inject struct fields as local variables so methods can reference them directly
        for (fname, fval) in &fields {
            self.declare_variable(fname, fval.clone());
        }

        // Bind explicit params
        for (param, val) in method_def.params.iter().zip(arg_values) {
            self.declare_variable(&param.name, val);
        }

        let mut return_value = Value::Void;
        for stmt in &method_def.body {
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
}
