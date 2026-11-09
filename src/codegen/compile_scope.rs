use std::{borrow::Cow, collections::HashMap, rc::Rc};

use crate::{
    ast::{
        AtomType, EnumDefinition, Expression, ExpressionKind, FunctionDefinition, StructDefinition,
        UnaryOp,
    },
    codegen::{
        CodegenErrorKind,
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
#[derive(Debug)]
pub struct CompileScope<'a> {
    pub parent: Option<&'a CompileScope<'a>>,
    variables: HashMap<String, (Address, Type)>,
    structs: HashMap<String, Rc<StructDefinition>>,
    enums: HashMap<String, Rc<EnumDefinition>>,
    functions: HashMap<String, Rc<FunctionDefinition>>,
    loop_context: Option<LoopContext>,
    function_context: Option<FunctionContext>,
    /// Bytes `rsp` sits below `rbp`. Each local reserves its own 8 byte slot as it is declared, so
    /// this is also the offset of the most recently declared local
    pub stack_depth: u32,
}

impl<'a> CompileScope<'a> {
    pub fn new(parent: Option<&'a CompileScope<'a>>) -> Self {
        Self {
            parent,
            variables: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            loop_context: None,
            function_context: None,
            stack_depth: parent.map_or(0, |p| p.stack_depth),
        }
    }

    pub fn new_loop(parent: &'a CompileScope<'a>, loop_context: LoopContext) -> Self {
        Self {
            loop_context: Some(loop_context),
            ..Self::new(Some(parent))
        }
    }

    pub fn new_function(parent: &'a CompileScope<'a>, function_context: FunctionContext) -> Self {
        Self {
            function_context: Some(function_context),
            stack_depth: 0,
            ..Self::new(Some(parent))
        }
    }

    pub fn loop_context(&self) -> Option<&LoopContext> {
        self.loop_context.as_ref()
    }

    pub fn function_context(&self) -> Option<&FunctionContext> {
        self.function_context
            .as_ref()
            .or_else(|| self.parent?.function_context())
    }

    pub fn lookup(&self, name: &str) -> Result<&(Address, Type), CodegenErrorKind> {
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
            ExpressionKind::Unary(UnaryOp::Negate, inner) | ExpressionKind::Postfix(_, inner) => {
                self.expression_type(inner)?
            }
            ExpressionKind::Unary(UnaryOp::AddressOf, inner) => Cow::Owned(Type::Pointer(
                Box::new(self.expression_type(inner)?.into_owned()),
            )),
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
            ExpressionKind::StructInstantiation {
                struct_type_name, ..
            } => Cow::Owned(
                self.lookup_struct(struct_type_name)
                    .map_or(Type::Unknown, |def| Type::Struct(def.clone())),
            ),
            ExpressionKind::FunctionCall(name, _) => self
                .lookup_function(name)
                .map_or(Cow::Owned(Type::Unknown), |f| Cow::Borrowed(&f.return_type)),
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

    pub fn loop_scope(&self) -> Option<&CompileScope<'a>> {
        if self.loop_context.is_some() {
            Some(self)
        } else {
            self.parent?.loop_scope()
        }
    }

    pub fn insert_struct(&mut self, def: Rc<StructDefinition>) {
        self.structs.insert(def.name.clone(), def);
    }

    pub fn insert_enum(&mut self, def: Rc<EnumDefinition>) {
        self.enums.insert(def.name.clone(), def);
    }

    pub fn insert_function(&mut self, name: String, def: Rc<FunctionDefinition>) {
        self.functions.insert(name, def);
    }

    pub fn lookup_function(&self, name: &str) -> Option<&Rc<FunctionDefinition>> {
        self.functions
            .get(name)
            .or_else(|| self.parent.and_then(|p| p.lookup_function(name)))
    }

    pub fn lookup_enum(&self, name: &str) -> Option<&Rc<EnumDefinition>> {
        self.enums
            .get(name)
            .or_else(|| self.parent.and_then(|p| p.lookup_enum(name)))
    }

    pub fn lookup_struct(&self, name: &str) -> Option<&Rc<StructDefinition>> {
        self.structs
            .get(name)
            .or_else(|| self.parent.and_then(|p| p.lookup_struct(name)))
    }
}
