use crate::tools::debugger::{DebugCommand, Debugger};
use crate::vm::VM;
use std::io::{self, Write};

pub struct Repl {
    debugger: Debugger,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            debugger: Debugger::new(),
        }
    }

    pub fn run(&mut self, vm: &mut VM) -> Result<i32, String> {
        println!("VarVM Debugger");
        println!("Type 'help' for available commands\n");

        // Set up debug callback
        let mut paused = true;
        vm.set_debug_mode(true);

        while vm.is_running() {
            if self.debugger.is_paused() {
                // Interactive prompt
                match self.prompt_command(vm) {
                    Ok(should_continue) => {
                        if !should_continue {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            } else {
                // Execute one instruction
                let ip = vm.get_ip();
                let program = vm.get_program();

                if ip >= program.instructions.len() {
                    println!("Program completed");
                    break;
                }

                // Check if we should break
                if self.debugger.should_break(vm, ip) {
                    self.show_current_instruction(vm);
                    continue;
                }

                // Execute the instruction through the VM
                // We need to manually call execute_instruction here
                // For now, we'll use the public run method with a callback
                break;
            }
        }

        Ok(0)
    }

    fn prompt_command(&mut self, vm: &mut VM) -> Result<bool, String> {
        print!("(vdb) ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read line: {}", e))?;

        let input = input.trim();
        if input.is_empty() {
            return Ok(true);
        }

        match self.parse_command(input) {
            Ok(command) => {
                let result = self.debugger.execute_command(vm, command)?;
                if !result.is_empty() {
                    println!("{}", result);
                }
                Ok(true)
            }
            Err(e) => {
                println!("Error: {}", e);
                Ok(true)
            }
        }
    }

    fn parse_command(&self, input: &str) -> Result<DebugCommand, String> {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command".to_string());
        }

        match parts[0] {
            "step" | "s" => Ok(DebugCommand::Step),
            "next" | "n" => Ok(DebugCommand::Next),
            "continue" | "c" => Ok(DebugCommand::Continue),
            "finish" | "f" => Ok(DebugCommand::Finish),
            "break" | "b" => {
                if parts.len() < 2 {
                    return Err("break requires an argument (IP or function name)".to_string());
                }
                // Try to parse as number first
                if let Ok(ip) = parts[1].parse::<usize>() {
                    Ok(DebugCommand::Break(ip))
                } else {
                    Ok(DebugCommand::BreakFunction(parts[1].to_string()))
                }
            }
            "delete" | "del" => {
                if parts.len() < 2 {
                    return Err("delete requires an IP argument".to_string());
                }
                let ip = parts[1]
                    .parse::<usize>()
                    .map_err(|_| "Invalid IP number".to_string())?;
                Ok(DebugCommand::DeleteBreakpoint(ip))
            }
            "list" => Ok(DebugCommand::ListBreakpoints),
            "print" | "p" => {
                if parts.len() < 2 {
                    return Err("print requires a variable name".to_string());
                }
                Ok(DebugCommand::Print(parts[1].to_string()))
            }
            "locals" => Ok(DebugCommand::Locals),
            "globals" => Ok(DebugCommand::Globals),
            "backtrace" | "bt" => Ok(DebugCommand::Backtrace),
            "disasm" | "d" => Ok(DebugCommand::Disasm),
            "registers" | "r" => Ok(DebugCommand::Registers),
            "l" => Ok(DebugCommand::List),
            "help" | "h" => Ok(DebugCommand::Help),
            "quit" | "q" => Ok(DebugCommand::Quit),
            _ => Err(format!("Unknown command: {}", parts[0])),
        }
    }

    fn show_current_instruction(&self, vm: &VM) {
        let ip = vm.get_ip();
        let program = vm.get_program();

        if ip < program.instructions.len() {
            let instr = &program.instructions[ip];
            println!("=> {:4} {:?}", ip, instr);
        }
    }
}
