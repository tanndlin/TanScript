use std::collections::HashMap;

pub const CODE_BASE: u64 = 0x4000_0000_0000;
pub const EXTERN_BASE: u64 = 0x5000_0000_0000;
pub const HALT: u64 = 0x6000_0000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Reg {
    pub index: usize,
    pub size: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Mem {
    pub base: Option<usize>,
    pub index: Option<usize>,
    pub scale: u8,
    pub disp: i64,
    pub symbol: Option<String>,
    pub size: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Operand {
    Reg(Reg),
    Imm(i64),
    Mem(Mem),
    Symbol(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cond {
    E,
    Ne,
    L,
    Le,
    G,
    Ge,
    B,
    Be,
    A,
    Ae,
    S,
    Ns,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Mov,
    Movzx,
    Movsx,
    Movsxd,
    Lea,
    Add,
    Sub,
    And,
    Or,
    Xor,
    Cmp,
    Test,
    Neg,
    Not,
    Inc,
    Dec,
    Imul,
    Div,
    Idiv,
    Cqo,
    Cdq,
    Shl,
    Shr,
    Sar,
    Push,
    Pop,
    Call,
    Ret,
    Jmp,
    Jcc(Cond),
    Set(Cond),
    Nop,
}

#[derive(Clone, Debug)]
pub struct Instr {
    pub op: Op,
    pub operands: Vec<Operand>,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct Program {
    pub instrs: Vec<Instr>,
    pub data: Vec<u8>,
    pub externs: Vec<String>,
    pub symbols: HashMap<String, u64>,
}

#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

enum Section {
    Text,
    Data,
}

const REGS64: [&str; 16] = [
    "rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13",
    "r14", "r15",
];
const REGS32: [&str; 16] = [
    "eax", "ecx", "edx", "ebx", "esp", "ebp", "esi", "edi", "r8d", "r9d", "r10d", "r11d", "r12d",
    "r13d", "r14d", "r15d",
];
const REGS16: [&str; 16] = [
    "ax", "cx", "dx", "bx", "sp", "bp", "si", "di", "r8w", "r9w", "r10w", "r11w", "r12w", "r13w",
    "r14w", "r15w",
];
const REGS8: [&str; 16] = [
    "al", "cl", "dl", "bl", "spl", "bpl", "sil", "dil", "r8b", "r9b", "r10b", "r11b", "r12b",
    "r13b", "r14b", "r15b",
];

pub fn register(name: &str) -> Option<Reg> {
    let name = name.to_ascii_lowercase();
    [(&REGS64, 8), (&REGS32, 4), (&REGS16, 2), (&REGS8, 1)]
        .into_iter()
        .find_map(|(table, size)| {
            table
                .iter()
                .position(|r| *r == name)
                .map(|index| Reg { index, size })
        })
}

fn size_keyword(word: &str) -> Option<u8> {
    match word.to_ascii_uppercase().as_str() {
        "BYTE" => Some(1),
        "WORD" => Some(2),
        "DWORD" => Some(4),
        "QWORD" => Some(8),
        _ => None,
    }
}

fn cond(suffix: &str) -> Option<Cond> {
    Some(match suffix {
        "e" | "z" => Cond::E,
        "ne" | "nz" => Cond::Ne,
        "l" | "nge" => Cond::L,
        "le" | "ng" => Cond::Le,
        "g" | "nle" => Cond::G,
        "ge" | "nl" => Cond::Ge,
        "b" | "c" | "nae" => Cond::B,
        "be" | "na" => Cond::Be,
        "a" | "nbe" => Cond::A,
        "ae" | "nc" | "nb" => Cond::Ae,
        "s" => Cond::S,
        "ns" => Cond::Ns,
        _ => return None,
    })
}

fn mnemonic(word: &str) -> Option<Op> {
    let word = word.to_ascii_lowercase();
    Some(match word.as_str() {
        "mov" => Op::Mov,
        "movzx" => Op::Movzx,
        "movsx" => Op::Movsx,
        "movsxd" => Op::Movsxd,
        "lea" => Op::Lea,
        "add" => Op::Add,
        "sub" => Op::Sub,
        "and" => Op::And,
        "or" => Op::Or,
        "xor" => Op::Xor,
        "cmp" => Op::Cmp,
        "test" => Op::Test,
        "neg" => Op::Neg,
        "not" => Op::Not,
        "inc" => Op::Inc,
        "dec" => Op::Dec,
        "imul" => Op::Imul,
        "div" => Op::Div,
        "idiv" => Op::Idiv,
        "cqo" => Op::Cqo,
        "cdq" => Op::Cdq,
        "shl" | "sal" => Op::Shl,
        "shr" => Op::Shr,
        "sar" => Op::Sar,
        "push" => Op::Push,
        "pop" => Op::Pop,
        "call" => Op::Call,
        "ret" => Op::Ret,
        "jmp" => Op::Jmp,
        "nop" => Op::Nop,
        _ => {
            if let Some(c) = word.strip_prefix("set").and_then(cond) {
                Op::Set(c)
            } else {
                Op::Jcc(word.strip_prefix('j').and_then(cond)?)
            }
        }
    })
}

fn strip_comment(line: &str) -> &str {
    let mut quote = None;
    for (i, c) in line.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            None if c == '"' || c == '\'' || c == '`' => quote = Some(c),
            None if c == ';' => return &line[..i],
            Some(_) | None => {}
        }
    }
    line
}

fn split_top_level(text: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut quote = None;
    let mut depth = 0;
    let mut start = 0;
    for (i, c) in text.char_indices() {
        match quote {
            Some(q) if c == q => quote = None,
            Some(_) => {}
            None => match c {
                '"' | '\'' | '`' => quote = Some(c),
                '[' => depth += 1,
                ']' => depth -= 1,
                ',' if depth == 0 => {
                    parts.push(text[start..i].trim());
                    start = i + 1;
                }
                _ => {}
            },
        }
    }
    let last = text[start..].trim();
    if !last.is_empty() || !parts.is_empty() {
        parts.push(last);
    }
    parts
}

fn number(text: &str) -> Option<i64> {
    let text = text.trim();
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest.trim()),
        None => (false, text),
    };
    let value = if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok()?
    } else if let Some(hex) = digits
        .strip_suffix('h')
        .or_else(|| digits.strip_suffix('H'))
        && digits.starts_with(|c: char| c.is_ascii_digit())
    {
        u64::from_str_radix(hex, 16).ok()?
    } else if digits.len() == 3 && digits.starts_with('\'') && digits.ends_with('\'') {
        u64::from(digits.as_bytes()[1])
    } else {
        digits.parse::<u64>().ok()?
    };
    let value = value as i64;
    Some(if negative {
        value.wrapping_neg()
    } else {
        value
    })
}

fn is_identifier(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '$' | '@' | '?'))
        && !text.starts_with(|c: char| c.is_ascii_digit())
}

fn memory(inner: &str, size: Option<u8>) -> Result<Mem, String> {
    let mut mem = Mem {
        base: None,
        index: None,
        scale: 1,
        disp: 0,
        symbol: None,
        size,
    };
    let mut terms = Vec::new();
    let mut sign = 1i64;
    let mut current = String::new();
    for c in inner.chars() {
        if c == '+' || c == '-' {
            if !current.trim().is_empty() {
                terms.push((sign, current.trim().to_string()));
            }
            current.clear();
            sign = if c == '-' { -1 } else { 1 };
        } else {
            current.push(c);
        }
    }
    if !current.trim().is_empty() {
        terms.push((sign, current.trim().to_string()));
    }

    for (sign, term) in terms {
        if let Some((a, b)) = term.split_once('*') {
            let (a, b) = (a.trim(), b.trim());
            let (reg, scale) = match (register(a), register(b)) {
                (Some(r), None) => (r, number(b)),
                (None, Some(r)) => (r, number(a)),
                _ => return Err(format!("bad scaled term `{term}`")),
            };
            let scale = scale.ok_or_else(|| format!("bad scale in `{term}`"))?;
            if sign < 0 || mem.index.is_some() || !matches!(scale, 1 | 2 | 4 | 8) {
                return Err(format!("bad scaled term `{term}`"));
            }
            mem.index = Some(reg.index);
            let scale = scale as u8;
            mem.scale = scale;
        } else if let Some(reg) = register(&term) {
            if sign < 0 {
                return Err(format!("cannot subtract register `{term}`"));
            }
            if mem.base.is_none() {
                mem.base = Some(reg.index);
            } else if mem.index.is_none() {
                mem.index = Some(reg.index);
            } else {
                return Err("too many registers in address".to_string());
            }
        } else if let Some(n) = number(&term) {
            mem.disp = mem.disp.wrapping_add(sign * n);
        } else if is_identifier(&term) && sign > 0 && mem.symbol.is_none() {
            mem.symbol = Some(term);
        } else {
            return Err(format!("bad address term `{term}`"));
        }
    }
    Ok(mem)
}

fn operand(text: &str) -> Result<Operand, String> {
    let mut text = text.trim();
    let mut size = None;
    if let Some((first, rest)) = text.split_once(char::is_whitespace)
        && let Some(s) = size_keyword(first)
    {
        size = Some(s);
        text = rest.trim();
    }
    if let Some(rest) = text.strip_prefix("rel ") {
        text = rest.trim();
    }

    if let Some(inner) = text.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
        return memory(inner, size).map(Operand::Mem);
    }
    if let Some(reg) = register(text) {
        return Ok(Operand::Reg(reg));
    }
    if let Some(n) = number(text) {
        return Ok(Operand::Imm(n));
    }
    if is_identifier(text) {
        return Ok(Operand::Symbol(text.to_string()));
    }
    Err(format!("bad operand `{text}`"))
}

fn data_items(text: &str, width: usize, out: &mut Vec<u8>) -> Result<(), String> {
    for item in split_top_level(text) {
        let quoted = item.len() >= 2
            && ((item.starts_with('"') && item.ends_with('"'))
                || (item.starts_with('\'') && item.ends_with('\'')));
        if quoted {
            let bytes = &item.as_bytes()[1..item.len() - 1];
            out.extend_from_slice(bytes);
            let pad = (width - bytes.len() % width) % width;
            out.extend(std::iter::repeat_n(0, pad));
        } else if let Some(n) = number(item) {
            out.extend_from_slice(&n.to_le_bytes()[..width]);
        } else {
            return Err(format!("bad data item `{item}`"));
        }
    }
    Ok(())
}

fn data_width(directive: &str) -> Option<usize> {
    match directive.to_ascii_lowercase().as_str() {
        "db" => Some(1),
        "dw" => Some(2),
        "dd" => Some(4),
        "dq" => Some(8),
        _ => None,
    }
}

pub fn parse(source: &str, data_base: u64) -> Result<Program, ParseError> {
    let mut program = Program::default();
    let mut section = Section::Text;
    let mut code_labels: Vec<(String, usize, usize)> = Vec::new();
    let mut data_labels: Vec<(String, usize, usize)> = Vec::new();

    for (number, raw) in source.lines().enumerate() {
        let line = number + 1;
        let err = |message: String| ParseError { line, message };
        let text = strip_comment(raw).trim();
        if text.is_empty() {
            continue;
        }
        let (head, rest) = match text.split_once(char::is_whitespace) {
            Some((h, r)) => (h, r.trim()),
            None => (text, ""),
        };

        match head.to_ascii_lowercase().as_str() {
            "bits" | "global" | "default" => continue,
            "extern" => {
                for name in rest.split(',') {
                    program.externs.push(name.trim().to_string());
                }
                continue;
            }
            "section" | "segment" => {
                let name = rest.split_whitespace().next().unwrap_or_default();
                section = if name.starts_with(".text") {
                    Section::Text
                } else {
                    Section::Data
                };
                continue;
            }
            _ => {}
        }

        let (label, body) = match text.split_once(':') {
            Some((l, b)) if is_identifier(l.trim()) && !l.contains(char::is_whitespace) => {
                (Some(l.trim()), b.trim())
            }
            _ => (None, text),
        };

        match section {
            Section::Text => {
                if let Some(label) = label {
                    code_labels.push((label.to_string(), program.instrs.len(), line));
                }
                if body.is_empty() {
                    continue;
                }
                let (word, args) = match body.split_once(char::is_whitespace) {
                    Some((w, a)) => (w, a.trim()),
                    None => (body, ""),
                };
                let op =
                    mnemonic(word).ok_or_else(|| err(format!("unknown instruction `{word}`")))?;
                let operands = split_top_level(args)
                    .into_iter()
                    .map(operand)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(err)?;
                program.instrs.push(Instr { op, operands, line });
            }
            Section::Data => {
                let mut words = body.splitn(2, char::is_whitespace);
                let first = words.next().unwrap_or_default();
                let after = words.next().unwrap_or_default().trim();
                let (name, directive, items) = if let Some(width) = data_width(first) {
                    (label, width, after)
                } else {
                    let mut more = after.splitn(2, char::is_whitespace);
                    let directive = more.next().unwrap_or_default();
                    let width = data_width(directive)
                        .ok_or_else(|| err(format!("unknown data directive `{directive}`")))?;
                    (Some(first), width, more.next().unwrap_or_default().trim())
                };
                if let Some(name) = name {
                    data_labels.push((name.to_string(), program.data.len(), line));
                }
                data_items(items, directive, &mut program.data).map_err(err)?;
            }
        }
    }

    for (i, name) in program.externs.iter().enumerate() {
        program.symbols.insert(name.clone(), EXTERN_BASE + i as u64);
    }
    for (name, index, line) in code_labels {
        if program
            .symbols
            .insert(name.clone(), CODE_BASE + index as u64)
            .is_some()
        {
            return Err(ParseError {
                line,
                message: format!("duplicate label `{name}`"),
            });
        }
    }
    for (name, offset, line) in data_labels {
        if program
            .symbols
            .insert(name.clone(), data_base + offset as u64)
            .is_some()
        {
            return Err(ParseError {
                line,
                message: format!("duplicate label `{name}`"),
            });
        }
    }

    let symbols = &program.symbols;
    for instr in &mut program.instrs {
        let line = instr.line;
        for op in &mut instr.operands {
            let resolve = |name: &str| {
                symbols.get(name).copied().ok_or_else(|| ParseError {
                    line,
                    message: format!("undefined symbol `{name}`"),
                })
            };
            match op {
                Operand::Symbol(name) => {
                    let value = resolve(name)? as i64;
                    *op = Operand::Imm(value);
                }
                Operand::Mem(mem) => {
                    if let Some(name) = mem.symbol.take() {
                        let value = resolve(&name)? as i64;
                        mem.disp = mem.disp.wrapping_add(value);
                    }
                }
                Operand::Reg(_) | Operand::Imm(_) => {}
            }
        }
    }

    Ok(program)
}
