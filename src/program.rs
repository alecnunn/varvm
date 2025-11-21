use crate::opcode::OpCode;
use crate::types::{DataType, Operand};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub dtype: DataType,
    pub is_global: bool,
    pub offset: usize,
    pub size: usize,
}

impl Variable {
    pub fn new(name: String, dtype: DataType, is_global: bool) -> Self {
        let size = Self::size_of(dtype);
        Self {
            name,
            dtype,
            is_global,
            offset: 0,
            size,
        }
    }

    fn size_of(dtype: DataType) -> usize {
        match dtype {
            DataType::I8 | DataType::U8 => 1,
            DataType::I16 | DataType::U16 => 2,
            DataType::I32 | DataType::U32 | DataType::F32 => 4,
            DataType::I64 | DataType::U64 | DataType::F64 | DataType::Ptr => 8,
            DataType::Void => 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: DataType,
    pub parameters: Vec<Variable>,
    pub locals: Vec<Variable>,
    pub start_ip: usize,
    pub end_ip: usize,
}

impl Function {
    pub fn new(name: String, return_type: DataType) -> Self {
        Self {
            name,
            return_type,
            parameters: Vec::new(),
            locals: Vec::new(),
            start_ip: 0,
            end_ip: 0,
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub instructions: Vec<OpCode>,
    pub globals: Vec<Variable>,
    pub functions: HashMap<String, Function>,
    pub labels: HashMap<String, usize>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            globals: Vec::new(),
            functions: HashMap::new(),
            labels: HashMap::new(),
        }
    }

    pub fn emit(&mut self, opcode: OpCode) -> usize {
        let ip = self.instructions.len();
        if let OpCode::Label { ref name } = opcode {
            self.labels.insert(name.clone(), ip);
        }
        self.instructions.push(opcode);
        ip
    }

    pub fn add_global(&mut self, var: Variable) {
        self.globals.push(var);
    }

    pub fn add_function(&mut self, func: Function) {
        self.functions.insert(func.name.clone(), func);
    }
}

impl Program {
    pub fn create_local(&mut self, dtype: DataType, name: &str) {
        self.emit(OpCode::CreateLocal {
            dtype,
            name: name.to_string(),
        });
    }

    pub fn set_var(&mut self, dest: &str, value: impl Into<Operand>) {
        self.emit(OpCode::SetVar {
            dest: dest.to_string(),
            value: value.into(),
        });
    }

    pub fn call(&mut self, result: Option<&str>, func: &str, args: &[&str]) {
        self.emit(OpCode::Call {
            result: result.map(String::from),
            func: func.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
        });
    }
}
