use std::collections::{HashMap, HashSet};

use crate::compile::Address;

// pub struct FunctionDefinition {
//     pub num_args: Option<u8>,
// }

pub struct CompileScope {
    pub variables: HashMap<String, Address>,
    pub functions: HashSet<String>,
    pub unknown_functions: HashSet<String>,
    pub num_variables: i32,
}

impl CompileScope {
    pub fn new() -> CompileScope {
        CompileScope {
            variables: HashMap::new(),
            functions: HashSet::new(),
            unknown_functions: HashSet::new(),
            num_variables: 0,
        }
    }

    pub fn get_variable(&self, name: &str) -> Result<&Address, String> {
        match self.variables.get(name) {
            Some(addr) => Ok(addr),
            None => Err(format!("Variable '{name}' not found")),
        }
    }

    pub fn add_variable(&mut self, name: String) -> Result<(), String> {
        if self.variables.contains_key(&name) {
            return Err(format!("Variable '{name}' already declared"));
        }

        self.variables
            .insert(name, Address::Stack((self.num_variables + 1) * 8));
        self.num_variables += 1;
        Ok(())
    }

    pub fn register_function(&mut self, name: &str) {
        if self.functions.get(name).is_none() {
            self.unknown_functions.insert(name.to_string());
        }
    }
}
