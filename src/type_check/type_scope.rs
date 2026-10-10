use std::collections::HashMap;

use crate::type_check::Type;

#[derive(Debug)]
pub struct TypeScope {
    scopes: Vec<HashMap<String, Type>>,
}

impl Default for TypeScope {
    fn default() -> Self {
        Self {
            scopes: vec![HashMap::new()],
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

    pub fn insert(&mut self, name: String, t: Type) {
        self.scopes.last_mut().unwrap().insert(name, t);
    }

    pub fn lookup(&self, name: &str) -> Option<&Type> {
        self.scopes.iter().rev().find_map(|s| s.get(name))
    }
}
