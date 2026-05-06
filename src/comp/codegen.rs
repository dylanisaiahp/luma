// src/comp/codegen.rs
use crate::ast::*;
use crate::interpreter::StructDef;
use crate::syntax::BinaryOp;

pub struct Codegen {
    output: String,
    indent: usize,
    var_counter: usize,
    current_return_type: String,
    current_file: String,
    struct_methods: Vec<(String, String)>,
    struct_defs: std::collections::HashMap<String, StructDef>,
    emitted_functions: std::collections::HashSet<String>,
    mutated_vars: std::collections::HashSet<String>,
    expected_type: Option<String>,
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
            current_file: String::new(),
            struct_methods: Vec::new(),
            struct_defs: std::collections::HashMap::new(),
            emitted_functions: std::collections::HashSet::new(),
            mutated_vars: std::collections::HashSet::new(),
            expected_type: None,
        }
    }

    pub fn with_file(mut self, file: &str) -> Self {
        self.current_file = file.to_string();
        self
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
            self.emit_line("fn luma_struct_dispatch(struct_name: &str, fields: std::collections::HashMap<String, Value>, method: &str, args: Vec<Value>, file: &str, line: u32, col: u32) -> Value {");
            self.indent += 1;
            self.emit_line("match (struct_name, method) {");
            self.indent += 1;
            for (struct_name, method_name) in &methods {
                self.emit_line(&format!(
                    "(\"{}\", \"{}\") => luma_struct_{}_{}(&fields, args, file, line, col),",
                    struct_name, method_name, struct_name, method_name
                ));
            }
            self.emit_line("_ => luma_runtime::runtime_error_with_location(&format!(\"{} has no method '{}'\", struct_name, method), file, line, col)");
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

    fn emit_line(&mut self, line: &str) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
        self.output.push_str(line);
        self.output.push('\n');
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

                // Collect mutated vars only from this function's body
                self.mutated_vars.clear();
                self.collect_mutated_vars(body);

                if name == "main" {
                    self.current_return_type = "void".to_string();
                    self.emit_line("fn main() {");
                } else {
                    let params_str = params
                        .iter()
                        .map(|p| {
                            if self.mutated_vars.contains(&p.name) {
                                format!("mut {}: Value", p.name)
                            } else {
                                format!("{}: Value", p.name)
                            }
                        })
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
                // Store struct def for field type lookup during codegen
                self.struct_defs.insert(
                    name.clone(),
                    StructDef {
                        fields: fields.clone(),
                        methods: methods.clone(),
                    },
                );

                for method in methods {
                    let ret = if method.return_type == "void" {
                        String::new()
                    } else {
                        " -> Value".to_string()
                    };
                    self.emit_line(&format!(
                        "fn luma_struct_{}_{}(fields: &std::collections::HashMap<String, Value>, args: Vec<Value>, _file: &str, _line: u32, _col: u32){} {{",
                        name, method.name, ret
                    ));
                    self.indent += 1;
                    for field in fields {
                        self.emit_line(&format!(
                            "let {} = fields.get(\"{}\").cloned().unwrap_or_else(|| luma_runtime::runtime_error_with_location(&format!(\"field '{{}}' not found on {{}}\", \"{}\", \"{}\"), _file, _line, _col));",
                            field.name, field.name, field.name, name
                        ));
                    }
                    for (i, param) in method.params.iter().enumerate() {
                        self.emit_line(&format!(
                            "let {} = args.get({}).cloned().unwrap_or_else(|| luma_runtime::runtime_error_with_location(&format!(\"missing argument '{{}}' on {{}}\", \"{}\", \"{}\"), _file, _line, _col));",
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
                type_name,
                ..
            } => {
                self.expected_type = Some(type_name.clone());
                let expr = self.emit_expr(value);
                self.expected_type = None;
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
                self.emit_line(&format!("let {} = if let Value::Integer(n) = {} {{ n }} else {{ luma_runtime::runtime_error_with_location(\"for range requires integers\", \"{}\", 0, 0) }};", tmp_s, s, self.current_file));
                self.emit_line(&format!("let {} = if let Value::Integer(n) = {} {{ n }} else {{ luma_runtime::runtime_error_with_location(\"for range requires integers\", \"{}\", 0, 0) }};", tmp_e, e, self.current_file));
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
                self.emit_line(&format!("let {} = if let Value::List(items) = {} {{ items }} else {{ luma_runtime::runtime_error_with_location(\"for..in requires a list\", \"{}\", 0, 0) }};", tmp, iter, self.current_file));
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
                self.emit_line(&format!("let {} = if let Value::Table(pairs) = {} {{ pairs }} else {{ luma_runtime::runtime_error_with_location(\"for..in requires a table\", \"{}\", 0, 0) }};", tmp, iter, self.current_file));
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
                    self.emit_line(&format!("luma_runtime::runtime_error_with_location(\"match must be exhaustive\", \"{}\", 0, 0);", self.current_file));
                    self.indent -= 1;
                    self.emit_line("}");
                }
                self.indent -= 1;
                self.emit_line("}");
                if self.current_return_type != "void" {
                    self.emit_line(&result.to_string());
                }
            }

            Stmt::Raise {
                message,
                line,
                column,
            } => {
                let msg = self.emit_expr(message);
                self.emit_line(&format!(
                    "luma_runtime::runtime_error_with_location(&format!(\"{{}}\", {}), \"{}\", {}, {});",
                    msg, self.current_file, line, column
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
            ExprKind::Empty => match self.expected_type.as_deref() {
                Some(t) if t.starts_with("table") => "Value::Table(vec![])".to_string(),
                Some(t) if t.starts_with("list") => "Value::List(vec![])".to_string(),
                _ => "Value::List(vec![])".to_string(),
            },
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
                    "{{ let _not_val = {}; if let Value::Boolean(b) = _not_val {{ Value::Boolean(!b) }} else {{ luma_runtime::runtime_error_with_location(\"'not' requires a boolean\", \"{}\", 0, 0) }} }}",
                    e, self.current_file
                )
            }

            ExprKind::BinaryOp { left, op, right } => match op {
                BinaryOp::Add => format!(
                    "luma_add({}, {})",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::Subtract => format!(
                    "luma_subtract({}, {})",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::Multiply => format!(
                    "luma_multiply({}, {})",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::Divide => format!(
                    "luma_divide({}, {})",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::Modulo => format!(
                    "luma_modulo({}, {})",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::Equal => format!(
                    "luma_compare(&{}, &{}, \"==\")",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::NotEqual => format!(
                    "luma_compare(&{}, &{}, \"!=\")",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::Greater => format!(
                    "luma_compare(&{}, &{}, \">\")",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::Less => format!(
                    "luma_compare(&{}, &{}, \"<\")",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::GreaterEqual => format!(
                    "luma_compare(&{}, &{}, \">=\")",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::LessEqual => format!(
                    "luma_compare(&{}, &{}, \"<=\")",
                    self.emit_expr(left),
                    self.emit_expr(right)
                ),
                BinaryOp::And => format!(
                    "{{ if let (Value::Boolean(l), Value::Boolean(r)) = ({}, {}) {{ Value::Boolean(l && r) }} else {{ luma_runtime::runtime_error_with_location(\"'and' requires booleans\", \"{}\", 0, 0) }} }}",
                    self.emit_expr(left),
                    self.emit_expr(right),
                    self.current_file
                ),
                BinaryOp::Or => format!(
                    "{{ if let (Value::Boolean(l), Value::Boolean(r)) = ({}, {}) {{ Value::Boolean(l || r) }} else {{ luma_runtime::runtime_error_with_location(\"'or' requires booleans\", \"{}\", 0, 0) }} }}",
                    self.emit_expr(left),
                    self.emit_expr(right),
                    self.current_file
                ),
            },

            ExprKind::Assign { name, value } => {
                self.mutated_vars.insert(name.clone());
                format!(
                    "{{ {} = {}; {}.clone() }}",
                    name,
                    self.emit_expr(value),
                    name
                )
            }

            ExprKind::AssignOp { name, op, value } => {
                let op_fn = match op {
                    crate::ast::AssignOpKind::Add => "luma_add",
                    crate::ast::AssignOpKind::Subtract => "luma_subtract",
                    crate::ast::AssignOpKind::Multiply => "luma_multiply",
                    crate::ast::AssignOpKind::Divide => "luma_divide",
                };
                self.mutated_vars.insert(name.clone());
                format!(
                    "{{ {} = {}({}.clone(), {}); {}.clone() }}",
                    name,
                    op_fn,
                    name,
                    self.emit_expr(value),
                    name
                )
            }

            ExprKind::Call { name, args } => {
                match name.as_str() {
                    "print" => {
                        // handled as Stmt::Print but may appear as expr
                        format!(
                            "{{ luma_print(&{}); Value::Void }}",
                            args.iter()
                                .map(|a| self.emit_expr(a))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                    "write" => {
                        format!(
                            "{{ luma_write(&{}); Value::Void }}",
                            args.iter()
                                .map(|a| self.emit_expr(a))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                    "read" => "luma_read()".to_string(),
                    "read_n" => format!("luma_read_n(&{})", self.emit_expr(&args[0])),
                    "random" => {
                        format!(
                            "luma_random(&{}, &{})",
                            self.emit_expr(&args[0]),
                            self.emit_expr(&args[1])
                        )
                    }
                    "int" => format!("luma_int(&{})", self.emit_expr(&args[0])),
                    "float" => format!("luma_float(&{})", self.emit_expr(&args[0])),
                    "string" => format!("luma_string(&{})", self.emit_expr(&args[0])),
                    "args" => "luma_args()".to_string(),
                    "fetch" => {
                        if args.is_empty() {
                            format!(
                                "luma_runtime::runtime_error_with_location(\"fetch() requires exactly one argument\", \"{}\", 0, 0)",
                                self.current_file
                            )
                        } else {
                            format!("Value::String({})", self.emit_expr(&args[0]))
                        }
                    }
                    "file" => {
                        if args.is_empty() {
                            format!(
                                "luma_runtime::runtime_error_with_location(\"file() requires exactly one argument\", \"{}\", 0, 0)",
                                self.current_file
                            )
                        } else {
                            self.emit_expr(&args[0])
                        }
                    }
                    "json" => {
                        if args.is_empty() {
                            format!(
                                "luma_runtime::runtime_error_with_location(\"json() requires exactly one argument\", \"{}\", 0, 0)",
                                self.current_file
                            )
                        } else {
                            format!("luma_json(&{})", self.emit_expr(&args[0]))
                        }
                    }
                    "toml" => {
                        if args.is_empty() {
                            format!(
                                "luma_runtime::runtime_error_with_location(\"toml() requires exactly one argument\", \"{}\", 0, 0)",
                                self.current_file
                            )
                        } else {
                            format!("luma_toml(&{})", self.emit_expr(&args[0]))
                        }
                    }
                    "run" => {
                        let parts = args
                            .iter()
                            .map(|a| format!("format!(\"{{}}\", {})", self.emit_expr(a)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("luma_run(&[{}])", parts)
                    }
                    "env" => format!("luma_env(&{})", self.emit_expr(&args[0])),
                    "home" => "luma_home()".to_string(),
                    "time" => "luma_time()".to_string(),
                    _ => {
                        // user-defined function
                        format!(
                            "luma_fn_{}({})",
                            name,
                            args.iter()
                                .map(|a| self.emit_expr(a))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
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
                            let path_expr = format!(
                                "luma_runtime::runtime_error_with_location(\"file() requires exactly one argument\", \"{}\", 0, 0)",
                                self.current_file
                            );
                            let path_str = format!(
                                "if let Value::String(ref _p) = {} {{ _p.clone() }} else {{ luma_runtime::runtime_error_with_location(\"file() requires a string path\", \"{}\", 0, 0) }}",
                                path_expr, self.current_file
                            );
                            let _ = path_str; // suppress warning
                            format!(
                                "{{ let _obj = {}; let _path = if let Value::String(ref p) = _obj {{ p.clone() }} else {{ luma_runtime::runtime_error_with_location(\"file() requires string\", \"{}\", 0, 0) }}; luma_file_method(&_path, \"{}\", {}) }}",
                                obj, self.current_file, method, args_slice
                            )
                        } else {
                            let path_expr = self.emit_expr(&args[0]);
                            let path_str = format!(
                                "if let Value::String(ref _p) = {} {{ _p.clone() }} else {{ luma_runtime::runtime_error_with_location(\"file() requires a string path\", \"{}\", 0, 0) }}",
                                path_expr, self.current_file
                            );
                            let _ = path_str; // suppress warning
                            format!(
                                "{{ let _obj = {}; let _path = if let Value::String(ref p) = _obj {{ p.clone() }} else {{ luma_runtime::runtime_error_with_location(\"file() requires string\", \"{}\", 0, 0) }}; luma_file_method(&_path, \"{}\", {}) }}",
                                obj, self.current_file, method, args_slice
                            )
                        }
                    }
                    ExprKind::Call { name, .. } if name == "fetch" => {
                        if args.is_empty() {
                            format!(
                                "{{ let _obj = {}; let _url = luma_runtime::runtime_error_with_location(\"fetch() requires exactly one argument\", \"{}\", 0, 0); luma_fetch_method(&_url, \"{}\", {}) }}",
                                obj, self.current_file, method, args_slice
                            )
                        } else {
                            format!(
                                "{{ let _obj = {}; let _url = if let Value::String(ref u) = _obj {{ u.clone() }} else {{ luma_runtime::runtime_error_with_location(\"fetch() requires string\", \"{}\", 0, 0) }}; luma_fetch_method(&_url, \"{}\", {}) }}",
                                obj, self.current_file, method, args_slice
                            )
                        }
                    }
                    ExprKind::Call { name, .. } if name == "input" => {
                        format!("luma_input_method(\"{}\")", method)
                    }
                    ExprKind::Call { name, .. } if name == "json" => {
                        format!(
                            "{{ let _obj = {}; let _s = if let Value::JsonHandle(ref s) = _obj {{ s.clone() }} else {{ luma_runtime::runtime_error_with_location(\"json() method called on non-json\", \"{}\", 0, 0) }}; luma_json_method(&_s, \"{}\", {}) }}",
                            obj, self.current_file, method, args_slice
                        )
                    }
                    ExprKind::Call { name, .. } if name == "toml" => {
                        format!(
                            "{{ let _obj = {}; let _s = if let Value::TomlHandle(ref s) = _obj {{ s.clone() }} else {{ luma_runtime::runtime_error_with_location(\"toml() method called on non-toml\", \"{}\", 0, 0) }}; luma_toml_method(&_s, \"{}\", {}) }}",
                            obj, self.current_file, method, args_slice
                        )
                    }
                    _ => {
                        let args_vec = if arg_exprs.is_empty() {
                            "vec![]".to_string()
                        } else {
                            format!("vec![{}]", arg_exprs.join(", "))
                        };
                        if self.struct_methods.is_empty() {
                            format!(
                                "luma_method_with_location({}, \"{}\", {}, \"{}\", {}, {})",
                                obj,
                                method,
                                args_vec,
                                self.current_file,
                                object.line,
                                object.column
                            )
                        } else {
                            format!(
                                "{{ let _obj = {}; let _method = \"{}\"; let _args = {}; if let Value::Struct {{ ref name, ref fields }} = _obj {{ luma_struct_dispatch(name, fields.clone(), _method, _args, \"{}\", {}, {}) }} else {{ luma_method_with_location(_obj, _method, _args, \"{}\", {}, {}) }}",
                                obj,
                                method,
                                args_vec,
                                self.current_file,
                                object.line,
                                object.column,
                                self.current_file,
                                object.line,
                                object.column
                            )
                        }
                    }
                }
            }

            ExprKind::FieldAccess { object, field } => {
                let obj = self.emit_expr(object);
                format!(
                    "{{ let _obj = {}; if let Value::Struct {{ name: ref _struct_name, fields }} = _obj {{ fields.get(\"{}\").cloned().unwrap_or_else(|| luma_runtime::runtime_error_with_location(&format!(\"field '{{}}' not found on struct '{{}}'\", \"{}\", _struct_name), \"{}\", {}, {})) }} else {{ luma_runtime::runtime_error_with_location(\"field access on non-struct\", \"{}\", {}, {}) }} }}",
                    obj,
                    field,
                    field,
                    self.current_file,
                    object.line,
                    object.column,
                    self.current_file,
                    object.line,
                    object.column
                )
            }

            ExprKind::StructInstantiate { name, fields } => {
                // Optimization: Check if this is a struct update (most fields from existing var)
                // For now, generate efficient struct construction
                let mut field_inserts = String::new();
                for (fname, fexpr) in fields {
                    // Look up field type from struct def to set expected_type for empty coercion
                    let field_type = self
                        .struct_defs
                        .get(name)
                        .and_then(|def| def.fields.iter().find(|f| &f.name == fname))
                        .map(|f| f.type_name.clone());
                    self.expected_type = field_type;
                    let v = self.emit_expr(fexpr);
                    self.expected_type = None;
                    field_inserts.push_str(&format!(
                        "_fields.insert(\"{}\".to_string(), {});\n",
                        fname, v
                    ));
                }
                // Use efficient pattern: avoid full reconstruction if possible
                format!(
                    "{{ let mut _fields = std::collections::HashMap::with_capacity({}); {} Value::Struct {{ name: \"{}\".to_string(), fields: _fields }} }}",
                    fields.len(),
                    field_inserts,
                    name
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
                    "luma_runtime::runtime_error_with_location(&format!(\"unknown constant {}.{}\", \"{}\", \"{}\"), \"{}\", 0, 0)",
                    type_name, constant, type_name, constant, self.current_file
                ),
            },
        }
    }
}
