use std::fmt;

use crate::{
    ast::StatementOrExpression,
    type_check::{TypeCheckError, TypeScope},
};

#[derive(Debug, Clone)]
pub struct Block {
    pub children: Vec<StatementOrExpression>,
}
impl Block {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        let result = self
            .children
            .iter_mut()
            .try_for_each(|child| child.check_types(type_scope));
        type_scope.pop();
        result
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        for child in &mut self.children {
            child.resolve_types(type_scope);
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;

        for child in &self.children {
            if first {
                first = false;
            } else {
                writeln!(f)?;
            }

            write!(f, "{child}")?;
        }

        Ok(())
    }
}
