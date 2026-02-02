use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::plugins;

pub type Compiler = fn(&Value) -> String;
pub type PluginCompiler = fn(&Value, Compiler, Compiler) -> String;

#[derive(Clone)]
pub struct Node {
    pub name: String,
    pub compiler: Option<PluginCompiler>,
    pub arg_types: Vec<String>,
    pub return_type: String
}

static NODE_REGISTRY: OnceLock<Mutex<HashMap<String, Node>>> = OnceLock::new();

static IMPORTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static COLLECTED_NAMESPACES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

static CURRENT_NAMESPACE: OnceLock<Mutex<String>> = OnceLock::new();

pub fn PLUGIN_COMPILER_PLACEHOLDER(x: &Value) -> String { String::new() }

pub fn collect_imports_plugin(node: &Value) -> String {
    collect_imports(node);
    return String::new();
}

pub fn collect_imports(nodes: &Value) {
    // 1. Create local temporary buffers (No locking yet)
    let mut local_imports = Vec::new();
    let mut local_namespaces = Vec::new();

    // 2. Process recursively using the local buffers
    // This allows plugins to call 'collect_imports' again without deadlocking,
    // because we aren't holding the global lock here.
    process_recursive(nodes, &mut local_imports, &mut local_namespaces);

    // 3. NOW lock the globals and merge the results
    let imports_lock = IMPORTS.get_or_init(|| Mutex::new(Vec::new()));
    let namespaces_lock = COLLECTED_NAMESPACES.get_or_init(|| Mutex::new(Vec::new()));

    // Merge Imports
    {
        let mut global_imports = imports_lock.lock().unwrap();
        for imp in local_imports {
            if !global_imports.contains(&imp) {
                global_imports.push(imp);
            }
        }
    } // Lock released here

    // Merge Namespaces
    {
        let mut global_namespaces = namespaces_lock.lock().unwrap();
        for ns in local_namespaces {
            if !global_namespaces.contains(&ns) {
                global_namespaces.push(ns);
            }
        }
    } // Lock released here
}

fn process_recursive(node: &Value, imports: &mut Vec<String>, collected_namespaces: &mut Vec<String>) {
    if let Some(node_list) = node.as_array() {
        for n in node_list {
            process_recursive(n, imports, collected_namespaces);
        }
        return;
    }

    let Some(name) = node["name"].as_str() else { return };
    let Some(reg_node) = get_reg(name) else { return };

    if let Some(node_compiler) = reg_node.compiler {
        node_compiler(node, collect_imports_plugin, collect_imports_plugin);
        return;
    }
    
    if !imports.contains(&name.to_string()) {
        let Some((namespace, _)) = name.split_once(':') else { return };

        if !collected_namespaces.contains(&namespace.to_string()) {
            collected_namespaces.push(namespace.to_string());
        }

        imports.push(name.to_string());
    }

    check_args(node, imports, collected_namespaces);
}

fn check_args(node: &Value, imports: &mut Vec<String>, collected_namespaces: &mut Vec<String>) {
    let Some(args) = node["args"].as_array() else { return };
    for arg in args {
        process_recursive(arg, imports, collected_namespaces);
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

        if let Some(plugin_compiler) = reg_node.compiler {
            return plugin_compiler(node, compile, compile_list);
        };

        let processed_aargs: Vec<String>  = args.iter()
            .map(|a| compile(a)) 
            .collect();
        let Some((namespace, func_name)) = name.split_once(':') else { return String::new(); };
        return format!("{}__{}({})", namespace, func_name, processed_aargs.join(", "));
    } else {
        return format!("{}", node);
    }
}

fn get_registry() -> &'static Mutex<HashMap<String, Node>> {
    NODE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn compile_collected_imports() -> String {
    let mut code = String::new();

    let namespaces_lock = COLLECTED_NAMESPACES.get_or_init(|| Mutex::new(Vec::new()));
    let namespaces = {
        let lock = namespaces_lock.lock().unwrap();
        lock.clone()
    };

    for namespace in namespaces {
        let Some(path) = plugins::get_plugin_path(&namespace) else { return code };
        code.push_str(&format!(
            "let lib_{} = Library::new(\"{}\")?;\n", namespace, path
        ));
    }


    let imports_lock = IMPORTS.get_or_init(|| Mutex::new(Vec::new()));
    let imports = {
        let lock = imports_lock.lock().unwrap();
        lock.clone()
    };

    for import in imports {

        let Some(reg_node) = get_reg(&import) else {
            println!("Node is not registered");
            return code;
        };

        let Some((namespace, func_name)) = import.split_once(':') else { return code; };
    
        code.push_str(&format!(
            "let {}__{}: Symbol<unsafe extern \"Rust\" fn({}) -> {}> = lib_{}.get(b\"{}\")?;\n",
            namespace, func_name, reg_node.arg_types.join(", "), reg_node.return_type, namespace, func_name
        ));
    }

    return code;
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

pub fn register(name: &str, compiler: Option<PluginCompiler>, arg_types: Vec<String>, return_type: &str) {
    let registry = get_registry();
    let mut map: std::sync::MutexGuard<'_, HashMap<String, Node>> = registry.lock().unwrap();
    let node = Node {
        name: String::from(name),
        compiler: compiler,
        arg_types: arg_types,
        return_type: String::from(return_type)
    };
    map.insert(format!("{}:{}", get_current_namespace(), name), node);
    println!("{}", name)
}

pub fn get_reg(name: &str) -> Option<Node> {
    let registry = get_registry();
    let map = registry.lock().unwrap();
    map.get(name).cloned()
}