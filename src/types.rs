use varvm_macros::ValueOps;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Ptr,
    Void,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Variable(String),
    Immediate(Value),
    Label(String),
    Type(DataType),
}

#[derive(Debug, Clone, PartialEq, ValueOps)]
pub enum Value {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Ptr(usize),
}

impl From<i32> for Operand {
    fn from(val: i32) -> Self {
        Operand::Immediate(Value::I32(val))
    }
}

impl From<&str> for Operand {
    fn from(val: &str) -> Self {
        Operand::Variable(val.to_string())
    }
}

// Non-macro implementations for Value
impl Value {
    pub fn is_zero(&self) -> bool {
        match self {
            Value::I8(v) => *v == 0,
            Value::I16(v) => *v == 0,
            Value::I32(v) => *v == 0,
            Value::I64(v) => *v == 0,
            Value::U8(v) => *v == 0,
            Value::U16(v) => *v == 0,
            Value::U32(v) => *v == 0,
            Value::U64(v) => *v == 0,
            Value::F32(v) => *v == 0.0,
            Value::F64(v) => *v == 0.0,
            Value::Ptr(v) => *v == 0,
        }
    }

    pub fn as_usize(&self) -> Result<usize, String> {
        match self {
            Value::I8(v) if *v >= 0 => Ok(*v as usize),
            Value::I16(v) if *v >= 0 => Ok(*v as usize),
            Value::I32(v) if *v >= 0 => Ok(*v as usize),
            Value::I64(v) if *v >= 0 => Ok(*v as usize),
            Value::U8(v) => Ok(*v as usize),
            Value::U16(v) => Ok(*v as usize),
            Value::U32(v) => Ok(*v as usize),
            Value::U64(v) => Ok(*v as usize),
            Value::Ptr(v) => Ok(*v),
            _ => Err("Cannot convert to usize (negative or invalid type)".to_string()),
        }
    }

    // Cross-type numeric equality comparison
    pub fn equals(&self, other: &Value) -> bool {
        // For floating point, convert both to f64 and compare
        if matches!(self, Value::F32(_) | Value::F64(_)) || matches!(other, Value::F32(_) | Value::F64(_)) {
            let a = match self {
                Value::I8(v) => *v as f64,
                Value::I16(v) => *v as f64,
                Value::I32(v) => *v as f64,
                Value::I64(v) => *v as f64,
                Value::U8(v) => *v as f64,
                Value::U16(v) => *v as f64,
                Value::U32(v) => *v as f64,
                Value::U64(v) => *v as f64,
                Value::F32(v) => *v as f64,
                Value::F64(v) => *v,
                Value::Ptr(v) => *v as f64,
            };
            let b = match other {
                Value::I8(v) => *v as f64,
                Value::I16(v) => *v as f64,
                Value::I32(v) => *v as f64,
                Value::I64(v) => *v as f64,
                Value::U8(v) => *v as f64,
                Value::U16(v) => *v as f64,
                Value::U32(v) => *v as f64,
                Value::U64(v) => *v as f64,
                Value::F32(v) => *v as f64,
                Value::F64(v) => *v,
                Value::Ptr(v) => *v as f64,
            };
            return a == b;
        }

        // For integers, convert both to i64 and compare
        let a = match self {
            Value::I8(v) => *v as i64,
            Value::I16(v) => *v as i64,
            Value::I32(v) => *v as i64,
            Value::I64(v) => *v,
            Value::U8(v) => *v as i64,
            Value::U16(v) => *v as i64,
            Value::U32(v) => *v as i64,
            Value::U64(v) => *v as i64,
            Value::Ptr(v) => *v as i64,
            _ => return false,
        };
        let b = match other {
            Value::I8(v) => *v as i64,
            Value::I16(v) => *v as i64,
            Value::I32(v) => *v as i64,
            Value::I64(v) => *v,
            Value::U8(v) => *v as i64,
            Value::U16(v) => *v as i64,
            Value::U32(v) => *v as i64,
            Value::U64(v) => *v as i64,
            Value::Ptr(v) => *v as i64,
            _ => return false,
        };
        a == b
    }
}
