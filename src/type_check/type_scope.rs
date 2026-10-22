use std::collections::HashMap;

use crate::{
    ast::StructDefinition,
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

    pub fn resolve(&self, t: &Type) -> Result<Type, TypeCheckError> {
        match t {
            Type::Named {
                name,
                concrete_generics,
                span,
            } => match self.lookup(name) {
                Some(Type::Struct(def)) => self.instantiate(def, concrete_generics, *span),
                Some(named_type @ (Type::Enum(_) | Type::Generic(_)))
                    if concrete_generics.is_empty() =>
                {
                    Ok(named_type.clone())
                }
                Some(Type::Enum(_) | Type::Generic(_)) => {
                    Err(TypeCheckErrorKind::IncorrectNumberOfGenerics {
                        name: name.clone(),
                        expected: 0,
                        given: concrete_generics.len(),
                    }
                    .at(*span))
                }
                Some(Type::Named { .. }) => {
                    Err(TypeCheckErrorKind::RecursiveStruct(name.clone()).at(*span))
                }
                _ => Err(TypeCheckErrorKind::UnknownType(name.clone()).at(*span)),
            },
            Type::Pointer(inner) => match inner.as_ref() {
                Type::Named { name, .. }
                    if matches!(self.lookup(name), Some(Type::Named { .. })) =>
                {
                    Ok(t.clone())
                }
                _ => Ok(Type::Pointer(Box::new(self.resolve(inner)?))),
            },
            _ => Ok(t.clone()),
        }
    }

    pub fn expand(&self, t: Type) -> Type {
        match &t {
            Type::Named { .. } => self.resolve(&t).unwrap_or(t),
            _ => t,
        }
    }

    /// Copies `def` with its generics replaced by `concrete_generics` in every field.
    fn instantiate(
        &self,
        def: &StructDefinition,
        concrete_generics: &[Type],
        span: Span,
    ) -> Result<Type, TypeCheckError> {
        if def.generics.len() != concrete_generics.len() {
            return Err(TypeCheckErrorKind::IncorrectNumberOfGenerics {
                name: def.name.clone(),
                expected: def.generics.len(),
                given: concrete_generics.len(),
            }
            .at(span));
        }

        let concrete_generics = concrete_generics
            .iter()
            .map(|t| self.resolve(t))
            .collect::<Result<Vec<_>, _>>()?;
        let mut def = def.clone();
        for field in &mut def.fields {
            field.ftype = substitute(&field.ftype, &def.generics, &concrete_generics);
        }
        Ok(Type::Struct(Box::new(def)))
    }
}

/// Replaces each generic in `t` with its concrete type, through pointers and type arguments.
fn substitute(t: &Type, generics: &[String], concrete_generics: &[Type]) -> Type {
    match t {
        Type::Generic(name) | Type::Named { name, .. }
            if let Some(index) = generics.iter().position(|g| g == name) =>
        {
            concrete_generics[index].clone()
        }
        Type::Named {
            name,
            concrete_generics: args,
            span,
        } => Type::Named {
            name: name.clone(),
            span: *span,
            concrete_generics: args
                .iter()
                .map(|arg| substitute(arg, generics, concrete_generics))
                .collect(),
        },
        Type::Pointer(inner) => {
            Type::Pointer(Box::new(substitute(inner, generics, concrete_generics)))
        }
        _ => t.clone(),
    }
}
