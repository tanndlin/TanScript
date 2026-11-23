use std::{borrow::Cow, collections::HashMap, rc::Rc};

use crate::{
    ast::{AtomType, EnumDefinition, Expression, ExpressionKind, StructDefinition, UnaryOp},
    codegen::{
        Codegen, CodegenErrorKind,
        address::{Address, Register},
        function_context::FunctionContext,
        instruction::Instruction,
        loop_context::LoopContext,
    },
    type_check::{Type, unify},
    types::IntType,
};

pub const SLOT_SIZE: u32 = 8;

#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct CompileScope {
    variables: HashMap<String, (Address, Type)>,
    structs: HashMap<String, Rc<StructDefinition>>,
    enums: HashMap<String, Rc<EnumDefinition>>,
    function_return_types: HashMap<String, Type>,
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

    pub fn allocate_variable(
        &mut self,
        name: String,
        t: Type,
    ) -> Result<Instruction, CodegenErrorKind> {
        let size = u32::try_from(t.size()).unwrap().next_multiple_of(SLOT_SIZE);
        let address = Address::Stack(i32::try_from(self.stack_depth + size).unwrap());
        self.add_variable(name, address, t)?;
        self.stack_depth += size;
        Ok(Instruction::sub(Register::RSP, size))
    }

    pub fn insert_struct(&mut self, def: Rc<StructDefinition>) {
        self.structs.insert(def.name.clone(), def);
    }

    pub fn insert_enum(&mut self, def: Rc<EnumDefinition>) {
        self.enums.insert(def.name.clone(), def);
    }

    pub fn insert_return_type(&mut self, name: String, return_type: Type) {
        self.function_return_types.insert(name, return_type);
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

    pub fn lookup(&self, name: &str) -> Result<&(Address, Type), CodegenErrorKind> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.variables.get(name))
            .ok_or_else(|| CodegenErrorKind::UndeclaredVariable(name.to_string()))
    }

    pub fn get_variable(&self, name: &str) -> Result<Address, CodegenErrorKind> {
        self.lookup(name).map(|(address, _)| *address)
    }

    pub fn get_type(&self, name: &str) -> Result<&Type, CodegenErrorKind> {
        self.lookup(name).map(|(_, t)| t)
    }

    pub fn expression_type<'s>(
        &'s self,
        expr: &'s Expression,
    ) -> Result<Cow<'s, Type>, CodegenErrorKind> {
        Ok(match &expr.kind {
            ExpressionKind::Atom(AtomType::Identifier(name)) => Cow::Borrowed(self.get_type(name)?),
            ExpressionKind::Atom(atom) => Cow::Owned(Type::from(atom)),
            ExpressionKind::Binary(op, _, _) if op.returns_boolean() => Cow::Owned(Type::Boolean),
            ExpressionKind::Binary(_, lhs, rhs) => self.unified_type(lhs, rhs)?,
            ExpressionKind::Unary(UnaryOp::Not, _) => Cow::Owned(Type::Boolean),
            ExpressionKind::Unary(UnaryOp::Negate, inner) => {
                match self.expression_type(inner)?.into_owned() {
                    Type::Int(t) => Cow::Owned(Type::Int(t.negated())),
                    t => Cow::Owned(t),
                }
            }
            ExpressionKind::Postfix(_, inner) => self.expression_type(inner)?,
            ExpressionKind::Unary(UnaryOp::AddressOf, inner) => {
                Cow::Owned(self.expression_type(inner)?.into_owned().pointer_to())
            }
            ExpressionKind::Unary(UnaryOp::Deref, _) | ExpressionKind::Index(_, _) => {
                Cow::Borrowed(&*expr.etype)
            }
            ExpressionKind::Cast(_, cast_type) => Cow::Borrowed(cast_type),
            ExpressionKind::SizeOf(_) => Cow::Owned(Type::Int(IntType::Coercible)),
            ExpressionKind::StructAccess { base, field_name } => {
                Cow::Owned(match self.expression_type(base)?.as_ref() {
                    Type::Struct(struct_def) => struct_def
                        .get_field_type(field_name)
                        .cloned()
                        .unwrap_or(Type::Unknown),
                    _ => Type::Unknown,
                })
            }
            ExpressionKind::StructInstantiation { .. } => Cow::Borrowed(&*expr.etype),
            ExpressionKind::FunctionCall(name, _) => self
                .lookup_return_type(name)
                .map_or(Cow::Owned(Type::Unknown), Cow::Borrowed),
            ExpressionKind::EnumVariant { enum_name, .. } => Cow::Owned(
                self.lookup_enum(enum_name)
                    .map_or(Type::Unknown, |def| Type::Enum(Rc::clone(def))),
            ),
        })
    }

    pub fn unified_type<'s>(
        &'s self,
        lhs: &'s Expression,
        rhs: &'s Expression,
    ) -> Result<Cow<'s, Type>, CodegenErrorKind> {
        let lhs = self.expression_type(lhs)?;
        let rhs = self.expression_type(rhs)?;
        Ok(Cow::Owned(
            unify(&lhs, &rhs)
                .or_else(|| unify(&rhs, &lhs))
                .cloned()
                .unwrap_or(Type::Unknown),
        ))
    }

    pub fn lookup_return_type(&self, name: &str) -> Option<&Type> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.function_return_types.get(name))
    }

    pub fn lookup_enum(&self, name: &str) -> Option<&Rc<EnumDefinition>> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.enums.get(name))
    }

    pub fn lookup_struct(&self, name: &str) -> Option<&Rc<StructDefinition>> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.structs.get(name))
    }
}
