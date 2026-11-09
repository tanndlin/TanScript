use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    rc::Rc,
};

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

    pub fn resolve<'t>(&self, t: &'t Type) -> Result<Cow<'t, Type>, TypeCheckError> {
        match t {
            Type::Named {
                name,
                concrete_generics,
                span,
            } => match self.lookup(name) {
                Some(Type::Struct(def)) => Ok(Cow::Owned(self.instantiate(
                    def,
                    concrete_generics,
                    *span,
                )?)),
                Some(Type::Named { .. }) => Ok(Cow::Borrowed(t)),
                Some(named_type @ (Type::Enum(_) | Type::Generic(_)))
                    if concrete_generics.is_empty() =>
                {
                    Ok(Cow::Owned(named_type.clone()))
                }
                Some(Type::Enum(_) | Type::Generic(_)) => {
                    Err(TypeCheckErrorKind::IncorrectNumberOfGenerics {
                        name: name.clone(),
                        expected: 0,
                        given: concrete_generics.len(),
                    }
                    .at(*span))
                }
                _ => Err(TypeCheckErrorKind::UnknownType(name.clone()).at(*span)),
            },
            Type::Pointer(inner) if self.mentions_placeholder(inner) => Ok(Cow::Borrowed(t)),
            Type::Pointer(inner) => Ok(match self.resolve(inner)? {
                Cow::Borrowed(_) => Cow::Borrowed(t),
                Cow::Owned(inner) => Cow::Owned(Type::Pointer(Box::new(inner))),
            }),
            _ => Ok(Cow::Borrowed(t)),
        }
    }

    // Only sets the type when a new type was narrowed, in whic case it is owned
    pub fn resolve_in_place(&self, t: &mut Type) -> Result<(), TypeCheckError> {
        if let Cow::Owned(resolved) = self.resolve(t)? {
            *t = resolved;
        }
        Ok(())
    }

    fn mentions_placeholder(&self, t: &Type) -> bool {
        match t {
            Type::Named {
                name,
                concrete_generics,
                ..
            } => {
                matches!(self.lookup(name), Some(Type::Named { .. }))
                    || concrete_generics
                        .iter()
                        .any(|generic| self.mentions_placeholder(generic))
            }
            Type::Pointer(inner) => self.mentions_placeholder(inner),
            _ => false,
        }
    }

    pub fn check_for_recursion(
        &self,
        t: &Type,
        span: Span,
        parent_structs: &mut HashSet<String>,
    ) -> Result<(), TypeCheckError> {
        let def = match t {
            Type::Struct(def) => def.clone(),
            Type::Named { name, .. } if parent_structs.contains(name) => {
                return Err(TypeCheckErrorKind::RecursiveStruct(name.clone()).at(span));
            }
            Type::Named { .. } => match self.resolve(t).as_deref() {
                Ok(Type::Struct(def)) => Rc::clone(def),
                _ => return Ok(()),
            },
            _ => return Ok(()),
        };

        if !parent_structs.insert(def.name.clone()) {
            return Err(TypeCheckErrorKind::RecursiveStruct(def.name.clone()).at(span));
        }
        for field in &def.fields {
            self.check_for_recursion(&field.ftype, span, parent_structs)?;
        }
        parent_structs.remove(&def.name);
        Ok(())
    }

    pub fn expand<'t>(&self, t: &'t Type) -> Cow<'t, Type> {
        match t {
            Type::Named { .. } => self.resolve(t).unwrap_or(Cow::Borrowed(t)),
            _ => Cow::Borrowed(t),
        }
    }

    /// Copies `def` with its generics replaced by `concrete_generics` in every field.
    fn instantiate(
        &self,
        def: &Rc<StructDefinition>,
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

        if def.generics.is_empty() {
            return Ok(Type::Struct(Rc::clone(def)));
        }

        let concrete_generics = concrete_generics
            .iter()
            .map(|t| self.resolve(t).map(Cow::into_owned))
            .collect::<Result<Vec<_>, _>>()?;
        let mut def = StructDefinition::clone(def);
        for field in &mut def.fields {
            field.ftype = substitute(&field.ftype, &def.generics, &concrete_generics);
        }
        Ok(Type::Struct(Rc::new(def)))
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
