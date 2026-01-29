use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone)]
struct Node {
    pub name: String,
    pub execute: fn(&Vec<Value>) -> Value,
    pub autocompile: bool
}

static NODE_REGISTRY: OnceLock<Mutex<HashMap<String, Node>>> = OnceLock::new();

pub fn compile_list(nodes: &Value) -> String {
    let mut code = String::new();
    if let Some(node_list) = nodes.as_array() {
        for node in node_list {
            code.push_str(&compile(node));

            if !code.trim().ends_with('}') {
                code.push(';');
            }
        }
    } else {
        println!("Failed to get list of nodes");
    }

    return code;
}

pub fn compile(node: &Value) -> String {
    if node.is_object() {
        let name = node["name"].as_str().unwrap();

        let Some(reg_node) = get_reg(name) else {
            println!("Node is not registered");
            return String::new();
        };

        let Some(args) = node["args"].as_array() else {
            println!("Invalid arguments");
            return String::new();
        };
        
        if reg_node.autocompile {
            let processed_aargs: Vec<String>  = args.iter()
                .map(|a| compile(a)) 
                .collect();
            return format!("{}(&vec![{}])", name, processed_aargs.join(", "));
        } else {
            let compiled = (reg_node.execute)(args);
            let Some(code) = compiled.as_str() else {
                println!("Invalid compile method");
                return String::new();
            };
            return String::from(code);
        }
    } else {
        return format!("serde_json::json!({})", node);
    }
}

/// Returns a reference to the global map, initializing it if necessary
fn get_registry() -> &'static Mutex<HashMap<String, Node>> {
    NODE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn register(name: &str, execute: fn(&Vec<Value>) -> Value, autocompile: bool) {
    let registry = get_registry();
    let mut map: std::sync::MutexGuard<'_, HashMap<String, Node>> = registry.lock().unwrap();
    let node = Node {
        name: name.to_string(),
        execute: execute,
        autocompile: autocompile
    };
    map.insert(name.to_string(), node);
    println!("{}, {}", name, autocompile)
}

pub fn get_reg(name: &str) -> Option<Node> {
    let registry = get_registry();
    let map = registry.lock().unwrap();
    map.get(name).cloned()
}