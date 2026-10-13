use std::collections::HashMap;

use crate::{
    ast::{AtomType, Expression, ExpressionKind},
    codegen::{CodegenErrorKind, address::Address, loop_context::LoopContext},
    type_check::Type,
};

pub const SLOT_SIZE: i32 = 8;

#[allow(dead_code)]
pub struct CompileScope<'a> {
    pub parent: Option<&'a CompileScope<'a>>,
    variables: HashMap<String, (Address, Type)>,
    loop_context: Option<LoopContext>,
    /// Bytes `rsp` sits below `rbp`. Each local reserves its own 8 byte slot as it is declared, so
    /// this is also the offset of the most recently declared local
    pub stack_depth: i32,
}

impl<'a> CompileScope<'a> {
    pub fn new(parent: Option<&'a CompileScope<'a>>) -> Self {
        Self {
            parent,
            variables: HashMap::new(),
            loop_context: None,
            stack_depth: parent.map_or(0, |p| p.stack_depth),
        }
    }

    pub fn new_loop(parent: &'a CompileScope<'a>, loop_context: LoopContext) -> Self {
        Self {
            loop_context: Some(loop_context),
            ..Self::new(Some(parent))
        }
    }

    pub fn loop_context(&self) -> Option<&LoopContext> {
        self.loop_context.as_ref()
    }

    fn lookup(&self, name: &str) -> Result<&(Address, Type), CodegenErrorKind> {
        if let Some(variable) = self.variables.get(name) {
            return Ok(variable);
        }

        let parent = self
            .parent
            .as_ref()
            .ok_or_else(|| CodegenErrorKind::UndeclaredVariable(name.to_string()))?;

        parent.lookup(name)
    }

    pub fn get_variable(&self, name: &str) -> Result<Address, CodegenErrorKind> {
        self.lookup(name).map(|(address, _)| *address)
    }

    pub fn get_type(&self, name: &str) -> Result<&Type, CodegenErrorKind> {
        self.lookup(name).map(|(_, t)| t)
    }

    pub fn element_type(&self, expr: &Expression) -> Result<Type, CodegenErrorKind> {
        let pointer = match &expr.kind {
            ExpressionKind::Atom(AtomType::Identifier(name)) => self.get_type(name)?.clone(),
            ExpressionKind::Index(lhs, _) => self.element_type(lhs)?,
            _ => Type::Unknown,
        };

        Ok(match pointer {
            Type::Pointer(inner) => *inner,
            _ => Type::Unknown,
        })
    }

    pub fn add_variable(
        &mut self,
        name: String,
        address: Address,
        t: Type,
    ) -> Result<(), CodegenErrorKind> {
        if self.variables.contains_key(&name) {
            return Err(CodegenErrorKind::AlreadyDeclared(name));
        }

        self.variables.insert(name, (address, t));
        Ok(())
    }

    pub fn allocate_variable(&mut self, name: String, t: Type) -> Result<String, CodegenErrorKind> {
        let address = Address::Stack(self.stack_depth + SLOT_SIZE);
        self.add_variable(name, address, t)?;
        self.stack_depth += SLOT_SIZE;
        Ok(format!("sub rsp, {SLOT_SIZE}"))
    }

    pub fn loop_scope(&self) -> Option<&CompileScope<'a>> {
        if self.loop_context.is_some() {
            Some(self)
        } else {
            self.parent?.loop_scope()
        }
    }
}
