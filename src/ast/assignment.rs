use std::fmt;

use crate::{
    ast::{AtomType, Expression, Statement},
    codegen::{CompileScope, RegisterHandler, SymbolTable},
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
        match &self.lhs {
            Expression::Atom(AtomType::Identifier(ident)) => {
                let address = compile_scope.get_variable(ident)?;
                self.expression
                    .compile(compile_scope, address, register_handler)
            }
            Expression::Index(lhs, index) => {
                register_handler.lease_with_scope(|register_handler, offset_ptr| {
                    // let lhs = lhs.compile(compile_scope, ptr), register_handler);
                    let data_size = 8; // TODO: This will have to change at some point. Assuming 8 bytes
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
                                format!("mov [{ptr}+{offset_ptr}], {value_dst}"),
                            ]
                            .join("\n"))
                        })
                    })
                })
            }
            _ => Err("Invalid LHS for assign".to_string()),
        }
    }
}

impl Statement for Assignment {
    fn compile(
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

    fn discover(&self, symbols: &mut SymbolTable) {
        self.expression.discover(symbols);
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.lhs, self.expression)
    }
}
