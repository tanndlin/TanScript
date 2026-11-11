use crate::machine::{Machine, R8, R9, RAX, RCX, RDX, RSP, sign_extend};

impl Machine {
    pub fn arg(&self, i: usize) -> Result<u64, String> {
        match i {
            0 => Ok(self.regs[RCX]),
            1 => Ok(self.regs[RDX]),
            2 => Ok(self.regs[R8]),
            3 => Ok(self.regs[R9]),
            _ => self.mem.read(self.regs[RSP] + 8 * i as u64, 8),
        }
    }

    fn ret(&mut self, value: u64) {
        self.regs[RAX] = value;
    }

    pub fn call_extern(&mut self, name: &str) -> Result<(), String> {
        match name {
            "printf" => {
                let format = self.arg(0)?;
                let text = self.format(format, 1)?;
                self.stdout.extend_from_slice(&text);
                self.ret(text.len() as u64);
            }
            "puts" => {
                let s = self.mem.cstr(self.arg(0)?)?.to_vec();
                self.stdout.extend_from_slice(&s);
                self.stdout.push(b'\n');
                self.ret(0);
            }
            "putchar" => {
                let c = self.arg(0)? as u8;
                self.stdout.push(c);
                self.ret(u64::from(c));
            }
            "exit" => {
                let code = self.arg(0)? as i32;
                self.exit_code = Some(code);
            }
            "fflush" => self.ret(0),
            "malloc" => {
                let ptr = self.malloc(self.arg(0)?)?;
                self.ret(ptr);
            }
            "calloc" => {
                let size = self.arg(0)?.wrapping_mul(self.arg(1)?);
                let ptr = self.malloc(size)?;
                self.mem.slice_mut(ptr, size)?.fill(0);
                self.ret(ptr);
            }
            "realloc" => {
                let ptr = self.realloc(self.arg(0)?, self.arg(1)?)?;
                self.ret(ptr);
            }
            "free" => {
                let ptr = self.arg(0)?;
                self.free(ptr)?;
            }
            "strlen" => {
                let len = self.mem.cstr(self.arg(0)?)?.len();
                self.ret(len as u64);
            }
            "strcmp" => {
                let a = self.mem.cstr(self.arg(0)?)?;
                let b = self.mem.cstr(self.arg(1)?)?;
                let value: i64 = match a.cmp(b) {
                    std::cmp::Ordering::Less => -1,
                    std::cmp::Ordering::Equal => 0,
                    std::cmp::Ordering::Greater => 1,
                };
                self.ret(value.cast_unsigned());
            }
            "time" => {
                let out = self.arg(0)?;
                if out != 0 {
                    self.mem.write(out, 8, self.time)?;
                }
                self.ret(self.time);
            }
            "srand" => self.rand_state = self.arg(0)? as u32,
            "rand" => {
                // MSVC's LCG, matching the Windows CRT the compiler targets.
                self.rand_state = self
                    .rand_state
                    .wrapping_mul(214_013)
                    .wrapping_add(2_531_011);
                self.ret(u64::from((self.rand_state >> 16) & 0x7fff));
            }
            _ => {
                if !self.call_io(name)? {
                    return Err(format!("unsupported extern function `{name}`"));
                }
            }
        }
        Ok(())
    }

    pub fn malloc(&mut self, size: u64) -> Result<u64, String> {
        let size = (size.max(1) + 15) & !15;
        if let Some(ptr) = self.heap.free.get_mut(&size).and_then(Vec::pop) {
            self.heap.sizes.insert(ptr, size);
            return Ok(ptr);
        }
        let ptr = self.heap.next;
        let end = ptr.checked_add(size).ok_or("out of memory")?;
        self.mem.grow(end)?;
        self.heap.next = end;
        self.heap.sizes.insert(ptr, size);
        Ok(ptr)
    }

    pub fn realloc(&mut self, ptr: u64, size: u64) -> Result<u64, String> {
        if ptr == 0 {
            return self.malloc(size);
        }
        let old = *self
            .heap
            .sizes
            .get(&ptr)
            .ok_or_else(|| format!("realloc of invalid pointer {ptr:#x}"))?;
        if size <= old {
            return Ok(ptr);
        }
        let new = self.malloc(size)?;
        let bytes = self.mem.slice(ptr, old)?.to_vec();
        self.mem.slice_mut(new, old)?.copy_from_slice(&bytes);
        self.free(ptr)?;
        Ok(new)
    }

    pub fn free(&mut self, ptr: u64) -> Result<(), String> {
        if ptr == 0 {
            return Ok(());
        }
        let size = self
            .heap
            .sizes
            .remove(&ptr)
            .ok_or_else(|| format!("free of invalid pointer {ptr:#x}"))?;
        self.heap.free.entry(size).or_default().push(ptr);
        Ok(())
    }

    pub fn format(&self, format: u64, first_arg: usize) -> Result<Vec<u8>, String> {
        let fmt = self.mem.cstr(format)?;
        let mut out = Vec::new();
        let mut next = first_arg;
        let mut take = |m: &Machine| -> Result<u64, String> {
            let v = m.arg(next)?;
            next += 1;
            Ok(v)
        };
        let mut i = 0;
        while i < fmt.len() {
            let c = fmt[i];
            i += 1;
            if c != b'%' {
                out.push(c);
                continue;
            }
            let mut left = false;
            let mut zero = false;
            let mut plus = false;
            let mut space = false;
            let mut alt = false;
            while i < fmt.len() {
                match fmt[i] {
                    b'-' => left = true,
                    b'0' => zero = true,
                    b'+' => plus = true,
                    b' ' => space = true,
                    b'#' => alt = true,
                    _ => break,
                }
                i += 1;
            }
            let mut width = 0usize;
            if fmt.get(i) == Some(&b'*') {
                let w = sign_extend(take(self)?, 4);
                if w < 0 {
                    left = true;
                }
                width = usize::try_from(w.unsigned_abs()).unwrap_or(0);
                i += 1;
            } else {
                while let Some(d) = fmt.get(i).filter(|d| d.is_ascii_digit()) {
                    width = width * 10 + usize::from(d - b'0');
                    i += 1;
                }
            }
            let mut precision = None;
            if fmt.get(i) == Some(&b'.') {
                i += 1;
                if fmt.get(i) == Some(&b'*') {
                    let p = sign_extend(take(self)?, 4);
                    precision = usize::try_from(p).ok();
                    i += 1;
                } else {
                    let mut p = 0usize;
                    while let Some(d) = fmt.get(i).filter(|d| d.is_ascii_digit()) {
                        p = p * 10 + usize::from(d - b'0');
                        i += 1;
                    }
                    precision = Some(p);
                }
            }
            let mut size = 4u8;
            loop {
                let rest = &fmt[i..];
                if rest.starts_with(b"I64") {
                    size = 8;
                    i += 3;
                } else if rest.starts_with(b"I32") {
                    size = 4;
                    i += 3;
                } else if rest.starts_with(b"ll") {
                    size = 8;
                    i += 2;
                } else if rest.starts_with(b"hh") {
                    size = 1;
                    i += 2;
                } else if let Some(&m) = rest.first().filter(|m| b"hlzjtLI".contains(m)) {
                    size = match m {
                        b'h' => 2,
                        b'l' | b'L' => 4,
                        _ => 8,
                    };
                    i += 1;
                } else {
                    break;
                }
            }
            let Some(&conv) = fmt.get(i) else {
                out.push(b'%');
                break;
            };
            i += 1;

            let (prefix, body): (Vec<u8>, Vec<u8>) = match conv {
                b'%' => {
                    out.push(b'%');
                    continue;
                }
                b'd' | b'i' => {
                    let v = sign_extend(take(self)?, size);
                    let sign = if v < 0 {
                        b"-".to_vec()
                    } else if plus {
                        b"+".to_vec()
                    } else if space {
                        b" ".to_vec()
                    } else {
                        Vec::new()
                    };
                    (sign, digits(v.unsigned_abs(), 10, false, precision))
                }
                b'u' | b'x' | b'X' | b'o' => {
                    let raw = take(self)?;
                    let v = if size == 8 {
                        raw
                    } else {
                        raw & ((1u64 << (u32::from(size) * 8)) - 1)
                    };
                    let base = match conv {
                        b'u' => 10,
                        b'o' => 8,
                        _ => 16,
                    };
                    let prefix = match conv {
                        b'x' if alt && v != 0 => b"0x".to_vec(),
                        b'X' if alt && v != 0 => b"0X".to_vec(),
                        b'o' if alt => b"0".to_vec(),
                        _ => Vec::new(),
                    };
                    (prefix, digits(v, base, conv == b'X', precision))
                }
                b'p' => (Vec::new(), format!("{:016X}", take(self)?).into_bytes()),
                b'c' => {
                    let ch = take(self)? as u8;
                    (Vec::new(), vec![ch])
                }
                b's' => {
                    let ptr = take(self)?;
                    let s = if ptr == 0 {
                        b"(null)".to_vec()
                    } else {
                        self.mem.cstr(ptr)?.to_vec()
                    };
                    let s = match precision {
                        Some(p) if p < s.len() => s[..p].to_vec(),
                        _ => s,
                    };
                    (Vec::new(), s)
                }
                other => {
                    return Err(format!(
                        "unsupported printf conversion `%{}`",
                        other as char
                    ));
                }
            };

            let len = prefix.len() + body.len();
            let pad = width.saturating_sub(len);
            let numeric = !matches!(conv, b's' | b'c');
            if left {
                out.extend(prefix);
                out.extend(body);
                out.extend(std::iter::repeat_n(b' ', pad));
            } else if zero && numeric && precision.is_none() {
                out.extend(prefix);
                out.extend(std::iter::repeat_n(b'0', pad));
                out.extend(body);
            } else {
                out.extend(std::iter::repeat_n(b' ', pad));
                out.extend(prefix);
                out.extend(body);
            }
        }
        Ok(out)
    }
}

fn digits(mut v: u64, base: u64, upper: bool, precision: Option<usize>) -> Vec<u8> {
    let table: &[u8] = if upper {
        b"0123456789ABCDEF"
    } else {
        b"0123456789abcdef"
    };
    let mut out = Vec::new();
    while v > 0 {
        out.push(table[(v % base) as usize]);
        v /= base;
    }
    let min = precision.unwrap_or(1);
    while out.len() < min {
        out.push(b'0');
    }
    out.reverse();
    out
}
