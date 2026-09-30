use std::fmt;

use crate::{
    ast::{Block, Statement},
    codegen::{Address, CompileScope, Register, RegisterHandler, SymbolTable},
};

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub args: Vec<String>,
    pub body: Block,
}

impl FunctionDefinition {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let mut scope = CompileScope::new(Some(compile_scope));

        let mut arg_setup = vec![];
        let arg_registers = [Register::RCX, Register::RDX, Register::R8, Register::R9];
        for (i, arg) in self.args.iter().enumerate() {
            // Win64: after `push rbp`, the shadow space for the first 4 args starts at rbp+16,
            // and any further args were placed directly after it by the caller
            let offset = 16 + i32::try_from(i).map_err(|_| "Index out of range")? * 8;
            let new_address = Address::Stack(-offset);

            // Move register args to their home in the shadow space
            if let Some(reg) = arg_registers.get(i) {
                arg_setup.push(format!("mov {new_address}, {reg}"));
            }

            // Update its location in the scope
            scope.add_variable(arg.clone(), Some(new_address))?;
            // This is not a variable that needs to be deallocated later
            scope.num_variables -= 1;
        }
        let arg_setup = arg_setup.join("\n\t");
        let instructions = self.body.compile(&mut scope, register_handler)?;
        let ret = if instructions.ends_with("ret") {
            ""
        } else {
            "pop rbp\n\tret"
        };

        let preamble = format!("{}:\n\tpush rbp\n\tmov rbp, rsp", self.name);
        Ok(format!(
            "{preamble}\n\t{arg_setup}\n{instructions}\n\t{ret}",
        ))
    }
}

impl Statement for FunctionDefinition {
    fn discover(&self, symbols: &mut SymbolTable) {
        symbols.functions.insert(self.name.clone(), self.clone());
        self.body.discover(symbols);
    }

    fn compile(
        &self,
        _compile_scope: &mut CompileScope,
        _register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        Ok(String::new())
    }

    fn requires_semicolon(&self) -> bool {
        false
    }
}

impl fmt::Display for FunctionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "def {}({}) {{{}}}",
            self.name,
            self.args.join(", "),
            self.body
        )
    }
}
