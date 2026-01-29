use serde_json::Value;


pub struct Node {
    pub name: &'static str,
    pub execute: fn(&Vec<Value>) -> Value,
    pub autocompile: bool
}

pub fn compile_list(nodes: &Value) -> String {
    let mut code = String::new();
    if let Some(node_list) = nodes.as_array() {
        for node in node_list {
            code.push_str(&compile(node));

            if !code.trim().ends_with('}') {
                code.push(';');
            }
        }
    } else {
        println!("Failed to get list of nodes");
    }

    return code;
}

pub fn compile(node: &Value) -> String {
    if node.is_object() {
        let name = node["name"].as_str().unwrap();

        let Some(reg_node) = get_reg(name) else {
            println!("No registered node found for name: {}", name);
            return String::new();
        };

        let Some(args) = node["args"].as_array() else {
            println!("Invalid arguments");
            return String::new();
        };
        
        if (reg_node.autocompile) {
            let processed_aargs: Vec<String>  = args.iter()
                .map(|a| compile(a)) 
                .collect();
            return format!("{}(&vec![{}])", name, processed_aargs.join(", "));
        } else {
            let compiled = ((reg_node.execute)(args));
            let Some(code) = compiled.as_str() else {
                println!("Invalid compile method");
                return String::new();
            };
            return String::from(code);
        }
    } else {
        return format!("serde_json::json!({})", node);
    }
}

pub fn handle(node: &Value) -> Value {
    let Some(_) = node.as_object() else {
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

    let Some(reg_node) = get_reg(name) else {
        println!("No registered node found for name: {}", name);
        return serde_json::json!(null);
    };

    let processed_args = if reg_node.autocompile {
        process_args(&args)
    } else {
        args.clone()
    };
    
    return (reg_node.execute)(&processed_args);
}

pub fn get_reg(name: &str) -> Option<&Node> {
    inventory::iter::<Node>().into_iter().find(|n| n.name == name)
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