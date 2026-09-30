use std::fmt;

use crate::ast::{AtomType, OperatorType};

#[derive(Debug, Clone)]
pub enum Expression {
    Atom(AtomType),
    Operation(OperatorType, Vec<Expression>),
    FunctionCall(String, Vec<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Atom: use {} not {:?>
            Expression::Atom(i) => write!(f, "{i}"),

            // Operation: same here
            Expression::Operation(head, rest) => {
                write!(f, "({head}")?;
                for s in rest {
                    write!(f, " {s}")?;
                }
                write!(f, ")")
            }
            Expression::FunctionCall(name, expressions) => write!(
                f,
                "{}({})",
                name,
                expressions
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}
