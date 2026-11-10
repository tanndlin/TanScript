use std::fmt;

use crate::{ast::Assignment, type_check::Type};

#[derive(Debug, Clone)]
pub struct Declaration {
    pub assign: Assignment,
    pub dtype: Option<Type>,
    pub binding: Option<usize>,
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {}", self.assign)
    }
}
