use varvm::asm;
use varvm::vm::VM;

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();

    let example_name = if args.len() > 1 {
        args[1].as_str()
    } else {
        println!("Running factorial test (default)");
        println!("Available tests: factorial, bitwise, cast, memory, comparison");
        println!("Usage: cargo run <test_name>\n");
        "factorial"
    };

    // Check if example exists
    let valid_examples = ["factorial", "bitwise", "cast", "memory", "comparison"];
    if !valid_examples.contains(&example_name) {
        eprintln!("Unknown test: {}", example_name);
        eprintln!("Available tests: factorial, bitwise, cast, memory, comparison");
        return;
    }

    // Load and assemble the .vasm file
    let example_path = PathBuf::from("examples").join(format!("{}.vasm", example_name));

    let source = match fs::read_to_string(&example_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read {}: {}", example_path.display(), e);
            return;
        }
    };

    let program = match asm::assemble(&source, example_path.to_string_lossy().to_string()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Assembly failed: {}", e);
            return;
        }
    };

    let mut vm = VM::new(program);
    match vm.run() {
        Ok(_) => println!("\nProgram executed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
