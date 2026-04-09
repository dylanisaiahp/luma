// src/interpreter/operations.rs
use crate::interpreter::value::{RuntimeError, Value};
use crate::syntax::BinaryOp;

pub fn evaluate_binary_op(
    left_val: Value,
    right_val: Value,
    op: &BinaryOp,
    line: usize,
    column: usize,
) -> Result<Value, RuntimeError> {
    match (left_val, right_val, op) {
        // Integer arithmetic
        (Value::Integer(l), Value::Integer(r), BinaryOp::Add) => Ok(Value::Integer(l + r)),
        (Value::Integer(l), Value::Integer(r), BinaryOp::Subtract) => Ok(Value::Integer(l - r)),
        (Value::Integer(l), Value::Integer(r), BinaryOp::Multiply) => Ok(Value::Integer(l * r)),
        (Value::Integer(l), Value::Integer(r), BinaryOp::Divide) => {
            if r == 0 {
                return Err(RuntimeError {
                    message: "Division by zero".to_string(),
                    file_path: String::new(),
                    line,
                    column,
                });
            }
            Ok(Value::Integer(l / r))
        }
        (Value::Integer(l), Value::Integer(r), BinaryOp::Modulo) => {
            if r == 0 {
                return Err(RuntimeError {
                    message: "Modulo by zero".to_string(),
                    file_path: String::new(),
                    line,
                    column,
                });
            }
            Ok(Value::Integer(l % r))
        }
        (Value::Float(l), Value::Float(r), BinaryOp::Modulo) => Ok(Value::Float(l % r)),
        (Value::Integer(l), Value::Float(r), BinaryOp::Modulo) => Ok(Value::Float(l as f64 % r)),
        (Value::Float(l), Value::Integer(r), BinaryOp::Modulo) => Ok(Value::Float(l % r as f64)),

        // Float arithmetic
        (Value::Float(l), Value::Float(r), BinaryOp::Add) => Ok(Value::Float(l + r)),
        (Value::Float(l), Value::Float(r), BinaryOp::Subtract) => Ok(Value::Float(l - r)),
        (Value::Float(l), Value::Float(r), BinaryOp::Multiply) => Ok(Value::Float(l * r)),
        (Value::Float(l), Value::Float(r), BinaryOp::Divide) => {
            if r == 0.0 {
                return Err(RuntimeError {
                    message: "Division by zero".to_string(),
                    file_path: String::new(),
                    line,
                    column,
                });
            }
            Ok(Value::Float(l / r))
        }

        // Mixed arithmetic (int + float)
        (Value::Integer(l), Value::Float(r), BinaryOp::Add) => Ok(Value::Float(l as f64 + r)),
        (Value::Float(l), Value::Integer(r), BinaryOp::Add) => Ok(Value::Float(l + r as f64)),
        (Value::Integer(l), Value::Float(r), BinaryOp::Subtract) => Ok(Value::Float(l as f64 - r)),
        (Value::Float(l), Value::Integer(r), BinaryOp::Subtract) => Ok(Value::Float(l - r as f64)),
        (Value::Integer(l), Value::Float(r), BinaryOp::Multiply) => Ok(Value::Float(l as f64 * r)),
        (Value::Float(l), Value::Integer(r), BinaryOp::Multiply) => Ok(Value::Float(l * r as f64)),
        (Value::Integer(l), Value::Float(r), BinaryOp::Divide) => {
            if r == 0.0 {
                return Err(RuntimeError {
                    message: "Division by zero".to_string(),
                    file_path: String::new(),
                    line,
                    column,
                });
            }
            Ok(Value::Float(l as f64 / r))
        }
        (Value::Float(l), Value::Integer(r), BinaryOp::Divide) => {
            if r == 0 {
                return Err(RuntimeError {
                    message: "Division by zero".to_string(),
                    file_path: String::new(),
                    line,
                    column,
                });
            }
            Ok(Value::Float(l / r as f64))
        }

        // String concatenation
        (Value::String(l), Value::String(r), BinaryOp::Add) => Ok(Value::String(l + &r)),
        (Value::String(l), Value::Integer(r), BinaryOp::Add) => {
            Ok(Value::String(l + &r.to_string()))
        }
        (Value::Integer(l), Value::String(r), BinaryOp::Add) => {
            Ok(Value::String(l.to_string() + &r))
        }
        (Value::String(l), Value::Float(r), BinaryOp::Add) => Ok(Value::String(l + &r.to_string())),
        (Value::Float(l), Value::String(r), BinaryOp::Add) => Ok(Value::String(l.to_string() + &r)),

        // Integer comparisons
        (Value::Integer(l), Value::Integer(r), BinaryOp::Greater) => Ok(Value::Boolean(l > r)),
        (Value::Integer(l), Value::Integer(r), BinaryOp::Less) => Ok(Value::Boolean(l < r)),
        (Value::Integer(l), Value::Integer(r), BinaryOp::GreaterEqual) => {
            Ok(Value::Boolean(l >= r))
        }
        (Value::Integer(l), Value::Integer(r), BinaryOp::LessEqual) => Ok(Value::Boolean(l <= r)),
        (Value::Integer(l), Value::Integer(r), BinaryOp::Equal) => Ok(Value::Boolean(l == r)),
        (Value::Integer(l), Value::Integer(r), BinaryOp::NotEqual) => Ok(Value::Boolean(l != r)),

        // Float comparisons
        (Value::Float(l), Value::Float(r), BinaryOp::Greater) => Ok(Value::Boolean(l > r)),
        (Value::Float(l), Value::Float(r), BinaryOp::Less) => Ok(Value::Boolean(l < r)),
        (Value::Float(l), Value::Float(r), BinaryOp::GreaterEqual) => Ok(Value::Boolean(l >= r)),
        (Value::Float(l), Value::Float(r), BinaryOp::LessEqual) => Ok(Value::Boolean(l <= r)),
        (Value::Float(l), Value::Float(r), BinaryOp::Equal) => Ok(Value::Boolean(l == r)),
        (Value::Float(l), Value::Float(r), BinaryOp::NotEqual) => Ok(Value::Boolean(l != r)),

        // Mixed comparisons (int vs float)
        (Value::Integer(l), Value::Float(r), BinaryOp::Greater) => {
            Ok(Value::Boolean((l as f64) > r))
        }
        (Value::Float(l), Value::Integer(r), BinaryOp::Greater) => Ok(Value::Boolean(l > r as f64)),
        (Value::Integer(l), Value::Float(r), BinaryOp::Less) => Ok(Value::Boolean((l as f64) < r)),
        (Value::Float(l), Value::Integer(r), BinaryOp::Less) => Ok(Value::Boolean(l < r as f64)),
        (Value::Integer(l), Value::Float(r), BinaryOp::GreaterEqual) => {
            Ok(Value::Boolean((l as f64) >= r))
        }
        (Value::Float(l), Value::Integer(r), BinaryOp::GreaterEqual) => {
            Ok(Value::Boolean(l >= r as f64))
        }
        (Value::Integer(l), Value::Float(r), BinaryOp::LessEqual) => {
            Ok(Value::Boolean((l as f64) <= r))
        }
        (Value::Float(l), Value::Integer(r), BinaryOp::LessEqual) => {
            Ok(Value::Boolean(l <= r as f64))
        }
        (Value::Integer(l), Value::Float(r), BinaryOp::Equal) => {
            Ok(Value::Boolean((l as f64) == r))
        }
        (Value::Float(l), Value::Integer(r), BinaryOp::Equal) => Ok(Value::Boolean(l == r as f64)),
        (Value::Integer(l), Value::Float(r), BinaryOp::NotEqual) => {
            Ok(Value::Boolean((l as f64) != r))
        }
        (Value::Float(l), Value::Integer(r), BinaryOp::NotEqual) => {
            Ok(Value::Boolean(l != r as f64))
        }

        // Boolean comparisons
        (Value::Boolean(l), Value::Boolean(r), BinaryOp::Equal) => Ok(Value::Boolean(l == r)),
        (Value::Boolean(l), Value::Boolean(r), BinaryOp::NotEqual) => Ok(Value::Boolean(l != r)),

        // String comparisons
        (Value::String(l), Value::String(r), BinaryOp::Equal) => Ok(Value::Boolean(l == r)),
        (Value::String(l), Value::String(r), BinaryOp::NotEqual) => Ok(Value::Boolean(l != r)),
        (Value::String(l), Value::String(r), BinaryOp::Greater) => Ok(Value::Boolean(l > r)),
        (Value::String(l), Value::String(r), BinaryOp::Less) => Ok(Value::Boolean(l < r)),
        (Value::String(l), Value::String(r), BinaryOp::GreaterEqual) => Ok(Value::Boolean(l >= r)),
        (Value::String(l), Value::String(r), BinaryOp::LessEqual) => Ok(Value::Boolean(l <= r)),

        // Logical operators
        (Value::Boolean(l), Value::Boolean(r), BinaryOp::And) => Ok(Value::Boolean(l && r)),
        (Value::Boolean(l), Value::Boolean(r), BinaryOp::Or) => Ok(Value::Boolean(l || r)),

        // Enum comparisons
        (
            Value::EnumVariant {
                enum_name: l_name,
                variant: l_var,
            },
            Value::EnumVariant {
                enum_name: r_name,
                variant: r_var,
            },
            BinaryOp::Equal,
        ) => Ok(Value::Boolean(l_name == r_name && l_var == r_var)),
        (
            Value::EnumVariant {
                enum_name: l_name,
                variant: l_var,
            },
            Value::EnumVariant {
                enum_name: r_name,
                variant: r_var,
            },
            BinaryOp::NotEqual,
        ) => Ok(Value::Boolean(l_name != r_name || l_var != r_var)),
        (
            Value::EnumVariantData {
                enum_name: l_name,
                variant: l_var,
                data: l_data,
            },
            Value::EnumVariantData {
                enum_name: r_name,
                variant: r_var,
                data: r_data,
            },
            BinaryOp::Equal,
        ) => Ok(Value::Boolean(
            l_name == r_name && l_var == r_var && l_data == r_data,
        )),
        (
            Value::EnumVariantData {
                enum_name: l_name,
                variant: l_var,
                data: l_data,
            },
            Value::EnumVariantData {
                enum_name: r_name,
                variant: r_var,
                data: r_data,
            },
            BinaryOp::NotEqual,
        ) => Ok(Value::Boolean(
            l_name != r_name || l_var != r_var || l_data != r_data,
        )),

        _ => Err(RuntimeError {
            message: "Type mismatch in binary operation".to_string(),
            file_path: String::new(),
            line,
            column,
        }),
    }
}
