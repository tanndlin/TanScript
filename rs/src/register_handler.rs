use std::collections::HashMap;

use crate::compile::Register;

pub struct RegisterHandler {
    pub data: Vec<String>,
    registers: HashMap<Register, bool>,
    reserved_registers: Vec<Register>,
    unique_id: u32,
}

impl RegisterHandler {
    pub fn new() -> RegisterHandler {
        RegisterHandler {
            data: vec![],
            registers: HashMap::from([
                (Register::RAX, false),
                (Register::RBX, false),
                (Register::RCX, false),
                (Register::RDX, false),
                (Register::R8, false),
                (Register::R9, false),
                (Register::R10, false),
                (Register::R11, false),
                (Register::R12, false),
                (Register::R13, false),
                (Register::R14, false),
                (Register::R15, false),
            ]),
            reserved_registers: vec![Register::RAX, Register::RDX],
            unique_id: 0,
        }
    }

    pub fn lease_register(&mut self) -> Result<Register, String> {
        let reg = self
            .registers
            .iter()
            .filter(|(reg, _)| !self.reserved_registers.contains(reg))
            .skip_while(|(_, used)| **used)
            .map(|(reg, _)| Ok(reg.clone()))
            .nth(0)
            .unwrap_or_else(|| Err("No registers available".to_string()))?;

        self.registers.insert(reg.clone(), true);
        Ok(reg)
    }

    pub fn release_register(&mut self, register: Register) {
        match self.registers.get(&register) {
            Some(_) => self.registers.insert(register, false),
            None => panic!("Released unleasable register? {register}"),
        };
    }

    pub fn add_data(&mut self, s: &str) -> String {
        let name = format!("data_{}", self.data.len());
        let format = format_data(&name, s);
        self.data.push(format);

        name
    }

    pub fn request_register(&mut self, register: &Register) -> Result<Register, String> {
        match self.registers.get(register) {
            None => Err(format!("Register {register} not leasable")),
            Some(used) => match used {
                true => Err(format!("Register {register} not available")),
                false => {
                    self.registers.insert(register.clone(), true);
                    Ok(register.clone())
                }
            },
        }
    }

    pub fn get_unique_id(&mut self) -> u32 {
        let id = self.unique_id;
        self.unique_id += 1;
        id
    }
}

fn format_data(name: &String, s: &str) -> String {
    let mut pieces: Vec<&str> = s.split("\\n").collect();
    if s.ends_with("\\n") {
        pieces.pop();
    }

    let mut newlines = pieces.join("\", 10, \"");
    if s.ends_with("\\n") {
        newlines.push_str("\", 10");
    } else {
        newlines.push('"');
    }

    format!("{name} db \"{newlines}, 0")
}
