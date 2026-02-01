mod node;
mod plugins;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use serde_json::Value;
use std::sync::{Mutex, OnceLock};

fn main() -> Result<(), Box<dyn Error>> {
    plugins::load_plugin("plugins/nodes.dll");
    println!("---------------");
    compile_all()?;
    Ok(())
}

fn compile_all() -> Result<(), Box<dyn Error>> {
    let mut main_code = "
    use serde_json::Value;
    use libloading::{Library, Symbol};

    fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
    ".to_string();

    let file = File::open("nodes.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;

    main_code.push_str(&node::import_functions(&nodes["main"]));
    main_code.push_str(&node::compile_list(&nodes["main"]));
    main_code.push_str("\n}Ok(())\n}");

    println!("{}", main_code);
    Ok(())
}