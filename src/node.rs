use serde_json::Value;

pub struct Node {
    pub name: &'static str,
    pub execute: fn(&Vec<Value>) -> Value,
}