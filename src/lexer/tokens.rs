// src/lexer/tokens.rs
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
    pub byte_pos: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Void,
    Int,
    Float,
    Bool,
    String,
    Print,
    If,
    Else,
    While,
    Return,
    Read,
    Match,
    True,
    False,

    // Literals
    Identifier(String),
    Number(i64),
    FloatLiteral(f64),
    StringLiteral(String),

    // Special patterns
    Underscore,

    // Arithmetic operators
    Plus,
    Minus,
    Star,
    Slash,

    // Comparison operators
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    EqualEqual,
    BangEqual,

    // Assignment operators
    Equals,
    PlusEquals,
    MinusEquals,
    StarEquals,
    SlashEquals,

    // Range operator
    DotDot,

    // Symbols
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Comma,
    Colon,

    // Interpolation
    Interpolation(String),

    // Special
    Illegal(String),
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Void => write!(f, "void"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Float => write!(f, "float"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::String => write!(f, "string"),
            TokenKind::Print => write!(f, "print"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Read => write!(f, "read"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Identifier(s) => write!(f, "{}", s),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::FloatLiteral(n) => write!(f, "{}", n),
            TokenKind::StringLiteral(s) => write!(f, "\"{}\"", s),
            TokenKind::Underscore => write!(f, "_"),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Greater => write!(f, ">"),
            TokenKind::Less => write!(f, "<"),
            TokenKind::GreaterEqual => write!(f, ">="),
            TokenKind::LessEqual => write!(f, "<="),
            TokenKind::EqualEqual => write!(f, "=="),
            TokenKind::BangEqual => write!(f, "!="),
            TokenKind::Equals => write!(f, "="),
            TokenKind::PlusEquals => write!(f, "+="),
            TokenKind::MinusEquals => write!(f, "-="),
            TokenKind::StarEquals => write!(f, "*="),
            TokenKind::SlashEquals => write!(f, "/="),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Interpolation(s) => write!(f, "&{{{}}}", s),
            TokenKind::Illegal(s) => write!(f, "Illegal({})", s),
            TokenKind::Eof => write!(f, "EOF"),
        }
    }
}

impl From<crate::syntax::Keyword> for TokenKind {
    fn from(k: crate::syntax::Keyword) -> Self {
        match k {
            crate::syntax::Keyword::Int => TokenKind::Int,
            crate::syntax::Keyword::Float => TokenKind::Float,
            crate::syntax::Keyword::String => TokenKind::String,
            crate::syntax::Keyword::Bool => TokenKind::Bool,
            crate::syntax::Keyword::Void => TokenKind::Void,
            crate::syntax::Keyword::If => TokenKind::If,
            crate::syntax::Keyword::Else => TokenKind::Else,
            crate::syntax::Keyword::While => TokenKind::While,
            crate::syntax::Keyword::Print => TokenKind::Print,
            crate::syntax::Keyword::Match => TokenKind::Match,
            crate::syntax::Keyword::Return => TokenKind::Return,
            crate::syntax::Keyword::Read => TokenKind::Read,
            crate::syntax::Keyword::True => TokenKind::True,
            crate::syntax::Keyword::False => TokenKind::False,
        }
    }
}
