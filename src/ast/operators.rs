use std::fmt;

use crate::ast::OperatorType;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    /// `+=` and friends, holding the arithmetic they apply
    Compound(BinaryOp),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PostfixOp {
    Increment,
    Decrement,
}

impl BinaryOp {
    pub fn from_token(op: &OperatorType) -> Option<BinaryOp> {
        Some(match op {
            OperatorType::Add => BinaryOp::Add,
            OperatorType::Subtract => BinaryOp::Subtract,
            OperatorType::Multiply => BinaryOp::Multiply,
            OperatorType::Divide => BinaryOp::Divide,
            OperatorType::Modulo => BinaryOp::Modulo,
            OperatorType::LessThan => BinaryOp::LessThan,
            OperatorType::LessOrEqual => BinaryOp::LessOrEqual,
            OperatorType::GreaterThan => BinaryOp::GreaterThan,
            OperatorType::GreaterOrEqual => BinaryOp::GreaterOrEqual,
            OperatorType::Equal => BinaryOp::Equal,
            OperatorType::NotEqual => BinaryOp::NotEqual,
            OperatorType::Or => BinaryOp::Or,
            OperatorType::And => BinaryOp::And,
            _ => return None,
        })
    }

    pub fn binding_power(self) -> (u8, u8) {
        match self {
            BinaryOp::And | BinaryOp::Or => (1, 2),
            BinaryOp::Equal | BinaryOp::NotEqual => (3, 4),
            BinaryOp::LessThan
            | BinaryOp::LessOrEqual
            | BinaryOp::GreaterThan
            | BinaryOp::GreaterOrEqual => (5, 6),
            BinaryOp::Add | BinaryOp::Subtract => (7, 8),
            BinaryOp::Multiply | BinaryOp::Divide | BinaryOp::Modulo => (9, 10),
        }
    }

    fn symbol(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Subtract => "-",
            BinaryOp::Multiply => "*",
            BinaryOp::Divide => "/",
            BinaryOp::Modulo => "%",
            BinaryOp::LessThan => "<",
            BinaryOp::LessOrEqual => "<=",
            BinaryOp::GreaterThan => ">",
            BinaryOp::GreaterOrEqual => ">=",
            BinaryOp::Equal => "==",
            BinaryOp::NotEqual => "!=",
            BinaryOp::Or => "||",
            BinaryOp::And => "&&",
        }
    }
}

impl UnaryOp {
    pub fn from_token(op: &OperatorType) -> Option<UnaryOp> {
        Some(match op {
            OperatorType::Subtract => UnaryOp::Negate,
            OperatorType::Not => UnaryOp::Not,
            _ => return None,
        })
    }

    pub const BINDING_POWER: u8 = 9;

    fn symbol(self) -> &'static str {
        match self {
            UnaryOp::Negate => "-",
            UnaryOp::Not => "!",
        }
    }
}

impl PostfixOp {
    pub fn from_token(op: &OperatorType) -> Option<PostfixOp> {
        Some(match op {
            OperatorType::Increment => PostfixOp::Increment,
            OperatorType::Decrement => PostfixOp::Decrement,
            _ => return None,
        })
    }

    pub const BINDING_POWER: u8 = 11;

    /// The arithmetic applied to the variable, i.e. `a++` is `a = a + 1`
    pub fn arithmetic(self) -> BinaryOp {
        match self {
            PostfixOp::Increment => BinaryOp::Add,
            PostfixOp::Decrement => BinaryOp::Subtract,
        }
    }

    fn symbol(self) -> &'static str {
        match self {
            PostfixOp::Increment => "++",
            PostfixOp::Decrement => "--",
        }
    }
}

impl OperatorType {
    pub fn as_assign(&self) -> Option<AssignOp> {
        Some(match self {
            OperatorType::Assign => AssignOp::Assign,
            OperatorType::AddAssign => AssignOp::Compound(BinaryOp::Add),
            OperatorType::SubAssign => AssignOp::Compound(BinaryOp::Subtract),
            _ => return None,
        })
    }
}

impl fmt::Display for BinaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl fmt::Display for UnaryOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

impl fmt::Display for PostfixOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}
