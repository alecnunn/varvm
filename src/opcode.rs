use crate::types::{DataType, Operand};

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    // variable management
    CreateLocal {
        dtype: DataType,
        name: String,
    },
    CreateGlobal {
        dtype: DataType,
        name: String,
    },
    DeleteLocal {
        name: String,
    },
    SetVar {
        dest: String,
        value: Operand,
    },
    CopyVar {
        dest: String,
        source: String,
    },

    // memory management
    Alloc {
        dest: String,
        size: Operand,
    },
    Free {
        ptr: String,
    },
    Load {
        dest: String,
        ptr: String,
        dtype: DataType,
    },
    Store {
        ptr: String,
        source: String,
        dtype: DataType,
    },
    GetAddr {
        dest: String,
        var: String,
    },

    // arithmetic operations
    Add {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Sub {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Mul {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Div {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Mod {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Neg {
        dest: String,
        source: Operand,
    },

    // bitwise operations
    And {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Or {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Xor {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Not {
        dest: String,
        source: Operand,
    },
    Shl {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Shr {
        dest: String,
        left: Operand,
        right: Operand,
    },

    // comparison operations
    Eq {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Ne {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Lt {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Le {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Gt {
        dest: String,
        left: Operand,
        right: Operand,
    },
    Ge {
        dest: String,
        left: Operand,
        right: Operand,
    },

    // control flow
    Label {
        name: String,
    },
    Jmp {
        label: String,
    },
    Jz {
        var: String,
        label: String,
    },
    Jnz {
        var: String,
        label: String,
    },

    // function operations
    FuncBegin {
        name: String,
        return_type: DataType,
    },
    FuncEnd,
    Call {
        result: Option<String>,
        func: String,
        args: Vec<String>,
    },
    Return {
        value: Option<Operand>,
    },
    PushArg {
        var: String,
    },
    PopArg {
        dest: String,
    },

    // type conversion
    Cast {
        dest: String,
        source: String,
        target_type: DataType,
    },

    // system operations
    Print {
        var: String,
    },
    Input {
        dest: String,
    },
    Exit {
        code: Operand,
    },
}
