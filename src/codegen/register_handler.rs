use std::collections::BTreeMap;

use crate::codegen::{
    CodegenErrorKind,
    address::{Address, Register},
    compile_call::NON_VOLATILE_REGISTERS,
    instruction::Instruction,
};

#[derive(Clone, Copy, Debug, Default)]
enum Liveness {
    #[default]
    Cold,
    Hot,
    Destination,
}

#[derive(Clone, Copy, Debug, Default)]
struct RegisterState {
    liveness: Liveness,
    last_used: u64,
}

pub struct RegisterHandler {
    pub data: Vec<String>,
    registers: BTreeMap<Register, RegisterState>,
    reserved_registers: Vec<Register>,
    pub touched: Vec<Register>,
    unique_id: u32,
    usage_counter: u64,
}

impl RegisterHandler {
    pub fn new() -> RegisterHandler {
        RegisterHandler {
            data: vec![],
            registers: [
                Register::RAX,
                Register::RBX,
                Register::RCX,
                Register::RDX,
                Register::R8,
                Register::R9,
                Register::R10,
                Register::R11,
                Register::R12,
                Register::R13,
                Register::R14,
                Register::R15,
            ]
            .into_iter()
            .map(|r| (r, RegisterState::default()))
            .collect(),
            reserved_registers: vec![Register::RAX, Register::RCX, Register::RDX],
            touched: vec![],
            unique_id: 0,
            usage_counter: 0,
        }
    }

    pub fn lease_register(&mut self) -> Result<Register, CodegenErrorKind> {
        let candidate = self
            .registers
            .iter()
            .filter(|(r, st)| {
                !matches!(st.liveness, Liveness::Hot) && !self.reserved_registers.contains(r)
            })
            .min_by_key(|(r, st)| (NON_VOLATILE_REGISTERS.contains(r), st.last_used))
            .map(|(r, _)| *r);

        let reg = candidate.ok_or(CodegenErrorKind::OutOfRegisters)?;
        let state = self.registers.get_mut(&reg).unwrap();
        state.liveness = Liveness::Hot;
        self.usage_counter = self.usage_counter.wrapping_add(1);
        state.last_used = self.usage_counter;
        self.touch(reg);
        Ok(reg)
    }

    fn touch(&mut self, register: Register) {
        if NON_VOLATILE_REGISTERS.contains(&register) && !self.touched.contains(&register) {
            self.touched.push(register);
        }
    }

    pub fn release_register(&mut self, register: Register) {
        if let Some(state) = self.registers.get_mut(&register) {
            state.liveness = Liveness::Cold;
            // mark as recently used so LRU prefers other registers next
            self.usage_counter = self.usage_counter.wrapping_add(1);
            state.last_used = self.usage_counter;
        }
    }

    pub fn lease_with_scope<E>(
        &mut self,
        function: impl FnOnce(&mut Self, Register) -> Result<Vec<Instruction>, E>,
    ) -> Result<Vec<Instruction>, E> {
        if let Ok(reg) = self.lease_register() {
            let ret = function(self, reg);
            self.release_register(reg);
            return ret;
        }

        // Everything is leased, so borrow the first reserved register and save its value around
        // the use. Holding it means nested code can't `request_register` it out from under us
        let reg = self.reserved_registers[0];
        let was_free = self.request_register(reg).is_ok();
        let ret = function(self, reg)?;
        if was_free {
            self.release_register(reg);
        }

        let mut out = vec![Instruction::push(reg)];
        out.extend(ret);
        out.push(Instruction::pop(reg));
        Ok(out)
    }

    /// Like `lease_with_scope`, but for a specific register. If it's already in use, its value is
    /// saved around `function`, unless `dst` is that register: the caller wants it overwritten
    pub fn request_with_scope<E>(
        &mut self,
        reg: Register,
        dst: Address,
        function: impl FnOnce(&mut Self, Register) -> Result<Vec<Instruction>, E>,
    ) -> Result<Vec<Instruction>, E> {
        if self.request_register(reg).is_ok() {
            let ret = function(self, reg);
            self.release_register(reg);
            return ret;
        }

        let ret = function(self, reg)?;
        if dst == Address::Register(reg) {
            Ok(ret)
        } else {
            let mut out = vec![Instruction::push(reg)];
            out.extend(ret);
            out.push(Instruction::pop(reg));
            Ok(out)
        }
    }

    pub fn add_data(&mut self, s: &str) -> String {
        let index = if let Some(index) = self.data.iter().position(|d| d == s) {
            index
        } else {
            self.data.push(s.to_string());
            self.data.len() - 1
        };

        data_name(index)
    }

    pub fn request_register(&mut self, register: Register) -> Result<Register, String> {
        match self.registers.get_mut(&register) {
            None => Err(format!("Register {register} not leasable")),
            Some(state) if matches!(state.liveness, Liveness::Hot) => {
                Err(format!("Register {register} not available"))
            }
            Some(state) => {
                state.liveness = Liveness::Hot;
                self.usage_counter = self.usage_counter.wrapping_add(1);
                state.last_used = self.usage_counter;
                self.touch(register);
                Ok(register)
            }
        }
    }

    pub fn mark_destination(&mut self, register: Register) -> bool {
        match self.registers.get_mut(&register) {
            Some(state) if matches!(state.liveness, Liveness::Hot) => {
                state.liveness = Liveness::Destination;
                true
            }
            _ => false,
        }
    }

    pub fn mark_hot(&mut self, register: Register) {
        if let Some(state) = self.registers.get_mut(&register) {
            state.liveness = Liveness::Hot;
            self.usage_counter = self.usage_counter.wrapping_add(1);
            state.last_used = self.usage_counter;
        }
    }

    pub fn is_hot(&self, register: Register) -> bool {
        self.registers
            .get(&register)
            .is_some_and(|state| matches!(state.liveness, Liveness::Hot))
    }

    pub fn get_unique_id(&mut self) -> u32 {
        let id = self.unique_id;
        self.unique_id += 1;
        id
    }
}

pub fn data_name(index: usize) -> String {
    format!("data_{index}")
}

#[cfg(test)]
mod tests {
    use crate::codegen::{
        address::Register, instruction::Instruction, register_handler::RegisterHandler,
    };

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
                Ok::<_, String>(vec![Instruction::mov(reg, 1)])
            })
            .unwrap();
        assert_eq!(
            asm,
            [
                Instruction::push(Register::RAX),
                Instruction::mov(Register::RAX, 1),
                Instruction::pop(Register::RAX),
            ]
        );
        assert!(rh.request_register(Register::RAX).is_ok());
    }
}
