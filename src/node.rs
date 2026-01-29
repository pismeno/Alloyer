use serde_json::Value;

pub struct Node {
    pub name: &'static str,
    pub execute: fn(&Vec<Value>) -> Value,
    pub resolve_args: bool
}

pub fn handle(node: &Value) -> Value {
    let Some(this) = node.as_object() else {
        return node.clone();
    };

    let Some(name) = node["name"].as_str() else {
        println!("Failed to get name of the node");
        return serde_json::json!(null);
    };

    let Some(args) = node["args"].as_array() else {
        println!("Failed to get args of the node");
        return serde_json::json!(null);
    };

    let Some(reg_node) = inventory::iter::<Node>().into_iter().find(|n| n.name == name) else {
        println!("No registered node found for name: {}", name);
        return serde_json::json!(null);
    };

    let processed_args = if reg_node.resolve_args {
        process_args(&args)
    } else {
        args.clone()
    };
    
    return (reg_node.execute)(&processed_args);
}

pub fn process_args(args: &[Value]) -> Vec<Value> {
    args.iter()
        .map(|arg| {
            if arg.is_object() {
                handle(arg)
            } else {
                arg.clone()
            }
        })
        .collect()
}