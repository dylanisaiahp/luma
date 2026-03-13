// src/syntax/keywords.rs
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    Int,
    Float,
    String,
    Bool,
    Void,
    If,
    Else,
    While,
    For,
    In,
    Print,
    Match,
    Return,
    Read,
    True,
    False,
    Not,
    Break,
    Use,
    Maybe,
    Raise,
    Worry,
    Empty,
    List,
    Table,
    Char,
    Word,
    Struct,
    Module,
}

impl FromStr for Keyword {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "int" => Ok(Keyword::Int),
            "float" => Ok(Keyword::Float),
            "string" => Ok(Keyword::String),
            "bool" => Ok(Keyword::Bool),
            "void" => Ok(Keyword::Void),
            "if" => Ok(Keyword::If),
            "else" => Ok(Keyword::Else),
            "while" => Ok(Keyword::While),
            "for" => Ok(Keyword::For),
            "in" => Ok(Keyword::In),
            "print" => Ok(Keyword::Print),
            "match" => Ok(Keyword::Match),
            "return" => Ok(Keyword::Return),
            "read" => Ok(Keyword::Read),
            "true" => Ok(Keyword::True),
            "false" => Ok(Keyword::False),
            "not" => Ok(Keyword::Not),
            "break" => Ok(Keyword::Break),
            "use" => Ok(Keyword::Use),
            "maybe" => Ok(Keyword::Maybe),
            "raise" => Ok(Keyword::Raise),
            "worry" => Ok(Keyword::Worry),
            "empty" => Ok(Keyword::Empty),
            "list" => Ok(Keyword::List),
            "table" => Ok(Keyword::Table),
            "char" => Ok(Keyword::Char),
            "word" => Ok(Keyword::Word),
            "struct" => Ok(Keyword::Struct),
            "module" => Ok(Keyword::Module),
            _ => Err(()),
        }
    }
}
