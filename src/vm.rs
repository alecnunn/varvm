use crate::opcode::OpCode;
use crate::program::Program;
use crate::tools::profiler::ProfileData;
use crate::types::{DataType, Operand, Value};
use std::collections::{HashMap, HashSet};
use std::io::Write;

// Macro for binary operations
macro_rules! binary_op {
    ($self:expr, $dest:expr, $left:expr, $right:expr, $method:ident) => {{
        let l = $self.resolve_operand(&$left)?;
        let r = $self.resolve_operand(&$right)?;
        $self.set_variable(&$dest, l.$method(&r)?)?;
    }};
}

// Macro for comparison operations
macro_rules! comparison_op {
    ($self:expr, $dest:expr, $left:expr, $right:expr, $method:ident) => {{
        let l = $self.resolve_operand(&$left)?;
        let r = $self.resolve_operand(&$right)?;
        $self.set_variable(&$dest, Value::I32(if l.$method(&r)? { 1 } else { 0 }))?;
    }};
}

// Macro for equality comparison
macro_rules! equality_op {
    ($self:expr, $dest:expr, $left:expr, $right:expr, ==) => {{
        let l = $self.resolve_operand(&$left)?;
        let r = $self.resolve_operand(&$right)?;
        $self.set_variable(&$dest, Value::I32(if l.equals(&r) { 1 } else { 0 }))?;
    }};
    ($self:expr, $dest:expr, $left:expr, $right:expr, !=) => {{
        let l = $self.resolve_operand(&$left)?;
        let r = $self.resolve_operand(&$right)?;
        $self.set_variable(&$dest, Value::I32(if !l.equals(&r) { 1 } else { 0 }))?;
    }};
}

// Macro for unary operations
macro_rules! unary_op {
    ($self:expr, $dest:expr, $source:expr, $method:ident) => {{
        let val = $self.resolve_operand(&$source)?;
        $self.set_variable(&$dest, val.$method()?)?;
    }};
}

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub function_name: String,
    pub return_ip: usize,
    pub locals: HashMap<String, Value>,
    pub return_dest: Option<String>,
    pub args: Vec<Value>,
}

pub type DebugCallback = Box<dyn FnMut(&mut VM, usize, &OpCode) -> Result<(), String>>;

pub struct VM {
    program: Program,
    ip: usize,
    globals: HashMap<String, Value>,
    call_stack: Vec<CallFrame>,
    current_frame: CallFrame,
    heap: HashMap<usize, Vec<u8>>,
    next_heap_addr: usize,
    running: bool,
    debug_mode: bool,
    breakpoints: HashSet<usize>,
    debug_callback: Option<DebugCallback>,
    profile_enabled: bool,
    profile_data: ProfileData,
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
            debug_mode: false,
            breakpoints: HashSet::new(),
            debug_callback: None,
            profile_enabled: false,
            profile_data: ProfileData::new(),
        };

        // Initialize globals
        for global in &vm.program.globals {
            vm.globals.insert(global.name.clone(), Value::I32(0));
        }

        // Initialize string literals
        vm.initialize_strings();

        vm
    }

    fn initialize_strings(&mut self) {
        for string_literal in &self.program.strings.clone() {
            // Allocate memory for the string (content + null terminator)
            let bytes = string_literal.content.as_bytes();
            let mut string_bytes = bytes.to_vec();
            string_bytes.push(0); // Add null terminator

            // Allocate memory
            let addr = self.next_heap_addr;
            self.next_heap_addr += string_bytes.len();

            // Store the string bytes in heap
            self.heap.insert(addr, string_bytes);

            // Set the global pointer variable to point to this string
            self.globals.insert(
                string_literal.global_name.clone(),
                Value::Ptr(addr),
            );
        }
    }

    pub fn run(&mut self) -> Result<i32, String> {
        // Start by calling main
        let main_func = self
            .program
            .functions
            .get("main")
            .ok_or_else(|| "No main function found".to_string())?
            .clone();
        self.ip = main_func.start_ip + 1;

        while self.running && self.ip < self.program.instructions.len() {
            self.execute_instruction()?;
        }
        Ok(0)
    }

    fn execute_instruction(&mut self) -> Result<(), String> {
        let current_ip = self.ip;
        let instruction = self.program.instructions[self.ip].clone();

        // Profile: record instruction execution if profiling enabled
        if self.profile_enabled {
            self.profile_data.record_instruction(current_ip, &instruction);
        }

        // Debug hook: call callback before executing if in debug mode
        if self.debug_mode || self.breakpoints.contains(&current_ip) {
            if self.debug_callback.is_some() {
                let mut callback = self.debug_callback.take().unwrap();
                let result = callback(self, current_ip, &instruction);
                self.debug_callback = Some(callback);
                result?;
            }
        }

        self.ip += 1;

        let result = self.execute_one(instruction);

        if let Err(e) = result {
            return Err(self.format_error(&e, current_ip));
        }

        Ok(())
    }

    fn execute_one(&mut self, instruction: OpCode) -> Result<(), String> {
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
                // Calculate how many bytes we need to read based on dtype
                let byte_count = match dtype {
                    DataType::I8 | DataType::U8 => 1,
                    DataType::I16 | DataType::U16 => 2,
                    DataType::I32 | DataType::U32 | DataType::F32 => 4,
                    DataType::I64 | DataType::U64 | DataType::F64 | DataType::Ptr => 8,
                    DataType::Void => 0,
                };
                let bytes = self.load_bytes_from_heap(addr, byte_count)?;
                let value = self.bytes_to_value(&bytes, dtype)?;
                self.set_variable(&dest, value)?;
            }

            OpCode::Store { ptr, source, dtype } => {
                let addr = self.get_variable(&ptr)?.as_usize()?;
                let value = self.get_variable(&source)?;
                let bytes = self.value_to_bytes(&value, dtype)?;
                self.store_bytes_to_heap(addr, bytes)?;
            }

            OpCode::GetAddr { dest, var } => {
                // Simulated address - in real impl would need actual memory addresses
                let fake_addr = var
                    .bytes()
                    .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
                self.set_variable(&dest, Value::Ptr(fake_addr))?;
            }

            OpCode::Add { dest, left, right } => binary_op!(self, dest, left, right, add),
            OpCode::Sub { dest, left, right } => binary_op!(self, dest, left, right, sub),
            OpCode::Mul { dest, left, right } => binary_op!(self, dest, left, right, mul),
            OpCode::Div { dest, left, right } => binary_op!(self, dest, left, right, div),
            OpCode::Mod { dest, left, right } => binary_op!(self, dest, left, right, modulo),
            OpCode::Neg { dest, source } => unary_op!(self, dest, source, neg),

            OpCode::Eq { dest, left, right } => equality_op!(self, dest, left, right, ==),
            OpCode::Ne { dest, left, right } => equality_op!(self, dest, left, right, !=),
            OpCode::Lt { dest, left, right } => comparison_op!(self, dest, left, right, lt),
            OpCode::Le { dest, left, right } => comparison_op!(self, dest, left, right, le),
            OpCode::Gt { dest, left, right } => comparison_op!(self, dest, left, right, gt),
            OpCode::Ge { dest, left, right } => comparison_op!(self, dest, left, right, ge),

            OpCode::And { dest, left, right } => binary_op!(self, dest, left, right, bitwise_and),
            OpCode::Or { dest, left, right } => binary_op!(self, dest, left, right, bitwise_or),
            OpCode::Xor { dest, left, right } => binary_op!(self, dest, left, right, bitwise_xor),
            OpCode::Not { dest, source } => unary_op!(self, dest, source, bitwise_not),
            OpCode::Shl { dest, left, right } => binary_op!(self, dest, left, right, shift_left),
            OpCode::Shr { dest, left, right } => binary_op!(self, dest, left, right, shift_right),

            OpCode::Cast {
                dest,
                source,
                target_type,
            } => {
                let val = self.get_variable(&source)?;
                self.set_variable(&dest, val.cast(target_type)?)?;
            }

            OpCode::Input { dest } => {
                use std::io::{self, BufRead};
                print!("Enter value for {}: ", dest);
                io::stdout().flush().ok();
                let stdin = io::stdin();
                let line = stdin
                    .lock()
                    .lines()
                    .next()
                    .ok_or_else(|| "Failed to read input".to_string())?
                    .map_err(|e| format!("IO error: {}", e))?;

                // Try to parse as i32 by default
                let value = line
                    .trim()
                    .parse::<i32>()
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
                self.ip = *self
                    .program
                    .labels
                    .get(&label)
                    .ok_or_else(|| format!("Unknown label: {}", label))?;
            }

            OpCode::Jz { var, label } => {
                let val = self.get_variable(&var)?;
                if val.is_zero() {
                    self.ip = *self
                        .program
                        .labels
                        .get(&label)
                        .ok_or_else(|| format!("Unknown label: {}", label))?;
                }
            }

            OpCode::Jnz { var, label } => {
                let val = self.get_variable(&var)?;
                if !val.is_zero() {
                    self.ip = *self
                        .program
                        .labels
                        .get(&label)
                        .ok_or_else(|| format!("Unknown label: {}", label))?;
                }
            }

            OpCode::FuncBegin { name, .. } => {
                // Skip to end of function if not being called
                let func = self
                    .program
                    .functions
                    .get(&name)
                    .ok_or_else(|| format!("Unknown function: {}", name))?;
                self.ip = func.end_ip + 1;
            }

            OpCode::FuncEnd => {
                // No-op, handled by Return
            }

            OpCode::Call { result, func, args } => {
                let func_def = self
                    .program
                    .functions
                    .get(&func)
                    .ok_or_else(|| format!("Unknown function: {}", func))?
                    .clone();

                // Get argument values before switching frames
                let mut arg_values = Vec::new();
                for arg in args {
                    let val = self.resolve_operand(&arg)?;
                    arg_values.push(val);
                }

                // Push current frame with return destination
                let mut frame = std::mem::replace(
                    &mut self.current_frame,
                    CallFrame {
                        function_name: func.clone(),
                        return_ip: 0,
                        locals: HashMap::new(),
                        return_dest: None,
                        args: arg_values,
                    },
                );
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

            OpCode::Sqrt { dest, source } => {
                let val = self.resolve_operand(&source)?;
                let result = match val {
                    Value::F32(v) => Value::F32(v.sqrt()),
                    Value::F64(v) => Value::F64(v.sqrt()),
                    _ => return Err(format!("sqrt requires float type, got {:?}", val)),
                };
                self.set_variable(&dest, result)?;
            }

            OpCode::Pow { dest, base, exp } => {
                let base_val = self.resolve_operand(&base)?;
                let exp_val = self.resolve_operand(&exp)?;
                let result = match (base_val, exp_val) {
                    (Value::F32(b), Value::F32(e)) => Value::F32(b.powf(e)),
                    (Value::F64(b), Value::F64(e)) => Value::F64(b.powf(e)),
                    (Value::I32(b), Value::I32(e)) => {
                        Value::I32((b as f64).powi(e) as i32)
                    }
                    _ => return Err("pow requires matching numeric types".to_string()),
                };
                self.set_variable(&dest, result)?;
            }

            OpCode::Abs { dest, source } => {
                let val = self.resolve_operand(&source)?;
                let result = match val {
                    Value::I32(v) => Value::I32(v.abs()),
                    Value::I64(v) => Value::I64(v.abs()),
                    Value::F32(v) => Value::F32(v.abs()),
                    Value::F64(v) => Value::F64(v.abs()),
                    _ => return Err(format!("abs requires numeric type, got {:?}", val)),
                };
                self.set_variable(&dest, result)?;
            }

            OpCode::Min { dest, a, b } => {
                let a_val = self.resolve_operand(&a)?;
                let b_val = self.resolve_operand(&b)?;
                let result = match (a_val, b_val) {
                    (Value::I32(x), Value::I32(y)) => Value::I32(x.min(y)),
                    (Value::I64(x), Value::I64(y)) => Value::I64(x.min(y)),
                    (Value::F32(x), Value::F32(y)) => Value::F32(x.min(y)),
                    (Value::F64(x), Value::F64(y)) => Value::F64(x.min(y)),
                    _ => return Err("min requires matching numeric types".to_string()),
                };
                self.set_variable(&dest, result)?;
            }

            OpCode::Max { dest, a, b } => {
                let a_val = self.resolve_operand(&a)?;
                let b_val = self.resolve_operand(&b)?;
                let result = match (a_val, b_val) {
                    (Value::I32(x), Value::I32(y)) => Value::I32(x.max(y)),
                    (Value::I64(x), Value::I64(y)) => Value::I64(x.max(y)),
                    (Value::F32(x), Value::F32(y)) => Value::F32(x.max(y)),
                    (Value::F64(x), Value::F64(y)) => Value::F64(x.max(y)),
                    _ => return Err("max requires matching numeric types".to_string()),
                };
                self.set_variable(&dest, result)?;
            }

            OpCode::Sin { dest, source } => {
                let val = self.resolve_operand(&source)?;
                let result = match val {
                    Value::F32(v) => Value::F32(v.sin()),
                    Value::F64(v) => Value::F64(v.sin()),
                    _ => return Err(format!("sin requires float type, got {:?}", val)),
                };
                self.set_variable(&dest, result)?;
            }

            OpCode::Cos { dest, source } => {
                let val = self.resolve_operand(&source)?;
                let result = match val {
                    Value::F32(v) => Value::F32(v.cos()),
                    Value::F64(v) => Value::F64(v.cos()),
                    _ => return Err(format!("cos requires float type, got {:?}", val)),
                };
                self.set_variable(&dest, result)?;
            }

            OpCode::Tan { dest, source } => {
                let val = self.resolve_operand(&source)?;
                let result = match val {
                    Value::F32(v) => Value::F32(v.tan()),
                    Value::F64(v) => Value::F64(v.tan()),
                    _ => return Err(format!("tan requires float type, got {:?}", val)),
                };
                self.set_variable(&dest, result)?;
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
        self.current_frame
            .locals
            .get(name)
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

    fn find_heap_allocation(&self, addr: usize) -> Result<(usize, usize), String> {
        // Find which allocation contains this address
        // Returns (base_address, offset_within_allocation)
        for (base_addr, bytes) in &self.heap {
            let end_addr = base_addr + bytes.len();
            if addr >= *base_addr && addr < end_addr {
                return Ok((*base_addr, addr - base_addr));
            }
        }
        Err(format!("Invalid pointer: {:#x}", addr))
    }

    fn load_bytes_from_heap(&self, addr: usize, count: usize) -> Result<Vec<u8>, String> {
        let (base_addr, offset) = self.find_heap_allocation(addr)?;
        let allocation = self.heap.get(&base_addr)
            .ok_or_else(|| format!("Heap allocation not found at {:#x}", base_addr))?;

        if offset + count > allocation.len() {
            return Err(format!("Read out of bounds at {:#x}", addr));
        }

        Ok(allocation[offset..offset + count].to_vec())
    }

    fn store_bytes_to_heap(&mut self, addr: usize, bytes: Vec<u8>) -> Result<(), String> {
        let (base_addr, offset) = self.find_heap_allocation(addr)?;
        let allocation = self.heap.get_mut(&base_addr)
            .ok_or_else(|| format!("Heap allocation not found at {:#x}", base_addr))?;

        if offset + bytes.len() > allocation.len() {
            return Err(format!("Write out of bounds at {:#x}", addr));
        }

        allocation[offset..offset + bytes.len()].copy_from_slice(&bytes);
        Ok(())
    }

    fn bytes_to_value(&self, bytes: &[u8], dtype: DataType) -> Result<Value, String> {
        match dtype {
            DataType::I8 => {
                if bytes.is_empty() {
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
                Ok(Value::I32(i32::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                ])))
            }
            DataType::I64 => {
                if bytes.len() < 8 {
                    return Err("Insufficient bytes for I64".to_string());
                }
                Ok(Value::I64(i64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ])))
            }
            DataType::U8 => {
                if bytes.is_empty() {
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
                Ok(Value::U32(u32::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                ])))
            }
            DataType::U64 => {
                if bytes.len() < 8 {
                    return Err("Insufficient bytes for U64".to_string());
                }
                Ok(Value::U64(u64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ])))
            }
            DataType::F32 => {
                if bytes.len() < 4 {
                    return Err("Insufficient bytes for F32".to_string());
                }
                Ok(Value::F32(f32::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3],
                ])))
            }
            DataType::F64 => {
                if bytes.len() < 8 {
                    return Err("Insufficient bytes for F64".to_string());
                }
                Ok(Value::F64(f64::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ])))
            }
            DataType::Ptr => {
                if bytes.len() < 8 {
                    return Err("Insufficient bytes for Ptr".to_string());
                }
                Ok(Value::Ptr(usize::from_le_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
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
            _ => Err(format!(
                "Type mismatch: cannot store {:?} as {:?}",
                value, dtype
            )),
        }
    }

    fn format_error(&self, error: &str, ip: usize) -> String {
        if let Some(source_map) = &self.program.source_map {
            if let Some(location) = source_map.instruction_locations.get(&ip) {
                return format!(
                    "Runtime error at {}:{}:{}\n  {}\n",
                    source_map.file.display(),
                    location.line,
                    location.column,
                    error
                );
            }
        }

        format!("Runtime error: {}", error)
    }

    // Debug methods
    pub fn set_debug_mode(&mut self, enabled: bool) {
        self.debug_mode = enabled;
    }

    pub fn set_debug_callback(&mut self, callback: DebugCallback) {
        self.debug_callback = Some(callback);
    }

    pub fn add_breakpoint(&mut self, ip: usize) {
        self.breakpoints.insert(ip);
    }

    pub fn remove_breakpoint(&mut self, ip: usize) {
        self.breakpoints.remove(&ip);
    }

    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    pub fn get_breakpoints(&self) -> &HashSet<usize> {
        &self.breakpoints
    }

    pub fn get_ip(&self) -> usize {
        self.ip
    }

    pub fn get_program(&self) -> &Program {
        &self.program
    }

    pub fn get_globals(&self) -> &HashMap<String, Value> {
        &self.globals
    }

    pub fn get_current_frame(&self) -> &CallFrame {
        &self.current_frame
    }

    pub fn get_call_stack(&self) -> &Vec<CallFrame> {
        &self.call_stack
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    // Profiling methods
    pub fn enable_profiling(&mut self) {
        self.profile_enabled = true;
        self.profile_data.start();
    }

    pub fn disable_profiling(&mut self) {
        self.profile_enabled = false;
        self.profile_data.stop();
    }

    pub fn get_profile_data(&self) -> &ProfileData {
        &self.profile_data
    }
}
