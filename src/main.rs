mod nodes;
mod node;
use node::Node;
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use serde_json::Value;

inventory::collect!(Node);

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("nodes.json")?;
    let reader = BufReader::new(file);
    let nodes: Value = serde_json::from_reader(reader)?;

    println!("{}", node::compile_list(&nodes["main"]));

    Ok(())
}