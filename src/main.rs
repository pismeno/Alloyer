mod node;
mod plugins;
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use serde_json::Value;

use crate::node::{compile, compile_list, set_current_namespace};

fn main() -> Result<(), Box<dyn Error>> {
    plugins::init_plugins();
    node::reg_custom_nodes()?;
    println!("---------------");
    compile_main()?;
    println!("---------------");
    compile_file(Path::new("nodes/func.json"))?;
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



fn compile_file(file: &Path) -> Result<(), Box<dyn Error>> {
    let file_o = File::open(file)?;
    let reader = BufReader::new(file_o);
    let nodes: Value = serde_json::from_reader(reader)?;

    let Some(obj) = nodes.as_object() else {
        return Err("11".into());
    };

    let Some(key) = obj.keys().next() else {
        return Err("12".into());
    };

    let mut code = String::new();

    let Some(reg_func) = node::get_func_reg(&format!("json@{}:{}", file.file_stem().unwrap().to_str().unwrap(), key)) else {
        return Err("13".into());
    };

    let args_string = reg_func.arg_names
        .iter()
        .zip(reg_func.arg_types.iter())
        .map(|(name, typ)| format!("{}: {}", name, typ))
        .collect::<Vec<String>>()
        .join(", ");

    code.push_str(&format!(
        "pub fn {}({}) -> {} {{", 
        key, 
        args_string, 
        reg_func.return_type
    ));

    code.push_str(&compile_list(&nodes[key]));

    println!("{}", code);

    Ok(())
}

fn compile_main() -> Result<(), Box<dyn Error>> {

    let file = File::open("nodes/main.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;

    node::collect_imports(&nodes["main"]);

    let mut main_code = node::compile_collected_mods();
    main_code.push_str("
    use serde_json::Value;
    use libloading::{Library, Symbol};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
    ");

    main_code.push_str(&node::compile_collected_imports());
    main_code.push_str(&node::compile_list(&nodes["main"]));
    main_code.push_str("\n}Ok(())\n}");

    println!("{}", main_code);
    Ok(())
}