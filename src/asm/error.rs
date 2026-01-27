use std::fmt;

#[derive(Debug, Clone)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub file: String,
}

#[derive(Debug, Clone)]
pub enum AsmError {
    LexError {
        message: String,
        location: Option<SourceLocation>,
    },
    ParseError {
        message: String,
        location: Option<SourceLocation>,
    },
    AssemblyError {
        message: String,
        location: Option<SourceLocation>,
    },
    IoError {
        message: String,
    },
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AsmError::LexError { message, location } => {
                if let Some(loc) = location {
                    write!(
                        f,
                        "Lexer error at {}:{}:{}: {}",
                        loc.file, loc.line, loc.column, message
                    )
                } else {
                    write!(f, "Lexer error: {}", message)
                }
            }
            AsmError::ParseError { message, location } => {
                if let Some(loc) = location {
                    write!(
                        f,
                        "Parse error at {}:{}:{}: {}",
                        loc.file, loc.line, loc.column, message
                    )
                } else {
                    write!(f, "Parse error: {}", message)
                }
            }
            AsmError::AssemblyError { message, location } => {
                if let Some(loc) = location {
                    write!(
                        f,
                        "Assembly error at {}:{}:{}: {}",
                        loc.file, loc.line, loc.column, message
                    )
                } else {
                    write!(f, "Assembly error: {}", message)
                }
            }
            AsmError::IoError { message } => {
                write!(f, "IO error: {}", message)
            }
        }
    }
}

impl std::error::Error for AsmError {}

impl From<std::io::Error> for AsmError {
    fn from(err: std::io::Error) -> Self {
        AsmError::IoError {
            message: err.to_string(),
        }
    }
}
