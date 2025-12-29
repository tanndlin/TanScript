use std::fmt;

use crate::{symbol_table::FunctionDefinition, types::LexerAtomType};

#[derive(Debug)]
pub struct Program {
    pub block: Block,
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.block)
    }
}

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

#[derive(Debug, Clone)]
pub struct Assignment {
    pub identifier: String,
    pub expression: Expression,
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub assign: Assignment,
}

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expression,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub block: Block,
    pub else_block: Option<Block>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Declaration(Declaration),
    Assign(Assignment),
    WhileLoop(WhileLoop),
    IfStatement(IfStatement),
    FunctionDefintion(FunctionDefinition),
    Return(Expression),
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
            Statement::Declaration(e) => write!(f, "{e}"),
            Statement::Assign(e) => write!(f, "{e}"),
            Statement::WhileLoop(while_loop) => {
                write!(
                    f,
                    "while {} {{\n{}\n}}",
                    while_loop.condition, while_loop.block
                )
            }
            Statement::IfStatement(if_statement) => {
                if let Some(else_block) = &if_statement.else_block {
                    write!(
                        f,
                        "if {} {{\n{}\n}} else {{\n{}\n}}",
                        if_statement.condition, if_statement.block, else_block
                    )
                } else {
                    write!(
                        f,
                        "if {} {{\n{}\n}}",
                        if_statement.condition, if_statement.block
                    )
                }
            }
            Statement::FunctionDefintion(def) => write!(
                f,
                "def {}({}) {{{}}}",
                def.name,
                def.args.join(", "),
                def.body
            ),
            Statement::Return(expr) => write!(f, "return {expr}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum StatementOrExpression {
    Statement(Statement),
    Expression(Expression),
}

impl fmt::Display for StatementOrExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StatementOrExpression::Expression(e) => write!(f, "{e}"),
            StatementOrExpression::Statement(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum AtomType {
    Number(i32),
    Identifier(String),
    String(String),
}

impl AtomType {
    pub fn from_lexer_atom(atom: &LexerAtomType) -> AtomType {
        match atom {
            LexerAtomType::Number(n) => AtomType::Number(*n),
            LexerAtomType::Identifier(s) => AtomType::Identifier(s.clone()),
            LexerAtomType::String(s) => AtomType::String(s.clone()),
            LexerAtomType::Semicolon => panic!("Unexpected semicolon"),
        }
    }
}

impl fmt::Display for AtomType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AtomType::Number(n) => write!(f, "{n}"),
            AtomType::Identifier(c) => write!(f, "{c}"),
            AtomType::String(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperatorType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
    Equal,
    NotEqual,
    Or,
    And,
    Assign,
    OpenCurly,
    CloseCurly,
    OpenParen,
    CloseParen,
    Not,
    Comma,
}

impl OperatorType {
    pub fn from_char(op: char) -> OperatorType {
        match op {
            '+' => OperatorType::Add,
            '-' => OperatorType::Subtract,
            '*' => OperatorType::Multiply,
            '/' => OperatorType::Divide,
            '%' => OperatorType::Modulo,
            '<' => OperatorType::LessThan,
            '>' => OperatorType::GreaterThan,
            '!' => OperatorType::Not,
            '=' => OperatorType::Assign,
            '{' => OperatorType::OpenCurly,
            '}' => OperatorType::CloseCurly,
            '(' => OperatorType::OpenParen,
            ')' => OperatorType::CloseParen,
            ',' => OperatorType::Comma,
            _ => panic!("Unknown operator: {op}"),
        }
    }
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperatorType::Add => write!(f, "+"),
            OperatorType::Subtract => write!(f, "-"),
            OperatorType::Multiply => write!(f, "*"),
            OperatorType::Divide => write!(f, "/"),
            OperatorType::Modulo => write!(f, "%"),
            OperatorType::LessThan => write!(f, "<"),
            OperatorType::LessOrEqual => write!(f, "<="),
            OperatorType::GreaterThan => write!(f, ">"),
            OperatorType::GreaterOrEqual => write!(f, ">="),
            OperatorType::Equal => write!(f, "=="),
            OperatorType::NotEqual => write!(f, "!="),
            OperatorType::Or => write!(f, "||"),
            OperatorType::And => write!(f, "&&"),
            OperatorType::Assign => write!(f, "="),
            OperatorType::OpenCurly => write!(f, "{{"),
            OperatorType::CloseCurly => write!(f, "}}"),
            OperatorType::OpenParen => write!(f, "("),
            OperatorType::CloseParen => write!(f, ")"),
            OperatorType::Not => write!(f, "!"),
            OperatorType::Comma => write!(f, ","),
        }
    }
}

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
