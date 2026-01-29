use crate::node::Node;
use serde_json::Value;

inventory::submit! {
    Node { 
        name: "out", 
        execute: out,
    }
}

inventory::submit! {
    Node {
        name: "put",
        execute: put,
    }
}

inventory::submit! {
    Node {
        name: "add",
        execute: add,
    }
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