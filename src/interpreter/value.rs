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
    FileHandle(String),
    JsonHandle(String),
    TomlHandle(String),
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
    pub fn type_name(&self) -> String {
        match self {
            Value::Integer(_) => "int".to_string(),
            Value::Float(_) => "float".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Char(_) => "char".to_string(),
            Value::Boolean(_) => "bool".to_string(),
            Value::Void => "void".to_string(),
            Value::Option(_) => "option".to_string(),
            Value::List(_) => "list".to_string(),
            Value::Table(_) => "table".to_string(),
            Value::FetchHandle(_) => "fetch".to_string(),
            Value::FileHandle(_) => "file".to_string(),
            Value::JsonHandle(_) => "json".to_string(),
            Value::TomlHandle(_) => "toml".to_string(),
            Value::Struct { name, .. } => name.clone(),
            Value::EnumVariant { enum_name, variant } => format!("{}.{}", enum_name, variant),
            Value::EnumVariantData {
                enum_name,
                variant,
                data: _,
            } => format!("{}.{}(..)", enum_name, variant),
        }
    }
}
