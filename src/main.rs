mod node;
mod plugins;
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use serde_json::Value;

use crate::node::{compile, set_current_namespace};

fn main() -> Result<(), Box<dyn Error>> {
    plugins::init_plugins();
    node::reg_custom_nodes()?;
    println!("---------------");
    compile_main()?;
    Ok(())
}

fn compile_all(dir: &Path) -> Result<(), Box<dyn Error>> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                compile_all(&path);
            } else if path.extension().map_or(false, |ext| ext == "dll") {
                compile_all(&path);
            }
        }
    }
    Ok(())
}



fn compile_file(file: &Path) {

}

fn compile_main() -> Result<(), Box<dyn Error>> {
    let mut main_code = node::compile_collected_mods();
    main_code.push_str("
    use serde_json::Value;
    use libloading::{Library, Symbol};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
    ");

    let file = File::open("nodes/main.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;

    node::collect_imports(&nodes["main"]);

    main_code.push_str(&node::compile_collected_imports());
    main_code.push_str(&node::compile_list(&nodes["main"]));
    main_code.push_str("\n}Ok(())\n}");

    println!("{}", main_code);
    Ok(())
}