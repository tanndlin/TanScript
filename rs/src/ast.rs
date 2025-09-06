use std::fmt;

use crate::types::LexerAtomType;

#[derive(Debug)]
pub struct Assignment {
    pub identifier: String,
    pub expression: Expression,
}

#[derive(Debug)]
pub struct Declaration {
    pub assign: Assignment,
}

#[derive(Debug)]
pub enum Statement {
    Declaration(Declaration),
    Assign(Assignment),
}

impl fmt::Display for Assignment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} = {}", self.identifier, self.expression)
    }
}

impl fmt::Display for Declaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "let {}", self.assign)
    }
}

impl fmt::Display for Statement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Statement::Declaration(e) => write!(f, "{}", e),
            Statement::Assign(e) => write!(f, "{}", e),
        }
    }
}

#[derive(Debug)]
pub enum StatementOrExpression {
    Statement(Statement),
    Expression(Expression),
}

impl fmt::Display for StatementOrExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementOrExpression::Expression(e) => write!(f, "{}", e),
            StatementOrExpression::Statement(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug)]
pub enum AtomType {
    Program,
    Number(i32),
    Identifier(String),
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl AtomType {
    pub fn from_char(op: char) -> AtomType {
        match op {
            '+' => AtomType::Add,
            '-' => AtomType::Subtract,
            '*' => AtomType::Multiply,
            '/' => AtomType::Divide,
            _ => panic!("Unknown operator: {}", op),
        }
    }
    pub fn from_lexer_atom(atom: LexerAtomType) -> AtomType {
        match atom {
            LexerAtomType::Number(n) => AtomType::Number(n),
            LexerAtomType::Identifier(s) => AtomType::Identifier(s),
        }
    }
}

impl fmt::Display for AtomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtomType::Add => write!(f, "+"),
            AtomType::Subtract => write!(f, "-"),
            AtomType::Multiply => write!(f, "*"),
            AtomType::Divide => write!(f, "/"),
            AtomType::Program => write!(f, "Program"),
            AtomType::Number(n) => write!(f, "{}", n),
            AtomType::Identifier(c) => write!(f, "{}", c),
        }
    }
}

#[derive(Debug)]
pub enum Expression {
    Atom(AtomType),
    Operation(AtomType, Vec<Expression>),
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Atom: use {} not {:?>
            Expression::Atom(i) => write!(f, "{}", i),

            // Operation: same here
            Expression::Operation(head, rest) => {
                write!(f, "({}", head)?;
                for s in rest {
                    write!(f, " {}", s)?;
                }
                write!(f, ")")
            }
        }
    }
}
