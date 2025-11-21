use std::collections::HashMap;
use std::io::Write;
use crate::types::{DataType, Operand, Value};
use crate::opcode::OpCode;
use crate::program::Program;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub return_ip: usize,
    pub locals: HashMap<String, Value>,
    pub return_dest: Option<String>,
    pub args: Vec<Value>,
}

pub struct VM {
    program: Program,
    ip: usize,
    globals: HashMap<String, Value>,
    call_stack: Vec<CallFrame>,
    current_frame: CallFrame,
    heap: HashMap<usize, Vec<u8>>,
    next_heap_addr: usize,
    running: bool,
}

impl VM {
    pub fn new(program: Program) -> Self {
        let mut vm = Self {
            program,
            ip: 0,
            globals: HashMap::new(),
            call_stack: Vec::new(),
            current_frame: CallFrame {
                function_name: "main".to_string(),
                return_ip: 0,
                locals: HashMap::new(),
                return_dest: None,
                args: Vec::new(),
            },
            heap: HashMap::new(),
            next_heap_addr: 0x1000,
            running: true,
        };

        // Initialize globals
        for global in &vm.program.globals {
            vm.globals.insert(global.name.clone(), Value::I32(0));
        }

        vm
    }

    pub fn run(&mut self) -> Result<i32, String> {
        // Start by calling main
        let main_func = self.program.functions.get("main")
            .ok_or_else(|| "No main function found".to_string())?
            .clone();
        self.ip = main_func.start_ip + 1;

        while self.running && self.ip < self.program.instructions.len() {
            self.execute_instruction()?;
        }
        Ok(0)
    }

    fn execute_instruction(&mut self) -> Result<(), String> {
        let instruction = self.program.instructions[self.ip].clone();
        self.ip += 1;

        match instruction {
            OpCode::CreateLocal { dtype, name } => {
                let value = self.default_value(dtype);
                self.current_frame.locals.insert(name, value);
            }

            OpCode::CreateGlobal { dtype, name } => {
                let value = self.default_value(dtype);
                self.globals.insert(name, value);
            }

            OpCode::DeleteLocal { name } => {
                self.current_frame.locals.remove(&name);
            }

            OpCode::SetVar { dest, value } => {
                let val = self.resolve_operand(&value)?;
                self.set_variable(&dest, val)?;
            }

            OpCode::CopyVar { dest, source } => {
                let val = self.get_variable(&source)?;
                self.set_variable(&dest, val)?;
            }

            OpCode::Alloc { dest, size } => {
                let size = self.resolve_operand(&size)?.as_usize()?;
                let addr = self.next_heap_addr;
                self.heap.insert(addr, vec![0u8; size]);
                self.next_heap_addr += size;
                self.set_variable(&dest, Value::Ptr(addr))?;
            }

            OpCode::Free { ptr } => {
                let addr = self.get_variable(&ptr)?.as_usize()?;
                self.heap.remove(&addr);
            }

            OpCode::Load { dest, ptr, dtype } => {
                let addr = self.get_variable(&ptr)?.as_usize()?;
                let bytes = self.heap.get(&addr)
                    .ok_or_else(|| format!("Invalid pointer: {:#x}", addr))?;
                let value = self.bytes_to_value(bytes, dtype)?;
                self.set_variable(&dest, value)?;
            }

            OpCode::Store { ptr, source, dtype } => {
                let addr = self.get_variable(&ptr)?.as_usize()?;
                let value = self.get_variable(&source)?;
                let bytes = self.value_to_bytes(&value, dtype)?;
                self.heap.insert(addr, bytes);
            }

            OpCode::GetAddr { dest, var } => {
                // Simulated address - in real impl would need actual memory addresses
                let fake_addr = var.bytes().fold(0usize, |acc, b| acc.wrapping_add(b as usize));
                self.set_variable(&dest, Value::Ptr(fake_addr))?;
            }

            OpCode::Add { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.add(&r)?)?;
            }

            OpCode::Sub { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.sub(&r)?)?;
            }

            OpCode::Mul { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.mul(&r)?)?;
            }

            OpCode::Div { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.div(&r)?)?;
            }

            OpCode::Mod { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.modulo(&r)?)?;
            }

            OpCode::Neg { dest, source } => {
                let val = self.resolve_operand(&source)?;
                self.set_variable(&dest, val.neg()?)?;
            }

            OpCode::Eq { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, Value::I32(if l == r { 1 } else { 0 }))?;
            }

            OpCode::Ne { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, Value::I32(if l != r { 1 } else { 0 }))?;
            }

            OpCode::Lt { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, Value::I32(if l.lt(&r)? { 1 } else { 0 }))?;
            }

            OpCode::Le { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, Value::I32(if l.le(&r)? { 1 } else { 0 }))?;
            }

            OpCode::Gt { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, Value::I32(if l.gt(&r)? { 1 } else { 0 }))?;
            }

            OpCode::Ge { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, Value::I32(if l.ge(&r)? { 1 } else { 0 }))?;
            }

            OpCode::And { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.bitwise_and(&r)?)?;
            }

            OpCode::Or { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.bitwise_or(&r)?)?;
            }

            OpCode::Xor { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.bitwise_xor(&r)?)?;
            }

            OpCode::Not { dest, source } => {
                let val = self.resolve_operand(&source)?;
                self.set_variable(&dest, val.bitwise_not()?)?;
            }

            OpCode::Shl { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.shift_left(&r)?)?;
            }

            OpCode::Shr { dest, left, right } => {
                let l = self.resolve_operand(&left)?;
                let r = self.resolve_operand(&right)?;
                self.set_variable(&dest, l.shift_right(&r)?)?;
            }

            OpCode::Cast { dest, source, target_type } => {
                let val = self.get_variable(&source)?;
                self.set_variable(&dest, val.cast(target_type)?)?;
            }

            OpCode::Input { dest } => {
                use std::io::{self, BufRead};
                print!("Enter value for {}: ", dest);
                io::stdout().flush().ok();
                let stdin = io::stdin();
                let line = stdin.lock().lines().next()
                    .ok_or_else(|| "Failed to read input".to_string())?
                    .map_err(|e| format!("IO error: {}", e))?;

                // Try to parse as i32 by default
                let value = line.trim().parse::<i32>()
                    .map(Value::I32)
                    .map_err(|_| format!("Invalid integer: {}", line))?;

                self.set_variable(&dest, value)?;
            }

            OpCode::PushArg { var } => {
                // This is for alternative calling convention - not used in current impl
                // Could be used to build up arguments before a call
                let _val = self.get_variable(&var)?;
                // Would push to an arg stack
            }

            OpCode::Label { .. } => {
                // Labels are no-ops during execution
            }

            OpCode::Jmp { label } => {
                self.ip = *self.program.labels.get(&label)
                    .ok_or_else(|| format!("Unknown label: {}", label))?;
            }

            OpCode::Jz { var, label } => {
                let val = self.get_variable(&var)?;
                if val.is_zero() {
                    self.ip = *self.program.labels.get(&label)
                        .ok_or_else(|| format!("Unknown label: {}", label))?;
                }
            }

            OpCode::Jnz { var, label } => {
                let val = self.get_variable(&var)?;
                if !val.is_zero() {
                    self.ip = *self.program.labels.get(&label)
                        .ok_or_else(|| format!("Unknown label: {}", label))?;
                }
            }

            OpCode::FuncBegin { name, .. } => {
                // Skip to end of function if not being called
                let func = self.program.functions.get(&name)
                    .ok_or_else(|| format!("Unknown function: {}", name))?;
                self.ip = func.end_ip + 1;
            }

            OpCode::FuncEnd => {
                // No-op, handled by Return
            }

            OpCode::Call { result, func, args } => {
                let func_def = self.program.functions.get(&func)
                    .ok_or_else(|| format!("Unknown function: {}", func))?
                    .clone();

                // Get argument values before switching frames
                let mut arg_values = Vec::new();
                for arg in &args {
                    let val = self.get_variable(arg)?;
                    arg_values.push(val);
                }

                // Push current frame with return destination
                let mut frame = std::mem::replace(&mut self.current_frame, CallFrame {
                    function_name: func.clone(),
                    return_ip: 0,
                    locals: HashMap::new(),
                    return_dest: None,
                    args: arg_values,
                });
                frame.return_ip = self.ip;
                frame.return_dest = result; // Store return dest in caller's frame
                self.call_stack.push(frame);

                self.ip = func_def.start_ip + 1;
            }

            OpCode::Return { value } => {
                let ret_val = value.map(|op| self.resolve_operand(&op)).transpose()?;

                if let Some(frame) = self.call_stack.pop() {
                    let return_dest = frame.return_dest.clone();
                    self.ip = frame.return_ip;
                    self.current_frame = frame;

                    // Set return value in the restored frame
                    if let (Some(val), Some(dest)) = (ret_val, &return_dest) {
                        self.set_variable(dest, val.clone())?;
                    }
                } else {
                    self.running = false;
                }
            }

            OpCode::PopArg { dest } => {
                // Pop arguments in reverse order (args are pushed in order, pop from end)
                if !self.current_frame.args.is_empty() {
                    let val = self.current_frame.args.remove(0); // Remove from front to preserve order
                    self.current_frame.locals.insert(dest, val);
                } else {
                    return Err("No arguments to pop".to_string());
                }
            }

            OpCode::Print { var } => {
                let val = self.get_variable(&var)?;
                println!("{}: {:?}", var, val);
            }

            OpCode::Exit { code: _ } => {
                self.running = false;
                return Ok(());
            }
        }

        Ok(())
    }

    fn resolve_operand(&self, operand: &Operand) -> Result<Value, String> {
        match operand {
            Operand::Variable(name) => self.get_variable(name),
            Operand::Immediate(val) => Ok(val.clone()),
            Operand::Label(_) => Err("Cannot resolve label as value".to_string()),
            Operand::Type(_) => Err("Cannot resolve type as value".to_string()),
        }
    }

    fn get_variable(&self, name: &str) -> Result<Value, String> {
        self.current_frame.locals.get(name)
            .or_else(|| self.globals.get(name))
            .cloned()
            .ok_or_else(|| format!("Unknown variable: {}", name))
    }

    fn set_variable(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.current_frame.locals.contains_key(name) {
            self.current_frame.locals.insert(name.to_string(), value);
        } else if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
        } else {
            return Err(format!("Unknown variable: {}", name));
        }
        Ok(())
    }

    fn default_value(&self, dtype: DataType) -> Value {
        match dtype {
            DataType::I8 => Value::I8(0),
            DataType::I16 => Value::I16(0),
            DataType::I32 => Value::I32(0),
            DataType::I64 => Value::I64(0),
            DataType::U8 => Value::U8(0),
            DataType::U16 => Value::U16(0),
            DataType::U32 => Value::U32(0),
            DataType::U64 => Value::U64(0),
            DataType::F32 => Value::F32(0.0),
            DataType::F64 => Value::F64(0.0),
            DataType::Ptr => Value::Ptr(0),
            DataType::Void => Value::I32(0),
        }
    }

    fn bytes_to_value(&self, bytes: &[u8], dtype: DataType) -> Result<Value, String> {
        match dtype {
            DataType::I8 => {
                if bytes.len() < 1 {
                    return Err("Insufficient bytes for I8".to_string());
                }
                Ok(Value::I8(i8::from_le_bytes([bytes[0]])))
            }
            DataType::I16 => {
                if bytes.len() < 2 {
                    return Err("Insufficient bytes for I16".to_string());
                }
                Ok(Value::I16(i16::from_le_bytes([bytes[0], bytes[1]])))
            }
            DataType::I32 => {
                if bytes.len() < 4 {
                    return Err("Insufficient bytes for I32".to_string());
                }
                Ok(Value::I32(i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])))
            }
            DataType::I64 => {
                if bytes.len() < 8 {
                    return Err("Insufficient bytes for I64".to_string());
                }
                Ok(Value::I64(i64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ])))
            }
            DataType::U8 => {
                if bytes.len() < 1 {
                    return Err("Insufficient bytes for U8".to_string());
                }
                Ok(Value::U8(bytes[0]))
            }
            DataType::U16 => {
                if bytes.len() < 2 {
                    return Err("Insufficient bytes for U16".to_string());
                }
                Ok(Value::U16(u16::from_le_bytes([bytes[0], bytes[1]])))
            }
            DataType::U32 => {
                if bytes.len() < 4 {
                    return Err("Insufficient bytes for U32".to_string());
                }
                Ok(Value::U32(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])))
            }
            DataType::U64 => {
                if bytes.len() < 8 {
                    return Err("Insufficient bytes for U64".to_string());
                }
                Ok(Value::U64(u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ])))
            }
            DataType::F32 => {
                if bytes.len() < 4 {
                    return Err("Insufficient bytes for F32".to_string());
                }
                Ok(Value::F32(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])))
            }
            DataType::F64 => {
                if bytes.len() < 8 {
                    return Err("Insufficient bytes for F64".to_string());
                }
                Ok(Value::F64(f64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ])))
            }
            DataType::Ptr => {
                if bytes.len() < 8 {
                    return Err("Insufficient bytes for Ptr".to_string());
                }
                Ok(Value::Ptr(usize::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                    bytes[4], bytes[5], bytes[6], bytes[7],
                ])))
            }
            DataType::Void => Err("Cannot read Void type from memory".to_string()),
        }
    }

    fn value_to_bytes(&self, value: &Value, dtype: DataType) -> Result<Vec<u8>, String> {
        match (value, dtype) {
            (Value::I8(v), DataType::I8) => Ok(v.to_le_bytes().to_vec()),
            (Value::I16(v), DataType::I16) => Ok(v.to_le_bytes().to_vec()),
            (Value::I32(v), DataType::I32) => Ok(v.to_le_bytes().to_vec()),
            (Value::I64(v), DataType::I64) => Ok(v.to_le_bytes().to_vec()),
            (Value::U8(v), DataType::U8) => Ok(vec![*v]),
            (Value::U16(v), DataType::U16) => Ok(v.to_le_bytes().to_vec()),
            (Value::U32(v), DataType::U32) => Ok(v.to_le_bytes().to_vec()),
            (Value::U64(v), DataType::U64) => Ok(v.to_le_bytes().to_vec()),
            (Value::F32(v), DataType::F32) => Ok(v.to_le_bytes().to_vec()),
            (Value::F64(v), DataType::F64) => Ok(v.to_le_bytes().to_vec()),
            (Value::Ptr(v), DataType::Ptr) => Ok(v.to_le_bytes().to_vec()),
            _ => Err(format!("Type mismatch: cannot store {:?} as {:?}", value, dtype)),
        }
    }
}
