use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::fs;
use std::path::Path;
use libloading::Library;
use libloading::Symbol;
use std::sync::{Mutex, MutexGuard, OnceLock};
use crate::node;

static PLUGIN_REGISTRY: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static PLUGIN_DIRECTORY: &str = "plugins/";

static LOADED_LIBRARIES: OnceLock<Mutex<Vec<Library>>> = OnceLock::new();

fn get_registry() -> &'static Mutex<HashMap<String, String>> {
    PLUGIN_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_library_storage() -> &'static Mutex<Vec<Library>> {
    LOADED_LIBRARIES.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn get_plugin_path(name: &str)-> Option<String> {
    let registry = get_registry();
    let map = registry.lock().unwrap();
    map.get(name).cloned()
}

pub fn init_plugins() {
    load_plugins(Path::new(PLUGIN_DIRECTORY));
}

fn load_plugins(dir: &Path) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                load_plugins(&path);
            } else if path.extension().map_or(false, |ext| ext == "dll") {
                load_plugin(&path);
            }
        }
    }
}

fn load_plugin(file: &Path) {
    unsafe {
        let lib = Library::new(file).unwrap();
        let init: Symbol<unsafe extern "C" fn() -> *const c_char> = lib.get(b"plugin_init").unwrap();

        let raw_name = init();
        let name = CStr::from_ptr(raw_name).to_string_lossy().into_owned();
        node::set_current_namespace(&name);

        let init_nodes: Symbol<unsafe extern "Rust" fn(fn(&str, Option<node::PluginCompiler>, Vec<String>, &str)) -> *const c_char> = lib.get(b"register_nodes").unwrap();
        init_nodes(node::register);

        let registry = get_registry();
        let mut map: MutexGuard<'_, HashMap<String, String>> = registry.lock().unwrap();

        let mut lib_storage = get_library_storage().lock().unwrap();
        lib_storage.push(lib);

        map.insert(name.to_string(), file.to_string_lossy().into_owned());
    }
}