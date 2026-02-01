use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use serde_json::Value;
use libloading::Library;
use libloading::Symbol;
use std::sync::{Mutex, MutexGuard, OnceLock};
use crate::node;

static PLUGIN_REGISTRY: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn get_registry() -> &'static Mutex<HashMap<String, String>> {
    PLUGIN_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_plugin_path(name: &str)-> Option<String> {
    let registry = get_registry();
    let map = registry.lock().unwrap();
    map.get(name).cloned()
}

pub fn load_plugin(path: &str) {
    unsafe {
        let lib = Library::new(path).unwrap();
        let init: Symbol<unsafe extern "C" fn() -> *const c_char> = lib.get(b"plugin_init").unwrap();

        let raw_name = init();
        let name = CStr::from_ptr(raw_name).to_string_lossy().into_owned();
        node::set_current_namespace(&name);

        let init_nodes: Symbol<unsafe extern "Rust" fn(fn(&str, bool, Vec<String>, &str)) -> *const c_char> = lib.get(b"register_nodes").unwrap();
        init_nodes(node::register);

        let registry = get_registry();
        let mut map: MutexGuard<'_, HashMap<String, String>> = registry.lock().unwrap();
        map.insert(name.to_string(), path.to_string());
    }
}