use std::fmt;

use crate::{
    ast::{Assignment, AtomType, ExpressionKind, Statement},
    codegen::{CompileScope, RegisterHandler, SymbolTable},
    lex::Span,
    type_check::{Type, TypeCheckError, TypeScope, infer_type_from_expression},
};

#[derive(Debug, Clone)]
pub struct Declaration {
    pub assign: Assignment,
    pub dtype: Option<Type>,
    pub span: Span,
}

impl Statement for Declaration {
    fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let ExpressionKind::Atom(AtomType::Identifier(ident)) = &self.assign.lhs.kind else {
            return Err("Declaration LHS can only be a identifier".to_string());
        };

        compile_scope.add_variable(ident.clone(), None)?;
        let assign = self
            .assign
            .compile_uncommented(compile_scope, register_handler)?;
        Ok(format!("; {self}\n{assign}"))
    }

    fn discover(&self, symbols: &mut SymbolTable) {
        self.assign.discover(symbols);
    }

    fn check_types(&self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let ExpressionKind::Atom(AtomType::Identifier(ident)) = &self.assign.lhs.kind else {
            panic!("Declaration LHS can only be a identifier");
        };

        let got = infer_type_from_expression(&self.assign.expression, type_scope);
        let Some(dtype) = &self.dtype else {
            type_scope.insert(ident.clone(), got);
            return Ok(());
        };

        if got != Type::CoercibleNumber && got != *dtype {
            return Err(TypeCheckError::TypeMismatch {
                expected: dtype.clone(),
                got,
                span: self.assign.expression.span,
            });
        }

        type_scope.insert(ident.clone(), dtype.clone());
        Ok(())
    }
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {}", self.assign)
    }
}
