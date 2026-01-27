use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use varvm::asm::{assemble, disassemble};
use varvm::bytecode::{encode, decode};
use varvm::vm::VM;

#[derive(Parser)]
#[command(name = "varvm")]
#[command(about = "VarVM - A variable-based virtual machine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Assemble .vasm file to bytecode")]
    Assemble {
        #[arg(help = "Input .vasm file")]
        input: PathBuf,

        #[arg(short, long, help = "Output .vbc bytecode file")]
        output: PathBuf,
    },

    #[command(about = "Run a .vbc bytecode file")]
    Run {
        #[arg(help = "Bytecode file to run")]
        input: PathBuf,
    },

    #[command(about = "Assemble and run a .vasm file")]
    AsmRun {
        #[arg(help = "Input .vasm file")]
        input: PathBuf,
    },

    #[command(about = "Disassemble bytecode or program back to .vasm")]
    Disasm {
        #[arg(help = "Input .vasm file")]
        input: PathBuf,

        #[arg(short, long, help = "Output .vasm file")]
        output: Option<PathBuf>,
    },

    #[command(about = "Profile a .vasm file")]
    Profile {
        #[arg(help = "Input .vasm file")]
        input: PathBuf,

        #[arg(long, default_value_t = 10, help = "Number of top items to show")]
        top: usize,
    },

    #[command(about = "List available standard library files")]
    ListStdlib,

    #[command(about = "Extract a standard library file")]
    ExtractStdlib {
        #[arg(help = "Library name (prelude, math)")]
        name: String,

        #[arg(short, long, help = "Output file (default: <name>.vasm)")]
        output: Option<PathBuf>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Assemble { input, output } => {
            if let Err(e) = assemble_command(input, output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Run { input } => {
            if let Err(e) = run_command(input) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::AsmRun { input } => {
            if let Err(e) = asm_run_command(input) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Disasm { input, output } => {
            if let Err(e) = disasm_command(input, output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Profile { input, top } => {
            if let Err(e) = profile_command(input, top) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::ListStdlib => {
            list_stdlib_command();
        },
        Commands::ExtractStdlib { name, output } => {
            if let Err(e) = extract_stdlib_command(name, output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}

fn assemble_command(input: PathBuf, output: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(&input)?;
    let filename = input.to_string_lossy().to_string();

    println!("Assembling {}...", filename);
    let program = assemble(&source, filename)?;

    println!("Encoding to bytecode...");
    let bytecode = encode(&program)?;

    fs::write(&output, bytecode)?;

    println!("Bytecode written to {}", output.display());
    println!("  Instructions: {}", program.instructions.len());
    println!("  Globals: {}", program.globals.len());
    println!("  Functions: {}", program.functions.len());
    println!("  Labels: {}", program.labels.len());

    Ok(())
}

fn run_command(input: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading bytecode from {}...", input.display());
    let bytecode = fs::read(&input)?;

    println!("Decoding bytecode...");
    let program = decode(&bytecode)?;

    println!("Running program...\n");
    let mut vm = VM::new(program);
    match vm.run() {
        Ok(exit_code) => {
            println!("\nProgram exited with code: {}", exit_code);
            Ok(())
        },
        Err(e) => {
            Err(format!("Runtime error: {}", e).into())
        }
    }
}

fn asm_run_command(input: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(&input)?;
    let filename = input.to_string_lossy().to_string();

    println!("Assembling {}...", filename);
    let program = assemble(&source, filename)?;

    println!("Running program...\n");
    let mut vm = VM::new(program);
    match vm.run() {
        Ok(exit_code) => {
            println!("\nProgram exited with code: {}", exit_code);
            Ok(())
        },
        Err(e) => {
            Err(format!("Runtime error: {}", e).into())
        }
    }
}

fn disasm_command(input: PathBuf, output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let source = fs::read_to_string(&input)?;
    let filename = input.to_string_lossy().to_string();

    println!("Assembling {}...", filename);
    let program = assemble(&source, filename)?;

    println!("Disassembling program...");
    let disasm = disassemble(&program);

    if let Some(output_path) = output {
        fs::write(&output_path, &disasm)?;
        println!("Disassembly written to {}", output_path.display());
    } else {
        println!("\n{}", disasm);
    }

    Ok(())
}

fn profile_command(input: PathBuf, top: usize) -> Result<(), Box<dyn std::error::Error>> {
    use varvm::tools::profiler::Profiler;

    let source = fs::read_to_string(&input)?;
    let filename = input.to_string_lossy().to_string();

    println!("Assembling {}...", filename);
    let program = assemble(&source, filename)?;

    println!("Running with profiling enabled...\n");
    let mut vm = VM::new(program);
    vm.enable_profiling();

    match vm.run() {
        Ok(exit_code) => {
            println!("\nProgram exited with code: {}", exit_code);
        },
        Err(e) => {
            eprintln!("\nRuntime error: {}", e);
        }
    }

    vm.disable_profiling();

    // Generate and display report
    let profile_data = vm.get_profile_data();
    let mut profiler = Profiler::new();
    // Copy the profile data (we need to create a method for this or expose the data differently)
    // For now, let's just use the data directly
    let report = generate_profile_report(profile_data, top);
    println!("\n{}", report);

    Ok(())
}

fn generate_profile_report(data: &varvm::tools::profiler::ProfileData, top_n: usize) -> String {
    use varvm::tools::profiler::Profiler;

    // Create a profiler and generate report
    // This is a bit hacky - we should refactor the Profiler to work better
    let mut report = String::new();

    report.push_str("=== VarVM Profile Report ===\n\n");

    // Summary
    report.push_str(&format!(
        "Total Instructions: {}\n",
        data.total_instructions
    ));

    if let Some(duration) = data.duration() {
        let millis = duration.as_millis();
        report.push_str(&format!("Execution Time: {}ms\n", millis));

        if millis > 0 {
            let ips = (data.total_instructions as u128 * 1000) / millis;
            report.push_str(&format!("Instructions/sec: {}\n", ips));
        }
    }

    report.push_str("\n");

    // Instruction breakdown
    report.push_str("Instruction Breakdown:\n");
    let mut sorted_instrs: Vec<_> = data.instruction_counts.iter().collect();
    sorted_instrs.sort_by(|a, b| b.1.cmp(a.1));

    for (instr, count) in sorted_instrs.iter().take(top_n) {
        let percentage = (**count as f64 / data.total_instructions as f64) * 100.0;
        report.push_str(&format!(
            "  {:15} {:8} ({:5.1}%)\n",
            instr, count, percentage
        ));
    }

    report.push_str("\n");

    // Function calls
    if !data.function_calls.is_empty() {
        report.push_str("Function Call Counts:\n");
        let mut sorted_funcs: Vec<_> = data.function_calls.iter().collect();
        sorted_funcs.sort_by(|a, b| b.1.cmp(a.1));

        for (func, count) in sorted_funcs.iter().take(top_n) {
            report.push_str(&format!("  {:20} {:8} calls\n", func, count));
        }

        report.push_str("\n");
    }

    // Hot spots
    report.push_str(&format!("Top {} Hot Spots (by IP):\n", top_n));
    let mut sorted_ips: Vec<_> = data.ip_counts.iter().collect();
    sorted_ips.sort_by(|a, b| b.1.cmp(a.1));

    for (ip, count) in sorted_ips.iter().take(top_n) {
        let percentage = (**count as f64 / data.total_instructions as f64) * 100.0;
        report.push_str(&format!(
            "  IP {:4} {:8} executions ({:5.1}%)\n",
            ip, count, percentage
        ));
    }

    report
}

fn list_stdlib_command() {
    use varvm::stdlib;

    println!("VarVM Standard Library Files:");
    println!();
    for name in stdlib::list_stdlib() {
        println!("  {}", name);
    }
    println!();
    println!("Extract a library file with:");
    println!("  varvm extract-stdlib <name>");
}

fn extract_stdlib_command(name: String, output: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    use varvm::stdlib;

    let content = stdlib::get_stdlib(&name)
        .ok_or_else(|| format!("Unknown library: {}. Use 'varvm list-stdlib' to see available libraries.", name))?;

    let output_path = output.unwrap_or_else(|| {
        let filename = if name.ends_with(".vasm") {
            name.clone()
        } else {
            format!("{}.vasm", name)
        };
        PathBuf::from(filename)
    });

    fs::write(&output_path, content)?;
    println!("Extracted {} to {}", name, output_path.display());

    Ok(())
}
