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

    println!("---------------");

    nodes::out(&vec![nodes::put(&vec![nodes::add(&vec![nodes::add(&vec![serde_json::json!(6), serde_json::json!(-2)]), serde_json::json!(-2)])])]);nodes::out(&vec![serde_json::json!("world")]);if(nodes::or(&vec![serde_json::json!(false), serde_json::json!(false)]).as_bool().unwrap_or(false)){nodes::out(&vec![serde_json::json!("IF block")]);}else{if(serde_json::json!(false).as_bool().unwrap_or(false)){nodes::out(&vec![serde_json::json!("2IF block")]);}else{nodes::out(&vec![serde_json::json!("2ELSE block")]);}}

    Ok(())
}