use crate::node;

use serde_json::Value;

pub fn init() {
    node::register("nodes::out", out, true);
    node::register("nodes::put", put, true);
    node::register("nodes::add", add, true);
    node::register("nodes::ifelse", ifelse, false);
    node::register("nodes::or", or, true);
}

pub fn put(input: &Vec<Value>) -> Value {
    input[0].clone()
}

pub fn out(input: &Vec<Value>) -> Value {
    println!("{}", input[0]);

    serde_json::json!(null)
}

pub fn add(input: &Vec<Value>) -> Value {
    if let (Some(a), Some(b)) = (input[0].as_f64(), input[1].as_f64()) {
        return serde_json::json!(a + b);
    }

    serde_json::json!(null)
}

pub fn or(input: &Vec<Value>) -> Value {
    if let (Some(a), Some(b)) = (input[0].as_bool(), input[1].as_bool()) {
        return serde_json::json!(a || b);
    }

    serde_json::json!(null)
}

pub fn ifelse(input: &Vec<Value>) -> Value {
    let cond = node::compile(&input[0]);
    let then_block = node::compile_list(&input[1]);
    let else_block = node::compile_list(&input[2]);

    Value::String(format!(
        "if({}.as_bool().unwrap_or(false)){{{}}}else{{{}}}",
        cond, then_block, else_block
    ))
}