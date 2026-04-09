// src/comp/codegen.rs
use crate::ast::*;
use crate::syntax::BinaryOp;

pub struct Codegen {
    output: String,
    indent: usize,
    var_counter: usize,
    current_return_type: String,
    struct_methods: Vec<(String, String)>,
    emitted_functions: std::collections::HashSet<String>,
    mutated_vars: std::collections::HashSet<String>,
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
            current_return_type: "void".to_string(),
            struct_methods: Vec::new(),
            emitted_functions: std::collections::HashSet::new(),
            mutated_vars: std::collections::HashSet::new(),
        }
    }

    fn collect_mutated_vars(&mut self, stmts: &[Stmt]) {
        for stmt in stmts {
            self.collect_from_stmt(stmt);
        }
    }

    fn collect_from_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Expression(expr) => {
                self.collect_from_expr(expr);
            }
            Stmt::Function { body, .. } => {
                for s in body {
                    self.collect_from_stmt(s);
                }
            }
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                for s in then_branch {
                    self.collect_from_stmt(s);
                }
                if let Some(else_stmts) = else_branch {
                    for s in else_stmts {
                        self.collect_from_stmt(s);
                    }
                }
            }
            Stmt::While { body, .. } => {
                for s in body {
                    self.collect_from_stmt(s);
                }
            }
            Stmt::For { body, .. } => {
                for s in body {
                    self.collect_from_stmt(s);
                }
            }
            Stmt::ForIn { body, .. } => {
                for s in body {
                    self.collect_from_stmt(s);
                }
            }
            Stmt::ForInTable { body, .. } => {
                for s in body {
                    self.collect_from_stmt(s);
                }
            }
            Stmt::Match { arms, else_arm, .. } => {
                for arm in arms {
                    for s in &arm.body {
                        self.collect_from_stmt(s);
                    }
                }
                if let Some(else_stmts) = else_arm {
                    for s in else_stmts {
                        self.collect_from_stmt(s);
                    }
                }
            }
            Stmt::Program(stmts) => {
                for s in stmts {
                    self.collect_from_stmt(s);
                }
            }
            _ => {}
        }
    }

    fn collect_from_expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Assign { name, .. } => {
                self.mutated_vars.insert(name.clone());
            }
            ExprKind::AssignOp { name, .. } => {
                self.mutated_vars.insert(name.clone());
            }
            ExprKind::BinaryOp { left, right, .. } => {
                self.collect_from_expr(left);
                self.collect_from_expr(right);
            }
            ExprKind::Not(operand) => {
                self.collect_from_expr(operand);
            }
            ExprKind::Some(inner) => {
                self.collect_from_expr(inner);
            }
            ExprKind::Call { args, .. } => {
                for arg in args {
                    self.collect_from_expr(arg);
                }
            }
            ExprKind::MethodCall { object, args, .. } => {
                self.collect_from_expr(object);
                for arg in args {
                    self.collect_from_expr(arg);
                }
            }
            ExprKind::FieldAccess { object, .. } => {
                self.collect_from_expr(object);
            }
            ExprKind::StructInstantiate { fields, .. } => {
                for (_, val) in fields {
                    self.collect_from_expr(val);
                }
            }
            ExprKind::EnumVariantData { data, .. } => {
                for d in data {
                    self.collect_from_expr(d);
                }
            }
            ExprKind::List(items) => {
                for item in items {
                    self.collect_from_expr(item);
                }
            }
            ExprKind::Table(pairs) => {
                for (key, val) in pairs {
                    self.collect_from_expr(key);
                    self.collect_from_expr(val);
                }
            }
            _ => {}
        }
    }

    pub fn generate(mut self, stmts: &[Stmt]) -> String {
        // File header
        self.emit_line("#![allow(unused_mut, unused_variables, unused_must_use)]");
        self.emit_line("mod luma_runtime;");
        self.emit_line("use luma_runtime::*;");
        self.emit_line("");

        // Collect variables that are mutated (reassigned) for making them mutable
        self.collect_mutated_vars(stmts);

        // Collect struct methods for dispatch
        for stmt in stmts {
            if let Stmt::StructDeclaration { name, methods, .. } = stmt {
                for method in methods {
                    self.struct_methods
                        .push((name.clone(), method.name.clone()));
                }
            }
        }

        // Collect and emit all function/struct declarations first
        for stmt in stmts {
            match stmt {
                Stmt::StructDeclaration { .. } => self.emit_stmt(stmt),
                Stmt::Function { name, .. } if name != "main" => self.emit_stmt(stmt),
                _ => {}
            }
        }

        // Emit struct dispatch function
        if !self.struct_methods.is_empty() {
            let methods = self.struct_methods.clone();
            self.emit_line("fn luma_struct_dispatch(struct_name: &str, fields: std::collections::HashMap<String, Value>, method: &str, args: Vec<Value>) -> Value {");
            self.indent += 1;
            self.emit_line("match (struct_name, method) {");
            self.indent += 1;
            for (struct_name, method_name) in &methods {
                self.emit_line(&format!(
                    "(\"{}\", \"{}\") => luma_struct_{}_{}(&fields, args),",
                    struct_name, method_name, struct_name, method_name
                ));
            }
            self.emit_line("_ => luma_runtime::runtime_error(&format!(\"{} has no method '{}'\", struct_name, method))");
            self.indent -= 1;
            self.emit_line("}");
            self.indent -= 1;
            self.emit_line("}");
            self.emit_line("");
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
                // Skip if we've already emitted this function (deduplication)
                if !self.emitted_functions.insert(name.clone()) {
                    return;
                }

                if name == "main" {
                    self.current_return_type = "void".to_string();
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
                    self.current_return_type = return_type.clone();
                    self.emit_line(&format!("fn luma_fn_{}({}){} {{", name, params_str, ret));
                }
                self.indent += 1;
                for s in body {
                    self.emit_stmt(s);
                }
                self.indent -= 1;
                self.emit_line("}");
                self.emit_line("");
            }

            Stmt::StructDeclaration {
                name,
                fields,
                methods,
            } => {
                for method in methods {
                    let ret = if method.return_type == "void" {
                        String::new()
                    } else {
                        " -> Value".to_string()
                    };
                    self.emit_line(&format!(
                        "fn luma_struct_{}_{}(fields: &std::collections::HashMap<String, Value>, args: Vec<Value>){} {{",
                        name, method.name, ret
                    ));
                    self.indent += 1;
                    for field in fields {
                        self.emit_line(&format!(
                            "let {} = fields.get(\"{}\").cloned().unwrap_or_else(|| luma_runtime::runtime_error(\"field '{}' not found on {}\"));",
                            field.name, field.name, field.name, name
                        ));
                    }
                    for (i, param) in method.params.iter().enumerate() {
                        self.emit_line(&format!(
                            "let {} = args.get({}).cloned().unwrap_or_else(|| luma_runtime::runtime_error(\"missing argument '{}' on {}\"));",
                            param.name, i, param.name, method.name
                        ));
                    }
                    let prev_return = self.current_return_type.clone();
                    self.current_return_type = method.return_type.clone();
                    for s in &method.body {
                        self.emit_stmt(s);
                    }
                    self.current_return_type = prev_return;
                    self.indent -= 1;
                    self.emit_line("}");
                    self.emit_line("");
                }
            }

            Stmt::EnumDeclaration { .. } => {
                // enums handled at load time, not emitted
            }

            Stmt::VariableDeclaration {
                name,
                value,
                mutable,
                ..
            } => {
                let expr = self.emit_expr(value);
                let is_mut = *mutable || self.mutated_vars.contains(name);
                let decl = if is_mut {
                    format!("let mut {} = {};", name, expr)
                } else {
                    format!("let {} = {};", name, expr)
                };
                self.emit_line(&decl);
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
                // Strip outer braces for single-line block expressions to avoid Rust warning
                let e = if !e.contains('\n') && e.starts_with('{') && e.ends_with('}') {
                    let inner = &e[1..e.len() - 1];
                    // Only strip if it's a simple pattern (if-let, etc.)
                    if inner.trim_start().starts_with("if ") || inner.trim_start().starts_with('{')
                    {
                        inner.to_string()
                    } else {
                        e
                    }
                } else {
                    e
                };
                if self.current_return_type == "void" {
                    self.emit_line(&format!("{};", e));
                    self.emit_line("return;");
                } else if self.current_return_type.starts_with("option(") {
                    let already_option = matches!(
                        &expr.kind,
                        ExprKind::Empty
                            | ExprKind::Some(_)
                            | ExprKind::None
                            | ExprKind::Identifier(_)
                    );
                    if already_option {
                        self.emit_line(&format!("return {};", e));
                    } else {
                        self.emit_line(&format!("return Value::Option(Some(Box::new({})));", e));
                    }
                } else {
                    self.emit_line(&format!("return {};", e));
                }
            }

            Stmt::Return(None) => {
                if self.current_return_type == "void" {
                    self.emit_line("return;");
                } else {
                    self.emit_line("return Value::Void;");
                }
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
                let result = self.fresh_var();
                self.emit_line(&format!("let {} = {};", tmp, val));
                self.emit_line(&format!("let mut {} = Value::Void;", result));
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
                } else {
                    self.emit_line("if !_luma_matched {");
                    self.indent += 1;
                    self.emit_line("luma_runtime::runtime_error(\"match must be exhaustive\");");
                    self.indent -= 1;
                    self.emit_line("}");
                }
                self.indent -= 1;
                self.emit_line("}");
                if self.current_return_type != "void" {
                    self.emit_line(&result.to_string());
                }
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
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t");
                format!(
                    "matches!({}, Value::String(ref _s) if _s == \"{}\")",
                    val_var, escaped
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
            MatchPattern::EnumVariant(enum_name, variant) => {
                format!(
                    "matches!({}, Value::EnumVariant {{ enum_name: ref _e, variant: ref _v }} if _e == \"{}\" && _v == \"{}\")",
                    val_var, enum_name, variant
                )
            }
        }
    }

    // --- Expressions ---

    fn emit_expr(&mut self, expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Integer(n) => format!("Value::Integer({})", n),
            ExprKind::Float(f) => format!("Value::Float({}f64)", f),
            ExprKind::Boolean(b) => format!("Value::Boolean({})", b),
            ExprKind::Empty => "Value::Option(None)".to_string(),
            ExprKind::Some(inner) => {
                let inner_code = self.emit_expr(inner);
                format!("Value::Option(Some(Box::new({})))", inner_code)
            }
            ExprKind::None => "Value::Option(None)".to_string(),

            ExprKind::String(s) => {
                let escaped = s
                    .replace('\\', "\\\\")
                    .replace('"', "\\\"")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
                    .replace('\t', "\\t")
                    .replace('\0', "\\0");
                format!("Value::String(\"{}\".to_string())", escaped)
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
                        if arg_exprs.is_empty() {
                            "luma_runtime::runtime_error(\"fetch() requires exactly one argument\")"
                                .to_string()
                        } else {
                            format!("Value::String({})", arg_exprs[0])
                        }
                    }
                    "file" => {
                        if arg_exprs.is_empty() {
                            "luma_runtime::runtime_error(\"file() requires exactly one argument\")"
                                .to_string()
                        } else {
                            arg_exprs[0].clone()
                        }
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
                let args_slice = if arg_exprs.is_empty() {
                    "&[]".to_string()
                } else {
                    format!("&[{}]", arg_exprs.join(", "))
                };

                // Special cases for fetch/file/input which need path/url
                match &object.kind {
                    ExprKind::Call { name, .. } if name == "file" => {
                        if args.is_empty() {
                            let path_expr = "luma_runtime::runtime_error(\"file() requires exactly one argument\")";
                            let path_str = format!(
                                "if let Value::String(ref _p) = {} {{ _p.clone() }} else {{ luma_runtime::runtime_error(\"file() requires a string path\") }}",
                                path_expr
                            );
                            let _ = path_str; // suppress warning
                            format!(
                                "{{ let _obj = {}; let _path = if let Value::String(ref p) = _obj {{ p.clone() }} else {{ luma_runtime::runtime_error(\"file() requires string\") }}; luma_file_method(&_path, \"{}\", {}) }}",
                                obj, method, args_slice
                            )
                        } else {
                            let path_expr = self.emit_expr(&args[0]);
                            let path_str = format!(
                                "if let Value::String(ref _p) = {} {{ _p.clone() }} else {{ luma_runtime::runtime_error(\"file() requires a string path\") }}",
                                path_expr
                            );
                            let _ = path_str; // suppress warning
                            format!(
                                "{{ let _obj = {}; let _path = if let Value::String(ref p) = _obj {{ p.clone() }} else {{ luma_runtime::runtime_error(\"file() requires string\") }}; luma_file_method(&_path, \"{}\", {}) }}",
                                obj, method, args_slice
                            )
                        }
                    }
                    ExprKind::Call { name, .. } if name == "fetch" => {
                        if args.is_empty() {
                            format!(
                                "{{ let _obj = {}; let _url = luma_runtime::runtime_error(\"fetch() requires exactly one argument\"); luma_fetch_method(&_url, \"{}\", {}) }}",
                                obj, method, args_slice
                            )
                        } else {
                            format!(
                                "{{ let _obj = {}; let _url = if let Value::String(ref u) = _obj {{ u.clone() }} else {{ luma_runtime::runtime_error(\"fetch() requires string\") }}; luma_fetch_method(&_url, \"{}\", {}) }}",
                                obj, method, args_slice
                            )
                        }
                    }
                    ExprKind::Call { name, .. } if name == "input" => {
                        format!("luma_input_method(\"{}\")", method)
                    }
                    _ => {
                        let args_vec = if arg_exprs.is_empty() {
                            "vec![]".to_string()
                        } else {
                            format!("vec![{}]", arg_exprs.join(", "))
                        };
                        if self.struct_methods.is_empty() {
                            format!("luma_method({}, \"{}\", {})", obj, method, args_vec)
                        } else {
                            format!(
                                "{{ let _obj = {}; let _method = \"{}\"; let _args = {}; if let Value::Struct {{ ref name, ref fields }} = _obj {{ luma_struct_dispatch(name, fields.clone(), _method, _args) }} else {{ luma_method(_obj, _method, _args) }} }}",
                                obj, method, args_vec
                            )
                        }
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

            ExprKind::EnumVariant { enum_name, variant } => {
                format!(
                    "Value::EnumVariant {{ enum_name: \"{}\".to_string(), variant: \"{}\".to_string() }}",
                    enum_name, variant
                )
            }

            ExprKind::EnumVariantData {
                enum_name,
                variant,
                data,
            } => {
                let data_exprs: Vec<String> = data.iter().map(|d| self.emit_expr(d)).collect();
                format!(
                    "Value::EnumVariantData {{ enum_name: \"{}\".to_string(), variant: \"{}\".to_string(), data: vec![{}] }}",
                    enum_name,
                    variant,
                    data_exprs.join(", ")
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
