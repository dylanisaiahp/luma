// src/comp/codegen.rs
use crate::ast::*;
use crate::syntax::BinaryOp;

pub struct Codegen {
    output: String,
    indent: usize,
    var_counter: usize,
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            var_counter: 0,
        }
    }

    pub fn generate(mut self, stmts: &[Stmt]) -> String {
        // File header
        self.emit_line("#![allow(unused_mut, unused_variables, unused_must_use)]");
        self.emit_line("mod luma_runtime;");
        self.emit_line("use luma_runtime::*;");
        self.emit_line("");

        // Collect and emit all function/struct declarations first
        for stmt in stmts {
            match stmt {
                Stmt::StructDeclaration { .. } => self.emit_stmt(stmt),
                Stmt::Function { name, .. } if name != "main" => self.emit_stmt(stmt),
                _ => {}
            }
        }

        // Emit main
        for stmt in stmts {
            if let Stmt::Function { name, .. } = stmt
                && name == "main"
            {
                self.emit_stmt(stmt);
            }
        }

        self.output
    }

    fn fresh_var(&mut self) -> String {
        self.var_counter += 1;
        format!("_luma_tmp_{}", self.var_counter)
    }

    fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }

    fn emit_line(&mut self, line: &str) {
        self.output
            .push_str(&format!("{}{}\n", self.indent_str(), line));
    }

    // --- Statements ---

    fn emit_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Function {
                name,
                params,
                body,
                return_type,
            } => {
                if name == "main" {
                    self.emit_line("fn main() {");
                } else {
                    let params_str = params
                        .iter()
                        .map(|p| format!("{}: Value", p.name))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let ret = if return_type == "void" {
                        String::new()
                    } else {
                        " -> Value".to_string()
                    };
                    self.emit_line(&format!("fn luma_fn_{}({}){} {{", name, params_str, ret));
                }
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                if return_type == "void" {
                    // no return needed
                }
                self.indent -= 1;
                self.emit_line("}");
                self.emit_line("");
            }

            Stmt::StructDeclaration { name, methods, .. } => {
                for method in methods {
                    let params_str = method
                        .params
                        .iter()
                        .map(|p| format!("{}: Value", p.name))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let ret = if method.return_type == "void" {
                        String::new()
                    } else {
                        " -> Value".to_string()
                    };
                    self.emit_line(&format!(
                        "fn luma_struct_{}_{}(fields: &std::collections::HashMap<String, Value>, {}){} {{",
                        name, method.name, params_str, ret
                    ));
                    self.indent += 1;
                    // inject fields as local vars
                    self.emit_line("// struct fields available as variables");
                    for s in &method.body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.emit_line("}");
                    self.emit_line("");
                }
            }

            Stmt::VariableDeclaration { name, value, .. } => {
                let expr = self.emit_expr(value);
                self.emit_line(&format!("let mut {} = {};", name, expr));
            }

            Stmt::Print(expr) => {
                let e = self.emit_expr(expr);
                self.emit_line(&format!("luma_print(&{});", e));
            }

            Stmt::Expression(expr) => {
                let e = self.emit_expr(expr);
                self.emit_line(&format!("{};", e));
            }

            Stmt::Return(Some(expr)) => {
                let e = self.emit_expr(expr);
                self.emit_line(&format!("return {};", e));
            }

            Stmt::Return(None) => {
                self.emit_line("return;");
            }

            Stmt::Break => {
                self.emit_line("break;");
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond = self.emit_expr(condition);
                self.emit_line(&format!("if let Value::Boolean(true) = {} {{", cond));
                self.indent += 1;
                for s in then_branch {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                if let Some(else_stmts) = else_branch {
                    self.emit_line("} else {");
                    self.indent += 1;
                    for s in else_stmts {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                }
                self.emit_line("}");
            }

            Stmt::While { condition, body } => {
                self.emit_line("loop {");
                self.indent += 1;
                let cond = self.emit_expr(condition);
                self.emit_line(&format!(
                    "if let Value::Boolean(false) = {} {{ break; }}",
                    cond
                ));
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::For {
                var,
                start,
                end,
                body,
            } => {
                let s = self.emit_expr(start);
                let e = self.emit_expr(end);
                let tmp_s = self.fresh_var();
                let tmp_e = self.fresh_var();
                self.emit_line(&format!("let {} = if let Value::Integer(n) = {} {{ n }} else {{ luma_runtime::runtime_error(\"for range requires integers\") }};", tmp_s, s));
                self.emit_line(&format!("let {} = if let Value::Integer(n) = {} {{ n }} else {{ luma_runtime::runtime_error(\"for range requires integers\") }};", tmp_e, e));
                self.emit_line(&format!("for _luma_i in {}..{} {{", tmp_s, tmp_e));
                self.indent += 1;
                self.emit_line(&format!("let mut {} = Value::Integer(_luma_i);", var));
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::ForIn {
                var,
                iterable,
                body,
            } => {
                let iter = self.emit_expr(iterable);
                let tmp = self.fresh_var();
                self.emit_line(&format!("let {} = if let Value::List(items) = {} {{ items }} else {{ luma_runtime::runtime_error(\"for..in requires a list\") }};", tmp, iter));
                self.emit_line(&format!("for _luma_item in {}.into_iter() {{", tmp));
                self.indent += 1;
                self.emit_line(&format!("let mut {} = _luma_item;", var));
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::ForInTable {
                key_var,
                val_var,
                iterable,
                body,
            } => {
                let iter = self.emit_expr(iterable);
                let tmp = self.fresh_var();
                self.emit_line(&format!("let {} = if let Value::Table(pairs) = {} {{ pairs }} else {{ luma_runtime::runtime_error(\"for..in requires a table\") }};", tmp, iter));
                self.emit_line(&format!("for (_luma_k, _luma_v) in {}.into_iter() {{", tmp));
                self.indent += 1;
                self.emit_line(&format!("let mut {} = _luma_k;", key_var));
                self.emit_line(&format!("let mut {} = _luma_v;", val_var));
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::Match {
                value,
                arms,
                else_arm,
            } => {
                let val = self.emit_expr(value);
                let tmp = self.fresh_var();
                self.emit_line(&format!("let {} = {};", tmp, val));
                self.emit_line("{");
                self.indent += 1;
                self.emit_line("let mut _luma_matched = false;");
                for arm in arms {
                    let cond = self.emit_match_pattern(&arm.pattern, &tmp);
                    self.emit_line(&format!("if {} {{", cond));
                    self.indent += 1;
                    self.emit_line("_luma_matched = true;");
                    for s in &arm.body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.emit_line("}");
                }
                if let Some(else_body) = else_arm {
                    self.emit_line("if !_luma_matched {");
                    self.indent += 1;
                    for s in else_body {
                        self.emit_stmt(s);
                    }
                    self.indent -= 1;
                    self.emit_line("}");
                }
                self.indent -= 1;
                self.emit_line("}");
            }

            Stmt::Raise { message, .. } => {
                let msg = self.emit_expr(message);
                self.emit_line(&format!(
                    "luma_runtime::runtime_error(&format!(\"{{}}\", {}));",
                    msg
                ));
            }

            Stmt::Program(stmts) => {
                for s in stmts {
                    self.emit_stmt(s);
                }
            }

            Stmt::Use { .. } | Stmt::ModuleDeclaration { .. } => {
                // handled at load time, not emitted
            }
        }
    }

    fn emit_match_pattern(&self, pattern: &MatchPattern, val_var: &str) -> String {
        match pattern {
            MatchPattern::Integer(n) => {
                format!("matches!({}, Value::Integer({}))", val_var, n)
            }
            MatchPattern::String(s) => {
                format!(
                    "matches!({}, Value::String(ref _s) if _s == \"{}\")",
                    val_var,
                    s.replace('"', "\\\"")
                )
            }
            MatchPattern::Range(start, end) => {
                format!(
                    "(if let Value::Integer(n) = {} {{ n >= {} && n <= {} }} else {{ false }})",
                    val_var, start, end
                )
            }
            MatchPattern::Set(patterns) => {
                let parts: Vec<String> = patterns
                    .iter()
                    .map(|p| self.emit_match_pattern(p, val_var))
                    .collect();
                format!("({})", parts.join(" || "))
            }
        }
    }

    // --- Expressions ---

    fn emit_expr(&mut self, expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Integer(n) => format!("Value::Integer({})", n),
            ExprKind::Float(f) => format!("Value::Float({}f64)", f),
            ExprKind::Boolean(b) => format!("Value::Boolean({})", b),
            ExprKind::Empty => "Value::Maybe(None)".to_string(),

            ExprKind::String(s) => {
                format!(
                    "Value::String(\"{}\".to_string())",
                    s.replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\t', "\\t")
                )
            }

            ExprKind::Char(s) => {
                let c = s.chars().next().unwrap_or('\0');
                format!("Value::Char('{}')", c.escape_default())
            }

            ExprKind::Identifier(name) => format!("{}.clone()", name),

            ExprKind::Interpolation(ident) => {
                format!("Value::String(format!(\"{{}}\", {}))", ident)
            }

            ExprKind::Not(operand) => {
                let e = self.emit_expr(operand);
                let _tmp = self.fresh_var();
                // emit as inline block
                format!(
                    "{{ let _not_val = {}; if let Value::Boolean(b) = _not_val {{ Value::Boolean(!b) }} else {{ luma_runtime::runtime_error(\"'not' requires a boolean\") }} }}",
                    e
                )
            }

            ExprKind::BinaryOp { left, op, right } => {
                let l = self.emit_expr(left);
                let r = self.emit_expr(right);
                match op {
                    BinaryOp::Add => format!("luma_add({}, {})", l, r),
                    BinaryOp::Subtract => format!("luma_subtract({}, {})", l, r),
                    BinaryOp::Multiply => format!("luma_multiply({}, {})", l, r),
                    BinaryOp::Divide => format!("luma_divide({}, {})", l, r),
                    BinaryOp::Modulo => format!("luma_modulo({}, {})", l, r),
                    BinaryOp::Equal => format!("luma_compare(&{}, &{}, \"==\")", l, r),
                    BinaryOp::NotEqual => format!("luma_compare(&{}, &{}, \"!=\")", l, r),
                    BinaryOp::Greater => format!("luma_compare(&{}, &{}, \">\")", l, r),
                    BinaryOp::Less => format!("luma_compare(&{}, &{}, \"<\")", l, r),
                    BinaryOp::GreaterEqual => format!("luma_compare(&{}, &{}, \">=\")", l, r),
                    BinaryOp::LessEqual => format!("luma_compare(&{}, &{}, \"<=\")", l, r),
                    BinaryOp::And => format!(
                        "{{ if let (Value::Boolean(l), Value::Boolean(r)) = ({}, {}) {{ Value::Boolean(l && r) }} else {{ luma_runtime::runtime_error(\"'and' requires booleans\") }} }}",
                        l, r
                    ),
                    BinaryOp::Or => format!(
                        "{{ if let (Value::Boolean(l), Value::Boolean(r)) = ({}, {}) {{ Value::Boolean(l || r) }} else {{ luma_runtime::runtime_error(\"'or' requires booleans\") }} }}",
                        l, r
                    ),
                }
            }

            ExprKind::Assign { name, value } => {
                let v = self.emit_expr(value);
                format!("{{ {} = {}; {}.clone() }}", name, v, name)
            }

            ExprKind::AssignOp { name, op, value } => {
                let v = self.emit_expr(value);
                let op_fn = match op {
                    crate::ast::AssignOpKind::Add => "luma_add",
                    crate::ast::AssignOpKind::Subtract => "luma_subtract",
                    crate::ast::AssignOpKind::Multiply => "luma_multiply",
                    crate::ast::AssignOpKind::Divide => "luma_divide",
                };
                format!(
                    "{{ {} = {}({}.clone(), {}); {}.clone() }}",
                    name, op_fn, name, v, name
                )
            }

            ExprKind::Call { name, args } => {
                let arg_exprs: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                match name.as_str() {
                    "print" => {
                        // handled as Stmt::Print but may appear as expr
                        format!("{{ luma_print(&{}); Value::Void }}", arg_exprs.join(", "))
                    }
                    "write" => {
                        format!("{{ luma_write(&{}); Value::Void }}", arg_exprs.join(", "))
                    }
                    "read" => "luma_read()".to_string(),
                    "random" => {
                        format!("luma_random(&{}, &{})", arg_exprs[0], arg_exprs[1])
                    }
                    "int" => format!("luma_int(&{})", arg_exprs[0]),
                    "float" => format!("luma_float(&{})", arg_exprs[0]),
                    "string" => format!("luma_string(&{})", arg_exprs[0]),
                    "input" => "Value::Void".to_string(), // handle via method
                    "fetch" => {
                        format!("Value::String({})", arg_exprs[0])
                        // fetch handle stored as string url, method dispatch resolves it
                    }
                    "file" => {
                        // file handle stored as string path
                        arg_exprs[0].clone()
                    }
                    "run" => {
                        let parts = arg_exprs
                            .iter()
                            .map(|a| format!("format!(\"{{}}\", {})", a))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("luma_run(&[{}])", parts)
                    }
                    _ => {
                        // user-defined function
                        format!("luma_fn_{}({})", name, arg_exprs.join(", "))
                    }
                }
            }

            ExprKind::MethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.emit_expr(object);
                let arg_exprs: Vec<String> = args.iter().map(|a| self.emit_expr(a)).collect();
                let args_vec = if arg_exprs.is_empty() {
                    "vec![]".to_string()
                } else {
                    format!("vec![{}]", arg_exprs.join(", "))
                };

                // Special cases for fetch/file/input which need path/url
                match &object.kind {
                    ExprKind::Call { name, .. } if name == "file" => {
                        let path_expr = self.emit_expr(&args[0]);
                        let path_str = format!(
                            "if let Value::String(ref _p) = {} {{ _p.clone() }} else {{ luma_runtime::runtime_error(\"file() requires a string path\") }}",
                            path_expr
                        );
                        let _ = path_str; // suppress warning
                        format!(
                            "{{ let _obj = {}; let _path = if let Value::String(ref p) = _obj {{ p.clone() }} else {{ luma_runtime::runtime_error(\"file() requires string\") }}; luma_file_method(&_path, \"{}\", {}) }}",
                            obj, method, args_vec
                        )
                    }
                    ExprKind::Call { name, .. } if name == "fetch" => {
                        format!(
                            "{{ let _obj = {}; let _url = if let Value::String(ref u) = _obj {{ u.clone() }} else {{ luma_runtime::runtime_error(\"fetch() requires string\") }}; luma_fetch_method(&_url, \"{}\", {}) }}",
                            obj, method, args_vec
                        )
                    }
                    ExprKind::Call { name, .. } if name == "input" => {
                        format!("luma_input_method(\"{}\")", method)
                    }
                    _ => {
                        format!("luma_method({}, \"{}\", {})", obj, method, args_vec)
                    }
                }
            }

            ExprKind::FieldAccess { object, field } => {
                let obj = self.emit_expr(object);
                format!(
                    "{{ let _obj = {}; if let Value::Struct {{ ref fields, .. }} = _obj {{ fields.get(\"{}\").cloned().unwrap_or_else(|| luma_runtime::runtime_error(\"field '{}' not found\")) }} else {{ luma_runtime::runtime_error(\"field access on non-struct\") }} }}",
                    obj, field, field
                )
            }

            ExprKind::StructInstantiate { name, fields } => {
                let mut field_inserts = String::new();
                for (fname, fexpr) in fields {
                    let v = self.emit_expr(fexpr);
                    field_inserts.push_str(&format!(
                        "_fields.insert(\"{}\".to_string(), {});",
                        fname, v
                    ));
                }
                format!(
                    "{{ let mut _fields = std::collections::HashMap::new(); {} Value::Struct {{ name: \"{}\".to_string(), fields: _fields }} }}",
                    field_inserts, name
                )
            }

            ExprKind::List(items) => {
                let item_exprs: Vec<String> = items.iter().map(|i| self.emit_expr(i)).collect();
                format!("Value::List(vec![{}])", item_exprs.join(", "))
            }

            ExprKind::Table(pairs) => {
                let pair_exprs: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| format!("({}, {})", self.emit_expr(k), self.emit_expr(v)))
                    .collect();
                format!("Value::Table(vec![{}])", pair_exprs.join(", "))
            }

            ExprKind::TypeConstant {
                type_name,
                constant,
            } => match (type_name.as_str(), constant.as_str()) {
                ("int", "max") => "Value::Integer(i64::MAX)".to_string(),
                ("int", "min") => "Value::Integer(i64::MIN)".to_string(),
                ("float", "max") => "Value::Float(f64::MAX)".to_string(),
                ("float", "min") => "Value::Float(f64::MIN)".to_string(),
                _ => format!(
                    "luma_runtime::runtime_error(\"unknown constant {}.{}\")",
                    type_name, constant
                ),
            },
        }
    }
}
