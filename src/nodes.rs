use crate::node;
use crate::node::Node;
use serde_json::Value;

inventory::submit! {
    Node { 
        name: "out", 
        execute: out,
        resolve_args: true
    }
}

inventory::submit! {
    Node {
        name: "put",
        execute: put,
        resolve_args: true
    }
}

inventory::submit! {
    Node {
        name: "add",
        execute: add,
        resolve_args: true
    }
}

inventory::submit! {
    Node {
        name: "ifelse",
        execute: ifelse,
        resolve_args: false
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

pub fn ifelse(input: &Vec<Value>) -> Value {
    let Some(condition) = node::handle(&input[0]).as_bool() else {
        println!("Condition is not a boolean");
        return serde_json::json!(null);
    };

    if condition {
        node::handle(&input[1]);
    } else {
        node::handle(&input[2]);
    }

    serde_json::json!(null)
}