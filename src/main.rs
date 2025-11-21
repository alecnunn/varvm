mod examples;
mod opcode;
mod program;
mod types;
mod vm;

use examples::{
    bitwise_operations_test, comparison_test, factorial_program, memory_operations_test,
    type_cast_test,
};
use std::env;
use vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();

    let program = if args.len() > 1 {
        match args[1].as_str() {
            "factorial" => factorial_program(),
            "bitwise" => bitwise_operations_test(),
            "cast" => type_cast_test(),
            "memory" => memory_operations_test(),
            "comparison" => comparison_test(),
            _ => {
                eprintln!("Unknown test: {}", args[1]);
                eprintln!("Available tests: factorial, bitwise, cast, memory, comparison");
                return;
            }
        }
    } else {
        println!("Running factorial test (default)");
        println!("Available tests: factorial, bitwise, cast, memory, comparison");
        println!("Usage: cargo run <test_name>\n");
        factorial_program()
    };

    let mut vm = VM::new(program);
    match vm.run() {
        Ok(_) => println!("\nProgram executed successfully"),
        Err(e) => eprintln!("Error: {}", e),
    }
}
