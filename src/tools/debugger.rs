use crate::asm::disassembler::disassemble;
use crate::opcode::OpCode;
use crate::program::Program;
use crate::types::Value;
use crate::vm::{CallFrame, VM};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DebugCommand {
    Step,
    Next,
    Continue,
    Finish,
    Break(usize),
    BreakFunction(String),
    DeleteBreakpoint(usize),
    ListBreakpoints,
    Print(String),
    Locals,
    Globals,
    Backtrace,
    Disasm,
    Registers,
    List,
    Help,
    Quit,
}

pub struct Debugger {
    paused: bool,
    step_mode: bool,
    next_depth: Option<usize>,
    finish_depth: Option<usize>,
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            paused: true,
            step_mode: false,
            next_depth: None,
            finish_depth: None,
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn execute_command(&mut self, vm: &mut VM, command: DebugCommand) -> Result<String, String> {
        match command {
            DebugCommand::Step => {
                self.step_mode = true;
                self.next_depth = None;
                self.finish_depth = None;
                self.resume();
                Ok("Stepping to next instruction".to_string())
            }
            DebugCommand::Next => {
                let depth = vm.get_call_stack().len();
                self.step_mode = false;
                self.next_depth = Some(depth);
                self.finish_depth = None;
                self.resume();
                Ok("Stepping over function calls".to_string())
            }
            DebugCommand::Continue => {
                self.step_mode = false;
                self.next_depth = None;
                self.finish_depth = None;
                self.resume();
                Ok("Continuing execution".to_string())
            }
            DebugCommand::Finish => {
                let depth = vm.get_call_stack().len();
                if depth == 0 {
                    return Err("Already in top-level frame".to_string());
                }
                self.step_mode = false;
                self.next_depth = None;
                self.finish_depth = Some(depth - 1);
                self.resume();
                Ok("Running until function returns".to_string())
            }
            DebugCommand::Break(ip) => {
                vm.add_breakpoint(ip);
                Ok(format!("Breakpoint set at IP {}", ip))
            }
            DebugCommand::BreakFunction(func_name) => {
                let program = vm.get_program();
                if let Some(func) = program.functions.get(&func_name) {
                    let ip = func.start_ip;
                    vm.add_breakpoint(ip);
                    Ok(format!("Breakpoint set at function '{}' (IP {})", func_name, ip))
                } else {
                    Err(format!("Function '{}' not found", func_name))
                }
            }
            DebugCommand::DeleteBreakpoint(ip) => {
                vm.remove_breakpoint(ip);
                Ok(format!("Breakpoint at IP {} removed", ip))
            }
            DebugCommand::ListBreakpoints => {
                let breakpoints = vm.get_breakpoints();
                if breakpoints.is_empty() {
                    Ok("No breakpoints set".to_string())
                } else {
                    let mut output = String::from("Breakpoints:\n");
                    for (i, &ip) in breakpoints.iter().enumerate() {
                        output.push_str(&format!("  {}. IP {}\n", i + 1, ip));
                    }
                    Ok(output)
                }
            }
            DebugCommand::Print(var_name) => {
                self.print_variable(vm, &var_name)
            }
            DebugCommand::Locals => {
                self.print_locals(vm)
            }
            DebugCommand::Globals => {
                self.print_globals(vm)
            }
            DebugCommand::Backtrace => {
                self.print_backtrace(vm)
            }
            DebugCommand::Disasm => {
                self.disassemble_current(vm)
            }
            DebugCommand::Registers => {
                self.print_registers(vm)
            }
            DebugCommand::List => {
                self.list_source(vm)
            }
            DebugCommand::Help => {
                Ok(Self::help_text())
            }
            DebugCommand::Quit => {
                vm.stop();
                Ok("Exiting debugger".to_string())
            }
        }
    }

    pub fn should_break(&mut self, vm: &VM, ip: usize) -> bool {
        // Check if we hit a breakpoint
        if vm.get_breakpoints().contains(&ip) {
            self.pause();
            return true;
        }

        // Check if we're in step mode
        if self.step_mode {
            self.pause();
            return true;
        }

        // Check if we're doing 'next' and returned to the right depth
        if let Some(target_depth) = self.next_depth {
            let current_depth = vm.get_call_stack().len();
            if current_depth <= target_depth {
                self.pause();
                self.next_depth = None;
                return true;
            }
        }

        // Check if we're doing 'finish' and returned to the right depth
        if let Some(target_depth) = self.finish_depth {
            let current_depth = vm.get_call_stack().len();
            if current_depth <= target_depth {
                self.pause();
                self.finish_depth = None;
                return true;
            }
        }

        false
    }

    fn print_variable(&self, vm: &VM, var_name: &str) -> Result<String, String> {
        let frame = vm.get_current_frame();

        // Try locals first
        if let Some(value) = frame.locals.get(var_name) {
            return Ok(format!("{}: {:?}", var_name, value));
        }

        // Try globals
        if let Some(value) = vm.get_globals().get(var_name) {
            return Ok(format!("{}: {:?}", var_name, value));
        }

        Err(format!("Variable '{}' not found", var_name))
    }

    fn print_locals(&self, vm: &VM) -> Result<String, String> {
        let frame = vm.get_current_frame();
        if frame.locals.is_empty() {
            Ok("No local variables".to_string())
        } else {
            let mut output = String::from("Local variables:\n");
            for (name, value) in &frame.locals {
                output.push_str(&format!("  {}: {:?}\n", name, value));
            }
            Ok(output)
        }
    }

    fn print_globals(&self, vm: &VM) -> Result<String, String> {
        let globals = vm.get_globals();
        if globals.is_empty() {
            Ok("No global variables".to_string())
        } else {
            let mut output = String::from("Global variables:\n");
            for (name, value) in globals {
                output.push_str(&format!("  {}: {:?}\n", name, value));
            }
            Ok(output)
        }
    }

    fn print_backtrace(&self, vm: &VM) -> Result<String, String> {
        let current = vm.get_current_frame();
        let stack = vm.get_call_stack();

        let mut output = String::from("Call stack:\n");
        output.push_str(&format!("  0. {} (current)\n", current.function_name));

        for (i, frame) in stack.iter().rev().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, frame.function_name));
        }

        Ok(output)
    }

    fn disassemble_current(&self, vm: &VM) -> Result<String, String> {
        let program = vm.get_program();
        let disasm = disassemble(program);
        Ok(disasm)
    }

    fn print_registers(&self, vm: &VM) -> Result<String, String> {
        let ip = vm.get_ip();
        let stack_depth = vm.get_call_stack().len();
        let frame = vm.get_current_frame();

        let mut output = String::from("Registers/State:\n");
        output.push_str(&format!("  IP: {}\n", ip));
        output.push_str(&format!("  Call Depth: {}\n", stack_depth));
        output.push_str(&format!("  Current Function: {}\n", frame.function_name));
        output.push_str(&format!("  Running: {}\n", vm.is_running()));

        Ok(output)
    }

    fn list_source(&self, vm: &VM) -> Result<String, String> {
        let ip = vm.get_ip();
        let program = vm.get_program();

        if ip >= program.instructions.len() {
            return Err("IP out of bounds".to_string());
        }

        let start = if ip >= 5 { ip - 5 } else { 0 };
        let end = (ip + 10).min(program.instructions.len());

        let mut output = String::new();
        for i in start..end {
            let prefix = if i == ip { "=>" } else { "  " };
            let instr = &program.instructions[i];
            output.push_str(&format!("{} {:4} {:?}\n", prefix, i, instr));
        }

        Ok(output)
    }

    fn help_text() -> String {
        r#"VarVM Debugger Commands:

Execution Control:
  step, s          Step to next instruction
  next, n          Step over function calls
  continue, c      Continue execution until breakpoint
  finish, f        Run until current function returns

Breakpoints:
  break <ip>       Set breakpoint at instruction pointer
  break <func>     Set breakpoint at function entry
  delete <ip>      Remove breakpoint
  list             List all breakpoints

Inspection:
  print <var>      Print variable value
  locals           Show local variables
  globals          Show global variables
  backtrace, bt    Show call stack
  registers, r     Show VM state
  list, l          List instructions around current IP
  disasm, d        Disassemble entire program

Other:
  help, h          Show this help
  quit, q          Exit debugger
"#.to_string()
    }
}
