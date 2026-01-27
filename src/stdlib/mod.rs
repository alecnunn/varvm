// Embedded standard library

pub const PRELUDE: &str = include_str!("../../stdlib/prelude.vasm");
pub const MATH: &str = include_str!("../../stdlib/math.vasm");

pub fn get_stdlib(name: &str) -> Option<&'static str> {
    match name {
        "stdlib/prelude.vasm" | "prelude.vasm" | "prelude" => Some(PRELUDE),
        "stdlib/math.vasm" | "math.vasm" | "math" => Some(MATH),
        _ => None,
    }
}

pub fn list_stdlib() -> Vec<&'static str> {
    vec!["prelude.vasm", "math.vasm"]
}
