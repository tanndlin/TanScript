#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i32),
    Boolean(bool),
    Operator(Operator),
    BitwiseOp(BitwiseOp),
    LParen,
    RParen,
    Eq,
    NotEq,
    LessThan,
    GreaterThan,
    Leq,
    Geq,
    Not,
    And,
    Or,

    Semi,
    Assign,
    ShortAssign(Operator),
    Increment,
    Decrement,

    LCurly,
    RCurly,

    Declare,
    Function,
    Comma,
    Return,
    If,
    Else,
    While,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BitwiseOp {
    And,
    Or,
    Not,
    Xor,
}

pub enum Keywords {
    Let,
    Return,
    If,
    Else,
    While,
    Function,
    True,
    False,
}

impl Keywords {
    pub fn from_string(s: &str) -> Option<Keywords> {
        match s {
            "let" => Some(Keywords::Let),
            "return" => Some(Keywords::Return),
            "if" => Some(Keywords::If),
            "else" => Some(Keywords::Else),
            "while" => Some(Keywords::While),
            "fn" => Some(Keywords::Function),
            "true" => Some(Keywords::True),
            "false" => Some(Keywords::False),
            _ => None,
        }
    }

    pub fn to_token(&self) -> Token {
        match self {
            Keywords::Let => Token::Declare,
            Keywords::Return => Token::Return,
            Keywords::If => Token::If,
            Keywords::Else => Token::Else,
            Keywords::While => Token::While,
            Keywords::Function => Token::Function,
            Keywords::True => Token::Boolean(true),
            Keywords::False => Token::Boolean(false),
            _ => panic!("Keyword not implemented"),
        }
    }
}

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
