use std::collections::HashMap;

use crate::codegen::address::{Address, Register};

#[derive(Clone, Copy, Debug)]
struct RegisterState {
    used: bool,
    last_used: u64,
}

impl RegisterState {
    fn new() -> Self {
        RegisterState {
            used: false,
            last_used: 0,
        }
    }
}

pub struct RegisterHandler {
    pub data: Vec<String>,
    registers: HashMap<Register, RegisterState>,
    reserved_registers: Vec<Register>,
    unique_id: u32,
    usage_counter: u64,
}

impl RegisterHandler {
    pub fn new() -> RegisterHandler {
        RegisterHandler {
            data: vec![],
            registers: HashMap::from([
                (Register::RAX, RegisterState::new()),
                (Register::RBX, RegisterState::new()),
                (Register::RCX, RegisterState::new()),
                (Register::RDX, RegisterState::new()),
                (Register::R8, RegisterState::new()),
                (Register::R9, RegisterState::new()),
                (Register::R10, RegisterState::new()),
                (Register::R11, RegisterState::new()),
                (Register::R12, RegisterState::new()),
                (Register::R13, RegisterState::new()),
                (Register::R14, RegisterState::new()),
                (Register::R15, RegisterState::new()),
            ]),
            reserved_registers: vec![Register::RAX, Register::RCX, Register::RDX],
            unique_id: 0,
            usage_counter: 0,
        }
    }

    pub fn lease_register(&mut self) -> Result<Register, String> {
        // Choose the least-recently-used free register (LRU heuristic) that is not reserved.
        let candidate = self
            .registers
            .iter()
            .filter(|(r, st)| !st.used && !self.reserved_registers.contains(r))
            .min_by_key(|(_, st)| st.last_used)
            .map(|(r, _)| *r);

        let reg = candidate.ok_or_else(|| "No registers available".to_string())?;
        let state = self.registers.get_mut(&reg).unwrap();
        state.used = true;
        self.usage_counter = self.usage_counter.wrapping_add(1);
        state.last_used = self.usage_counter;
        Ok(reg)
    }

    pub fn release_register(&mut self, register: Register) {
        match self.registers.get_mut(&register) {
            Some(state) => {
                state.used = false;
                // mark as recently used so LRU prefers other registers next
                self.usage_counter = self.usage_counter.wrapping_add(1);
                state.last_used = self.usage_counter;
            }
            None => panic!("Released unleasable register? {register}"),
        }
    }

    pub fn lease_with_scope(
        &mut self,
        function: impl FnOnce(&mut Self, Register) -> Result<String, String>,
    ) -> Result<String, String> {
        if let Ok(reg) = self.lease_register() {
            let ret = function(self, reg);
            self.release_register(reg);
            return ret;
        }

        // Everything is leased, so borrow the first reserved register and save its value around
        // the use. Holding it means nested code can't `request_register` it out from under us
        let reg = self.reserved_registers[0];
        let was_free = self.request_register(reg).is_ok();
        let ret = function(self, reg);
        if was_free {
            self.release_register(reg);
        }
        Ok(format!("push {reg}\n{}\npop {reg}", ret?))
    }

    /// Like `lease_with_scope`, but for a specific register. If it's already in use, its value is
    /// saved around `function`, unless `dst` is that register: the caller wants it overwritten
    pub fn request_with_scope(
        &mut self,
        reg: Register,
        dst: Address,
        function: impl FnOnce(&mut Self, Register) -> Result<String, String>,
    ) -> Result<String, String> {
        if self.request_register(reg).is_ok() {
            let ret = function(self, reg);
            self.release_register(reg);
            return ret;
        }

        let ret = function(self, reg)?;
        if dst == Address::Register(reg) {
            Ok(ret)
        } else {
            Ok(format!("push {reg}\n{ret}\npop {reg}"))
        }
    }

    pub fn add_data(&mut self, s: &str) -> String {
        let name = format!("data_{}", self.data.len());
        let format = format_data(&name, s);
        self.data.push(format);

        name
    }

    pub fn request_register(&mut self, register: Register) -> Result<Register, String> {
        match self.registers.get_mut(&register) {
            None => Err(format!("Register {register} not leasable")),
            Some(state) if state.used => Err(format!("Register {register} not available")),
            Some(state) => {
                state.used = true;
                self.usage_counter = self.usage_counter.wrapping_add(1);
                state.last_used = self.usage_counter;
                Ok(register)
            }
        }
    }

    pub fn is_used(&self, register: Register) -> bool {
        self.registers
            .get(&register)
            .is_some_and(|state| state.used)
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

#[cfg(test)]
mod tests {
    use crate::codegen::{address::Register, register_handler::RegisterHandler};

    #[test]
    fn prevent_double_lease() {
        let mut rh = RegisterHandler::new();
        let reg = rh.request_register(Register::RAX);
        assert!(reg.is_ok());

        let reg2 = rh.request_register(Register::RAX);
        assert!(reg2.is_err());
    }

    #[test]
    fn allow_release_then_lease() {
        let mut rh = RegisterHandler::new();

        let rax = rh.request_register(Register::RAX);
        assert!(rax.is_ok());
        let rax = rax.unwrap();

        rh.release_register(rax);
        assert!(rh.request_register(rax).is_ok());
    }

    fn exhaust(rh: &mut RegisterHandler) -> Vec<Register> {
        std::iter::from_fn(|| rh.lease_register().ok()).collect()
    }

    #[test]
    fn lease_with_scope_releases_on_error() {
        let mut rh = RegisterHandler::new();
        let before = exhaust(&mut rh).len();
        let mut rh = RegisterHandler::new();

        let _ = rh.lease_with_scope(|_, _| Err("fail".to_string()));
        assert_eq!(exhaust(&mut rh).len(), before);
    }

    #[test]
    fn lease_with_scope_fallback_blocks_nested_request() {
        let mut rh = RegisterHandler::new();
        exhaust(&mut rh);

        let asm = rh
            .lease_with_scope(|rh, reg| {
                assert_eq!(reg, Register::RAX);
                assert!(rh.request_register(Register::RAX).is_err());
                Ok(format!("mov {reg}, 1"))
            })
            .unwrap();
        assert_eq!(asm, "push rax\nmov rax, 1\npop rax");
        assert!(rh.request_register(Register::RAX).is_ok());
    }
}
