use crate::types::DataType;

#[derive(Debug, Clone)]
pub struct SourceLoc {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Section,
    Data,
    Text,
    Func,
    FuncBegin,
    FuncEnd,
    Local,
    Include,
    Define,

    // Data types
    Type(DataType),

    // Identifiers and literals
    Identifier(String),
    Label(String),
    Integer(i64),
    Float(f64),
    String(String),

    // Symbols
    Colon,
    Comma,
    Equals,
    LeftBracket,   // [
    RightBracket,  // ]
    LeftBrace,     // {
    RightBrace,    // }

    // Comments and whitespace
    Comment(String),
    Newline,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    Data,
    Text,
}

#[derive(Debug, Clone)]
pub struct DataDeclaration {
    pub name: String,
    pub dtype: DataType,
    pub value: Option<Immediate>,
}

#[derive(Debug, Clone)]
pub enum Immediate {
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone)]
pub enum Operand {
    Variable(String),
    Immediate(Immediate),
    Label(String),
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: String,
    pub operands: Vec<Operand>,
    pub location: SourceLoc,
}

#[derive(Debug, Clone)]
pub struct LocalDeclaration {
    pub name: String,
    pub dtype: DataType,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Label(String),
    Instruction(Instruction),
    LocalDecl(LocalDeclaration),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefineValue {
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone)]
pub struct Define {
    pub name: String,
    pub value: DefineValue,
}

#[derive(Debug, Clone)]
pub struct AsmProgram {
    pub defines: Vec<Define>,
    pub data_section: Vec<DataDeclaration>,
    pub text_section: Vec<Statement>,
    pub includes: Vec<String>,
}

impl AsmProgram {
    pub fn new() -> Self {
        Self {
            defines: Vec::new(),
            data_section: Vec::new(),
            text_section: Vec::new(),
            includes: Vec::new(),
        }
    }

    pub fn merge(&mut self, other: AsmProgram) {
        self.defines.extend(other.defines);
        self.data_section.extend(other.data_section);
        self.text_section.extend(other.text_section);
        // Don't merge includes to avoid re-including
    }
}
