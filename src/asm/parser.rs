use crate::asm::ast::*;
use crate::asm::error::AsmError;
use crate::types::DataType;

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
    current_line: usize,
    current_column: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            current_line: 1,
            current_column: 1,
        }
    }

    pub fn parse(&mut self) -> Result<AsmProgram, AsmError> {
        let mut program = AsmProgram::new();

        self.skip_newlines();

        while !self.is_at_end() && !matches!(self.current(), Token::Eof) {
            if self.check_keyword("define") {
                self.parse_define(&mut program)?;
            } else if self.check_keyword("include") {
                self.parse_include(&mut program)?;
            } else if self.check_keyword("section") {
                self.parse_section(&mut program)?;
            } else {
                return Err(AsmError::ParseError {
                    message: format!("Expected 'define', 'include', or 'section', got {:?}", self.current()),
                    location: None,
                });
            }
            self.skip_newlines();
        }

        Ok(program)
    }

    fn parse_define(&mut self, program: &mut AsmProgram) -> Result<(), AsmError> {
        use crate::asm::ast::{Define, DefineValue};

        self.expect_keyword("define")?;

        // Get the constant name
        let name = self.expect_identifier()?;

        // Get the value (integer, float, or string)
        let value = match self.current().clone() {
            Token::Integer(i) => {
                self.advance();
                DefineValue::Integer(i)
            },
            Token::Float(f) => {
                self.advance();
                DefineValue::Float(f)
            },
            Token::String(s) => {
                self.advance();
                DefineValue::String(s)
            },
            _ => {
                return Err(AsmError::ParseError {
                    message: format!("Expected integer, float, or string after define name, got {:?}", self.current()),
                    location: None,
                });
            }
        };

        self.expect_newline()?;

        program.defines.push(Define { name, value });

        Ok(())
    }

    fn parse_include(&mut self, program: &mut AsmProgram) -> Result<(), AsmError> {
        self.expect_keyword("include")?;

        // Expect a string literal with the file path
        let path = match self.current().clone() {
            Token::String(s) => {
                self.advance();
                s
            },
            _ => {
                return Err(AsmError::ParseError {
                    message: format!("Expected string path after 'include', got {:?}", self.current()),
                    location: None,
                });
            }
        };

        self.expect_newline()?;

        // Store the include path
        program.includes.push(path);

        Ok(())
    }

    fn parse_section(&mut self, program: &mut AsmProgram) -> Result<(), AsmError> {
        self.expect_keyword("section")?;

        let section_name = match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                name
            },
            Token::Label(name) => {
                self.advance();
                name
            },
            _ => {
                return Err(AsmError::ParseError {
                    message: format!("Expected section name, got {:?}", self.current()),
                    location: None,
                });
            }
        };

        self.expect_newline()?;
        self.skip_newlines();

        match section_name.as_str() {
            ".data" => self.parse_data_section(program)?,
            ".text" => self.parse_text_section(program)?,
            _ => {
                return Err(AsmError::ParseError {
                    message: format!("Unknown section: {}", section_name),
                    location: None,
                });
            }
        }

        Ok(())
    }

    fn parse_data_section(&mut self, program: &mut AsmProgram) -> Result<(), AsmError> {
        use crate::asm::ast::Immediate;
        use crate::types::DataType;

        while !self.is_at_end() && !self.check_keyword("section") {
            if matches!(self.current(), Token::Newline) {
                self.advance();
                continue;
            }

            if matches!(self.current(), Token::Eof) {
                break;
            }

            let name = self.expect_identifier()?;
            self.expect(Token::Colon)?;

            // Check if this is a string declaration (str type)
            if matches!(self.current(), Token::Identifier(s) if s == "str") {
                self.advance();

                // Expect a string literal
                let string_value = match self.current().clone() {
                    Token::String(s) => {
                        self.advance();
                        s
                    },
                    _ => {
                        return Err(AsmError::ParseError {
                            message: format!("Expected string literal after 'str', got {:?}", self.current()),
                            location: None,
                        });
                    }
                };

                program.data_section.push(DataDeclaration {
                    name,
                    dtype: DataType::Ptr,  // Strings are stored as pointers
                    value: Some(Immediate::String(string_value)),
                });
            } else {
                // Regular type declaration
                let dtype = self.expect_type()?;

                let value = if self.check(Token::Equals) {
                    self.advance();
                    Some(self.parse_immediate()?)
                } else {
                    None
                };

                program.data_section.push(DataDeclaration {
                    name,
                    dtype,
                    value,
                });
            }

            self.skip_newlines();
        }

        Ok(())
    }

    fn parse_text_section(&mut self, program: &mut AsmProgram) -> Result<(), AsmError> {
        while !self.is_at_end() && !self.check_keyword("section") {
            if matches!(self.current(), Token::Newline) {
                self.advance();
                continue;
            }

            if matches!(self.current(), Token::Eof) {
                break;
            }

            let token = self.current().clone();

            match token {
                Token::Identifier(_) | Token::FuncBegin | Token::FuncEnd => {
                    let next_token = self.peek(1);
                    if matches!(next_token, Some(Token::Colon)) && matches!(token, Token::Identifier(_)) {
                        let name = self.expect_identifier()?;
                        self.expect(Token::Colon)?;
                        program.text_section.push(Statement::Label(name));
                        self.skip_newlines();
                    } else {
                        let instruction = self.parse_instruction()?;
                        program.text_section.push(Statement::Instruction(instruction));
                        self.skip_newlines();
                    }
                },
                Token::Label(name) => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    program.text_section.push(Statement::Label(name));
                    self.skip_newlines();
                },
                Token::Local => {
                    self.advance();
                    let name = match self.current().clone() {
                        Token::Identifier(n) => {
                            self.advance();
                            n
                        },
                        Token::Type(_) => {
                            return Err(AsmError::ParseError {
                                message: "Variable name cannot be a type keyword".to_string(),
                                location: None,
                            });
                        },
                        _ => {
                            return Err(AsmError::ParseError {
                                message: format!("Expected variable name, got {:?}", self.current()),
                                location: None,
                            });
                        }
                    };
                    self.expect(Token::Colon)?;
                    let dtype = self.expect_type()?;
                    program.text_section.push(Statement::LocalDecl(LocalDeclaration {
                        name,
                        dtype,
                    }));
                    self.skip_newlines();
                },
                _ => {
                    return Err(AsmError::ParseError {
                        message: format!("Unexpected token in text section: {:?}", token),
                        location: None,
                    });
                }
            }
        }

        Ok(())
    }

    fn parse_instruction(&mut self) -> Result<Instruction, AsmError> {
        let location = SourceLoc {
            line: self.current_line,
            column: self.current_column,
        };

        let opcode = match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                name
            },
            Token::FuncBegin => {
                self.advance();
                "func_begin".to_string()
            },
            Token::FuncEnd => {
                self.advance();
                "func_end".to_string()
            },
            _ => {
                return Err(AsmError::ParseError {
                    message: format!("Expected instruction opcode, got {:?}", self.current()),
                    location: None,
                });
            }
        };

        let mut operands = Vec::new();

        if !matches!(self.current(), Token::Newline | Token::Eof) {
            loop {
                let operand = self.parse_operand()?;
                operands.push(operand);

                if self.check(Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(Instruction { opcode, operands, location })
    }

    fn parse_operand(&mut self) -> Result<Operand, AsmError> {
        match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(Operand::Variable(name))
            },
            Token::Label(name) => {
                self.advance();
                Ok(Operand::Label(name))
            },
            Token::Integer(_) | Token::Float(_) => {
                let imm = self.parse_immediate()?;
                Ok(Operand::Immediate(imm))
            },
            Token::Type(dtype) => {
                self.advance();
                let type_name = match dtype {
                    crate::types::DataType::I8 => "i8",
                    crate::types::DataType::I16 => "i16",
                    crate::types::DataType::I32 => "i32",
                    crate::types::DataType::I64 => "i64",
                    crate::types::DataType::U8 => "u8",
                    crate::types::DataType::U16 => "u16",
                    crate::types::DataType::U32 => "u32",
                    crate::types::DataType::U64 => "u64",
                    crate::types::DataType::F32 => "f32",
                    crate::types::DataType::F64 => "f64",
                    crate::types::DataType::Ptr => "ptr",
                    crate::types::DataType::Void => "void",
                };
                Ok(Operand::Variable(type_name.to_string()))
            },
            _ => Err(AsmError::ParseError {
                message: format!("Expected operand, got {:?}", self.current()),
                location: None,
            }),
        }
    }

    fn parse_immediate(&mut self) -> Result<Immediate, AsmError> {
        match self.current().clone() {
            Token::Integer(val) => {
                self.advance();
                Ok(Immediate::Integer(val))
            },
            Token::Float(val) => {
                self.advance();
                Ok(Immediate::Float(val))
            },
            _ => Err(AsmError::ParseError {
                message: format!("Expected immediate value, got {:?}", self.current()),
                location: None,
            }),
        }
    }

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), AsmError> {
        if !self.check_keyword(keyword) {
            return Err(AsmError::ParseError {
                message: format!("Expected keyword '{}', got {:?}", keyword, self.current()),
                location: None,
            });
        }
        self.advance();
        Ok(())
    }

    fn expect_identifier(&mut self) -> Result<String, AsmError> {
        match self.current().clone() {
            Token::Identifier(name) => {
                self.advance();
                Ok(name)
            },
            _ => Err(AsmError::ParseError {
                message: format!("Expected identifier, got {:?}", self.current()),
                location: None,
            }),
        }
    }

    fn expect_type(&mut self) -> Result<DataType, AsmError> {
        match self.current().clone() {
            Token::Type(dtype) => {
                self.advance();
                Ok(dtype)
            },
            _ => Err(AsmError::ParseError {
                message: format!("Expected type, got {:?}", self.current()),
                location: None,
            }),
        }
    }

    fn expect(&mut self, expected: Token) -> Result<(), AsmError> {
        if !self.check(expected.clone()) {
            return Err(AsmError::ParseError {
                message: format!("Expected {:?}, got {:?}", expected, self.current()),
                location: None,
            });
        }
        self.advance();
        Ok(())
    }

    fn expect_newline(&mut self) -> Result<(), AsmError> {
        if !matches!(self.current(), Token::Newline | Token::Eof) {
            return Err(AsmError::ParseError {
                message: format!("Expected newline, got {:?}", self.current()),
                location: None,
            });
        }
        if matches!(self.current(), Token::Newline) {
            self.advance();
        }
        Ok(())
    }

    fn check_keyword(&self, keyword: &str) -> bool {
        match self.current() {
            Token::Section if keyword == "section" => true,
            Token::Data if keyword == "data" => true,
            Token::Text if keyword == "text" => true,
            Token::Local if keyword == "local" => true,
            Token::Include if keyword == "include" => true,
            Token::Define if keyword == "define" => true,
            Token::FuncBegin if keyword == "func_begin" => true,
            Token::FuncEnd if keyword == "func_end" => true,
            _ => false,
        }
    }

    fn check(&self, token: Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        std::mem::discriminant(self.current()) == std::mem::discriminant(&token)
    }

    fn skip_newlines(&mut self) {
        while matches!(self.current(), Token::Newline) {
            self.current_line += 1;
            self.current_column = 1;
            self.advance();
        }
    }

    fn current(&self) -> &Token {
        if self.position >= self.tokens.len() {
            &Token::Eof
        } else {
            &self.tokens[self.position]
        }
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        let pos = self.position + offset;
        if pos < self.tokens.len() {
            Some(&self.tokens[pos])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || matches!(self.current(), Token::Eof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asm::lexer::Lexer;

    #[test]
    fn test_parse_data_section() {
        let input = "section .data\n    result: i32\n";
        let mut lexer = Lexer::new(input, "test.vasm".to_string());
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.data_section.len(), 1);
        assert_eq!(program.data_section[0].name, "result");
    }
}
