// src/interpreter/value.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),
    Void,
    Option(Option<Box<Value>>),
    List(Vec<Value>),
    Table(Vec<(Value, Value)>),
    FetchHandle(String),
    InputHandle,
    FileHandle(String),
    Struct {
        name: String,
        fields: HashMap<String, Value>,
    },
    EnumVariant {
        enum_name: String,
        variant: String,
    },
    EnumVariantData {
        enum_name: String,
        variant: String,
        data: Vec<Value>,
    },
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
    pub file_path: String,
    pub line: usize,
    pub column: usize,
}

impl RuntimeError {
    #[allow(dead_code)]
    pub fn new(message: String, line: usize, column: usize) -> Self {
        Self {
            message,
            file_path: String::new(),
            line,
            column,
        }
    }

    pub fn with_file(mut self, file_path: &str) -> Self {
        if self.file_path.is_empty() {
            self.file_path = file_path.to_string();
        }
        self
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Char(_) => "char",
            Value::Boolean(_) => "bool",
            Value::Void => "void",
            Value::Option(_) => "option",
            Value::List(_) => "list",
            Value::Table(_) => "table",
            Value::FetchHandle(_) => "fetch",
            Value::InputHandle => "input",
            Value::FileHandle(_) => "file",
            Value::Struct { .. } => "struct",
            Value::EnumVariant { .. } => "enum",
            Value::EnumVariantData { .. } => "enum",
        }
    }
}
