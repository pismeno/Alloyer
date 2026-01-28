use crate::node::Node;
use serde_json::Value;

inventory::submit! {
    Node { 
        name: "out", 
        execute: out,
        has_follow_up: false
    }
}

inventory::submit! {
    Node {
        name: "put",
        execute: put,
        has_follow_up: true
    }
}

pub fn put(input: Value) -> Value {
    input.clone()
}

pub fn out(input: Value) -> Value {
    println!("{}", input["0"]);

    serde_json::json!(null)
}