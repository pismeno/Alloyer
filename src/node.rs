use serde_json::Value;
use std::collections::HashMap;
use std::fmt::format;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use std::path::Path;

use crate::plugins;

pub type Compiler = fn(&Value) -> String;
pub type PluginCompiler = fn(&Vec<Value>, Compiler, Compiler) -> String;

#[derive(Clone)]
pub struct Node {
    pub compiler: Option<PluginCompiler>,
    pub arg_types: Vec<String>,
    pub return_type: String
}

#[derive(Clone)]
pub struct FuncNode {
    pub compiler: Option<PluginCompiler>,
    pub arg_names: Vec<String>,
    pub arg_types: Vec<String>,
    pub return_type: String
}

static NODE_REGISTRY: OnceLock<Mutex<HashMap<String, Node>>> = OnceLock::new();
static FUNC_NODE_REGISTRY: OnceLock<Mutex<HashMap<String, FuncNode>>> = OnceLock::new();

static IMPORTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static MOD_IMPORTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static COLLECTED_NAMESPACES: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

static CURRENT_NAMESPACE: OnceLock<Mutex<String>> = OnceLock::new();

pub fn collect_imports_plugin(node: &Value) -> String {
    collect_imports(node, false);
    return String::new();
}

pub fn collect_imports(nodes: &Value, clear: bool) {
    let mut local_imports: Vec<String> = Vec::new();
    let mut local_mod_imports: Vec<String> = Vec::new();
    let mut local_namespaces: Vec<String> = Vec::new();

    import_recursive(nodes, &mut local_imports, &mut local_mod_imports, &mut local_namespaces);

    let imports_lock = IMPORTS.get_or_init(|| Mutex::new(Vec::new()));
    let mod_imports_lock = MOD_IMPORTS.get_or_init(|| Mutex::new(Vec::new()));
    let namespaces_lock = COLLECTED_NAMESPACES.get_or_init(|| Mutex::new(Vec::new()));

    let mut global_imports = imports_lock.lock().unwrap();
    if clear { global_imports.clear() };
    for imp in local_imports {
        if !global_imports.contains(&imp) {
            global_imports.push(imp);
        }
    }

    let mut global_mod_imports = mod_imports_lock.lock().unwrap();
    for imp in local_mod_imports {
        if !global_mod_imports.contains(&imp) {
            global_mod_imports.push(imp);
        }
    }
    
    let mut global_namespaces = namespaces_lock.lock().unwrap();
    if clear { global_namespaces.clear(); }
    for ns in local_namespaces {
        if !global_namespaces.contains(&ns) {
            global_namespaces.push(ns);
        }
    }
}

fn import_recursive(node: &Value, imports: &mut Vec<String>, mod_imports: &mut Vec<String>, collected_namespaces: &mut Vec<String>) {
    // 1. Handle Arrays (recursion)
    if let Some(node_list) = node.as_array() {
        for n in node_list {
            import_recursive(n, imports, mod_imports, collected_namespaces);
        }
        return;
    }

    // 2. Extract Name
    let Some(name) = node["name"].as_str() else { return };
    
    // 3. Handle Special Compiler Nodes (Recursion for arguments)
    if let Some(reg_node) = get_any_reg(name) {
        if let Some(node_compiler) = reg_node.compiler {
            if let Some(args) = node["args"].as_array() {
                // If the node has a custom compiler, we still need to recurse into its args!
                 for arg in args {
                    import_recursive(arg, imports, mod_imports, collected_namespaces);
                }
            }
            // Even if it has a compiler, we might return here, BUT we must ensure
            // we don't return before checking if the node ITSELF needs importing.
            // (Assuming custom compiler nodes don't need DLL imports, we return).
            return; 
        }
    }

    // 4. MAIN IMPORT LOGIC
    if !imports.contains(&name.to_string()) {
        if let Some((namespace, _)) = name.split_once(':') {
            
            // CASE A: Local JSON/Mod Import (contains '@')
            if let Some((prefix, md)) = namespace.split_once('@') {
                if prefix == "json" {
                     if !mod_imports.contains(&md.to_string()) {
                        mod_imports.push(md.to_string());
                     }
                }
            } 
            // CASE B: External DLL Import (No '@')
            else {
                // This is the part that was failing!
                if !collected_namespaces.contains(&namespace.to_string()) {
                    collected_namespaces.push(namespace.to_string());
                }
                imports.push(name.to_string());
            }
        }
    }

    // 5. Recurse into Arguments (Standard Nodes)
    if let Some(args) = node["args"].as_array() {
        for arg in args {
            import_recursive(arg, imports, mod_imports, collected_namespaces);
        }
    }
}

pub fn reg_custom_nodes() -> Result<(), Box<dyn Error>> {
    let file = File::open("nodes/nodes.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;

    let Some(node_list) = nodes.as_array() else {
        return Err("1".into());
    };

    for node in node_list {
        let Some(file) = node["file"].as_str() else {
            return Err("2".into());
        };
        let Some(func) = node["fn"].as_str() else {
            return Err("3".into());
        };
        let Some(return_type) = node["return_type"].as_str() else {
            return Err("4".into());
        };
        let Some(args) = node["args"].as_array() else {
            return Err("5".into());
        };
        let stem = Path::new(file).file_stem().unwrap().to_str().unwrap();
        set_current_namespace(&format!("json@{}", stem));
        let mut arg_names: Vec<String> = Vec::new();
        let mut arg_types: Vec<String> = Vec::new();
        for arg in args {
            let Some(typ) = arg["type"].as_str() else {
                return Err("6".into());
            };
            let Some(name) = arg["name"].as_str() else {
                return Err("7".into());
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

        let args = &extract_args(node);
        let proccessed_args = proccess_args(args);

        if let Some(reg_node) = get_func_reg(name) {
            let md = name.split('@').nth(1).unwrap_or("").split(':').nth(0).unwrap_or("");
            let f_name = name.split(':').nth(1).unwrap_or("");
            return format!("{}::{}({})", md, f_name, proccessed_args);
        }

        let Some(reg_node) = get_reg(name) else {
            println!("Node is not registered");
            return String::new();
        };

        if let Some(node_compiler) = reg_node.compiler {
            return node_compiler(args, compile, compile_list);
        };

        let Some((namespace, func_name)) = name.split_once(':') else { return String::new(); };
        return format!("{}__{}({})", namespace, func_name, proccessed_args);
    } else {
        if let Some(str_val) = node.as_str() {
            if let Some(raw_code) = str_val.strip_prefix('$') {
                return raw_code.to_string();
            }
        }

        return format!("{}", node);
    }
}

fn extract_args(val: &Value) -> Vec<Value> {
    val["args"]
        .as_array()
        .cloned() 
        .unwrap_or_else(|| {
            println!("Invalid arguments");
            vec![]
        })
}

fn proccess_args(args: &Vec<Value>) -> String {
    let proccessed_aargs: Vec<String>  = args.iter()
            .map(|a| compile(a)) 
            .collect();
    
    return proccessed_aargs.join(", ");
}

fn get_registry() -> &'static Mutex<HashMap<String, Node>> {
    NODE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_func_registry() -> &'static Mutex<HashMap<String, FuncNode>> {
    FUNC_NODE_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn compile_collected_imports(add_question_mark: bool) -> String {
    let mut code = String::new();

    // 1. Generate Library Loaders
    let namespaces_lock = COLLECTED_NAMESPACES.get_or_init(|| Mutex::new(Vec::new()));
    let namespaces = namespaces_lock.lock().unwrap().clone();

    for namespace in namespaces {
        // Skip local namespaces starting with "json@"
        if namespace.starts_with("json@") {
            continue;
        }

        let Some(path) = plugins::get_plugin_path(&namespace) else { continue };
        if add_question_mark {
            code.push_str(&format!(
                "let lib_{} = Library::new(\"{}\")?;\n", namespace, path
            ));
        } else {
            code.push_str(&format!(
                "let lib_{} = Library::new(\"{}\").expect(\"Failed to load symbol '{}' from library\");\n", namespace, path, namespace
            ));
        }
    }
        

    // 2. Generate Symbol Definitions
    let imports_lock = IMPORTS.get_or_init(|| Mutex::new(Vec::new()));
    let imports = imports_lock.lock().unwrap().clone();

    for import in imports {
        // SKIP local compiled nodes! They are called directly via mod::func
        if import.starts_with("json@") {
            continue;
        }

        // Only look for DLL nodes in the registry
        let Some(reg_node) = get_reg(&import) else {
            println!("Node {} is not registered", import);
            continue; // Use continue, NOT return!
        };

        let Some((namespace, func_name)) = import.split_once(':') else { continue };
        
        if add_question_mark {
            code.push_str(&format!(
                "let {}__{}: Symbol<extern \"Rust\" fn({}) -> {}> = lib_{}.get(b\"{}\")?;\n",
                namespace, func_name, reg_node.arg_types.join(", "), reg_node.return_type, namespace, func_name
            ));
        } else {
            code.push_str(&format!(
                "let {}__{}: Symbol<extern \"Rust\" fn({}) -> {}> = lib_{}.get(b\"{}\").expect(\"Failed to load symbol '{}' from library\");\n",
                namespace, func_name, reg_node.arg_types.join(", "), reg_node.return_type, namespace, func_name, func_name
            ));
        }
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

pub fn get_func_reg(name: &str) -> Option<FuncNode> {
    let registry = get_func_registry();
    let map = registry.lock().unwrap();
    map.get(name).cloned()
}

pub fn get_any_reg(name: &str) -> Option<Node> {
    if name.starts_with("json@") {
        let registry = get_func_registry();
        let map = registry.lock().unwrap();
        map.get(name).map(|f| Node {
            compiler: f.compiler,
            arg_types: f.arg_types.clone(),
            return_type: f.return_type.clone(),
        })
    } else {
        get_reg(name)
    }
}