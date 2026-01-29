use crate::node;
use crate::node::Node;
use serde_json::Value;

inventory::submit! {
    Node { 
        name: "nodes::out", 
        execute: out,
        autocompile: true
    }
}

inventory::submit! {
    Node {
        name: "nodes::put",
        execute: put,
        autocompile: true
    }
}

inventory::submit! {
    Node {
        name: "nodes::add",
        execute: add,
        autocompile: true
    }
}

inventory::submit! {
    Node {
        name: "nodes::ifelse",
        execute: ifelse,
        autocompile: false
    }
}

inventory::submit! {
    Node {
        name: "nodes::or",
        execute: or,
        autocompile: true
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

pub fn or(input: &Vec<Value>) -> Value {
    if let (Some(a), Some(b)) = (input[0].as_bool(), input[1].as_bool()) {
        return serde_json::json!(a || b);
    }

    serde_json::json!(null)
}

pub fn ifelse(input: &Vec<Value>) -> Value {
    let mut code = String::from("if(");

    code.push_str(&format!("{}.as_bool().unwrap_or(false)", &node::compile(&input[0])));
    code.push_str("){");
    code.push_str(&node::compile_list(&input[1]));
    code.push_str("}else{");
    code.push_str(&node::compile_list(&input[2]));
    code.push('}');

    return Value::String(code);
}