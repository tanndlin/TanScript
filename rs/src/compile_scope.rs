use std::{
    cell::RefCell,
    collections::HashMap,
    rc::{Rc, Weak},
};

use crate::compile::Address;

#[allow(dead_code)]
pub struct CompileScope {
    pub parent: Option<Weak<RefCell<CompileScope>>>,
    pub variables: HashMap<String, Address>,
    pub num_variables: i32,
}

impl CompileScope {
    pub fn new(parent: Option<&Rc<RefCell<CompileScope>>>) -> CompileScope {
        CompileScope {
            parent: parent.map(Rc::downgrade),
            variables: HashMap::new(),
            num_variables: 0,
        }
    }

    pub fn get_variable(&self, name: &str) -> Result<Address, String> {
        if let Some(addr) = self.variables.get(name) {
            return Ok(addr.clone());
        }

        let parent = self
            .parent
            .as_ref()
            .and_then(Weak::upgrade)
            .ok_or_else(|| format!("Variable '{name}' not found"))?;

        parent.borrow().get_variable(name)
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

    pub fn add_variable_at_address(
        &mut self,
        name: String,
        address: Address,
    ) -> Result<(), String> {
        if self.variables.contains_key(&name) {
            return Err(format!("Variable '{name}' already declared"));
        }

        self.variables.insert(name, address);
        self.num_variables += 1;
        Ok(())
    }
}
