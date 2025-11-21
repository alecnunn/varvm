# VarVM

A simple stack-based virtual machine implementation in Rust with support for multiple data types, dynamic memory allocation, and function calls.

## Building

```bash
cargo build
```

## Running Examples

The project includes several example programs demonstrating different VM features:

```bash
cargo run                # factorial (default)
cargo run factorial      # recursive factorial calculation
cargo run bitwise        # bitwise operations (AND, OR, XOR, NOT, shifts)
cargo run cast           # type conversions between numeric types
cargo run memory         # dynamic memory allocation and access
cargo run comparison     # comparison operators
```

## Instruction Set

The VM supports a fairly complete set of operations:

**Variable Management**
- `CreateLocal`, `CreateGlobal`, `DeleteLocal` - variable lifecycle
- `SetVar`, `CopyVar` - assignment operations

**Memory Operations**
- `Alloc`, `Free` - dynamic memory allocation
- `Load`, `Store` - memory access with type information
- `GetAddr` - get variable address

**Arithmetic**
- `Add`, `Sub`, `Mul`, `Div`, `Mod`, `Neg`

**Bitwise**
- `And`, `Or`, `Xor`, `Not`, `Shl`, `Shr`

**Comparisons**
- `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`

**Control Flow**
- `Label`, `Jmp`, `Jz`, `Jnz` - labels and conditional/unconditional jumps
- `FuncBegin`, `FuncEnd`, `Call`, `Return` - function definitions and calls
- `PopArg` - retrieve function arguments

**Misc**
- `Cast` - type conversions
- `Print` - debug output
- `Input` - read from stdin
- `Exit` - halt execution

## Type System

Supported types: `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64`, `F32`, `F64`, `Ptr`, `Void`

All arithmetic and bitwise operations check types at runtime and return appropriate errors for mismatches.

## Project Structure

- `src/types.rs` - type definitions and value operations
- `src/opcode.rs` - instruction set definition
- `src/program.rs` - program structure and builders
- `src/vm.rs` - execution engine
- `src/examples.rs` - example programs
- `src/main.rs` - entry point
