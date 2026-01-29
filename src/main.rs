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

    if let Some(node_list) = nodes["main"].as_array() {
        for node in node_list {
            node::handle(node);
        }
    } else {
        println!("Failed to get list of nodes");
    }

    Ok(())
}