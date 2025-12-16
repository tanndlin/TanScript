use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::{Rc, Weak},
};

use crate::{ast::Block, compile::Address};

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub args: Vec<String>,
    pub body: Block,
}

pub struct CompileScope {
    pub parent: Option<Weak<RefCell<CompileScope>>>,
    pub variables: HashMap<String, Address>,
    pub functions: HashMap<String, FunctionDefinition>,
    pub num_variables: i32,
}

impl CompileScope {
    pub fn new(parent: Option<&Rc<RefCell<CompileScope>>>) -> CompileScope {
        CompileScope {
            parent: parent.map(Rc::downgrade),
            variables: HashMap::new(),
            functions: HashMap::new(),
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

    pub fn add_function(&mut self, name: &String, def: FunctionDefinition) -> Result<(), String> {
        if self.functions.contains_key(name) {
            return Err(format!("Function {name} already exists"));
        }

        self.functions.insert(name.clone(), def);

        Ok(())
    }
}
