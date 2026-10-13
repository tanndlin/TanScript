use std::fmt;

use crate::{
    ast::{AtomType, Expression, ExpressionKind},
    codegen::{CompileScope, RegisterHandler, SymbolTable, store_element},
    type_check::{
        Type, TypeCheckError, TypeScope, check_types_expression, expect_type, narrow_expression,
    },
};

#[derive(Debug, Clone)]
pub struct Assignment {
    pub lhs: Expression,
    pub expression: Expression,
}

impl Assignment {
    /// Compiles without the leading debug comment, for callers that print their own
    pub fn compile_uncommented(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        match &self.lhs.kind {
            ExpressionKind::Atom(AtomType::Identifier(ident)) => {
                let address = compile_scope.get_variable(ident)?;
                self.expression
                    .compile(compile_scope, address, register_handler)
            }
            ExpressionKind::Index(lhs, index) => {
                let element = compile_scope.element_type(lhs)?;
                register_handler.lease_with_scope(|register_handler, offset_ptr| {
                    let data_size = element.size();
                    let index_asm = index.compile(compile_scope, offset_ptr, register_handler)?;
                    register_handler.lease_with_scope(|register_handler, ptr| {
                        let lhs = lhs.compile(compile_scope, ptr, register_handler)?;

                        register_handler.lease_with_scope(|register_handler, value_dst| {
                            let rhs = self.expression.compile(
                                compile_scope,
                                value_dst,
                                register_handler,
                            )?;
                            Ok([
                                index_asm,
                                format!("imul {offset_ptr}, {offset_ptr}, {data_size}"),
                                lhs,
                                rhs,
                                store_element(
                                    &element,
                                    &format!("[{ptr}+{offset_ptr}]"),
                                    value_dst,
                                ),
                            ]
                            .join("\n"))
                        })
                    })
                })
            }
            _ => Err("Invalid LHS for assign".to_string()),
        }
    }

    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<String, String> {
        let asm = self.compile_uncommented(compile_scope, register_handler)?;
        Ok(format!(
            "; {self}
{asm}"
        ))
    }

    pub fn discover(&self, symbols: &mut SymbolTable) {
        self.expression.discover(symbols);
    }

    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let expected = check_types_expression(&self.lhs, type_scope)?;
        let got = check_types_expression(&self.expression, type_scope)?;

        if expected == Type::CoercibleNumber && got.is_number_type() {
            narrow_expression(&self.lhs, got, type_scope);
            return Ok(());
        }
        expect_type(&self.expression, &expected, &got, type_scope)
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.lhs, self.expression)
    }
}
