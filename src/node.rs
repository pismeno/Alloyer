use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::plugins;

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub autocompile: bool,
    pub arg_types: Vec<String>,
    pub return_type: String
}

static NODE_REGISTRY: OnceLock<Mutex<HashMap<String, Node>>> = OnceLock::new();

static IMPORTED: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static NAMESPACES_LOADED: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

static CURRENT_NAMESPACE: OnceLock<Mutex<String>> = OnceLock::new();

pub fn import_functions(nodes: &Value) -> String {
    let imported_lock = IMPORTED.get_or_init(|| Mutex::new(Vec::new()));
    let namespaces_lock = NAMESPACES_LOADED.get_or_init(|| Mutex::new(Vec::new()));

    let mut imported = imported_lock.lock().unwrap();
    let mut namespaces_loaded = namespaces_lock.lock().unwrap();
    let mut imports = String::new();

    process_recursive(nodes, &mut imports, &mut imported, &mut namespaces_loaded);

    imports
}

fn process_recursive(node: &Value, imports: &mut String, imported: &mut Vec<String>, namespaces_loaded: &mut Vec<String>,) {
    if let Some(node_list) = node.as_array() {
        for n in node_list {
            process_recursive(n, imports, imported, namespaces_loaded);
        }
        return;
    }

    let Some(name) = node["name"].as_str() else { return };
    let Some(reg_node) = get_reg(name) else { return };
    
    if reg_node.autocompile && !imported.contains(&name.to_string()) {
        let Some((namespace, func_name)) = name.split_once(':') else { return };
        let Some(path) = plugins::get_plugin_path(namespace) else { return };

        if !namespaces_loaded.contains(&namespace.to_string()) {
            imports.push_str(&format!("    let lib_{} = Library::new(\"{}\")?;\n", namespace, path));
            namespaces_loaded.push(namespace.to_string());
        }

        imports.push_str(&format!(
            "    let {}__{}: Symbol<unsafe extern \"Rust\" fn({}) -> {}> = lib_{}.get(b\"{}\")?;\n",
            namespace, func_name, reg_node.arg_types.join(", "), reg_node.return_type, namespace, func_name
        ));

        imported.push(name.to_string());
    }

    check_args(node, imports, imported, namespaces_loaded);
}

fn check_args(node: &Value, imports: &mut String, imported: &mut Vec<String>, namespaces_loaded: &mut Vec<String>) {
    let Some(args) = node["args"].as_array() else { return };
    for arg in args {
        process_recursive(arg, imports, imported, namespaces_loaded);
    }
}

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
            let Some((namespace, func_name)) = name.split_once(':') else { return String::new(); };
            return format!("{}__{}({})", namespace, func_name, processed_aargs.join(", "));
        } else {
            // TODO compiling
            /*
            let compiled = (reg_node.execute)(args);
            let Some(code) = compiled.as_str() else {
                println!("Invalid compile method");
                return String::new();
            };
            return String::from(code);
            */
            return String::new();
        }
    } else {
        return format!("{}", node);
    }
}

fn get_registry() -> &'static Mutex<HashMap<String, Node>> {
    NODE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn set_current_namespace(namespace: &str) {
    let mutex = CURRENT_NAMESPACE.get_or_init(|| Mutex::new(String::from("default")));
    if let Ok(mut guard) = mutex.lock() {
        *guard = namespace.to_string();
    }
}

fn get_current_namespace() -> String {
    CURRENT_NAMESPACE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|guard| guard.clone())
        .unwrap_or_else(|| "".to_string())
}

pub fn register(name: &str, autocompile: bool, arg_types: Vec<String>, return_type: &str) {
    let registry = get_registry();
    let mut map: std::sync::MutexGuard<'_, HashMap<String, Node>> = registry.lock().unwrap();
    let node = Node {
        name: String::from(name),
        autocompile: autocompile,
        arg_types: arg_types,
        return_type: String::from(return_type)
    };
    map.insert(format!("{}:{}", get_current_namespace(), name), node);
    println!("{}, {}", name, autocompile)
}

pub fn get_reg(name: &str) -> Option<Node> {
    let registry = get_registry();
    let map = registry.lock().unwrap();
    map.get(name).cloned()
}