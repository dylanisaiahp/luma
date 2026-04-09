#![allow(unused_mut, unused_variables, unused_must_use)]
mod luma_runtime;
use luma_runtime::*;

fn main() {
    let raw = Value::String("{\"params\": {\"textDocument\": {\"uri\": \"test.slt\", \"text\": \"void main(){}\"}}}".to_string());
    let data = { let _obj = luma_json(&raw.clone()); let _s = if let Value::JsonHandle(ref s) = _obj { s.clone() } else { luma_runtime::runtime_error("json() method called on non-json") }; luma_json_method(&_s, "parse", &[]) };
    luma_print(&luma_method(data.clone(), "get", vec![Value::String("params.textDocument.uri".to_string())]));
    luma_print(&luma_method(data.clone(), "get", vec![Value::String("params.textDocument.text".to_string())]));
}

