use std::collections::HashMap;

use crate::compile::Address;

#[allow(dead_code)]
pub struct CompileScope<'a> {
    pub parent: Option<&'a CompileScope<'a>>,
    pub variables: HashMap<String, Address>,
    pub num_variables: i32,
}

impl<'a> CompileScope<'a> {
    pub fn new(parent: Option<&'a CompileScope<'a>>) -> Self {
        Self {
            parent,
            variables: HashMap::new(),
            num_variables: 0,
        }
    }

    pub fn get_variable(&self, name: &str) -> Result<Address, String> {
        if let Some(addr) = self.variables.get(name) {
            return Ok(*addr);
        }

        let parent = self
            .parent
            .as_ref()
            .ok_or_else(|| format!("Variable '{name}' not found"))?;

        parent.get_variable(name)
    }

    pub fn add_variable(&mut self, name: String, address: Option<Address>) -> Result<(), String> {
        if self.variables.contains_key(&name) {
            return Err(format!("Variable '{name}' already declared"));
        }

        if let Some(address) = address {
            self.variables.insert(name, address);
        } else {
            self.variables
                .insert(name, Address::Stack((self.num_variables + 1) * 8));
        }

        self.num_variables += 1;
        Ok(())
    }
}
