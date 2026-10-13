use std::fmt;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    RAX,
    RBX,
    RCX,
    RDX,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    RSP,
    RBP,
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Register::RAX => write!(f, "rax"),
            Register::RBX => write!(f, "rbx"),
            Register::RCX => write!(f, "rcx"),
            Register::RDX => write!(f, "rdx"),
            Register::R8 => write!(f, "r8"),
            Register::R9 => write!(f, "r9"),
            Register::R10 => write!(f, "r10"),
            Register::R11 => write!(f, "r11"),
            Register::R12 => write!(f, "r12"),
            Register::R13 => write!(f, "r13"),
            Register::R14 => write!(f, "r14"),
            Register::R15 => write!(f, "r15"),
            Register::RSP => write!(f, "rsp"),
            Register::RBP => write!(f, "rbp"),
        }
    }
}

impl Register {
    pub fn lower_32_bits(self) -> &'static str {
        match self {
            Register::RAX => "eax",
            Register::RBX => "ebx",
            Register::RCX => "ecx",
            Register::RDX => "edx",
            Register::R8 => "r8d",
            Register::R9 => "r9d",
            Register::R10 => "r10d",
            Register::R11 => "r11d",
            Register::R12 => "r12d",
            Register::R13 => "r13d",
            Register::R14 => "r14d",
            Register::R15 => "r15d",
            Register::RSP => "esp",
            Register::RBP => "ebp",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Address {
    Register(Register),
    Stack(i32), // Offset below rbp; negative offsets are above rbp (e.g. function args)
    StackArg(i32),
    Indexed { base: Register, index: Register },
}

impl Address {
    pub fn lower_8_bits(self) -> String {
        match self {
            Address::Stack(_) | Address::StackArg(_) | Address::Indexed { .. } => self.to_string(),
            Address::Register(reg) => match reg {
                Register::RAX => "al".to_string(),
                Register::RBX => "bl".to_string(),
                Register::RCX => "cl".to_string(),
                Register::RDX => "dl".to_string(),
                Register::R8 => "r8b".to_string(),
                Register::R9 => "r9b".to_string(),
                Register::R10 => "r10b".to_string(),
                Register::R11 => "r11b".to_string(),
                Register::R12 => "r12b".to_string(),
                Register::R13 => "r13b".to_string(),
                Register::R14 => "r14b".to_string(),
                Register::R15 => "r15b".to_string(),
                Register::RSP => "spl".to_string(),
                Register::RBP => "bpl".to_string(),
            },
        }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Address::Register(reg) => write!(f, "{reg}"),
            Address::Stack(off) if *off < 0 => write!(f, "[rbp + {}]", -off),
            Address::Stack(off) => write!(f, "[rbp - {off}]"),
            Address::StackArg(off) => write!(f, "[rsp + {off}]"),
            Address::Indexed { base, index } => write!(f, "[{base}+{index}]"),
        }
    }
}

impl From<Register> for Address {
    fn from(reg: Register) -> Self {
        Address::Register(reg)
    }
}
