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

fn handle_node(node: &Value) {
    if let Some(name) = node["name"].as_str() {
        let args = &node["args"];
        if let Some(reg_node) = inventory::iter::<Node>().into_iter().find(|n| n.name == name) {
            if (reg_node.has_follow_up) {
                let returned = (reg_node.execute)(args.clone());
                handle_follow_up(node["follow_up"]["name"].clone(), returned);
            } else {
                (reg_node.execute)(args.clone());
            }
        } else {
            println!("No registered node found for name: {}", name);
        }
    } else {
        println!("Failed to get name of node");
    }
}

fn handle_follow_up(name_value: Value, args: Value) {
    if let Some(name) =  name_value.as_str() {
        if let Some(reg_node) = inventory::iter::<Node>().into_iter().find(|n| n.name == name) {
            (reg_node.execute)(args.clone());
        } else {
            println!("No registered node found for name: {}", name);
        }
    }
}