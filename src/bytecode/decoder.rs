use crate::bytecode::{MAGIC, VERSION};
use crate::opcode::OpCode;
use crate::program::{Function, Program, Variable};
use crate::types::{DataType, Operand, Value};
use std::collections::HashMap;
use std::io::{self, Read};

pub fn decode(data: &[u8]) -> io::Result<Program> {
    let mut cursor = 0;

    let magic = read_u32(data, &mut cursor)?;
    if magic != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid magic number: expected {:x}, got {:x}", MAGIC, magic),
        ));
    }

    let version = read_u32(data, &mut cursor)?;
    if version != VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unsupported version: {}", version),
        ));
    }

    let mut program = Program::new();

    program.globals = decode_globals(data, &mut cursor)?;
    program.functions = decode_functions(data, &mut cursor)?;
    program.labels = decode_labels(data, &mut cursor)?;
    program.instructions = decode_instructions(data, &mut cursor)?;

    Ok(program)
}

fn decode_globals(data: &[u8], cursor: &mut usize) -> io::Result<Vec<Variable>> {
    let count = read_u32(data, cursor)? as usize;
    let mut globals = Vec::with_capacity(count);

    for _ in 0..count {
        let name = read_string(data, cursor)?;
        let dtype = read_datatype(data, cursor)?;
        globals.push(Variable::new(name, dtype, true));
    }

    Ok(globals)
}

fn decode_functions(
    data: &[u8],
    cursor: &mut usize,
) -> io::Result<HashMap<String, Function>> {
    let count = read_u32(data, cursor)? as usize;
    let mut functions = HashMap::new();

    for _ in 0..count {
        let name = read_string(data, cursor)?;
        let return_type = read_datatype(data, cursor)?;
        let start_ip = read_u32(data, cursor)? as usize;
        let end_ip = read_u32(data, cursor)? as usize;

        let func = Function {
            name: name.clone(),
            return_type,
            parameters: Vec::new(),
            locals: Vec::new(),
            start_ip,
            end_ip,
        };

        functions.insert(name, func);
    }

    Ok(functions)
}

fn decode_labels(data: &[u8], cursor: &mut usize) -> io::Result<HashMap<String, usize>> {
    let count = read_u32(data, cursor)? as usize;
    let mut labels = HashMap::new();

    for _ in 0..count {
        let name = read_string(data, cursor)?;
        let addr = read_u32(data, cursor)? as usize;
        labels.insert(name, addr);
    }

    Ok(labels)
}

fn decode_instructions(data: &[u8], cursor: &mut usize) -> io::Result<Vec<OpCode>> {
    let count = read_u32(data, cursor)? as usize;
    let mut instructions = Vec::with_capacity(count);

    for _ in 0..count {
        instructions.push(decode_opcode(data, cursor)?);
    }

    Ok(instructions)
}

fn decode_opcode(data: &[u8], cursor: &mut usize) -> io::Result<OpCode> {
    let opcode_id = read_u8(data, cursor)?;

    match opcode_id {
        0 => {
            let dtype = read_datatype(data, cursor)?;
            let name = read_string(data, cursor)?;
            Ok(OpCode::CreateLocal { dtype, name })
        },
        1 => {
            let dtype = read_datatype(data, cursor)?;
            let name = read_string(data, cursor)?;
            Ok(OpCode::CreateGlobal { dtype, name })
        },
        2 => {
            let name = read_string(data, cursor)?;
            Ok(OpCode::DeleteLocal { name })
        },
        3 => {
            let dest = read_string(data, cursor)?;
            let value = read_operand(data, cursor)?;
            Ok(OpCode::SetVar { dest, value })
        },
        4 => {
            let dest = read_string(data, cursor)?;
            let source = read_string(data, cursor)?;
            Ok(OpCode::CopyVar { dest, source })
        },
        10 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Add { dest, left, right })
        },
        11 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Sub { dest, left, right })
        },
        12 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Mul { dest, left, right })
        },
        13 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Div { dest, left, right })
        },
        14 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Mod { dest, left, right })
        },
        15 => {
            let dest = read_string(data, cursor)?;
            let source = read_operand(data, cursor)?;
            Ok(OpCode::Neg { dest, source })
        },
        5 => {
            let dest = read_string(data, cursor)?;
            let size = read_operand(data, cursor)?;
            Ok(OpCode::Alloc { dest, size })
        },
        6 => {
            let ptr = read_string(data, cursor)?;
            Ok(OpCode::Free { ptr })
        },
        7 => {
            let dest = read_string(data, cursor)?;
            let ptr = read_string(data, cursor)?;
            let dtype = read_datatype(data, cursor)?;
            Ok(OpCode::Load { dest, ptr, dtype })
        },
        8 => {
            let ptr = read_string(data, cursor)?;
            let source = read_string(data, cursor)?;
            let dtype = read_datatype(data, cursor)?;
            Ok(OpCode::Store { ptr, source, dtype })
        },
        9 => {
            let dest = read_string(data, cursor)?;
            let var = read_string(data, cursor)?;
            Ok(OpCode::GetAddr { dest, var })
        },
        16 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::And { dest, left, right })
        },
        17 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Or { dest, left, right })
        },
        18 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Xor { dest, left, right })
        },
        19 => {
            let dest = read_string(data, cursor)?;
            let source = read_operand(data, cursor)?;
            Ok(OpCode::Not { dest, source })
        },
        20 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Shl { dest, left, right })
        },
        21 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Shr { dest, left, right })
        },
        30 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Eq { dest, left, right })
        },
        31 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Ne { dest, left, right })
        },
        32 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Lt { dest, left, right })
        },
        33 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Le { dest, left, right })
        },
        34 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Gt { dest, left, right })
        },
        35 => {
            let dest = read_string(data, cursor)?;
            let left = read_operand(data, cursor)?;
            let right = read_operand(data, cursor)?;
            Ok(OpCode::Ge { dest, left, right })
        },
        50 => {
            let name = read_string(data, cursor)?;
            Ok(OpCode::Label { name })
        },
        51 => {
            let label = read_string(data, cursor)?;
            Ok(OpCode::Jmp { label })
        },
        52 => {
            let var = read_string(data, cursor)?;
            let label = read_string(data, cursor)?;
            Ok(OpCode::Jz { var, label })
        },
        53 => {
            let var = read_string(data, cursor)?;
            let label = read_string(data, cursor)?;
            Ok(OpCode::Jnz { var, label })
        },
        60 => {
            let name = read_string(data, cursor)?;
            let return_type = read_datatype(data, cursor)?;
            Ok(OpCode::FuncBegin { name, return_type })
        },
        61 => Ok(OpCode::FuncEnd),
        62 => {
            let has_result = read_u8(data, cursor)?;
            let result = if has_result == 1 {
                Some(read_string(data, cursor)?)
            } else {
                None
            };
            let func = read_string(data, cursor)?;
            let arg_count = read_u32(data, cursor)? as usize;
            let mut args = Vec::with_capacity(arg_count);
            for _ in 0..arg_count {
                args.push(read_operand(data, cursor)?);
            }
            Ok(OpCode::Call { result, func, args })
        },
        63 => {
            let has_value = read_u8(data, cursor)?;
            let value = if has_value == 1 {
                Some(read_operand(data, cursor)?)
            } else {
                None
            };
            Ok(OpCode::Return { value })
        },
        64 => {
            let var = read_string(data, cursor)?;
            Ok(OpCode::PushArg { var })
        },
        65 => {
            let dest = read_string(data, cursor)?;
            Ok(OpCode::PopArg { dest })
        },
        80 => {
            let dest = read_string(data, cursor)?;
            let source = read_string(data, cursor)?;
            let target_type = read_datatype(data, cursor)?;
            Ok(OpCode::Cast { dest, source, target_type })
        },
        90 => {
            let dest = read_string(data, cursor)?;
            let source = read_operand(data, cursor)?;
            Ok(OpCode::Sqrt { dest, source })
        },
        91 => {
            let dest = read_string(data, cursor)?;
            let base = read_operand(data, cursor)?;
            let exp = read_operand(data, cursor)?;
            Ok(OpCode::Pow { dest, base, exp })
        },
        92 => {
            let dest = read_string(data, cursor)?;
            let source = read_operand(data, cursor)?;
            Ok(OpCode::Abs { dest, source })
        },
        93 => {
            let dest = read_string(data, cursor)?;
            let a = read_operand(data, cursor)?;
            let b = read_operand(data, cursor)?;
            Ok(OpCode::Min { dest, a, b })
        },
        94 => {
            let dest = read_string(data, cursor)?;
            let a = read_operand(data, cursor)?;
            let b = read_operand(data, cursor)?;
            Ok(OpCode::Max { dest, a, b })
        },
        95 => {
            let dest = read_string(data, cursor)?;
            let source = read_operand(data, cursor)?;
            Ok(OpCode::Sin { dest, source })
        },
        96 => {
            let dest = read_string(data, cursor)?;
            let source = read_operand(data, cursor)?;
            Ok(OpCode::Cos { dest, source })
        },
        97 => {
            let dest = read_string(data, cursor)?;
            let source = read_operand(data, cursor)?;
            Ok(OpCode::Tan { dest, source })
        },
        70 => {
            let var = read_string(data, cursor)?;
            Ok(OpCode::Print { var })
        },
        72 => {
            let dest = read_string(data, cursor)?;
            Ok(OpCode::Input { dest })
        },
        71 => {
            let code = read_operand(data, cursor)?;
            Ok(OpCode::Exit { code })
        },
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown opcode ID: {}", opcode_id),
        )),
    }
}

fn read_operand(data: &[u8], cursor: &mut usize) -> io::Result<Operand> {
    let operand_type = read_u8(data, cursor)?;

    match operand_type {
        0 => {
            let name = read_string(data, cursor)?;
            Ok(Operand::Variable(name))
        },
        1 => {
            let value_type = read_u8(data, cursor)?;
            let value = match value_type {
                0 => {
                    let v = read_i32(data, cursor)?;
                    Value::I32(v)
                },
                1 => {
                    let v = read_i64(data, cursor)?;
                    Value::I64(v)
                },
                2 => {
                    let v = read_f32(data, cursor)?;
                    Value::F32(v)
                },
                3 => {
                    let v = read_f64(data, cursor)?;
                    Value::F64(v)
                },
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Unknown value type: {}", value_type),
                    ));
                }
            };
            Ok(Operand::Immediate(value))
        },
        2 => {
            let name = read_string(data, cursor)?;
            Ok(Operand::Label(name))
        },
        3 => {
            let dtype = read_datatype(data, cursor)?;
            Ok(Operand::Type(dtype))
        },
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Unknown operand type: {}", operand_type),
        )),
    }
}

fn read_u8(data: &[u8], cursor: &mut usize) -> io::Result<u8> {
    if *cursor >= data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected end of bytecode",
        ));
    }
    let value = data[*cursor];
    *cursor += 1;
    Ok(value)
}

fn read_u32(data: &[u8], cursor: &mut usize) -> io::Result<u32> {
    if *cursor + 4 > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected end of bytecode",
        ));
    }
    let bytes = [data[*cursor], data[*cursor + 1], data[*cursor + 2], data[*cursor + 3]];
    *cursor += 4;
    Ok(u32::from_le_bytes(bytes))
}

fn read_i32(data: &[u8], cursor: &mut usize) -> io::Result<i32> {
    if *cursor + 4 > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected end of bytecode",
        ));
    }
    let bytes = [data[*cursor], data[*cursor + 1], data[*cursor + 2], data[*cursor + 3]];
    *cursor += 4;
    Ok(i32::from_le_bytes(bytes))
}

fn read_i64(data: &[u8], cursor: &mut usize) -> io::Result<i64> {
    if *cursor + 8 > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected end of bytecode",
        ));
    }
    let bytes = [
        data[*cursor],
        data[*cursor + 1],
        data[*cursor + 2],
        data[*cursor + 3],
        data[*cursor + 4],
        data[*cursor + 5],
        data[*cursor + 6],
        data[*cursor + 7],
    ];
    *cursor += 8;
    Ok(i64::from_le_bytes(bytes))
}

fn read_f32(data: &[u8], cursor: &mut usize) -> io::Result<f32> {
    if *cursor + 4 > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected end of bytecode",
        ));
    }
    let bytes = [data[*cursor], data[*cursor + 1], data[*cursor + 2], data[*cursor + 3]];
    *cursor += 4;
    Ok(f32::from_le_bytes(bytes))
}

fn read_f64(data: &[u8], cursor: &mut usize) -> io::Result<f64> {
    if *cursor + 8 > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected end of bytecode",
        ));
    }
    let bytes = [
        data[*cursor],
        data[*cursor + 1],
        data[*cursor + 2],
        data[*cursor + 3],
        data[*cursor + 4],
        data[*cursor + 5],
        data[*cursor + 6],
        data[*cursor + 7],
    ];
    *cursor += 8;
    Ok(f64::from_le_bytes(bytes))
}

fn read_string(data: &[u8], cursor: &mut usize) -> io::Result<String> {
    let len = read_u32(data, cursor)? as usize;
    if *cursor + len > data.len() {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "Unexpected end of bytecode",
        ));
    }
    let bytes = &data[*cursor..*cursor + len];
    *cursor += len;
    String::from_utf8(bytes.to_vec()).map_err(|e| {
        io::Error::new(io::ErrorKind::InvalidData, format!("Invalid UTF-8: {}", e))
    })
}

fn read_datatype(data: &[u8], cursor: &mut usize) -> io::Result<DataType> {
    let dtype_id = read_u8(data, cursor)?;
    Ok(match dtype_id {
        0 => DataType::I8,
        1 => DataType::I16,
        2 => DataType::I32,
        3 => DataType::I64,
        4 => DataType::U8,
        5 => DataType::U16,
        6 => DataType::U32,
        7 => DataType::U64,
        8 => DataType::F32,
        9 => DataType::F64,
        10 => DataType::Ptr,
        11 => DataType::Void,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown data type: {}", dtype_id),
            ));
        }
    })
}
