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
            handle_node(node);
        }
    } else {
        println!("Failed to get list of nodes");
    }

    Ok(())
}

fn handle_node(node: &Value) -> Value {
    if let Some(name) = node["name"].as_str() {
        if let Some(args) = node["args"].as_array() {

            let processed_args: Vec<Value> = match node["args"].as_array() {
                Some(args) => args
                    .iter()
                    .map(|arg| {
                        if arg.is_object() {
                            // If it's a node, resolve it first
                            handle_node(arg)
                        } else {
                            // If it's a simple entry (string/number), just clone it
                            arg.clone()
                        }
                    })
                    .collect(),
                None => Vec::new(),
            };

            if let Some(reg_node) = inventory::iter::<Node>().into_iter().find(|n| n.name == name) {
                return (reg_node.execute)(&processed_args);
            } else {
                println!("No registered node found for name: {}", name);
            }
        }
    } else {
        println!("Failed to get name of node");
    }

    serde_json::json!(null)
}