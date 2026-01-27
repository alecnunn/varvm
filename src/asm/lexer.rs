use crate::asm::ast::Token;
use crate::asm::error::{AsmError, SourceLocation};
use crate::types::DataType;

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    file: String,
}

impl Lexer {
    pub fn new(input: &str, file: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            file,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, AsmError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_except_newline();

            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;

            match token {
                Token::Comment(_) => {},
                Token::Newline => {
                    if let Some(last) = tokens.last() {
                        if !matches!(last, Token::Newline) {
                            tokens.push(token);
                        }
                    }
                },
                _ => tokens.push(token),
            }
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, AsmError> {
        if self.is_at_end() {
            return Ok(Token::Eof);
        }

        let ch = self.current_char();

        match ch {
            ';' => self.read_comment(),
            '\n' => {
                self.advance();
                self.line += 1;
                self.column = 1;
                Ok(Token::Newline)
            },
            ':' => {
                self.advance();
                Ok(Token::Colon)
            },
            ',' => {
                self.advance();
                Ok(Token::Comma)
            },
            '=' => {
                self.advance();
                Ok(Token::Equals)
            },
            '[' => {
                self.advance();
                Ok(Token::LeftBracket)
            },
            ']' => {
                self.advance();
                Ok(Token::RightBracket)
            },
            '{' => {
                self.advance();
                Ok(Token::LeftBrace)
            },
            '}' => {
                self.advance();
                Ok(Token::RightBrace)
            },
            '"' => self.read_string(),
            '.' => self.read_local_label(),
            '-' | '0'..='9' => self.read_number(),
            'a'..='z' | 'A'..='Z' | '_' => self.read_identifier_or_keyword(),
            _ => Err(AsmError::LexError {
                message: format!("Unexpected character: '{}'", ch),
                location: Some(self.location()),
            }),
        }
    }

    fn read_comment(&mut self) -> Result<Token, AsmError> {
        self.advance();
        let start = self.position;

        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }

        let comment: String = self.input[start..self.position].iter().collect();
        Ok(Token::Comment(comment))
    }

    fn read_string(&mut self) -> Result<Token, AsmError> {
        self.advance(); // Skip opening quote
        let mut string_val = String::new();

        while !self.is_at_end() && self.current_char() != '"' {
            let ch = self.current_char();

            if ch == '\\' && !self.is_at_end() {
                self.advance();
                if !self.is_at_end() {
                    let escaped = self.current_char();
                    match escaped {
                        'n' => string_val.push('\n'),
                        't' => string_val.push('\t'),
                        'r' => string_val.push('\r'),
                        '\\' => string_val.push('\\'),
                        '"' => string_val.push('"'),
                        _ => {
                            string_val.push('\\');
                            string_val.push(escaped);
                        }
                    }
                    self.advance();
                }
            } else if ch == '\n' {
                return Err(AsmError::LexError {
                    message: "Unterminated string literal".to_string(),
                    location: Some(self.location()),
                });
            } else {
                string_val.push(ch);
                self.advance();
            }
        }

        if self.is_at_end() {
            return Err(AsmError::LexError {
                message: "Unterminated string literal".to_string(),
                location: Some(self.location()),
            });
        }

        self.advance(); // Skip closing quote
        Ok(Token::String(string_val))
    }

    fn read_local_label(&mut self) -> Result<Token, AsmError> {
        let start = self.position;
        self.advance();

        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let label: String = self.input[start..self.position].iter().collect();
        Ok(Token::Label(label))
    }

    fn read_number(&mut self) -> Result<Token, AsmError> {
        let start_pos = self.position;
        let mut is_negative = false;
        let mut is_float = false;
        let mut is_hex = false;
        let mut is_binary = false;

        if self.current_char() == '-' {
            is_negative = true;
            self.advance();
        }

        if self.current_char() == '0' && !self.is_at_end() {
            self.advance();
            if !self.is_at_end() {
                match self.current_char() {
                    'x' | 'X' => {
                        is_hex = true;
                        self.advance();
                    },
                    'b' | 'B' => {
                        is_binary = true;
                        self.advance();
                    },
                    '0'..='9' | '.' => {},
                    _ => {
                        return Ok(Token::Integer(0));
                    }
                }
            }
        }

        while !self.is_at_end() {
            let ch = self.current_char();
            if is_hex && ch.is_ascii_hexdigit() {
                self.advance();
            } else if is_binary && (ch == '0' || ch == '1') {
                self.advance();
            } else if ch.is_numeric() {
                self.advance();
            } else if ch == '.' && !is_float && !is_hex && !is_binary {
                is_float = true;
                self.advance();
            } else {
                break;
            }
        }

        let num_str: String = self.input[start_pos..self.position].iter().collect();

        if is_float {
            let value: f64 = num_str.parse().map_err(|_| AsmError::LexError {
                message: format!("Invalid float: {}", num_str),
                location: Some(self.location()),
            })?;
            Ok(Token::Float(value))
        } else if is_hex {
            let hex_part = &num_str[if is_negative { 3 } else { 2 }..];
            let value = i64::from_str_radix(hex_part, 16).map_err(|_| AsmError::LexError {
                message: format!("Invalid hexadecimal: {}", num_str),
                location: Some(self.location()),
            })?;
            Ok(Token::Integer(if is_negative { -value } else { value }))
        } else if is_binary {
            let bin_part = &num_str[if is_negative { 3 } else { 2 }..];
            let value = i64::from_str_radix(bin_part, 2).map_err(|_| AsmError::LexError {
                message: format!("Invalid binary: {}", num_str),
                location: Some(self.location()),
            })?;
            Ok(Token::Integer(if is_negative { -value } else { value }))
        } else {
            let value: i64 = num_str.parse().map_err(|_| AsmError::LexError {
                message: format!("Invalid integer: {}", num_str),
                location: Some(self.location()),
            })?;
            Ok(Token::Integer(value))
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Result<Token, AsmError> {
        let start = self.position;

        while !self.is_at_end() {
            let ch = self.current_char();
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text: String = self.input[start..self.position].iter().collect();

        let token = match text.as_str() {
            "section" => Token::Section,
            "data" => Token::Data,
            "text" => Token::Text,
            "func_begin" => Token::FuncBegin,
            "func_end" => Token::FuncEnd,
            "local" => Token::Local,
            "include" => Token::Include,
            "define" => Token::Define,

            "str" => Token::Identifier("str".to_string()), // str is special, not a DataType
            "i8" => Token::Type(DataType::I8),
            "i16" => Token::Type(DataType::I16),
            "i32" => Token::Type(DataType::I32),
            "i64" => Token::Type(DataType::I64),
            "u8" => Token::Type(DataType::U8),
            "u16" => Token::Type(DataType::U16),
            "u32" => Token::Type(DataType::U32),
            "u64" => Token::Type(DataType::U64),
            "f32" => Token::Type(DataType::F32),
            "f64" => Token::Type(DataType::F64),
            "ptr" => Token::Type(DataType::Ptr),
            "void" => Token::Type(DataType::Void),

            _ => Token::Identifier(text),
        };

        Ok(token)
    }

    fn skip_whitespace_except_newline(&mut self) {
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn current_char(&self) -> char {
        self.input[self.position]
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
            self.column += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn location(&self) -> SourceLocation {
        SourceLocation {
            line: self.line,
            column: self.column,
            file: self.file.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokenization() {
        let input = "section .data\n    result: i32\n";
        let mut lexer = Lexer::new(input, "test.vasm".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0], Token::Section));
        assert!(matches!(tokens[1], Token::Label(_))); // .data is parsed as Label
    }

    #[test]
    fn test_number_parsing() {
        let input = "42 0x2A 0b101010 3.14";
        let mut lexer = Lexer::new(input, "test.vasm".to_string());
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0], Token::Integer(42)));
        assert!(matches!(tokens[1], Token::Integer(42)));
        assert!(matches!(tokens[2], Token::Integer(42)));
        assert!(matches!(tokens[3], Token::Float(_)));
    }
}
