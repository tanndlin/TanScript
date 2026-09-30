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

impl Statement for Assignment {
    fn compile(
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
                                format!("; {} = {}", self.lhs, self.expression),
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

    fn discover(&self, symbols: &mut SymbolTable) {
        self.expression.discover(symbols);
    }
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.lhs, self.expression)
    }
}
