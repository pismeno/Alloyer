mod node;
mod plugins;
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use serde_json::Value;

use crate::node::compile_collected_imports;

fn main() -> Result<(), Box<dyn Error>> {
    plugins::init_plugins();
    node::reg_custom_nodes()?;
    compile_all(Path::new("nodes/"))?;
    Ok(())
}

fn compile_all(dir: &Path) -> Result<(), Box<dyn Error>> {
    compile_main()?;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = path.file_stem().unwrap().to_str().unwrap();
            if path.is_dir() {
                compile_all(&path)?;
            } else if path.extension().map_or(false, |ext| ext == "json") && filename != "main" && filename != "nodes" {
                compile_file(&path)?;
            }
        }
    }
    Ok(())
}



fn compile_file(file: &Path) -> Result<(), Box<dyn Error>> {
    let file_o = File::open(file)?;
    let reader = BufReader::new(file_o);
    let nodes: Value = serde_json::from_reader(reader)?;

    let mut file_code = String::from("use libloading::{Library, Symbol};");

    let Some(obj) = nodes.as_object() else {
        return Err("11".into());
    };

    let mut keys_iter = obj.keys();
    let filename = file.file_stem().unwrap().to_str().unwrap();
    while let Some(key) = keys_iter.next() {
        node::collect_imports(&nodes[key], true);
        let mut code = String::new();

        let Some(reg_func) = node::get_func_reg(&format!("json@{}:{}", filename, key)) else {
            return Err("13".into());
        };

        let args_string = reg_func.arg_names
            .iter()
            .zip(reg_func.arg_types.iter())
            .map(|(name, typ)| format!("{}: {}", name, typ))
            .collect::<Vec<String>>()
            .join(", ");

        code.push_str(&format!(
            "pub fn {}({}) -> {} {{unsafe {{", 
            key, 
            args_string, 
            reg_func.return_type
        ));

        code.push_str(&compile_collected_imports(false));
        code.push_str(&node::compile_list(&nodes[key]));
        code.push_str("}}");

        file_code.push_str(&code);
    }

    build_file(&file_code, &filename)?;

    Ok(())
}

fn compile_main() -> Result<(), Box<dyn Error>> {

    let file = File::open("nodes/main.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;

    node::collect_imports(&nodes["main"], true);

    let mut main_code = node::compile_collected_mods();
    main_code.push_str("
    use libloading::{Library, Symbol};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
    ");

    main_code.push_str(&node::compile_collected_imports(true));
    main_code.push_str(&node::compile_list(&nodes["main"]));
    main_code.push_str("\n}Ok(())\n}");

    build_file(&main_code, "main")?;
    Ok(())
}

fn build_file(code: &str, filename: &str) -> Result<(), Box<dyn Error>> {
    let path = Path::new("buildrs/src").join(format!("{}.rs", filename));
    fs::write(path, code)?;
    Ok(())
}