// src/ast/mod.rs

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: usize,
    pub column: usize,
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Integer(i64),
    Float(f64),
    String(String),
    Char(String), // single character: 'x'
    Word(String), // single word (no whitespace): 'hello'
    Boolean(bool),
    Identifier(String),
    Interpolation(String),
    Not(Box<Expr>),
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
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    TypeConstant {
        type_name: String,
        constant: String,
    },
    List(Vec<Expr>),
    Table(Vec<(Expr, Expr)>),
    Empty,
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
    Break,
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
    // for item in list
    ForIn {
        var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    // for (key, value) in table
    ForInTable {
        key_var: String,
        val_var: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    Match {
        value: Expr,
        arms: Vec<MatchArm>,
        else_arm: Option<Vec<Stmt>>,
    },
    Use {
        module: String,
    },
}
