use crate::machine::{File, Machine, RAX, sign_extend};

const EOF: u64 = u64::MAX;

impl Machine {
    pub fn call_io(&mut self, name: &str) -> Result<bool, String> {
        let value = match name {
            "fopen" => {
                let path = String::from_utf8_lossy(self.mem.cstr(self.arg(0)?)?).into_owned();
                let mode = self.mem.cstr(self.arg(1)?)?.to_vec();
                let data = if mode.first() == Some(&b'r') {
                    self.files.get(&path).cloned()
                } else {
                    Some(Vec::new())
                };
                match data {
                    Some(data) => {
                        let handle = self.next_handle;
                        self.next_handle += 16;
                        self.open_files.insert(handle, File { data, pos: 0 });
                        handle
                    }
                    None => 0,
                }
            }
            "fclose" => {
                let handle = self.arg(0)?;
                self.open_files
                    .remove(&handle)
                    .ok_or_else(|| format!("fclose of invalid file {handle:#x}"))?;
                0
            }
            "fseek" => {
                let offset = sign_extend(self.arg(1)?, 4);
                let whence = self.arg(2)? & 0xFFFF_FFFF;
                let file = self.file(self.arg(0)?)?;
                let base = match whence {
                    0 => 0,
                    1 => file.pos as i64,
                    2 => file.data.len() as i64,
                    _ => return Err(format!("fseek with invalid origin {whence}")),
                };
                match usize::try_from(base + offset) {
                    Ok(pos) => {
                        file.pos = pos;
                        0
                    }
                    Err(_) => u64::MAX,
                }
            }
            "ftell" => self.file(self.arg(0)?)?.pos as u64,
            "fgetc" | "getc" => {
                let file = self.file(self.arg(0)?)?;
                match file.data.get(file.pos) {
                    Some(&b) => {
                        file.pos += 1;
                        u64::from(b)
                    }
                    None => EOF,
                }
            }
            "fread" => {
                let buf = self.arg(0)?;
                let size = self.arg(1)?;
                let count = self.arg(2)?;
                let file = self.file(self.arg(3)?)?;
                let remaining = (file.data.len().saturating_sub(file.pos)) as u64;
                let items = remaining.checked_div(size).map_or(0, |n| count.min(n));
                let len = items * size;
                let start = file.pos;
                let bytes = file.data[start..start + len as usize].to_vec();
                file.pos += len as usize;
                self.mem.slice_mut(buf, len)?.copy_from_slice(&bytes);
                items
            }
            "fscanf" => {
                let handle = self.arg(0)?;
                let format = self.mem.cstr(self.arg(1)?)?.to_vec();
                let file = self.file(handle)?;
                let (data, mut pos) = (std::mem::take(&mut file.data), file.pos);
                let result = self.scan(&data, &mut pos, &format, 2);
                let file = self.file(handle)?;
                file.data = data;
                file.pos = pos;
                result?
            }
            "sscanf" => {
                let input = self.mem.cstr(self.arg(0)?)?.to_vec();
                let format = self.mem.cstr(self.arg(1)?)?.to_vec();
                self.scan(&input, &mut 0, &format, 2)?
            }
            _ => return Ok(false),
        };
        self.regs[RAX] = value;
        Ok(true)
    }

    fn file(&mut self, handle: u64) -> Result<&mut File, String> {
        self.open_files
            .get_mut(&handle)
            .ok_or_else(|| format!("invalid FILE pointer {handle:#x}"))
    }

    fn scan(
        &mut self,
        input: &[u8],
        pos: &mut usize,
        format: &[u8],
        first_arg: usize,
    ) -> Result<u64, String> {
        let mut assigned = 0u64;
        let mut next = first_arg;
        let mut i = 0;
        let fail = |assigned: u64, pos: usize| {
            if assigned == 0 && pos >= input.len() {
                EOF
            } else {
                assigned
            }
        };

        while i < format.len() {
            let c = format[i];
            i += 1;
            if c.is_ascii_whitespace() {
                while input.get(*pos).is_some_and(u8::is_ascii_whitespace) {
                    *pos += 1;
                }
                continue;
            }
            if c != b'%' || format.get(i) == Some(&b'%') {
                if c == b'%' {
                    i += 1;
                }
                if input.get(*pos) == Some(&c) {
                    *pos += 1;
                    continue;
                }
                return Ok(fail(assigned, *pos));
            }

            let suppress = format.get(i) == Some(&b'*');
            if suppress {
                i += 1;
            }
            let mut width = 0usize;
            while let Some(d) = format.get(i).filter(|d| d.is_ascii_digit()) {
                width = width * 10 + usize::from(d - b'0');
                i += 1;
            }
            let width = if width == 0 { usize::MAX } else { width };
            let rest = &format[i..];
            let (size, skip) = if rest.starts_with(b"ll") || rest.starts_with(b"I64") {
                (8u8, if rest[0] == b'I' { 3 } else { 2 })
            } else if rest.starts_with(b"hh") {
                (1, 2)
            } else {
                match rest.first() {
                    Some(b'h') => (2, 1),
                    Some(b'l') => (4, 1),
                    Some(b'z' | b'j' | b't') => (8, 1),
                    _ => (4, 0),
                }
            };
            i += skip;
            let Some(&conv) = format.get(i) else {
                return Err("truncated scanf format".to_string());
            };
            i += 1;

            match conv {
                b'd' | b'i' | b'u' | b'x' => {
                    while input.get(*pos).is_some_and(u8::is_ascii_whitespace) {
                        *pos += 1;
                    }
                    let start = *pos;
                    let mut end = start;
                    if matches!(input.get(end), Some(b'-' | b'+')) {
                        end += 1;
                    }
                    let radix = if conv == b'x' { 16 } else { 10 };
                    while end - start < width
                        && input.get(end).is_some_and(|b| (*b as char).is_digit(radix))
                    {
                        end += 1;
                    }
                    let text = std::str::from_utf8(&input[start..end]).unwrap_or_default();
                    let Ok(value) = i64::from_str_radix(text, radix) else {
                        return Ok(fail(assigned, *pos));
                    };
                    *pos = end;
                    if !suppress {
                        let addr = self.arg(next)?;
                        next += 1;
                        self.mem.write(addr, size, value.cast_unsigned())?;
                        assigned += 1;
                    }
                }
                b's' | b'c' | b'[' => {
                    let accept: Box<dyn Fn(u8) -> bool> = match conv {
                        b's' => {
                            while input.get(*pos).is_some_and(u8::is_ascii_whitespace) {
                                *pos += 1;
                            }
                            Box::new(|b: u8| !b.is_ascii_whitespace())
                        }
                        b'c' => Box::new(|_| true),
                        _ => {
                            let negate = format.get(i) == Some(&b'^');
                            if negate {
                                i += 1;
                            }
                            let start = i;
                            if format.get(i) == Some(&b']') {
                                i += 1;
                            }
                            while format.get(i).is_some_and(|&b| b != b']') {
                                i += 1;
                            }
                            let set = format[start..i].to_vec();
                            i += 1;
                            Box::new(move |b: u8| set.contains(&b) != negate)
                        }
                    };
                    let width = if conv == b'c' && width == usize::MAX {
                        1
                    } else {
                        width
                    };
                    let start = *pos;
                    while *pos - start < width && input.get(*pos).is_some_and(|&b| accept(b)) {
                        *pos += 1;
                    }
                    if *pos == start || (conv == b'c' && *pos - start < width) {
                        return Ok(fail(assigned, *pos));
                    }
                    if !suppress {
                        let addr = self.arg(next)?;
                        next += 1;
                        let bytes = input[start..*pos].to_vec();
                        let len = bytes.len() as u64;
                        let terminate = u64::from(conv != b'c');
                        let dst = self.mem.slice_mut(addr, len + terminate)?;
                        dst[..bytes.len()].copy_from_slice(&bytes);
                        if terminate == 1 {
                            dst[bytes.len()] = 0;
                        }
                        assigned += 1;
                    }
                }
                other => return Err(format!("unsupported scanf conversion `%{}`", other as char)),
            }
        }
        Ok(assigned)
    }
}
