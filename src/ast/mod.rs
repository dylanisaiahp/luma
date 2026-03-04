// src/ast/mod.rs

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Identifier(String),
    Interpolation(String),
    BinaryOp {
        left: Box<Expr>,
        op: crate::syntax::BinaryOp,
        right: Box<Expr>,
    },
    Assign {
        name: String,
        value: Box<Expr>,
    },
    AssignOp {
        name: String,
        op: AssignOpKind,
        value: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AssignOpKind {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Integer(i64),
    Range(i64, i64),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub type_name: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Program(Vec<Stmt>),
    Function {
        name: String,
        body: Vec<Stmt>,
    },
    UserFunction {
        return_type: String,
        name: String,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    VariableDeclaration {
        type_name: String,
        name: String,
        value: Expr,
    },
    Print(Expr),
    Expression(Expr),
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        start: Expr,
        end: Expr,
        body: Vec<Stmt>,
    },
    Match {
        value: Expr,
        arms: Vec<MatchArm>,
        else_arm: Option<Vec<Stmt>>,
    },
}
