# Alloyer

An engine for programming languages that operates based on "nodes." These are written in JSON and subsequently compiled into Rust code. Each node references either another node (defined as a function in `nodes.json`) or a function from a `.dll` plugin, which the plugin registers during compilation.

# Attentntion
This project has been abandoned.  

## Goal
The goal is to create a GUI desktop application that generates the JSON and then calls the Alloyer engine. The engine is intended to be versatile enough to create any type of project.

## Future Plans
- The ability to compile into either a library or an executable.

## Libraries and Languages

### Engine:
**Rust**, using libraries:
- `serde_json`
- `libloading`

### GUI:
Most likely **C#**, using libraries:
- Probably **WPF**

## How to run?

1. **Clone the project.**
2. **Run the engine:** Use the command `cargo run`. This will compile the `.json` files from the `nodes/` directory into Rust code in `buildrs/src`.

### Sample Files
The repository currently includes the following sample files:
* `main.json`: The standard main function where the code execution begins.
* `func.json`: Sample custom functions created from already existing nodes.
* `nodes.json`: The file where custom functions are registered. Here, you must specify the file where the function is defined, its name, arguments (and their types), and the return type.

### Important Note on Plugins
In order for the sample `.json` files to work, you must place the sample plugin `nodes.dll` (found in the classroom) into a folder named `plugins` in the project root.
