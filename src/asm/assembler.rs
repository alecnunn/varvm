use crate::asm::ast::{AsmProgram, Immediate, Operand as AsmOperand, Statement};
use crate::asm::error::AsmError;
use crate::asm::lexer::Lexer;
use crate::asm::parser::Parser;
use crate::opcode::OpCode;
use crate::program::{Function, Program, SourceLocation, SourceMap, Variable};
use crate::types::{DataType, Operand, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub fn assemble(source: &str, filename: String) -> Result<Program, AsmError> {
    let mut included_files = HashSet::new();
    let base_path = Path::new(&filename)
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let ast = parse_with_includes(source, &filename, &base_path, &mut included_files)?;

    let mut assembler = Assembler::new(filename);
    assembler.assemble_program(ast)
}

fn parse_with_includes(
    source: &str,
    filename: &str,
    base_path: &Path,
    included_files: &mut HashSet<String>,
) -> Result<AsmProgram, AsmError> {
    // Check for circular includes
    let canonical_path = PathBuf::from(filename)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(filename))
        .to_string_lossy()
        .to_string();

    if included_files.contains(&canonical_path) {
        return Err(AsmError::ParseError {
            message: format!("Circular include detected: {}", filename),
            location: None,
        });
    }
    included_files.insert(canonical_path);

    // Parse the current file
    let mut lexer = Lexer::new(source, filename.to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let mut ast = parser.parse()?;

    // Process includes
    let includes = ast.includes.clone();
    ast.includes.clear(); // Clear to avoid re-processing

    for include_path in includes {
        let included_ast = resolve_and_parse_include(&include_path, base_path, included_files)?;
        ast.merge(included_ast);
    }

    Ok(ast)
}

fn resolve_and_parse_include(
    path: &str,
    base_path: &Path,
    included_files: &mut HashSet<String>,
) -> Result<AsmProgram, AsmError> {
    // First, check if it's a stdlib reference
    if let Some(stdlib_content) = crate::stdlib::get_stdlib(path) {
        // Parse the stdlib content
        // Use a virtual filename for stdlib
        let virtual_filename = format!("stdlib:{}", path);
        return parse_with_includes(stdlib_content, &virtual_filename, base_path, included_files);
    }

    // Otherwise, try to read from filesystem
    let full_path = base_path.join(path);

    let source = fs::read_to_string(&full_path).map_err(|e| {
        AsmError::ParseError {
            message: format!("Failed to read include file '{}': {}", path, e),
            location: None,
        }
    })?;

    let filename = full_path.to_string_lossy().to_string();
    let new_base_path = full_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| base_path.to_path_buf());

    parse_with_includes(&source, &filename, &new_base_path, included_files)
}

struct Assembler {
    program: Program,
    filename: String,
    label_positions: HashMap<String, usize>,
    function_starts: HashMap<String, usize>,
    current_function: Option<String>,
    instruction_locations: HashMap<usize, SourceLocation>,
    defines: HashMap<String, crate::asm::ast::DefineValue>,
}

impl Assembler {
    fn new(filename: String) -> Self {
        Self {
            program: Program::new(),
            filename,
            label_positions: HashMap::new(),
            function_starts: HashMap::new(),
            current_function: None,
            instruction_locations: HashMap::new(),
            defines: HashMap::new(),
        }
    }

    // Qualify local labels (starting with '.') with the current function name
    fn qualify_label(&self, label: &str) -> String {
        if label.starts_with('.') {
            if let Some(func) = &self.current_function {
                format!("{}:{}", func, label)
            } else {
                label.to_string()
            }
        } else {
            label.to_string()
        }
    }

    fn assemble_program(&mut self, ast: AsmProgram) -> Result<Program, AsmError> {
        // Store defines for later substitution
        for define in ast.defines {
            self.defines.insert(define.name, define.value);
        }

        // Process data section
        for decl in ast.data_section {
            // Check if this is a string literal
            if let Some(Immediate::String(ref content)) = decl.value {
                // Add string to program's string pool
                self.program.add_string(content.clone(), decl.name.clone());
            }

            // Add global variable (Ptr type for strings)
            let var = Variable::new(decl.name.clone(), decl.dtype, true);
            self.program.add_global(var);
        }

        for statement in ast.text_section {
            self.assemble_statement(statement)?;
        }

        self.resolve_labels()?;

        let source_map = SourceMap {
            file: PathBuf::from(self.filename.clone()),
            instruction_locations: self.instruction_locations.clone(),
        };
        self.program.source_map = Some(source_map);

        Ok(self.program.clone())
    }

    fn assemble_statement(&mut self, statement: Statement) -> Result<(), AsmError> {
        match statement {
            Statement::Label(name) => {
                let qualified_name = self.qualify_label(&name);
                let ip = self.program.instructions.len();
                self.label_positions.insert(qualified_name.clone(), ip);
                self.program.emit(OpCode::Label { name: qualified_name });
            },
            Statement::LocalDecl(decl) => {
                self.program.emit(OpCode::CreateLocal {
                    dtype: decl.dtype,
                    name: decl.name,
                });
            },
            Statement::Instruction(instr) => {
                let ip = self.program.instructions.len();
                let loc = SourceLocation {
                    line: instr.location.line,
                    column: instr.location.column,
                    snippet: String::new(),
                };
                self.instruction_locations.insert(ip, loc);
                self.assemble_instruction(instr)?;
            },
        }
        Ok(())
    }

    fn assemble_instruction(
        &mut self,
        instr: crate::asm::ast::Instruction,
    ) -> Result<(), AsmError> {
        let opcode_name = instr.opcode.to_lowercase();

        match opcode_name.as_str() {
            "func_begin" => {
                if instr.operands.len() != 1 {
                    return Err(AsmError::AssemblyError {
                        message: format!(
                            "func_begin expects 1 operand (return type), got {}",
                            instr.operands.len()
                        ),
                        location: None,
                    });
                }

                let func_name = if let Some(prev_label) = self.last_label() {
                    prev_label
                } else {
                    return Err(AsmError::AssemblyError {
                        message: "func_begin must follow a label".to_string(),
                        location: None,
                    });
                };

                let return_type = self.operand_to_datatype(&instr.operands[0])?;

                self.current_function = Some(func_name.clone());
                self.function_starts
                    .insert(func_name.clone(), self.program.instructions.len());

                self.program.emit(OpCode::FuncBegin {
                    name: func_name,
                    return_type,
                });
            },
            "func_end" => {
                if let Some(func_name) = &self.current_function {
                    let start_ip = *self.function_starts.get(func_name).unwrap();
                    let end_ip = self.program.instructions.len();

                    let func = Function {
                        name: func_name.clone(),
                        return_type: DataType::I32,
                        parameters: Vec::new(),
                        locals: Vec::new(),
                        start_ip,
                        end_ip,
                    };
                    self.program.add_function(func);
                    self.current_function = None;
                }

                self.program.emit(OpCode::FuncEnd);
            },
            "set" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("set expects 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let dest = self.operand_to_string(&instr.operands[0])?;
                let value = self.operand_to_operand(&instr.operands[1])?;

                self.program.emit(OpCode::SetVar { dest, value });
            },
            "copy" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("copy expects 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let dest = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_string(&instr.operands[1])?;

                self.program.emit(OpCode::CopyVar { dest, source });
            },
            "add" | "sub" | "mul" | "div" | "mod" | "and" | "or" | "xor" | "shl" | "shr"
            | "eq" | "ne" | "lt" | "le" | "gt" | "ge" => {
                if instr.operands.len() != 3 {
                    return Err(AsmError::AssemblyError {
                        message: format!(
                            "{} expects 3 operands, got {}",
                            opcode_name,
                            instr.operands.len()
                        ),
                        location: None,
                    });
                }

                let dest = self.operand_to_string(&instr.operands[0])?;
                let left = self.operand_to_operand(&instr.operands[1])?;
                let right = self.operand_to_operand(&instr.operands[2])?;

                let opcode = match opcode_name.as_str() {
                    "add" => OpCode::Add { dest, left, right },
                    "sub" => OpCode::Sub { dest, left, right },
                    "mul" => OpCode::Mul { dest, left, right },
                    "div" => OpCode::Div { dest, left, right },
                    "mod" => OpCode::Mod { dest, left, right },
                    "and" => OpCode::And { dest, left, right },
                    "or" => OpCode::Or { dest, left, right },
                    "xor" => OpCode::Xor { dest, left, right },
                    "shl" => OpCode::Shl { dest, left, right },
                    "shr" => OpCode::Shr { dest, left, right },
                    "eq" => OpCode::Eq { dest, left, right },
                    "ne" => OpCode::Ne { dest, left, right },
                    "lt" => OpCode::Lt { dest, left, right },
                    "le" => OpCode::Le { dest, left, right },
                    "gt" => OpCode::Gt { dest, left, right },
                    "ge" => OpCode::Ge { dest, left, right },
                    _ => unreachable!(),
                };

                self.program.emit(opcode);
            },
            "neg" | "not" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!(
                            "{} expects 2 operands, got {}",
                            opcode_name,
                            instr.operands.len()
                        ),
                        location: None,
                    });
                }

                let dest = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_operand(&instr.operands[1])?;

                let opcode = match opcode_name.as_str() {
                    "neg" => OpCode::Neg { dest, source },
                    "not" => OpCode::Not { dest, source },
                    _ => unreachable!(),
                };

                self.program.emit(opcode);
            },
            "jmp" => {
                if instr.operands.len() != 1 {
                    return Err(AsmError::AssemblyError {
                        message: format!("jmp expects 1 operand, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let label = self.operand_to_string(&instr.operands[0])?;
                let qualified_label = self.qualify_label(&label);
                self.program.emit(OpCode::Jmp { label: qualified_label });
            },
            "jz" | "jnz" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!(
                            "{} expects 2 operands, got {}",
                            opcode_name,
                            instr.operands.len()
                        ),
                        location: None,
                    });
                }

                let var = self.operand_to_string(&instr.operands[0])?;
                let label = self.operand_to_string(&instr.operands[1])?;
                let qualified_label = self.qualify_label(&label);

                let opcode = match opcode_name.as_str() {
                    "jz" => OpCode::Jz { var, label: qualified_label },
                    "jnz" => OpCode::Jnz { var, label: qualified_label },
                    _ => unreachable!(),
                };

                self.program.emit(opcode);
            },
            "call" => {
                if instr.operands.len() < 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("call expects at least 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let result = Some(self.operand_to_string(&instr.operands[0])?);
                let func = self.operand_to_string(&instr.operands[1])?;
                let args = instr.operands[2..]
                    .iter()
                    .map(|op| self.operand_to_operand(op))
                    .collect::<Result<Vec<_>, _>>()?;

                self.program.emit(OpCode::Call { result, func, args });
            },
            "ret" => {
                let value = if instr.operands.len() > 0 {
                    Some(self.operand_to_operand(&instr.operands[0])?)
                } else {
                    None
                };

                self.program.emit(OpCode::Return { value });
            },
            "pop_arg" => {
                if instr.operands.len() != 1 {
                    return Err(AsmError::AssemblyError {
                        message: format!("pop_arg expects 1 operand, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let dest = self.operand_to_string(&instr.operands[0])?;
                self.program.emit(OpCode::PopArg { dest });
            },
            "push_arg" => {
                if instr.operands.len() != 1 {
                    return Err(AsmError::AssemblyError {
                        message: format!("push_arg expects 1 operand, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let var = self.operand_to_string(&instr.operands[0])?;
                self.program.emit(OpCode::PushArg { var });
            },
            "print" => {
                if instr.operands.len() != 1 {
                    return Err(AsmError::AssemblyError {
                        message: format!("print expects 1 operand, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let var = self.operand_to_string(&instr.operands[0])?;
                self.program.emit(OpCode::Print { var });
            },
            "alloc" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("alloc expects 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let dest = self.operand_to_string(&instr.operands[0])?;
                let size = self.operand_to_operand(&instr.operands[1])?;

                self.program.emit(OpCode::Alloc { dest, size });
            },
            "free" => {
                if instr.operands.len() != 1 {
                    return Err(AsmError::AssemblyError {
                        message: format!("free expects 1 operand, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let ptr = self.operand_to_string(&instr.operands[0])?;
                self.program.emit(OpCode::Free { ptr });
            },
            "load" => {
                if instr.operands.len() != 3 {
                    return Err(AsmError::AssemblyError {
                        message: format!("load expects 3 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let dest = self.operand_to_string(&instr.operands[0])?;
                let ptr = self.operand_to_string(&instr.operands[1])?;
                let dtype = self.operand_to_datatype(&instr.operands[2])?;

                self.program.emit(OpCode::Load { dest, ptr, dtype });
            },
            "store" => {
                if instr.operands.len() != 3 {
                    return Err(AsmError::AssemblyError {
                        message: format!("store expects 3 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let ptr = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_string(&instr.operands[1])?;
                let dtype = self.operand_to_datatype(&instr.operands[2])?;

                self.program.emit(OpCode::Store { ptr, source, dtype });
            },
            "cast" => {
                if instr.operands.len() != 3 {
                    return Err(AsmError::AssemblyError {
                        message: format!("cast expects 3 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }

                let dest = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_string(&instr.operands[1])?;
                let target_type = self.operand_to_datatype(&instr.operands[2])?;

                self.program.emit(OpCode::Cast {
                    dest,
                    source,
                    target_type,
                });
            },
            "sqrt" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("sqrt expects 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }
                let dest = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_operand(&instr.operands[1])?;
                self.program.emit(OpCode::Sqrt { dest, source });
            },
            "pow" => {
                if instr.operands.len() != 3 {
                    return Err(AsmError::AssemblyError {
                        message: format!("pow expects 3 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }
                let dest = self.operand_to_string(&instr.operands[0])?;
                let base = self.operand_to_operand(&instr.operands[1])?;
                let exp = self.operand_to_operand(&instr.operands[2])?;
                self.program.emit(OpCode::Pow { dest, base, exp });
            },
            "abs" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("abs expects 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }
                let dest = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_operand(&instr.operands[1])?;
                self.program.emit(OpCode::Abs { dest, source });
            },
            "min" => {
                if instr.operands.len() != 3 {
                    return Err(AsmError::AssemblyError {
                        message: format!("min expects 3 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }
                let dest = self.operand_to_string(&instr.operands[0])?;
                let a = self.operand_to_operand(&instr.operands[1])?;
                let b = self.operand_to_operand(&instr.operands[2])?;
                self.program.emit(OpCode::Min { dest, a, b });
            },
            "max" => {
                if instr.operands.len() != 3 {
                    return Err(AsmError::AssemblyError {
                        message: format!("max expects 3 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }
                let dest = self.operand_to_string(&instr.operands[0])?;
                let a = self.operand_to_operand(&instr.operands[1])?;
                let b = self.operand_to_operand(&instr.operands[2])?;
                self.program.emit(OpCode::Max { dest, a, b });
            },
            "sin" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("sin expects 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }
                let dest = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_operand(&instr.operands[1])?;
                self.program.emit(OpCode::Sin { dest, source });
            },
            "cos" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("cos expects 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }
                let dest = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_operand(&instr.operands[1])?;
                self.program.emit(OpCode::Cos { dest, source });
            },
            "tan" => {
                if instr.operands.len() != 2 {
                    return Err(AsmError::AssemblyError {
                        message: format!("tan expects 2 operands, got {}", instr.operands.len()),
                        location: None,
                    });
                }
                let dest = self.operand_to_string(&instr.operands[0])?;
                let source = self.operand_to_operand(&instr.operands[1])?;
                self.program.emit(OpCode::Tan { dest, source });
            },
            _ => {
                return Err(AsmError::AssemblyError {
                    message: format!("Unknown opcode: {}", opcode_name),
                    location: None,
                });
            }
        }

        Ok(())
    }

    fn operand_to_string(&self, operand: &AsmOperand) -> Result<String, AsmError> {
        match operand {
            AsmOperand::Variable(name) => Ok(name.clone()),
            AsmOperand::Label(name) => Ok(name.clone()),
            _ => Err(AsmError::AssemblyError {
                message: format!("Expected variable or label, got {:?}", operand),
                location: None,
            }),
        }
    }

    fn operand_to_operand(&self, operand: &AsmOperand) -> Result<Operand, AsmError> {
        use crate::asm::ast::DefineValue;

        match operand {
            AsmOperand::Variable(name) => {
                // Check if this is a define constant
                if let Some(define_value) = self.defines.get(name) {
                    let value = match define_value {
                        DefineValue::Integer(val) => Value::I32(*val as i32),
                        DefineValue::Float(val) => Value::F32(*val as f32),
                        DefineValue::String(_) => {
                            return Err(AsmError::AssemblyError {
                                message: format!("Cannot use string define '{}' as numeric operand", name),
                                location: None,
                            });
                        }
                    };
                    Ok(Operand::Immediate(value))
                } else {
                    Ok(Operand::Variable(name.clone()))
                }
            },
            AsmOperand::Label(name) => Ok(Operand::Label(name.clone())),
            AsmOperand::Immediate(imm) => {
                let value = match imm {
                    Immediate::Integer(val) => Value::I32(*val as i32),
                    Immediate::Float(val) => Value::F32(*val as f32),
                    Immediate::String(_) => {
                        return Err(AsmError::AssemblyError {
                            message: "String immediates are not supported as instruction operands".to_string(),
                            location: None,
                        });
                    }
                };
                Ok(Operand::Immediate(value))
            },
        }
    }

    fn operand_to_datatype(&self, operand: &AsmOperand) -> Result<DataType, AsmError> {
        match operand {
            AsmOperand::Variable(name) => {
                match name.as_str() {
                    "i8" => Ok(DataType::I8),
                    "i16" => Ok(DataType::I16),
                    "i32" => Ok(DataType::I32),
                    "i64" => Ok(DataType::I64),
                    "u8" => Ok(DataType::U8),
                    "u16" => Ok(DataType::U16),
                    "u32" => Ok(DataType::U32),
                    "u64" => Ok(DataType::U64),
                    "f32" => Ok(DataType::F32),
                    "f64" => Ok(DataType::F64),
                    "ptr" => Ok(DataType::Ptr),
                    "void" => Ok(DataType::Void),
                    _ => Err(AsmError::AssemblyError {
                        message: format!("Unknown data type: {}", name),
                        location: None,
                    }),
                }
            },
            _ => Err(AsmError::AssemblyError {
                message: format!("Expected data type, got {:?}", operand),
                location: None,
            }),
        }
    }

    fn last_label(&self) -> Option<String> {
        for i in (0..self.program.instructions.len()).rev() {
            if let OpCode::Label { name } = &self.program.instructions[i] {
                return Some(name.clone());
            }
        }
        None
    }

    fn resolve_labels(&mut self) -> Result<(), AsmError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_simple() {
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
        assert!(program.functions.contains_key("main"));
        assert_eq!(program.globals.len(), 1);
    }
}
