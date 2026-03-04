// src/syntax/keywords.rs
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Keyword {
    // Types
    Int,
    Float,
    String,
    Bool,
    Void,
    // Statements
    If,
    Else,
    While,
    For,
    In,
    Print,
    Match,
    // Functions
    Return,
    Read,
    // Literals
    True,
    False,
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
            _ => Err(()),
        }
    }
}
