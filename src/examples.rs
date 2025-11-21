use crate::types::DataType;
use crate::opcode::OpCode;
use crate::program::{Program, Variable, Function};

pub fn bitwise_operations_test() -> Program {
    let mut prog = Program::new();

    prog.emit(OpCode::FuncBegin {
        name: "main".to_string(),
        return_type: DataType::I32,
    });

    // Test AND
    prog.create_local(DataType::I32, "a");
    prog.create_local(DataType::I32, "b");
    prog.create_local(DataType::I32, "result");

    prog.set_var("a", 0b1100);
    prog.set_var("b", 0b1010);
    prog.emit(OpCode::And {
        dest: "result".to_string(),
        left: "a".into(),
        right: "b".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be 0b1000 = 8

    // Test OR
    prog.emit(OpCode::Or {
        dest: "result".to_string(),
        left: "a".into(),
        right: "b".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be 0b1110 = 14

    // Test XOR
    prog.emit(OpCode::Xor {
        dest: "result".to_string(),
        left: "a".into(),
        right: "b".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be 0b0110 = 6

    // Test NOT
    prog.set_var("a", 0);
    prog.emit(OpCode::Not {
        dest: "result".to_string(),
        source: "a".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be -1

    // Test Shift Left
    prog.set_var("a", 5);
    prog.set_var("b", 2);
    prog.emit(OpCode::Shl {
        dest: "result".to_string(),
        left: "a".into(),
        right: "b".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be 20 (5 << 2)

    // Test Shift Right
    prog.set_var("a", 20);
    prog.set_var("b", 2);
    prog.emit(OpCode::Shr {
        dest: "result".to_string(),
        left: "a".into(),
        right: "b".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be 5 (20 >> 2)

    prog.emit(OpCode::Return { value: Some(0.into()) });
    let end = prog.emit(OpCode::FuncEnd);

    let mut main_func = Function::new("main".to_string(), DataType::I32);
    main_func.start_ip = 0;
    main_func.end_ip = end;
    prog.add_function(main_func);

    prog
}

pub fn type_cast_test() -> Program {
    let mut prog = Program::new();

    prog.emit(OpCode::FuncBegin {
        name: "main".to_string(),
        return_type: DataType::I32,
    });

    prog.create_local(DataType::I32, "i32_val");
    prog.create_local(DataType::F32, "f32_val");
    prog.create_local(DataType::I64, "i64_val");

    // Cast i32 to f32
    prog.set_var("i32_val", 42);
    prog.emit(OpCode::Cast {
        dest: "f32_val".to_string(),
        source: "i32_val".to_string(),
        target_type: DataType::F32,
    });
    prog.emit(OpCode::Print { var: "f32_val".to_string() }); // Should be 42.0

    // Cast f32 to i64
    prog.emit(OpCode::Cast {
        dest: "i64_val".to_string(),
        source: "f32_val".to_string(),
        target_type: DataType::I64,
    });
    prog.emit(OpCode::Print { var: "i64_val".to_string() }); // Should be 42

    prog.emit(OpCode::Return { value: Some(0.into()) });
    let end = prog.emit(OpCode::FuncEnd);

    let mut main_func = Function::new("main".to_string(), DataType::I32);
    main_func.start_ip = 0;
    main_func.end_ip = end;
    prog.add_function(main_func);

    prog
}

pub fn memory_operations_test() -> Program {
    let mut prog = Program::new();

    prog.emit(OpCode::FuncBegin {
        name: "main".to_string(),
        return_type: DataType::I32,
    });

    prog.create_local(DataType::Ptr, "ptr");
    prog.create_local(DataType::I32, "value");
    prog.create_local(DataType::I32, "loaded");

    // Allocate memory for one i32
    prog.emit(OpCode::Alloc {
        dest: "ptr".to_string(),
        size: 4.into(),
    });

    // Store value
    prog.set_var("value", 123);
    prog.emit(OpCode::Store {
        ptr: "ptr".to_string(),
        source: "value".to_string(),
        dtype: DataType::I32,
    });

    // Load value
    prog.emit(OpCode::Load {
        dest: "loaded".to_string(),
        ptr: "ptr".to_string(),
        dtype: DataType::I32,
    });
    prog.emit(OpCode::Print { var: "loaded".to_string() }); // Should be 123

    // Free memory
    prog.emit(OpCode::Free {
        ptr: "ptr".to_string(),
    });

    prog.emit(OpCode::Return { value: Some(0.into()) });
    let end = prog.emit(OpCode::FuncEnd);

    let mut main_func = Function::new("main".to_string(), DataType::I32);
    main_func.start_ip = 0;
    main_func.end_ip = end;
    prog.add_function(main_func);

    prog
}

pub fn comparison_test() -> Program {
    let mut prog = Program::new();

    prog.emit(OpCode::FuncBegin {
        name: "main".to_string(),
        return_type: DataType::I32,
    });

    prog.create_local(DataType::I32, "a");
    prog.create_local(DataType::I32, "b");
    prog.create_local(DataType::I32, "result");

    prog.set_var("a", 10);
    prog.set_var("b", 5);

    // Test Gt (greater than)
    prog.emit(OpCode::Gt {
        dest: "result".to_string(),
        left: "a".into(),
        right: "b".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be 1

    // Test Ge (greater or equal)
    prog.emit(OpCode::Ge {
        dest: "result".to_string(),
        left: "a".into(),
        right: "b".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be 1

    prog.set_var("a", 5);
    prog.emit(OpCode::Ge {
        dest: "result".to_string(),
        left: "a".into(),
        right: "b".into(),
    });
    prog.emit(OpCode::Print { var: "result".to_string() }); // Should be 1

    prog.emit(OpCode::Return { value: Some(0.into()) });
    let end = prog.emit(OpCode::FuncEnd);

    let mut main_func = Function::new("main".to_string(), DataType::I32);
    main_func.start_ip = 0;
    main_func.end_ip = end;
    prog.add_function(main_func);

    prog
}

pub fn factorial_program() -> Program {
    let mut prog = Program::new();

    // set global to store result
    prog.add_global(Variable::new("result".to_string(), DataType::I32, true));

    // main function
    let main_start = prog.emit(OpCode::FuncBegin {
        name: "main".to_string(),
        return_type: DataType::I32,
    });
    prog.create_local(DataType::I32, "n");
    prog.set_var("n", 5);
    prog.call(Some("result"), "factorial", &["n"]);
    prog.emit(OpCode::Print { var: "result".to_string() });
    prog.emit(OpCode::Return { value: Some(0.into()) });
    let main_end = prog.emit(OpCode::FuncEnd);

    let mut main_func = Function::new("main".to_string(), DataType::I32);
    main_func.start_ip = main_start;
    main_func.end_ip = main_end;
    prog.add_function(main_func);

    let factorial_start = prog.emit(OpCode::FuncBegin {
        name: "factorial".to_string(),
        return_type: DataType::I32,
    });
    prog.emit(OpCode::PopArg { dest: "n".to_string() });
    prog.create_local(DataType::I32, "temp");
    prog.set_var("temp", 1);
    prog.emit(OpCode::Le {
        dest: "temp".to_string(),
        left: "n".into(),
        right: 1.into(),
    });
    prog.emit(OpCode::Jnz {
        var: "temp".to_string(),
        label: "base_case".to_string(),
    });

    // recursive case
    prog.create_local(DataType::I32, "n_minus_1");
    prog.emit(OpCode::Sub {
        dest: "n_minus_1".to_string(),
        left: "n".into(),
        right: 1.into(),
    });
    prog.call(Some("temp"), "factorial", &["n_minus_1"]);
    prog.emit(OpCode::Mul {
        dest: "temp".to_string(),
        left: "n".into(),
        right: "temp".into(),
    });
    prog.emit(OpCode::Return { value: Some("temp".into()) });

    // Base case
    prog.emit(OpCode::Label { name: "base_case".to_string() });
    prog.emit(OpCode::Return { value: Some(1.into()) });
    let factorial_end = prog.emit(OpCode::FuncEnd);

    let mut factorial_func = Function::new("factorial".to_string(), DataType::I32);
    factorial_func.start_ip = factorial_start;
    factorial_func.end_ip = factorial_end;
    prog.add_function(factorial_func);

    prog
}
