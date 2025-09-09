use std::collections::HashMap;

use crate::compile::Address;

pub struct FunctionDefinition {
    pub name: String,
    pub num_args: u8,
}

pub struct CompileScope {
    pub variables: HashMap<String, Address>,
    pub functions: HashMap<String, FunctionDefinition>,
    pub num_variables: i32,
}

impl CompileScope {
    pub fn new() -> CompileScope {
        CompileScope {
            variables: HashMap::new(),
            functions: HashMap::new(),
            num_variables: 0,
        }
    }

    pub fn get_variable(&self, name: &str) -> Address {
        match self.variables.get(name) {
            None => panic!("Variable {} not found", name),
            Some(addr) => (*addr).clone(),
        }
    }

    pub fn add_variable(&mut self, name: String) {
        self.variables
            .insert(name, Address::Stack((self.num_variables + 1) * 8));
        self.num_variables += 1
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionDefinition> {
        self.functions.get(name)
    }
}
