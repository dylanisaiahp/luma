// src/interpreter/value.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Void,
    Maybe(Option<Box<Value>>),
    List(Vec<Value>),
    Table(Vec<(Value, Value)>),
    FetchHandle(String), // holds the URL, .get() and .send() called as methods
    InputHandle, // returned by input() with no args, .flag() and .option() called as methods
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
            Value::Boolean(_) => "bool",
            Value::Void => "void",
            Value::Maybe(_) => "maybe",
            Value::List(_) => "list",
            Value::Table(_) => "table",
            Value::FetchHandle(_) => "fetch",
            Value::InputHandle => "input",
        }
    }
}
