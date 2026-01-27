use crate::bytecode::{MAGIC, VERSION};
use crate::opcode::OpCode;
use crate::program::{Function, Program, Variable};
use crate::types::DataType;
use std::io::{self, Write};

pub fn encode(program: &Program) -> io::Result<Vec<u8>> {
    let mut buffer = Vec::new();

    buffer.write_all(&MAGIC.to_le_bytes())?;
    buffer.write_all(&VERSION.to_le_bytes())?;

    encode_globals(&mut buffer, &program.globals)?;
    encode_functions(&mut buffer, &program.functions)?;
    encode_labels(&mut buffer, &program.labels)?;
    encode_instructions(&mut buffer, &program.instructions)?;

    Ok(buffer)
}

fn encode_globals(buffer: &mut Vec<u8>, globals: &[Variable]) -> io::Result<()> {
    buffer.write_all(&(globals.len() as u32).to_le_bytes())?;

    for global in globals {
        encode_string(buffer, &global.name)?;
        buffer.write_all(&(global.dtype as u8).to_le_bytes())?;
    }

    Ok(())
}

fn encode_functions(
    buffer: &mut Vec<u8>,
    functions: &std::collections::HashMap<String, Function>,
) -> io::Result<()> {
    buffer.write_all(&(functions.len() as u32).to_le_bytes())?;

    for (name, func) in functions {
        encode_string(buffer, name)?;
        buffer.write_all(&(func.return_type as u8).to_le_bytes())?;
        buffer.write_all(&(func.start_ip as u32).to_le_bytes())?;
        buffer.write_all(&(func.end_ip as u32).to_le_bytes())?;
    }

    Ok(())
}

fn encode_labels(
    buffer: &mut Vec<u8>,
    labels: &std::collections::HashMap<String, usize>,
) -> io::Result<()> {
    buffer.write_all(&(labels.len() as u32).to_le_bytes())?;

    for (name, addr) in labels {
        encode_string(buffer, name)?;
        buffer.write_all(&(*addr as u32).to_le_bytes())?;
    }

    Ok(())
}

fn encode_instructions(buffer: &mut Vec<u8>, instructions: &[OpCode]) -> io::Result<()> {
    buffer.write_all(&(instructions.len() as u32).to_le_bytes())?;

    for instr in instructions {
        encode_opcode(buffer, instr)?;
    }

    Ok(())
}

fn encode_opcode(buffer: &mut Vec<u8>, opcode: &OpCode) -> io::Result<()> {
    match opcode {
        OpCode::CreateLocal { dtype, name } => {
            buffer.write_all(&[0])?;
            buffer.write_all(&(*dtype as u8).to_le_bytes())?;
            encode_string(buffer, name)?;
        },
        OpCode::CreateGlobal { dtype, name } => {
            buffer.write_all(&[1])?;
            buffer.write_all(&(*dtype as u8).to_le_bytes())?;
            encode_string(buffer, name)?;
        },
        OpCode::DeleteLocal { name } => {
            buffer.write_all(&[2])?;
            encode_string(buffer, name)?;
        },
        OpCode::SetVar { dest, value } => {
            buffer.write_all(&[3])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, value)?;
        },
        OpCode::CopyVar { dest, source } => {
            buffer.write_all(&[4])?;
            encode_string(buffer, dest)?;
            encode_string(buffer, source)?;
        },
        OpCode::Add { dest, left, right } => {
            buffer.write_all(&[10])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Sub { dest, left, right } => {
            buffer.write_all(&[11])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Mul { dest, left, right } => {
            buffer.write_all(&[12])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Div { dest, left, right } => {
            buffer.write_all(&[13])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Mod { dest, left, right } => {
            buffer.write_all(&[14])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Neg { dest, source } => {
            buffer.write_all(&[15])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, source)?;
        },
        OpCode::Alloc { dest, size } => {
            buffer.write_all(&[5])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, size)?;
        },
        OpCode::Free { ptr } => {
            buffer.write_all(&[6])?;
            encode_string(buffer, ptr)?;
        },
        OpCode::Load { dest, ptr, dtype } => {
            buffer.write_all(&[7])?;
            encode_string(buffer, dest)?;
            encode_string(buffer, ptr)?;
            buffer.write_all(&(*dtype as u8).to_le_bytes())?;
        },
        OpCode::Store { ptr, source, dtype } => {
            buffer.write_all(&[8])?;
            encode_string(buffer, ptr)?;
            encode_string(buffer, source)?;
            buffer.write_all(&(*dtype as u8).to_le_bytes())?;
        },
        OpCode::GetAddr { dest, var } => {
            buffer.write_all(&[9])?;
            encode_string(buffer, dest)?;
            encode_string(buffer, var)?;
        },
        OpCode::And { dest, left, right } => {
            buffer.write_all(&[16])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Or { dest, left, right } => {
            buffer.write_all(&[17])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Xor { dest, left, right } => {
            buffer.write_all(&[18])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Not { dest, source } => {
            buffer.write_all(&[19])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, source)?;
        },
        OpCode::Shl { dest, left, right } => {
            buffer.write_all(&[20])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Shr { dest, left, right } => {
            buffer.write_all(&[21])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Eq { dest, left, right } => {
            buffer.write_all(&[30])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Ne { dest, left, right } => {
            buffer.write_all(&[31])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Lt { dest, left, right } => {
            buffer.write_all(&[32])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Le { dest, left, right } => {
            buffer.write_all(&[33])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Gt { dest, left, right } => {
            buffer.write_all(&[34])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Ge { dest, left, right } => {
            buffer.write_all(&[35])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, left)?;
            encode_operand(buffer, right)?;
        },
        OpCode::Label { name } => {
            buffer.write_all(&[50])?;
            encode_string(buffer, name)?;
        },
        OpCode::Jmp { label } => {
            buffer.write_all(&[51])?;
            encode_string(buffer, label)?;
        },
        OpCode::Jz { var, label } => {
            buffer.write_all(&[52])?;
            encode_string(buffer, var)?;
            encode_string(buffer, label)?;
        },
        OpCode::Jnz { var, label } => {
            buffer.write_all(&[53])?;
            encode_string(buffer, var)?;
            encode_string(buffer, label)?;
        },
        OpCode::FuncBegin { name, return_type } => {
            buffer.write_all(&[60])?;
            encode_string(buffer, name)?;
            buffer.write_all(&(*return_type as u8).to_le_bytes())?;
        },
        OpCode::FuncEnd => {
            buffer.write_all(&[61])?;
        },
        OpCode::Call { result, func, args } => {
            buffer.write_all(&[62])?;
            match result {
                Some(r) => {
                    buffer.write_all(&[1])?;
                    encode_string(buffer, r)?;
                },
                None => {
                    buffer.write_all(&[0])?;
                }
            }
            encode_string(buffer, func)?;
            buffer.write_all(&(args.len() as u32).to_le_bytes())?;
            for arg in args {
                encode_operand(buffer, arg)?;
            }
        },
        OpCode::Return { value } => {
            buffer.write_all(&[63])?;
            match value {
                Some(v) => {
                    buffer.write_all(&[1])?;
                    encode_operand(buffer, v)?;
                },
                None => {
                    buffer.write_all(&[0])?;
                }
            }
        },
        OpCode::PushArg { var } => {
            buffer.write_all(&[64])?;
            encode_string(buffer, var)?;
        },
        OpCode::PopArg { dest } => {
            buffer.write_all(&[65])?;
            encode_string(buffer, dest)?;
        },
        OpCode::Cast { dest, source, target_type } => {
            buffer.write_all(&[80])?;
            encode_string(buffer, dest)?;
            encode_string(buffer, source)?;
            buffer.write_all(&(*target_type as u8).to_le_bytes())?;
        },
        OpCode::Sqrt { dest, source } => {
            buffer.write_all(&[90])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, source)?;
        },
        OpCode::Pow { dest, base, exp } => {
            buffer.write_all(&[91])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, base)?;
            encode_operand(buffer, exp)?;
        },
        OpCode::Abs { dest, source } => {
            buffer.write_all(&[92])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, source)?;
        },
        OpCode::Min { dest, a, b } => {
            buffer.write_all(&[93])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, a)?;
            encode_operand(buffer, b)?;
        },
        OpCode::Max { dest, a, b } => {
            buffer.write_all(&[94])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, a)?;
            encode_operand(buffer, b)?;
        },
        OpCode::Sin { dest, source } => {
            buffer.write_all(&[95])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, source)?;
        },
        OpCode::Cos { dest, source } => {
            buffer.write_all(&[96])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, source)?;
        },
        OpCode::Tan { dest, source } => {
            buffer.write_all(&[97])?;
            encode_string(buffer, dest)?;
            encode_operand(buffer, source)?;
        },
        OpCode::Print { var } => {
            buffer.write_all(&[70])?;
            encode_string(buffer, var)?;
        },
        OpCode::Input { dest } => {
            buffer.write_all(&[72])?;
            encode_string(buffer, dest)?;
        },
        OpCode::Exit { code } => {
            buffer.write_all(&[71])?;
            encode_operand(buffer, code)?;
        }
    }

    Ok(())
}

fn encode_operand(buffer: &mut Vec<u8>, operand: &crate::types::Operand) -> io::Result<()> {
    use crate::types::{Operand, Value};

    match operand {
        Operand::Variable(name) => {
            buffer.write_all(&[0])?;
            encode_string(buffer, name)?;
        },
        Operand::Immediate(value) => {
            buffer.write_all(&[1])?;
            match value {
                Value::I32(v) => {
                    buffer.write_all(&[0])?;
                    buffer.write_all(&v.to_le_bytes())?;
                },
                Value::I64(v) => {
                    buffer.write_all(&[1])?;
                    buffer.write_all(&v.to_le_bytes())?;
                },
                Value::F32(v) => {
                    buffer.write_all(&[2])?;
                    buffer.write_all(&v.to_le_bytes())?;
                },
                Value::F64(v) => {
                    buffer.write_all(&[3])?;
                    buffer.write_all(&v.to_le_bytes())?;
                },
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unsupported value type in bytecode",
                    ));
                }
            }
        },
        Operand::Label(name) => {
            buffer.write_all(&[2])?;
            encode_string(buffer, name)?;
        },
        Operand::Type(dtype) => {
            buffer.write_all(&[3])?;
            buffer.write_all(&(*dtype as u8).to_le_bytes())?;
        },
    }

    Ok(())
}

fn encode_string(buffer: &mut Vec<u8>, s: &str) -> io::Result<()> {
    let bytes = s.as_bytes();
    buffer.write_all(&(bytes.len() as u32).to_le_bytes())?;
    buffer.write_all(bytes)?;
    Ok(())
}

fn encode_datatype(_dtype: DataType) -> u8 {
    _dtype as u8
}
