// src/interpreter/value.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),   // single character: 'x'
    Word(String), // single whitespace-free token: 'hello'
    Boolean(bool),
    Void,
    Maybe(Option<Box<Value>>),
    List(Vec<Value>),
    Table(Vec<(Value, Value)>),
    FetchHandle(String),
    InputHandle,
    FileHandle(String),
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Char(_) => "char",
            Value::Word(_) => "word",
            Value::Boolean(_) => "bool",
            Value::Void => "void",
            Value::Maybe(_) => "maybe",
            Value::List(_) => "list",
            Value::Table(_) => "table",
            Value::FetchHandle(_) => "fetch",
            Value::InputHandle => "input",
            Value::FileHandle(_) => "file",
        }
    }
}
