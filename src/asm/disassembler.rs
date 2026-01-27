use crate::opcode::OpCode;
use crate::program::Program;
use crate::types::{DataType, Operand, Value};

pub fn disassemble(program: &Program) -> String {
    let mut output = String::new();
    let mut disasm = Disassembler::new(program);

    output.push_str(&disasm.disassemble_data_section());
    output.push_str("\n");
    output.push_str(&disasm.disassemble_text_section());

    output
}

struct Disassembler<'a> {
    program: &'a Program,
    current_function: Option<String>,
}

impl<'a> Disassembler<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            program,
            current_function: None,
        }
    }

    fn disassemble_data_section(&self) -> String {
        if self.program.globals.is_empty() {
            return String::new();
        }

        let mut output = String::from("section .data\n");

        for global in &self.program.globals {
            output.push_str(&format!(
                "    {}: {}\n",
                global.name,
                self.format_datatype(global.dtype)
            ));
        }

        output
    }

    fn disassemble_text_section(&mut self) -> String {
        let mut output = String::from("section .text\n");

        for (idx, instr) in self.program.instructions.iter().enumerate() {
            let line = self.disassemble_instruction(instr, idx);
            if !line.is_empty() {
                output.push_str(&line);
                output.push('\n');
            }
        }

        output
    }

    fn disassemble_instruction(&mut self, instr: &OpCode, _idx: usize) -> String {
        match instr {
            OpCode::Label { name } => {
                if name.starts_with('.') {
                    format!("{}:", name)
                } else {
                    format!("\n{}:", name)
                }
            },
            OpCode::FuncBegin { name, return_type } => {
                self.current_function = Some(name.clone());
                format!("    func_begin {}", self.format_datatype(*return_type))
            },
            OpCode::FuncEnd => {
                self.current_function = None;
                String::from("    func_end")
            },
            OpCode::CreateLocal { dtype, name } => {
                format!("    local {}: {}", name, self.format_datatype(*dtype))
            },
            OpCode::CreateGlobal { dtype, name } => {
                format!("    global {}: {}", name, self.format_datatype(*dtype))
            },
            OpCode::DeleteLocal { name } => {
                format!("    delete_local {}", name)
            },
            OpCode::SetVar { dest, value } => {
                format!("    set {}, {}", dest, self.format_operand(value))
            },
            OpCode::CopyVar { dest, source } => {
                format!("    copy {}, {}", dest, source)
            },
            OpCode::Add { dest, left, right } => {
                format!("    add {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Sub { dest, left, right } => {
                format!("    sub {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Mul { dest, left, right } => {
                format!("    mul {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Div { dest, left, right } => {
                format!("    div {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Mod { dest, left, right } => {
                format!("    mod {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Neg { dest, source } => {
                format!("    neg {}, {}", dest, self.format_operand(source))
            },
            OpCode::And { dest, left, right } => {
                format!("    and {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Or { dest, left, right } => {
                format!("    or {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Xor { dest, left, right } => {
                format!("    xor {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Not { dest, source } => {
                format!("    not {}, {}", dest, self.format_operand(source))
            },
            OpCode::Shl { dest, left, right } => {
                format!("    shl {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Shr { dest, left, right } => {
                format!("    shr {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Eq { dest, left, right } => {
                format!("    eq {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Ne { dest, left, right } => {
                format!("    ne {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Lt { dest, left, right } => {
                format!("    lt {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Le { dest, left, right } => {
                format!("    le {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Gt { dest, left, right } => {
                format!("    gt {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Ge { dest, left, right } => {
                format!("    ge {}, {}, {}", dest, self.format_operand(left), self.format_operand(right))
            },
            OpCode::Jmp { label } => {
                format!("    jmp {}", label)
            },
            OpCode::Jz { var, label } => {
                format!("    jz {}, {}", var, label)
            },
            OpCode::Jnz { var, label } => {
                format!("    jnz {}, {}", var, label)
            },
            OpCode::Call { result, func, args } => {
                let result_str = result.as_ref().map(|s| s.as_str()).unwrap_or("_");
                let args_str = args.iter()
                    .map(|arg| self.format_operand(arg))
                    .collect::<Vec<_>>()
                    .join(", ");
                if args.is_empty() {
                    format!("    call {}, {}", result_str, func)
                } else {
                    format!("    call {}, {}, {}", result_str, func, args_str)
                }
            },
            OpCode::Return { value } => {
                if let Some(v) = value {
                    format!("    ret {}", self.format_operand(v))
                } else {
                    String::from("    ret")
                }
            },
            OpCode::PushArg { var } => {
                format!("    push_arg {}", var)
            },
            OpCode::PopArg { dest } => {
                format!("    pop_arg {}", dest)
            },
            OpCode::Alloc { dest, size } => {
                format!("    alloc {}, {}", dest, self.format_operand(size))
            },
            OpCode::Free { ptr } => {
                format!("    free {}", ptr)
            },
            OpCode::Load { dest, ptr, dtype } => {
                format!("    load {}, {}, {}", dest, ptr, self.format_datatype(*dtype))
            },
            OpCode::Store { ptr, source, dtype } => {
                format!("    store {}, {}, {}", ptr, source, self.format_datatype(*dtype))
            },
            OpCode::GetAddr { dest, var } => {
                format!("    get_addr {}, {}", dest, var)
            },
            OpCode::Cast { dest, source, target_type } => {
                format!("    cast {}, {}, {}", dest, source, self.format_datatype(*target_type))
            },
            OpCode::Sqrt { dest, source } => {
                format!("    sqrt {}, {}", dest, self.format_operand(source))
            },
            OpCode::Pow { dest, base, exp } => {
                format!("    pow {}, {}, {}", dest, self.format_operand(base), self.format_operand(exp))
            },
            OpCode::Abs { dest, source } => {
                format!("    abs {}, {}", dest, self.format_operand(source))
            },
            OpCode::Min { dest, a, b } => {
                format!("    min {}, {}, {}", dest, self.format_operand(a), self.format_operand(b))
            },
            OpCode::Max { dest, a, b } => {
                format!("    max {}, {}, {}", dest, self.format_operand(a), self.format_operand(b))
            },
            OpCode::Sin { dest, source } => {
                format!("    sin {}, {}", dest, self.format_operand(source))
            },
            OpCode::Cos { dest, source } => {
                format!("    cos {}, {}", dest, self.format_operand(source))
            },
            OpCode::Tan { dest, source } => {
                format!("    tan {}, {}", dest, self.format_operand(source))
            },
            OpCode::Print { var } => {
                format!("    print {}", var)
            },
            OpCode::Input { dest } => {
                format!("    input {}", dest)
            },
            OpCode::Exit { code } => {
                format!("    exit {}", self.format_operand(code))
            },
        }
    }

    fn format_operand(&self, operand: &Operand) -> String {
        match operand {
            Operand::Variable(name) => name.clone(),
            Operand::Immediate(value) => self.format_value(value),
            Operand::Label(name) => name.clone(),
            Operand::Type(dtype) => self.format_datatype(*dtype),
        }
    }

    fn format_value(&self, value: &Value) -> String {
        match value {
            Value::I8(v) => v.to_string(),
            Value::I16(v) => v.to_string(),
            Value::I32(v) => v.to_string(),
            Value::I64(v) => v.to_string(),
            Value::U8(v) => v.to_string(),
            Value::U16(v) => v.to_string(),
            Value::U32(v) => v.to_string(),
            Value::U64(v) => v.to_string(),
            Value::F32(v) => {
                if v.fract() == 0.0 {
                    format!("{}.0", v)
                } else {
                    v.to_string()
                }
            },
            Value::F64(v) => {
                if v.fract() == 0.0 {
                    format!("{}.0", v)
                } else {
                    v.to_string()
                }
            },
            Value::Ptr(v) => format!("0x{:x}", v),
        }
    }

    fn format_datatype(&self, dtype: DataType) -> String {
        match dtype {
            DataType::I8 => "i8".to_string(),
            DataType::I16 => "i16".to_string(),
            DataType::I32 => "i32".to_string(),
            DataType::I64 => "i64".to_string(),
            DataType::U8 => "u8".to_string(),
            DataType::U16 => "u16".to_string(),
            DataType::U32 => "u32".to_string(),
            DataType::U64 => "u64".to_string(),
            DataType::F32 => "f32".to_string(),
            DataType::F64 => "f64".to_string(),
            DataType::Ptr => "ptr".to_string(),
            DataType::Void => "void".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asm::assemble;

    #[test]
    fn test_disassemble_simple() {
        let source = r#"
section .data
    result: i32

section .text

main:
    func_begin i32
    local n: i32
    set n, 5
    print n
    ret 0
    func_end
"#;

        let program = assemble(source, "test.vasm".to_string()).unwrap();
        let disasm = disassemble(&program);

        assert!(disasm.contains("section .data"));
        assert!(disasm.contains("result: i32"));
        assert!(disasm.contains("main:"));
        assert!(disasm.contains("func_begin i32"));
        assert!(disasm.contains("local n: i32"));
        assert!(disasm.contains("set n, 5"));
        assert!(disasm.contains("print n"));
        assert!(disasm.contains("ret 0"));
        assert!(disasm.contains("func_end"));
    }
}
