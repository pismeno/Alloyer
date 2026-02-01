mod node;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use serde_json::Value;
use libloading::Library;
use libloading::Symbol;
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::sync::Arc;

static PLUGIN_REGISTRY: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
fn main() -> Result<(), Box<dyn Error>> {
    load_plugin("plugins/nodes.dll");

    println!("---------------");

    compile_all();

    Ok(())
}

fn get_registry() -> &'static Mutex<HashMap<String, String>> {
    PLUGIN_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_plugin_path(name: &str)-> Option<String> {
    let registry = get_registry();
    let map = registry.lock().unwrap();
    map.get(name).cloned()
}

fn load_plugin(path: &str) {
    unsafe {
        let lib = Library::new(path).unwrap();
        let init: Symbol<unsafe extern "C" fn() -> *const c_char> = lib.get(b"plugin_init").unwrap();

        let raw_name = init();
        let name = CStr::from_ptr(raw_name).to_string_lossy().into_owned();

        let init_nodes: Symbol<unsafe extern "C" fn(fn(&str, fn(&Vec<Value>) -> Value, bool)) -> *const c_char> = lib.get(b"register_nodes").unwrap();
        init_nodes(node::register);

        let registry = get_registry();
        let mut map: MutexGuard<'_, HashMap<String, String>> = registry.lock().unwrap();
        map.insert(name.to_string(), path.to_string());
    }
}

fn compile_all() -> Result<(), Box<dyn Error>> {
    let mut main = "
    use serde_json::Value;

    fn main() {
    ".to_string();

    let file = File::open("nodes.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;
    let imports = import_all_functions(&nodes);

    main.push_str(&imports);
    main.push_str(&node::compile_list(&nodes["main"]));
    main.push('}');

    println!("{}", main);

    Ok(())
}

fn import_all_functions(nodes: &Value) -> String {

    let mut imported: Vec<String> = Vec::new();
    let mut imports = String::new();

    let Some(node_list) = nodes["main"].as_array() else {
        println!("Not a list");
        return String::new();
    };

    for node in node_list {
        let name = node["name"].as_str().unwrap();

        let Some(reg_node) = node::get_reg(name) else {
            println!("Node is not registered");
            return String::new();
        };

        if reg_node.autocompile == true {
            if (imported.contains(&name.to_string())) { continue; };

            let Some((namespace, func_name)) = name.split_once(':') else { continue; };
            let Some(path) = get_plugin_path(namespace) else { continue; };
            imports.push_str(&format!(
                "
                let lib = Library::new(\"{}\")?;
                let {}__{}: Symbol<unsafe extern \"C\" fn(&Vec<Value>) -> Value> = lib.get(b\"add\")?;
                ",
                path, namespace, func_name
            ));
            imported.push(name.to_string());
        }
    }

    return imports;
}