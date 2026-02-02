use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;

use crate::plugins;

pub type Compiler = fn(&Value) -> String;
pub type PluginCompiler = fn(&Vec<Value>, Compiler, Compiler) -> String;

#[derive(Clone)]
pub struct Node {
    pub compiler: Option<PluginCompiler>,
    pub arg_types: Vec<String>,
    pub return_type: String
}

struct FuncNode {
    compiler: Option<PluginCompiler>,
    arg_names: Vec<String>,
    arg_types: Vec<String>,
    return_type: String
}

static NODE_REGISTRY: OnceLock<Mutex<HashMap<String, Node>>> = OnceLock::new();
static FUNC_NODE_REGISTRY: OnceLock<Mutex<HashMap<String, FuncNode>>> = OnceLock::new();

static IMPORTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static MOD_IMPORTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static COLLECTED_NAMESPACES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

static CURRENT_NAMESPACE: OnceLock<Mutex<String>> = OnceLock::new();

pub fn collect_imports_plugin(node: &Value) -> String {
    collect_imports(node);
    return String::new();
}

pub fn collect_imports(nodes: &Value) {
    let mut local_imports: Vec<String> = Vec::new();
    let mut local_mod_imports: Vec<String> = Vec::new();
    let mut local_namespaces: Vec<String> = Vec::new();

    import_recursive(nodes, &mut local_imports, &mut local_mod_imports, &mut local_namespaces);

    let imports_lock = IMPORTS.get_or_init(|| Mutex::new(Vec::new()));
    let namespaces_lock = COLLECTED_NAMESPACES.get_or_init(|| Mutex::new(Vec::new()));

    let mut global_imports = imports_lock.lock().unwrap();
    for imp in local_imports {
        if !global_imports.contains(&imp) {
            global_imports.push(imp);
        }
    }
    
    let mut global_namespaces = namespaces_lock.lock().unwrap();
    for ns in local_namespaces {
        if !global_namespaces.contains(&ns) {
            global_namespaces.push(ns);
        }
    }
}

fn import_recursive(node: &Value, imports: &mut Vec<String>, mod_imports: &mut Vec<String>, collected_namespaces: &mut Vec<String>) {
    if let Some(node_list) = node.as_array() {
        for n in node_list {
            import_recursive(n, imports, mod_imports, collected_namespaces);
        }
        return;
    }

    let Some(name) = node["name"].as_str() else { return };
    let Some(reg_node) = get_reg(name) else { return };

    if let Some(node_compiler) = reg_node.compiler {
        if let Some(args) = node["args"].as_array() {
            node_compiler(args, collect_imports_plugin, collect_imports_plugin);
        }
        return; 
    }
    
    if !imports.contains(&name.to_string()) {
        let Some((namespace, _)) = name.split_once(':') else { return };
        if let Some((json, md)) = namespace.split_once('@') { 
            if json != "json" {
                return;
            }
            mod_imports.push(md.to_string());
         };

        if !collected_namespaces.contains(&namespace.to_string()) {
            collected_namespaces.push(namespace.to_string());
        }
        imports.push(name.to_string());
    }

    let Some(args) = node["args"].as_array() else { return };
    for arg in args {
        import_recursive(arg, imports, mod_imports, collected_namespaces);
    }
}

pub fn reg_custom_nodes() -> Result<(), Box<dyn Error>> {
    let file = File::open("nodes/nodes.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;

    let Some(node_list) = nodes.as_array() else {
        return Err("".into());
    };

    for node in node_list {
        let Some(file) = node["file"].as_str() else {
            return Err("".into());
        };
        let Some(func) = node["func"].as_str() else {
            return Err("".into());
        };
        let Some(return_type) = node["return_type"].as_str() else {
            return Err("".into());
        };
        let Some(args) = node["args"].as_array() else {
            return Err("".into());
        };
        set_current_namespace(&format!("json@{}", file));
        let mut arg_names: Vec<String> = Vec::new();
        let mut arg_types: Vec<String> = Vec::new();
        for arg in args {
            let Some(typ) = arg["type"].as_str() else {
                return Err("".into());
            };
            let Some(name) = arg["name"].as_str() else {
                return Err("".into());
            };
            arg_names.push(name.to_string());
            arg_types.push(typ.to_string());
        }
        register_func(func, None, arg_names, arg_types, return_type);
    }

    Ok(())
}

pub fn compile_list(nodes: &Value) -> String {
    let mut code = String::new();
    if let Some(node_list) = nodes.as_array() {
        for node in node_list {
            code.push_str(&compile(node));

            let trimmed = code.trim();
            if !trimmed.ends_with('}') && !trimmed.ends_with(';') {
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

        if let Some(node_compiler) = reg_node.compiler {
            return node_compiler(args, compile, compile_list);
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

fn get_func_registry() -> &'static Mutex<HashMap<String, FuncNode>> {
    FUNC_NODE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
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
            "let {}__{}: Symbol<extern \"Rust\" fn({}) -> {}> = lib_{}.get(b\"{}\")?;\n",
            namespace, func_name, reg_node.arg_types.join(", "), reg_node.return_type, namespace, func_name
        ));
    }

    return code;
}

pub fn compile_collected_mods () -> String {
    let mut code = String::new();

    let mods_lock = MOD_IMPORTS.get_or_init(|| Mutex::new(Vec::new()));
    let mods = {
        let lock = mods_lock.lock().unwrap();
        lock.clone()
    };

    for md in mods {
        code.push_str(&format!("mod {};", md));
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

fn register_func(name: &str, compiler: Option<PluginCompiler>, arg_names: Vec<String> ,arg_types: Vec<String>, return_type: &str) {
    let registry = get_func_registry();
    let mut map: MutexGuard<'_, HashMap<String, FuncNode>> = registry.lock().unwrap();
    let node = FuncNode {
        compiler: compiler,
        arg_names: arg_names,
        arg_types: arg_types,
        return_type: String::from(return_type)
    };
    map.insert(format!("{}:{}", get_current_namespace(), name), node);
    println!("{}", name)
}

pub fn register(name: &str, compiler: Option<PluginCompiler>, arg_types: Vec<String>, return_type: &str) {
    let registry = get_registry();
    let mut map: MutexGuard<'_, HashMap<String, Node>> = registry.lock().unwrap();
    let node = Node {
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