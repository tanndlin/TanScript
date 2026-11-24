use std::fmt;

use crate::{ast::Assignment, type_check::Type};

#[derive(Debug, Clone)]
pub struct Declaration {
    pub assign: Assignment,
    pub annotation: Option<Type>,
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {}", self.assign)
    }
}
