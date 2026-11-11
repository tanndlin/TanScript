use std::collections::HashMap;

use crate::parse::{Cond, EXTERN_BASE, HALT, Instr, Mem, Op, Operand, Program, Reg};

pub const NULL_GUARD: u64 = 0x1_0000;
pub const STACK_SIZE: u64 = 0x10_0000;
pub const STACK_TOP: u64 = NULL_GUARD + STACK_SIZE;
pub const DATA_BASE: u64 = STACK_TOP;

pub const RAX: usize = 0;
pub const RCX: usize = 1;
pub const RDX: usize = 2;
pub const RSP: usize = 4;
pub const R8: usize = 8;
pub const R9: usize = 9;

pub struct Memory {
    pub bytes: Vec<u8>,
    pub limit: usize,
}

impl Memory {
    fn check(&self, addr: u64, len: u64) -> Result<usize, String> {
        if addr < NULL_GUARD {
            return Err(if addr >= NULL_GUARD - 0x1000 {
                "stack overflow".to_string()
            } else {
                format!("invalid memory access at {addr:#x} (null pointer?)")
            });
        }
        match addr.checked_add(len) {
            Some(end) if end <= self.bytes.len() as u64 => {
                usize::try_from(addr).map_err(|e| e.to_string())
            }
            _ => Err(format!("invalid memory access at {addr:#x}")),
        }
    }

    pub fn read(&self, addr: u64, size: u8) -> Result<u64, String> {
        let start = self.check(addr, u64::from(size))?;
        let mut buf = [0u8; 8];
        buf[..size as usize].copy_from_slice(&self.bytes[start..start + size as usize]);
        Ok(u64::from_le_bytes(buf))
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) -> Result<(), String> {
        let start = self.check(addr, u64::from(size))?;
        self.bytes[start..start + size as usize]
            .copy_from_slice(&value.to_le_bytes()[..size as usize]);
        Ok(())
    }

    pub fn slice(&self, addr: u64, len: u64) -> Result<&[u8], String> {
        let start = self.check(addr, len)?;
        Ok(&self.bytes[start..start + usize::try_from(len).map_err(|e| e.to_string())?])
    }

    pub fn slice_mut(&mut self, addr: u64, len: u64) -> Result<&mut [u8], String> {
        let start = self.check(addr, len)?;
        Ok(&mut self.bytes[start..start + usize::try_from(len).map_err(|e| e.to_string())?])
    }

    pub fn cstr(&self, addr: u64) -> Result<&[u8], String> {
        let start = self.check(addr, 1)?;
        let len = self.bytes[start..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| format!("unterminated string at {addr:#x}"))?;
        Ok(&self.bytes[start..start + len])
    }

    pub fn grow(&mut self, end: u64) -> Result<(), String> {
        let end = usize::try_from(end).map_err(|e| e.to_string())?;
        if end > self.limit {
            return Err("out of memory".to_string());
        }
        if end > self.bytes.len() {
            let new_len = end.max(self.bytes.len() * 2).min(self.limit);
            self.bytes.resize(new_len, 0);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct Flags {
    pub zf: bool,
    pub sf: bool,
    pub of: bool,
    pub cf: bool,
}

pub struct Heap {
    pub next: u64,
    pub sizes: HashMap<u64, u64>,
    pub free: HashMap<u64, Vec<u64>>,
}

pub struct File {
    pub data: Vec<u8>,
    pub pos: usize,
}

pub struct Machine {
    pub regs: [u64; 16],
    pub flags: Flags,
    pub mem: Memory,
    pub heap: Heap,
    pub stdout: Vec<u8>,
    pub files: HashMap<String, Vec<u8>>,
    pub open_files: HashMap<u64, File>,
    pub next_handle: u64,
    pub exit_code: Option<i32>,
    pub time: u64,
    pub rand_state: u32,
}

pub enum Flow {
    Next,
    Jump(u64),
    Call(u64),
    Halt,
}

fn mask(size: u8) -> u64 {
    if size == 8 {
        u64::MAX
    } else {
        (1u64 << (u32::from(size) * 8)) - 1
    }
}

fn sign_bit(size: u8) -> u64 {
    1u64 << (u32::from(size) * 8 - 1)
}

pub fn sign_extend(value: u64, size: u8) -> i64 {
    let shift = 64 - u32::from(size) * 8;
    ((value << shift) as i64) >> shift
}

impl Machine {
    pub fn new(program: &Program, memory_limit: usize) -> Result<Self, String> {
        let data_end = DATA_BASE + program.data.len() as u64;
        let heap_start = (data_end + 15) & !15;
        let mut mem = Memory {
            bytes: Vec::new(),
            limit: memory_limit,
        };
        mem.grow(heap_start.max(DATA_BASE + 0x1000))?;
        mem.slice_mut(DATA_BASE, program.data.len() as u64)?
            .copy_from_slice(&program.data);
        let mut machine = Machine {
            regs: [0; 16],
            flags: Flags::default(),
            mem,
            heap: Heap {
                next: heap_start,
                sizes: HashMap::new(),
                free: HashMap::new(),
            },
            stdout: Vec::new(),
            files: HashMap::new(),
            open_files: HashMap::new(),
            next_handle: 0x7000_0000,
            exit_code: None,
            time: 0,
            rand_state: 1,
        };
        machine.regs[RSP] = STACK_TOP;
        machine.push(HALT)?;
        Ok(machine)
    }

    pub fn get_reg(&self, reg: Reg) -> u64 {
        self.regs[reg.index] & mask(reg.size)
    }

    pub fn set_reg(&mut self, reg: Reg, value: u64) {
        let slot = &mut self.regs[reg.index];
        *slot = match reg.size {
            8 => value,
            4 => value & 0xFFFF_FFFF,
            size => (*slot & !mask(size)) | (value & mask(size)),
        };
    }

    pub fn address(&self, mem: &Mem) -> u64 {
        let mut addr = mem.disp.cast_unsigned();
        if let Some(base) = mem.base {
            addr = addr.wrapping_add(self.regs[base]);
        }
        if let Some(index) = mem.index {
            addr = addr.wrapping_add(self.regs[index].wrapping_mul(u64::from(mem.scale)));
        }
        addr
    }

    pub fn push(&mut self, value: u64) -> Result<(), String> {
        self.regs[RSP] = self.regs[RSP].wrapping_sub(8);
        self.mem.write(self.regs[RSP], 8, value)
    }

    pub fn pop(&mut self) -> Result<u64, String> {
        let value = self.mem.read(self.regs[RSP], 8)?;
        self.regs[RSP] = self.regs[RSP].wrapping_add(8);
        Ok(value)
    }

    fn read(&self, op: &Operand, size: u8) -> Result<u64, String> {
        Ok(match op {
            Operand::Reg(reg) => self.get_reg(*reg),
            Operand::Imm(n) => n.cast_unsigned() & mask(size),
            Operand::Mem(mem) => self.mem.read(self.address(mem), size)?,
            Operand::Symbol(name) => return Err(format!("unresolved symbol `{name}`")),
        })
    }

    fn write(&mut self, op: &Operand, size: u8, value: u64) -> Result<(), String> {
        match op {
            Operand::Reg(reg) => {
                self.set_reg(Reg { size, ..*reg }, value);
                Ok(())
            }
            Operand::Mem(mem) => self.mem.write(self.address(mem), size, value & mask(size)),
            Operand::Imm(_) | Operand::Symbol(_) => Err("cannot write to an immediate".to_string()),
        }
    }

    fn set_result_flags(&mut self, result: u64, size: u8) {
        self.flags.zf = result & mask(size) == 0;
        self.flags.sf = result & sign_bit(size) != 0;
    }

    fn add_flags(&mut self, a: u64, b: u64, size: u8) -> u64 {
        let result = a.wrapping_add(b) & mask(size);
        let sb = sign_bit(size);
        self.flags.cf = result < a;
        self.flags.of = (a & sb) == (b & sb) && (result & sb) != (a & sb);
        self.set_result_flags(result, size);
        result
    }

    fn sub_flags(&mut self, a: u64, b: u64, size: u8) -> u64 {
        let result = a.wrapping_sub(b) & mask(size);
        let sb = sign_bit(size);
        self.flags.cf = a < b;
        self.flags.of = (a & sb) != (b & sb) && (result & sb) != (a & sb);
        self.set_result_flags(result, size);
        result
    }

    fn logic_flags(&mut self, result: u64, size: u8) -> u64 {
        self.flags.cf = false;
        self.flags.of = false;
        self.set_result_flags(result, size);
        result & mask(size)
    }

    pub fn condition(&self, cond: Cond) -> bool {
        let f = &self.flags;
        match cond {
            Cond::E => f.zf,
            Cond::Ne => !f.zf,
            Cond::L => f.sf != f.of,
            Cond::Le => f.zf || f.sf != f.of,
            Cond::G => !f.zf && f.sf == f.of,
            Cond::Ge => f.sf == f.of,
            Cond::B => f.cf,
            Cond::Be => f.cf || f.zf,
            Cond::A => !f.cf && !f.zf,
            Cond::Ae => !f.cf,
            Cond::S => f.sf,
            Cond::Ns => !f.sf,
        }
    }

    fn operand_size(ops: &[Operand]) -> Result<u8, String> {
        ops.iter()
            .find_map(|op| match op {
                Operand::Reg(r) => Some(r.size),
                _ => None,
            })
            .or_else(|| {
                ops.iter().find_map(|op| match op {
                    Operand::Mem(m) => m.size,
                    _ => None,
                })
            })
            .ok_or_else(|| "operation size not specified".to_string())
    }

    fn own_size(op: &Operand) -> Result<u8, String> {
        match op {
            Operand::Reg(r) => Ok(r.size),
            Operand::Mem(m) => m
                .size
                .ok_or_else(|| "operation size not specified".to_string()),
            _ => Err("expected register or memory operand".to_string()),
        }
    }

    pub fn exec(&mut self, instr: &Instr, program: &Program) -> Result<Flow, String> {
        let ops = instr.operands.as_slice();
        let arity = |n: usize| {
            if ops.len() == n {
                Ok(())
            } else {
                Err(format!("expected {n} operand(s), found {}", ops.len()))
            }
        };

        match instr.op {
            Op::Nop => arity(0)?,
            Op::Mov => {
                arity(2)?;
                let size = match &ops[0] {
                    Operand::Reg(r) => r.size,
                    _ => Self::operand_size(ops)?,
                };
                let value = self.read(&ops[1], size)?;
                self.write(&ops[0], size, value)?;
            }
            Op::Movzx | Op::Movsx | Op::Movsxd => {
                arity(2)?;
                let dst = Self::own_size(&ops[0])?;
                let src = Self::own_size(&ops[1])?;
                let value = self.read(&ops[1], src)?;
                let value = if instr.op == Op::Movzx {
                    value
                } else {
                    sign_extend(value, src).cast_unsigned()
                };
                self.write(&ops[0], dst, value)?;
            }
            Op::Lea => {
                arity(2)?;
                let Operand::Mem(mem) = &ops[1] else {
                    return Err("lea needs a memory operand".to_string());
                };
                let size = Self::own_size(&ops[0])?;
                let addr = self.address(mem);
                self.write(&ops[0], size, addr)?;
            }
            Op::Add | Op::Sub | Op::Cmp | Op::And | Op::Or | Op::Xor | Op::Test => {
                arity(2)?;
                let size = Self::operand_size(ops)?;
                let a = self.read(&ops[0], size)?;
                let b = self.read(&ops[1], size)?;
                let result = match instr.op {
                    Op::Add => self.add_flags(a, b, size),
                    Op::Sub | Op::Cmp => self.sub_flags(a, b, size),
                    Op::And | Op::Test => self.logic_flags(a & b, size),
                    Op::Or => self.logic_flags(a | b, size),
                    _ => self.logic_flags(a ^ b, size),
                };
                if !matches!(instr.op, Op::Cmp | Op::Test) {
                    self.write(&ops[0], size, result)?;
                }
            }
            Op::Neg | Op::Not | Op::Inc | Op::Dec => {
                arity(1)?;
                let size = Self::own_size(&ops[0])?;
                let a = self.read(&ops[0], size)?;
                let result = match instr.op {
                    Op::Neg => {
                        let r = self.sub_flags(0, a, size);
                        self.flags.cf = a != 0;
                        r
                    }
                    Op::Not => !a & mask(size),
                    Op::Inc => {
                        let cf = self.flags.cf;
                        let r = self.add_flags(a, 1, size);
                        self.flags.cf = cf;
                        r
                    }
                    _ => {
                        let cf = self.flags.cf;
                        let r = self.sub_flags(a, 1, size);
                        self.flags.cf = cf;
                        r
                    }
                };
                self.write(&ops[0], size, result)?;
            }
            Op::Imul => {
                let (dst, a, b) = match ops {
                    [dst, src] => (dst, dst, src),
                    [dst, src, imm] => (dst, src, imm),
                    _ => return Err("imul needs 2 or 3 operands".to_string()),
                };
                let size = Self::own_size(dst)?;
                let a = sign_extend(self.read(a, size)?, size);
                let b = sign_extend(self.read(b, size)?, size);
                let full = i128::from(a) * i128::from(b);
                let result = full as u64 & mask(size);
                let overflow = i128::from(sign_extend(result, size)) != full;
                self.flags.cf = overflow;
                self.flags.of = overflow;
                self.set_result_flags(result, size);
                self.write(dst, size, result)?;
            }
            Op::Div | Op::Idiv => {
                arity(1)?;
                let size = Self::own_size(&ops[0])?;
                let divisor = self.read(&ops[0], size)?;
                if divisor == 0 {
                    return Err("division by zero".to_string());
                }
                let bits = u32::from(size) * 8;
                let rax = Reg { index: RAX, size };
                let rdx = Reg { index: RDX, size };
                let (lo, hi) = (self.get_reg(rax), self.get_reg(rdx));
                let (q, r) = if instr.op == Op::Div {
                    let dividend = (u128::from(hi) << bits) | u128::from(lo);
                    let q = dividend / u128::from(divisor);
                    if q > u128::from(mask(size)) {
                        return Err("division overflow".to_string());
                    }
                    let r = (dividend % u128::from(divisor)) as u64;
                    (q as u64, r)
                } else {
                    let dividend = (i128::from(sign_extend(hi, size)) << bits) | i128::from(lo);
                    let d = i128::from(sign_extend(divisor, size));
                    let q = dividend / d;
                    let max = i128::from(sign_extend(sign_bit(size) - 1, size));
                    if q > max || q < -max - 1 {
                        return Err("division overflow".to_string());
                    }
                    (q as u64, (dividend % d) as u64)
                };
                self.set_reg(rax, q);
                self.set_reg(rdx, r);
            }
            Op::Cqo => {
                arity(0)?;
                self.regs[RDX] = if sign_extend(self.regs[RAX], 8) < 0 {
                    u64::MAX
                } else {
                    0
                };
            }
            Op::Cdq => {
                arity(0)?;
                let edx = if sign_extend(self.regs[RAX], 4) < 0 {
                    0xFFFF_FFFF
                } else {
                    0
                };
                self.set_reg(
                    Reg {
                        index: RDX,
                        size: 4,
                    },
                    edx,
                );
            }
            Op::Shl | Op::Shr | Op::Sar => {
                arity(2)?;
                let size = Self::own_size(&ops[0])?;
                let count = (self.read(&ops[1], 1)? & if size == 8 { 63 } else { 31 }) as u32;
                let a = self.read(&ops[0], size)?;
                if count != 0 {
                    let result = match instr.op {
                        Op::Shl => (a << count) & mask(size),
                        Op::Shr => a >> count,
                        _ => (sign_extend(a, size) >> count).cast_unsigned() & mask(size),
                    };
                    self.flags.cf = match instr.op {
                        Op::Shl => {
                            count <= u32::from(size) * 8
                                && (a >> (u32::from(size) * 8 - count)) & 1 != 0
                        }
                        _ => (a >> (count - 1)) & 1 != 0,
                    };
                    self.set_result_flags(result, size);
                    self.write(&ops[0], size, result)?;
                }
            }
            Op::Push => {
                arity(1)?;
                let value = match &ops[0] {
                    Operand::Imm(n) => n.cast_unsigned(),
                    op => self.read(op, 8)?,
                };
                self.push(value)?;
            }
            Op::Pop => {
                arity(1)?;
                let value = self.pop()?;
                self.write(&ops[0], 8, value)?;
            }
            Op::Set(cond) => {
                arity(1)?;
                let value = u64::from(self.condition(cond));
                self.write(&ops[0], 1, value)?;
            }
            Op::Jmp => {
                arity(1)?;
                return Ok(Flow::Jump(self.read(&ops[0], 8)?));
            }
            Op::Jcc(cond) => {
                arity(1)?;
                if self.condition(cond) {
                    return Ok(Flow::Jump(self.read(&ops[0], 8)?));
                }
            }
            Op::Call => {
                arity(1)?;
                let target = self.read(&ops[0], 8)?;
                if (EXTERN_BASE..EXTERN_BASE + program.externs.len() as u64).contains(&target) {
                    let name = &program.externs[(target - EXTERN_BASE) as usize];
                    self.call_extern(name)?;
                    if self.exit_code.is_some() {
                        return Ok(Flow::Halt);
                    }
                } else {
                    return Ok(Flow::Call(target));
                }
            }
            Op::Ret => {
                arity(0)?;
                let target = self.pop()?;
                if target == HALT {
                    let code = self.regs[RAX] as i32;
                    self.exit_code = Some(code);
                    return Ok(Flow::Halt);
                }
                return Ok(Flow::Jump(target));
            }
        }
        Ok(Flow::Next)
    }
}
