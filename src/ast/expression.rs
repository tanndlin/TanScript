use std::fmt;

use crate::ast::{AtomType, BinaryOp, PostfixOp, UnaryOp};

#[derive(Debug, Clone)]
pub enum Expression {
    Atom(AtomType),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Postfix(PostfixOp, String),
    FunctionCall(String, Vec<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expression::Atom(i) => write!(f, "{i}"),
            Expression::Binary(op, left, right) => write!(f, "({op} {left} {right})"),
            Expression::Unary(op, child) => write!(f, "({op} {child})"),
            Expression::Postfix(op, identifier) => write!(f, "({op} {identifier})"),
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
