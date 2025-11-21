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

#[derive(Debug, Clone, PartialEq)]
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

// Value arithmetic operations
impl Value {
    pub fn add(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a + b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a + b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a + b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a + b)),
            _ => Err("Type mismatch in addition".to_string()),
        }
    }

    pub fn sub(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a - b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a - b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a - b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a - b)),
            _ => Err("Type mismatch in subtraction".to_string()),
        }
    }

    pub fn mul(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a * b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a * b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a * b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a * b)),
            _ => Err("Type mismatch in multiplication".to_string()),
        }
    }

    pub fn div(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) if *b != 0 => Ok(Value::I32(a / b)),
            (Value::I64(a), Value::I64(b)) if *b != 0 => Ok(Value::I64(a / b)),
            (Value::F32(a), Value::F32(b)) => Ok(Value::F32(a / b)),
            (Value::F64(a), Value::F64(b)) => Ok(Value::F64(a / b)),
            _ => Err("Type mismatch or division by zero".to_string()),
        }
    }

    pub fn modulo(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) if *b != 0 => Ok(Value::I32(a % b)),
            (Value::I64(a), Value::I64(b)) if *b != 0 => Ok(Value::I64(a % b)),
            _ => Err("Type mismatch or modulo by zero".to_string()),
        }
    }

    pub fn neg(&self) -> Result<Value, String> {
        match self {
            Value::I32(a) => Ok(Value::I32(-a)),
            Value::I64(a) => Ok(Value::I64(-a)),
            Value::F32(a) => Ok(Value::F32(-a)),
            Value::F64(a) => Ok(Value::F64(-a)),
            _ => Err("Cannot negate this type".to_string()),
        }
    }

    pub fn lt(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Ok(a < b),
            (Value::I64(a), Value::I64(b)) => Ok(a < b),
            (Value::F32(a), Value::F32(b)) => Ok(a < b),
            (Value::F64(a), Value::F64(b)) => Ok(a < b),
            _ => Err("Type mismatch in comparison".to_string()),
        }
    }

    pub fn le(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::I32(a), Value::I32(b)) => Ok(a <= b),
            (Value::I64(a), Value::I64(b)) => Ok(a <= b),
            (Value::F32(a), Value::F32(b)) => Ok(a <= b),
            (Value::F64(a), Value::F64(b)) => Ok(a <= b),
            _ => Err("Type mismatch in comparison".to_string()),
        }
    }

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

    pub fn gt(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::I8(a), Value::I8(b)) => Ok(a > b),
            (Value::I16(a), Value::I16(b)) => Ok(a > b),
            (Value::I32(a), Value::I32(b)) => Ok(a > b),
            (Value::I64(a), Value::I64(b)) => Ok(a > b),
            (Value::U8(a), Value::U8(b)) => Ok(a > b),
            (Value::U16(a), Value::U16(b)) => Ok(a > b),
            (Value::U32(a), Value::U32(b)) => Ok(a > b),
            (Value::U64(a), Value::U64(b)) => Ok(a > b),
            (Value::F32(a), Value::F32(b)) => Ok(a > b),
            (Value::F64(a), Value::F64(b)) => Ok(a > b),
            _ => Err("Type mismatch in comparison".to_string()),
        }
    }

    pub fn ge(&self, other: &Value) -> Result<bool, String> {
        match (self, other) {
            (Value::I8(a), Value::I8(b)) => Ok(a >= b),
            (Value::I16(a), Value::I16(b)) => Ok(a >= b),
            (Value::I32(a), Value::I32(b)) => Ok(a >= b),
            (Value::I64(a), Value::I64(b)) => Ok(a >= b),
            (Value::U8(a), Value::U8(b)) => Ok(a >= b),
            (Value::U16(a), Value::U16(b)) => Ok(a >= b),
            (Value::U32(a), Value::U32(b)) => Ok(a >= b),
            (Value::U64(a), Value::U64(b)) => Ok(a >= b),
            (Value::F32(a), Value::F32(b)) => Ok(a >= b),
            (Value::F64(a), Value::F64(b)) => Ok(a >= b),
            _ => Err("Type mismatch in comparison".to_string()),
        }
    }

    pub fn bitwise_and(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::I8(a), Value::I8(b)) => Ok(Value::I8(a & b)),
            (Value::I16(a), Value::I16(b)) => Ok(Value::I16(a & b)),
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a & b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a & b)),
            (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a & b)),
            (Value::U16(a), Value::U16(b)) => Ok(Value::U16(a & b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a & b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a & b)),
            _ => Err("Type mismatch or invalid type for bitwise AND".to_string()),
        }
    }

    pub fn bitwise_or(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::I8(a), Value::I8(b)) => Ok(Value::I8(a | b)),
            (Value::I16(a), Value::I16(b)) => Ok(Value::I16(a | b)),
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a | b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a | b)),
            (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a | b)),
            (Value::U16(a), Value::U16(b)) => Ok(Value::U16(a | b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a | b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a | b)),
            _ => Err("Type mismatch or invalid type for bitwise OR".to_string()),
        }
    }

    pub fn bitwise_xor(&self, other: &Value) -> Result<Value, String> {
        match (self, other) {
            (Value::I8(a), Value::I8(b)) => Ok(Value::I8(a ^ b)),
            (Value::I16(a), Value::I16(b)) => Ok(Value::I16(a ^ b)),
            (Value::I32(a), Value::I32(b)) => Ok(Value::I32(a ^ b)),
            (Value::I64(a), Value::I64(b)) => Ok(Value::I64(a ^ b)),
            (Value::U8(a), Value::U8(b)) => Ok(Value::U8(a ^ b)),
            (Value::U16(a), Value::U16(b)) => Ok(Value::U16(a ^ b)),
            (Value::U32(a), Value::U32(b)) => Ok(Value::U32(a ^ b)),
            (Value::U64(a), Value::U64(b)) => Ok(Value::U64(a ^ b)),
            _ => Err("Type mismatch or invalid type for bitwise XOR".to_string()),
        }
    }

    pub fn bitwise_not(&self) -> Result<Value, String> {
        match self {
            Value::I8(a) => Ok(Value::I8(!a)),
            Value::I16(a) => Ok(Value::I16(!a)),
            Value::I32(a) => Ok(Value::I32(!a)),
            Value::I64(a) => Ok(Value::I64(!a)),
            Value::U8(a) => Ok(Value::U8(!a)),
            Value::U16(a) => Ok(Value::U16(!a)),
            Value::U32(a) => Ok(Value::U32(!a)),
            Value::U64(a) => Ok(Value::U64(!a)),
            _ => Err("Invalid type for bitwise NOT".to_string()),
        }
    }

    pub fn shift_left(&self, other: &Value) -> Result<Value, String> {
        let shift = match other {
            Value::I32(s) if *s >= 0 => *s as u32,
            Value::U32(s) => *s,
            _ => return Err("Shift amount must be non-negative integer".to_string()),
        };

        match self {
            Value::I8(a) => Ok(Value::I8(a.wrapping_shl(shift))),
            Value::I16(a) => Ok(Value::I16(a.wrapping_shl(shift))),
            Value::I32(a) => Ok(Value::I32(a.wrapping_shl(shift))),
            Value::I64(a) => Ok(Value::I64(a.wrapping_shl(shift))),
            Value::U8(a) => Ok(Value::U8(a.wrapping_shl(shift))),
            Value::U16(a) => Ok(Value::U16(a.wrapping_shl(shift))),
            Value::U32(a) => Ok(Value::U32(a.wrapping_shl(shift))),
            Value::U64(a) => Ok(Value::U64(a.wrapping_shl(shift))),
            _ => Err("Invalid type for shift left".to_string()),
        }
    }

    pub fn shift_right(&self, other: &Value) -> Result<Value, String> {
        let shift = match other {
            Value::I32(s) if *s >= 0 => *s as u32,
            Value::U32(s) => *s,
            _ => return Err("Shift amount must be non-negative integer".to_string()),
        };

        match self {
            Value::I8(a) => Ok(Value::I8(a.wrapping_shr(shift))),
            Value::I16(a) => Ok(Value::I16(a.wrapping_shr(shift))),
            Value::I32(a) => Ok(Value::I32(a.wrapping_shr(shift))),
            Value::I64(a) => Ok(Value::I64(a.wrapping_shr(shift))),
            Value::U8(a) => Ok(Value::U8(a.wrapping_shr(shift))),
            Value::U16(a) => Ok(Value::U16(a.wrapping_shr(shift))),
            Value::U32(a) => Ok(Value::U32(a.wrapping_shr(shift))),
            Value::U64(a) => Ok(Value::U64(a.wrapping_shr(shift))),
            _ => Err("Invalid type for shift right".to_string()),
        }
    }

    pub fn cast(&self, target_type: DataType) -> Result<Value, String> {
        match target_type {
            DataType::I8 => self.as_i8(),
            DataType::I16 => self.as_i16(),
            DataType::I32 => self.as_i32(),
            DataType::I64 => self.as_i64(),
            DataType::U8 => self.as_u8(),
            DataType::U16 => self.as_u16(),
            DataType::U32 => self.as_u32(),
            DataType::U64 => self.as_u64(),
            DataType::F32 => self.as_f32(),
            DataType::F64 => self.as_f64(),
            DataType::Ptr => Ok(Value::Ptr(self.as_usize()?)),
            DataType::Void => Err("Cannot cast to Void type".to_string()),
        }
    }

    fn as_i8(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::I8(*v)),
            Value::I16(v) => Ok(Value::I8(*v as i8)),
            Value::I32(v) => Ok(Value::I8(*v as i8)),
            Value::I64(v) => Ok(Value::I8(*v as i8)),
            Value::U8(v) => Ok(Value::I8(*v as i8)),
            Value::U16(v) => Ok(Value::I8(*v as i8)),
            Value::U32(v) => Ok(Value::I8(*v as i8)),
            Value::U64(v) => Ok(Value::I8(*v as i8)),
            Value::F32(v) => Ok(Value::I8(*v as i8)),
            Value::F64(v) => Ok(Value::I8(*v as i8)),
            Value::Ptr(v) => Ok(Value::I8(*v as i8)),
        }
    }

    fn as_i16(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::I16(*v as i16)),
            Value::I16(v) => Ok(Value::I16(*v)),
            Value::I32(v) => Ok(Value::I16(*v as i16)),
            Value::I64(v) => Ok(Value::I16(*v as i16)),
            Value::U8(v) => Ok(Value::I16(*v as i16)),
            Value::U16(v) => Ok(Value::I16(*v as i16)),
            Value::U32(v) => Ok(Value::I16(*v as i16)),
            Value::U64(v) => Ok(Value::I16(*v as i16)),
            Value::F32(v) => Ok(Value::I16(*v as i16)),
            Value::F64(v) => Ok(Value::I16(*v as i16)),
            Value::Ptr(v) => Ok(Value::I16(*v as i16)),
        }
    }

    fn as_i32(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::I32(*v as i32)),
            Value::I16(v) => Ok(Value::I32(*v as i32)),
            Value::I32(v) => Ok(Value::I32(*v)),
            Value::I64(v) => Ok(Value::I32(*v as i32)),
            Value::U8(v) => Ok(Value::I32(*v as i32)),
            Value::U16(v) => Ok(Value::I32(*v as i32)),
            Value::U32(v) => Ok(Value::I32(*v as i32)),
            Value::U64(v) => Ok(Value::I32(*v as i32)),
            Value::F32(v) => Ok(Value::I32(*v as i32)),
            Value::F64(v) => Ok(Value::I32(*v as i32)),
            Value::Ptr(v) => Ok(Value::I32(*v as i32)),
        }
    }

    fn as_i64(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::I64(*v as i64)),
            Value::I16(v) => Ok(Value::I64(*v as i64)),
            Value::I32(v) => Ok(Value::I64(*v as i64)),
            Value::I64(v) => Ok(Value::I64(*v)),
            Value::U8(v) => Ok(Value::I64(*v as i64)),
            Value::U16(v) => Ok(Value::I64(*v as i64)),
            Value::U32(v) => Ok(Value::I64(*v as i64)),
            Value::U64(v) => Ok(Value::I64(*v as i64)),
            Value::F32(v) => Ok(Value::I64(*v as i64)),
            Value::F64(v) => Ok(Value::I64(*v as i64)),
            Value::Ptr(v) => Ok(Value::I64(*v as i64)),
        }
    }

    fn as_u8(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::U8(*v as u8)),
            Value::I16(v) => Ok(Value::U8(*v as u8)),
            Value::I32(v) => Ok(Value::U8(*v as u8)),
            Value::I64(v) => Ok(Value::U8(*v as u8)),
            Value::U8(v) => Ok(Value::U8(*v)),
            Value::U16(v) => Ok(Value::U8(*v as u8)),
            Value::U32(v) => Ok(Value::U8(*v as u8)),
            Value::U64(v) => Ok(Value::U8(*v as u8)),
            Value::F32(v) => Ok(Value::U8(*v as u8)),
            Value::F64(v) => Ok(Value::U8(*v as u8)),
            Value::Ptr(v) => Ok(Value::U8(*v as u8)),
        }
    }

    fn as_u16(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::U16(*v as u16)),
            Value::I16(v) => Ok(Value::U16(*v as u16)),
            Value::I32(v) => Ok(Value::U16(*v as u16)),
            Value::I64(v) => Ok(Value::U16(*v as u16)),
            Value::U8(v) => Ok(Value::U16(*v as u16)),
            Value::U16(v) => Ok(Value::U16(*v)),
            Value::U32(v) => Ok(Value::U16(*v as u16)),
            Value::U64(v) => Ok(Value::U16(*v as u16)),
            Value::F32(v) => Ok(Value::U16(*v as u16)),
            Value::F64(v) => Ok(Value::U16(*v as u16)),
            Value::Ptr(v) => Ok(Value::U16(*v as u16)),
        }
    }

    fn as_u32(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::U32(*v as u32)),
            Value::I16(v) => Ok(Value::U32(*v as u32)),
            Value::I32(v) => Ok(Value::U32(*v as u32)),
            Value::I64(v) => Ok(Value::U32(*v as u32)),
            Value::U8(v) => Ok(Value::U32(*v as u32)),
            Value::U16(v) => Ok(Value::U32(*v as u32)),
            Value::U32(v) => Ok(Value::U32(*v)),
            Value::U64(v) => Ok(Value::U32(*v as u32)),
            Value::F32(v) => Ok(Value::U32(*v as u32)),
            Value::F64(v) => Ok(Value::U32(*v as u32)),
            Value::Ptr(v) => Ok(Value::U32(*v as u32)),
        }
    }

    fn as_u64(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::U64(*v as u64)),
            Value::I16(v) => Ok(Value::U64(*v as u64)),
            Value::I32(v) => Ok(Value::U64(*v as u64)),
            Value::I64(v) => Ok(Value::U64(*v as u64)),
            Value::U8(v) => Ok(Value::U64(*v as u64)),
            Value::U16(v) => Ok(Value::U64(*v as u64)),
            Value::U32(v) => Ok(Value::U64(*v as u64)),
            Value::U64(v) => Ok(Value::U64(*v)),
            Value::F32(v) => Ok(Value::U64(*v as u64)),
            Value::F64(v) => Ok(Value::U64(*v as u64)),
            Value::Ptr(v) => Ok(Value::U64(*v as u64)),
        }
    }

    fn as_f32(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::F32(*v as f32)),
            Value::I16(v) => Ok(Value::F32(*v as f32)),
            Value::I32(v) => Ok(Value::F32(*v as f32)),
            Value::I64(v) => Ok(Value::F32(*v as f32)),
            Value::U8(v) => Ok(Value::F32(*v as f32)),
            Value::U16(v) => Ok(Value::F32(*v as f32)),
            Value::U32(v) => Ok(Value::F32(*v as f32)),
            Value::U64(v) => Ok(Value::F32(*v as f32)),
            Value::F32(v) => Ok(Value::F32(*v)),
            Value::F64(v) => Ok(Value::F32(*v as f32)),
            Value::Ptr(v) => Ok(Value::F32(*v as f32)),
        }
    }

    fn as_f64(&self) -> Result<Value, String> {
        match self {
            Value::I8(v) => Ok(Value::F64(*v as f64)),
            Value::I16(v) => Ok(Value::F64(*v as f64)),
            Value::I32(v) => Ok(Value::F64(*v as f64)),
            Value::I64(v) => Ok(Value::F64(*v as f64)),
            Value::U8(v) => Ok(Value::F64(*v as f64)),
            Value::U16(v) => Ok(Value::F64(*v as f64)),
            Value::U32(v) => Ok(Value::F64(*v as f64)),
            Value::U64(v) => Ok(Value::F64(*v as f64)),
            Value::F32(v) => Ok(Value::F64(*v as f64)),
            Value::F64(v) => Ok(Value::F64(*v)),
            Value::Ptr(v) => Ok(Value::F64(*v as f64)),
        }
    }
}
