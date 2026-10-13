use std::fmt;

use crate::{
    ast::{Assignment, AtomType, ExpressionKind},
    codegen::{CompileScope, RegisterHandler, SymbolTable},
    lex::Span,
    type_check::{Type, TypeCheckError, TypeScope, check_types_expression, expect_type},
};

#[derive(Debug, Clone)]
pub struct Declaration {
    pub assign: Assignment,
    pub dtype: Option<Type>,
    pub binding: Option<usize>,
    pub _span: Span,
}

impl Declaration {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let ExpressionKind::Atom(AtomType::Identifier(ident)) = &self.assign.lhs.kind else {
            return Err("Declaration LHS can only be a identifier".to_string());
        };

        let dtype = self.dtype.clone().unwrap_or(Type::Unknown);
        compile_scope.add_variable(ident.clone(), None, dtype)?;
        let assign = self
            .assign
            .compile_uncommented(compile_scope, register_handler)?;
        Ok(format!("; {self}\n{assign}"))
    }

    pub fn discover(&self, symbols: &mut SymbolTable) {
        self.assign.discover(symbols);
    }

    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let ExpressionKind::Atom(AtomType::Identifier(ident)) = &self.assign.lhs.kind else {
            panic!("Declaration LHS can only be a identifier");
        };

        let got = check_types_expression(&self.assign.expression, type_scope)?;
        let Some(expected) = self.dtype.clone() else {
            self.binding = Some(type_scope.insert(ident.clone(), got));
            return Ok(());
        };

        expect_type(&self.assign.expression, &expected, &got, type_scope)?;
        self.binding = Some(type_scope.insert(ident.clone(), expected));
        Ok(())
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        if let Some(id) = self.binding {
            self.dtype = Some(type_scope.binding(id).clone());
        }
    }
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {}", self.assign)
    }
}
