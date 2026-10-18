use std::collections::HashMap;

use crate::{
    error::At,
    type_check::{Type, TypeCheckError, TypeCheckErrorKind},
    types::{IntType, Span},
};

#[derive(Debug)]
pub struct TypeScope {
    scopes: Vec<HashMap<String, usize>>,
    bindings: Vec<Type>,
    functions: Vec<Type>,
}

impl Default for TypeScope {
    fn default() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            bindings: vec![],
            functions: vec![],
        }
    }
}

impl TypeScope {
    pub fn push_function(&mut self, return_type: Type) {
        self.functions.push(return_type);
    }

    pub fn pop_function(&mut self) {
        self.functions.pop();
    }

    pub fn expected_return_type(&self) -> Option<&Type> {
        self.functions.last()
    }

    pub fn push(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop(&mut self) {
        self.scopes.pop();
    }

    pub fn insert(&mut self, name: String, t: Type) -> usize {
        let id = self.bindings.len();
        self.bindings.push(t);
        self.scopes.last_mut().unwrap().insert(name, id);
        id
    }

    fn find(&self, name: &str) -> Option<usize> {
        self.scopes.iter().rev().find_map(|s| s.get(name)).copied()
    }

    pub fn narrow(&mut self, name: &str, t: Type) {
        if let Some(id) = self.find(name)
            && self.bindings[id] == Type::Int(IntType::Coercible)
        {
            self.bindings[id] = t;
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.find(name).map(|id| &self.bindings[id])
    }

    pub fn binding(&self, id: usize) -> &Type {
        &self.bindings[id]
    }

    pub fn resolve(&self, t: &Type, span: Span) -> Result<Type, TypeCheckError> {
        match t {
            Type::Named(name) => match self.lookup(name) {
                Some(struct_type @ Type::Struct(_)) => Ok(struct_type.clone()),
                _ => Err(TypeCheckErrorKind::UnknownType(name.clone()).at(span)),
            },
            Type::Pointer(inner) => Ok(Type::Pointer(Box::new(self.resolve(inner, span)?))),
            _ => Ok(t.clone()),
        }
    }
}
