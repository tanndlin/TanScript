mod address;
mod codegen_error;
mod compile;
mod compile_call;
mod compile_memory;
mod compile_operators;
mod compile_scope;
mod compile_statement;
mod function_context;
mod instruction;
mod loop_context;
mod register_handler;
mod symbol_table;

pub use codegen_error::{CodegenError, CodegenErrorKind};

use std::collections::HashSet;

use crate::codegen::{compile_scope::CompileScope, register_handler::RegisterHandler};

pub struct Codegen {
    scopes: Vec<CompileScope>,
    pub registers: RegisterHandler,
    pub defined_functions: HashSet<String>,
    pub data: Vec<String>,
    unique_id: u32,
}

impl Default for Codegen {
    fn default() -> Self {
        Self {
            scopes: vec![CompileScope::default()],
            registers: RegisterHandler::default(),
            defined_functions: HashSet::new(),
            data: vec![],
            unique_id: 0,
        }
    }
}

impl Codegen {
    pub fn add_data(&mut self, s: &str) -> String {
        let index = if let Some(index) = self.data.iter().position(|d| d == s) {
            index
        } else {
            self.data.push(s.to_string());
            self.data.len() - 1
        };

        data_name(index)
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
