use std::collections::HashMap;

use crate::compile::Address;

pub struct FunctionDefinition {
    pub num_args: Option<u8>,
}

pub struct CompileScope {
    pub variables: HashMap<String, Address>,
    pub functions: HashMap<String, FunctionDefinition>,
    pub num_variables: i32,
}

impl CompileScope {
    pub fn new() -> CompileScope {
        let mut functions = HashMap::new();
        functions.insert("printf".to_string(), FunctionDefinition { num_args: None });
        functions.insert(
            "malloc".to_string(),
            FunctionDefinition { num_args: Some(1) },
        );
        functions.insert(
            "fopen".to_string(),
            FunctionDefinition { num_args: Some(2) },
        );
        functions.insert(
            "fclose".to_string(),
            FunctionDefinition { num_args: Some(1) },
        );
        functions.insert(
            "fread".to_string(),
            FunctionDefinition { num_args: Some(4) },
        );

        CompileScope {
            variables: HashMap::new(),
            functions,
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

    pub fn get_function(&self, name: &str) -> Result<&FunctionDefinition, String> {
        match self.functions.get(name) {
            Some(func) => Ok(func),
            None => Err(format!("Function '{name}' not found")),
        }
    }
}
