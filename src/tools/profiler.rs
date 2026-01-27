use crate::opcode::OpCode;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ProfileData {
    pub total_instructions: u64,
    pub instruction_counts: HashMap<String, u64>,
    pub function_calls: HashMap<String, u64>,
    pub ip_counts: HashMap<usize, u64>,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
}

impl ProfileData {
    pub fn new() -> Self {
        Self {
            total_instructions: 0,
            instruction_counts: HashMap::new(),
            function_calls: HashMap::new(),
            ip_counts: HashMap::new(),
            start_time: None,
            end_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        self.end_time = Some(Instant::now());
    }

    pub fn record_instruction(&mut self, ip: usize, opcode: &OpCode) {
        self.total_instructions += 1;

        // Count by instruction type
        let opcode_name = Self::opcode_name(opcode);
        *self.instruction_counts.entry(opcode_name).or_insert(0) += 1;

        // Count by IP (hot spots)
        *self.ip_counts.entry(ip).or_insert(0) += 1;

        // Track function calls
        if let OpCode::Call { func, .. } = opcode {
            *self.function_calls.entry(func.clone()).or_insert(0) += 1;
        }
    }

    pub fn duration(&self) -> Option<Duration> {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => Some(end.duration_since(start)),
            _ => None,
        }
    }

    fn opcode_name(opcode: &OpCode) -> String {
        match opcode {
            OpCode::CreateLocal { .. } => "CreateLocal",
            OpCode::CreateGlobal { .. } => "CreateGlobal",
            OpCode::DeleteLocal { .. } => "DeleteLocal",
            OpCode::SetVar { .. } => "SetVar",
            OpCode::CopyVar { .. } => "CopyVar",
            OpCode::Alloc { .. } => "Alloc",
            OpCode::Free { .. } => "Free",
            OpCode::Load { .. } => "Load",
            OpCode::Store { .. } => "Store",
            OpCode::GetAddr { .. } => "GetAddr",
            OpCode::Add { .. } => "Add",
            OpCode::Sub { .. } => "Sub",
            OpCode::Mul { .. } => "Mul",
            OpCode::Div { .. } => "Div",
            OpCode::Mod { .. } => "Mod",
            OpCode::Neg { .. } => "Neg",
            OpCode::And { .. } => "And",
            OpCode::Or { .. } => "Or",
            OpCode::Xor { .. } => "Xor",
            OpCode::Not { .. } => "Not",
            OpCode::Shl { .. } => "Shl",
            OpCode::Shr { .. } => "Shr",
            OpCode::Eq { .. } => "Eq",
            OpCode::Ne { .. } => "Ne",
            OpCode::Lt { .. } => "Lt",
            OpCode::Le { .. } => "Le",
            OpCode::Gt { .. } => "Gt",
            OpCode::Ge { .. } => "Ge",
            OpCode::Label { .. } => "Label",
            OpCode::Jmp { .. } => "Jmp",
            OpCode::Jz { .. } => "Jz",
            OpCode::Jnz { .. } => "Jnz",
            OpCode::FuncBegin { .. } => "FuncBegin",
            OpCode::FuncEnd => "FuncEnd",
            OpCode::Call { .. } => "Call",
            OpCode::Return { .. } => "Return",
            OpCode::PushArg { .. } => "PushArg",
            OpCode::PopArg { .. } => "PopArg",
            OpCode::Cast { .. } => "Cast",
            OpCode::Sqrt { .. } => "Sqrt",
            OpCode::Pow { .. } => "Pow",
            OpCode::Abs { .. } => "Abs",
            OpCode::Min { .. } => "Min",
            OpCode::Max { .. } => "Max",
            OpCode::Sin { .. } => "Sin",
            OpCode::Cos { .. } => "Cos",
            OpCode::Tan { .. } => "Tan",
            OpCode::Print { .. } => "Print",
            OpCode::Input { .. } => "Input",
            OpCode::Exit { .. } => "Exit",
        }
        .to_string()
    }
}

pub struct Profiler {
    data: ProfileData,
}

impl Profiler {
    pub fn new() -> Self {
        Self {
            data: ProfileData::new(),
        }
    }

    pub fn start(&mut self) {
        self.data.start();
    }

    pub fn stop(&mut self) {
        self.data.stop();
    }

    pub fn record(&mut self, ip: usize, opcode: &OpCode) {
        self.data.record_instruction(ip, opcode);
    }

    pub fn get_data(&self) -> &ProfileData {
        &self.data
    }

    pub fn generate_report(&self, top_n: usize) -> String {
        let mut report = String::new();

        report.push_str("=== VarVM Profile Report ===\n\n");

        // Summary
        report.push_str(&format!(
            "Total Instructions: {}\n",
            self.data.total_instructions
        ));

        if let Some(duration) = self.data.duration() {
            let millis = duration.as_millis();
            report.push_str(&format!("Execution Time: {}ms\n", millis));

            if millis > 0 {
                let ips = (self.data.total_instructions as u128 * 1000) / millis;
                report.push_str(&format!("Instructions/sec: {}\n", ips));
            }
        }

        report.push_str("\n");

        // Instruction breakdown
        report.push_str("Instruction Breakdown:\n");
        let mut sorted_instrs: Vec<_> = self.data.instruction_counts.iter().collect();
        sorted_instrs.sort_by(|a, b| b.1.cmp(a.1));

        for (instr, count) in sorted_instrs.iter().take(top_n) {
            let percentage = (**count as f64 / self.data.total_instructions as f64) * 100.0;
            report.push_str(&format!(
                "  {:15} {:8} ({:5.1}%)\n",
                instr, count, percentage
            ));
        }

        report.push_str("\n");

        // Function calls
        if !self.data.function_calls.is_empty() {
            report.push_str("Function Call Counts:\n");
            let mut sorted_funcs: Vec<_> = self.data.function_calls.iter().collect();
            sorted_funcs.sort_by(|a, b| b.1.cmp(a.1));

            for (func, count) in sorted_funcs.iter().take(top_n) {
                report.push_str(&format!("  {:20} {:8} calls\n", func, count));
            }

            report.push_str("\n");
        }

        // Hot spots (most executed IPs)
        report.push_str(&format!("Top {} Hot Spots (by IP):\n", top_n));
        let mut sorted_ips: Vec<_> = self.data.ip_counts.iter().collect();
        sorted_ips.sort_by(|a, b| b.1.cmp(a.1));

        for (ip, count) in sorted_ips.iter().take(top_n) {
            let percentage = (**count as f64 / self.data.total_instructions as f64) * 100.0;
            report.push_str(&format!(
                "  IP {:4} {:8} executions ({:5.1}%)\n",
                ip, count, percentage
            ));
        }

        report
    }
}
