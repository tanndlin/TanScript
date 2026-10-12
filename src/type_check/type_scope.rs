use std::collections::HashMap;

use crate::type_check::{Type, TypeCheckError};

#[derive(Debug)]
pub struct TypeScope {
    scopes: Vec<HashMap<String, usize>>,
    bindings: Vec<Type>,
}

impl Default for TypeScope {
    fn default() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            bindings: vec![],
        }
    }
}

impl TypeScope {
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
            && self.bindings[id] == Type::CoercibleNumber
        {
            self.bindings[id] = t;
        }
    }

    pub fn lookup(&self, name: &str) -> Result<&Type, TypeCheckError> {
        self.find(name)
            .map(|id| &self.bindings[id])
            .ok_or(TypeCheckError::IdentifierNotFound(name.to_string()))
    }

    pub fn binding(&self, id: usize) -> &Type {
        &self.bindings[id]
    }
}
