mod nodes;
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut PLUGIN_REGISTRY: HashMap<String, Library> = HashMap::new();

    nodes::init();
    load_plugin("../example_plugin/target/release/example_plugin.dll", &mut PLUGIN_REGISTRY);

    let file = File::open("nodes.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;

    println!("{}", node::compile_list(&nodes["main"]));

    println!("---------------");

    nodes::out(&vec![nodes::put(&vec![nodes::add(&vec![nodes::add(&vec![serde_json::json!(6), serde_json::json!(-2)]), serde_json::json!(-2)])])]);nodes::out(&vec![serde_json::json!("world")]);if(nodes::or(&vec![serde_json::json!(false), serde_json::json!(false)]).as_bool().unwrap_or(false)){nodes::out(&vec![serde_json::json!("IF block")]);}else{if(serde_json::json!(false).as_bool().unwrap_or(false)){nodes::out(&vec![serde_json::json!("2IF block")]);}else{nodes::out(&vec![serde_json::json!("2ELSE block")]);}}

    Ok(())
}

fn load_plugin(path: &str, PLUGIN_REGISTRY: &mut HashMap<String, Library>) {
    unsafe {
        let lib = Library::new(path).unwrap();
        let init: Symbol<unsafe extern "C" fn() -> *const c_char> = lib.get(b"plugin_init").unwrap();

        let raw_name = init();
        let name = CStr::from_ptr(raw_name).to_string_lossy().into_owned();

        let init_nodes: Symbol<unsafe extern "C" fn(fn(&str, fn(&Vec<Value>) -> Value, bool)) -> *const c_char> = lib.get(b"register_nodes").unwrap();
        init_nodes(node::register);

        PLUGIN_REGISTRY.insert(name, lib);
    }
}