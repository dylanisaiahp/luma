// src/syntax/keywords.rs

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
    Print,
    Match,

    // Functions
    Return,
    Read,

    // Literals
    True,
    False,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "int" => Some(Keyword::Int),
            "float" => Some(Keyword::Float),
            "string" => Some(Keyword::String),
            "bool" => Some(Keyword::Bool),
            "void" => Some(Keyword::Void),
            "if" => Some(Keyword::If),
            "else" => Some(Keyword::Else),
            "while" => Some(Keyword::While),
            "print" => Some(Keyword::Print),
            "match" => Some(Keyword::Match),
            "return" => Some(Keyword::Return),
            "read" => Some(Keyword::Read),
            "true" => Some(Keyword::True),
            "false" => Some(Keyword::False),
            _ => None,
        }
    }
}
