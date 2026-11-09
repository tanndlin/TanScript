#![allow(
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::too_many_lines,
    clippy::struct_excessive_bools,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

pub mod io;
pub mod libc;
pub mod machine;
pub mod parse;

use std::{collections::HashMap, fmt};

use machine::{DATA_BASE, Flow, Machine};
use parse::{CODE_BASE, parse};

pub struct Options {
    pub entry: String,
    pub max_steps: u64,
    pub memory_limit: usize,
    pub files: HashMap<String, Vec<u8>>,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            entry: "main".to_string(),
            max_steps: 1_000_000_000,
            memory_limit: 256 * 1024 * 1024,
            files: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub line: Option<usize>,
    pub message: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

pub struct Run {
    pub stdout: Vec<u8>,
    pub steps: u64,
    pub result: Result<i32, Error>,
}

impl Run {
    pub fn stdout_lossy(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }
}

fn failed(line: Option<usize>, message: String) -> Run {
    Run {
        stdout: Vec::new(),
        steps: 0,
        result: Err(Error { line, message }),
    }
}

pub fn run(source: &str, options: Options) -> Run {
    let program = match parse(source, DATA_BASE) {
        Ok(p) => p,
        Err(e) => return failed(Some(e.line), e.message),
    };
    let mut machine = match Machine::new(&program, options.memory_limit) {
        Ok(m) => m,
        Err(e) => return failed(None, e),
    };
    machine.files = options.files;

    let Some(&entry) = program.symbols.get(&options.entry) else {
        return failed(None, format!("entry point `{}` not found", options.entry));
    };
    let code_end = CODE_BASE + program.instrs.len() as u64;
    let to_pc = |target: u64| -> Result<usize, String> {
        if (CODE_BASE..code_end).contains(&target) {
            usize::try_from(target - CODE_BASE).map_err(|e| e.to_string())
        } else if target == code_end {
            Err("execution ran past the end of the program".to_string())
        } else {
            Err(format!("jump to invalid address {target:#x}"))
        }
    };

    let mut steps = 0u64;
    let mut line = None;
    let result = (|| -> Result<i32, String> {
        let mut pc = to_pc(entry)?;
        loop {
            let instr = program
                .instrs
                .get(pc)
                .ok_or("execution ran past the end of the program")?;
            line = Some(instr.line);
            if steps >= options.max_steps {
                return Err(format!("step limit of {} exceeded", options.max_steps));
            }
            steps += 1;
            pc += 1;
            match machine.exec(instr, &program)? {
                Flow::Next => {}
                Flow::Jump(target) => pc = to_pc(target)?,
                Flow::Call(target) => {
                    let dest = to_pc(target)?;
                    machine.push(CODE_BASE + pc as u64)?;
                    pc = dest;
                }
                Flow::Halt => return Ok(machine.exit_code.unwrap_or(0)),
            }
        }
    })();

    Run {
        stdout: machine.stdout,
        steps,
        result: result.map_err(|message| Error { line, message }),
    }
}
