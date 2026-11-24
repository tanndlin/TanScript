use std::collections::HashMap;

use crate::{
    codegen::{
        Codegen, CodegenErrorKind,
        address::{Address, Register},
        function_context::FunctionContext,
        instruction::Instruction,
        loop_context::LoopContext,
    },
    type_check::Type,
};

pub const SLOT_SIZE: u32 = 8;

#[derive(Debug, Default)]
pub struct CompileScope {
    variables: HashMap<String, Address>,
    loop_context: Option<LoopContext>,
    function_context: Option<FunctionContext>,
    /// Bytes `rsp` sits below `rbp`. Each local reserves its own 8 byte slot as it is declared, so
    /// this is also the offset of the most recently declared local
    pub stack_depth: u32,
}

impl CompileScope {
    pub fn loop_context(&self) -> Option<&LoopContext> {
        self.loop_context.as_ref()
    }

    pub fn add_variable(&mut self, name: String, address: Address) -> Result<(), CodegenErrorKind> {
        if self.variables.contains_key(&name) {
            return Err(CodegenErrorKind::AlreadyDeclared(name));
        }

        self.variables.insert(name, address);
        Ok(())
    }

    pub fn allocate_variable(
        &mut self,
        name: String,
        t: &Type,
    ) -> Result<Instruction, CodegenErrorKind> {
        let size = u32::try_from(t.size()).unwrap().next_multiple_of(SLOT_SIZE);
        let address = Address::Stack(i32::try_from(self.stack_depth + size).unwrap());
        self.add_variable(name, address)?;
        self.stack_depth += size;
        Ok(Instruction::sub(Register::RSP, size))
    }
}

impl Codegen {
    pub fn scope(&self) -> &CompileScope {
        self.scopes
            .last()
            .expect("the global scope is never popped")
    }

    pub fn scope_mut(&mut self) -> &mut CompileScope {
        self.scopes
            .last_mut()
            .expect("the global scope is never popped")
    }

    pub fn stack_depth(&self) -> u32 {
        self.scope().stack_depth
    }

    pub fn with_block<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        let stack_depth = self.stack_depth();
        self.with_scope(
            CompileScope {
                stack_depth,
                ..Default::default()
            },
            f,
        )
    }

    pub fn with_loop<T>(&mut self, loop_context: LoopContext, f: impl FnOnce(&mut Self) -> T) -> T {
        let stack_depth = self.stack_depth();
        self.with_scope(
            CompileScope {
                loop_context: Some(loop_context),
                stack_depth,
                ..Default::default()
            },
            f,
        )
    }

    pub fn with_function<T>(
        &mut self,
        function_context: FunctionContext,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        self.with_scope(
            CompileScope {
                function_context: Some(function_context),
                ..Default::default()
            },
            f,
        )
    }

    fn with_scope<T>(&mut self, scope: CompileScope, f: impl FnOnce(&mut Self) -> T) -> T {
        self.scopes.push(scope);
        let result = f(self);
        self.scopes.pop();
        result
    }

    pub fn function_context(&self) -> Option<&FunctionContext> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.function_context.as_ref())
    }

    pub fn loop_scope(&self) -> Option<&CompileScope> {
        self.scopes
            .iter()
            .rev()
            .find(|scope| scope.loop_context.is_some())
    }

    pub fn get_variable(&self, name: &str) -> Result<Address, CodegenErrorKind> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.variables.get(name))
            .copied()
            .ok_or_else(|| CodegenErrorKind::UndeclaredVariable(name.to_string()))
    }
}
