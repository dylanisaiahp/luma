#![allow(unused_mut, unused_variables, unused_must_use)]
mod luma_runtime;
use luma_runtime::*;

fn main() {
    let raw = Value::String("method = \"initialize\"\nid = 1".to_string());
    let data = { let _obj = luma_toml(&raw.clone()); let _s = if let Value::TomlHandle(ref s) = _obj { s.clone() } else { luma_runtime::runtime_error("toml() method called on non-toml") }; luma_toml_method(&_s, "parse", &[]) };
    luma_print(&luma_method(data.clone(), "get", vec![Value::String("method".to_string())]));
}

