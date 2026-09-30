use std::fmt;

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
