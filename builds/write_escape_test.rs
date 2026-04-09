#![allow(unused_mut, unused_variables, unused_must_use)]
mod luma_runtime;
use luma_runtime::*;

fn main() {
    { luma_write(&Value::String("Hello\nWorld\\r\n".to_string())); Value::Void };
    { luma_write(&Value::String("Tab:\there\\r\nDone".to_string())); Value::Void };
}

