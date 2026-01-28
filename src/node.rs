use serde_json::Value;

pub struct Node {
    pub name: &'static str,
    pub execute: fn(Value) -> Value,
    pub has_follow_up: bool
}