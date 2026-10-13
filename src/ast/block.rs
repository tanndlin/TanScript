use std::fmt;

use crate::ast::StatementOrExpression;

#[derive(Debug, Clone)]
pub struct Block {
    pub children: Vec<StatementOrExpression>,
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
