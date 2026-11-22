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

use crate::codegen::{compile_scope::CompileScope, register_handler::RegisterHandler};

pub struct Codegen {
    scopes: Vec<CompileScope>,
    pub registers: RegisterHandler,
}

impl Default for Codegen {
    fn default() -> Self {
        Self {
            scopes: vec![CompileScope::default()],
            registers: RegisterHandler::default(),
        }
    }
}
