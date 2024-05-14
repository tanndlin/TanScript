#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i32),
    Boolean(bool),
    Operator(Operator),
    BitwiseOp(BitwiseOp),
    Type(DataType),
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
    Colon,
    Assign,
    ShortAssign(Operator),
    Increment,
    Decrement,

    LCurly,
    RCurly,

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

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Pointer(Box<DataType>),
    Integer,
    Float,
    Boolean,
}

impl DataType {
    pub fn to_string(&self) -> String {
        match self {
            DataType::Integer => "int".to_string(),
            DataType::Float => "float".to_string(),
            DataType::Boolean => "bool".to_string(),
            DataType::Pointer(t) => format!("{}*", t.to_string()),
        }
    }
}

pub enum Keywords {
    Return,
    If,
    Else,
    While,
    Function,
    True,
    False,
    Integer,
    Float,
    Boolean,
}

impl Keywords {
    pub fn from_string(s: &str) -> Option<Keywords> {
        match s {
            "return" => Some(Keywords::Return),
            "if" => Some(Keywords::If),
            "else" => Some(Keywords::Else),
            "while" => Some(Keywords::While),
            "fn" => Some(Keywords::Function),
            "true" => Some(Keywords::True),
            "false" => Some(Keywords::False),
            "int" => Some(Keywords::Integer),
            "float" => Some(Keywords::Float),
            "bool" => Some(Keywords::Boolean),
            _ => None,
        }
    }

    pub fn to_token(&self) -> Token {
        match self {
            Keywords::Return => Token::Return,
            Keywords::If => Token::If,
            Keywords::Else => Token::Else,
            Keywords::While => Token::While,
            Keywords::Function => Token::Function,
            Keywords::True => Token::Boolean(true),
            Keywords::False => Token::Boolean(false),
            Keywords::Integer => Token::Type(DataType::Integer),
            Keywords::Float => Token::Type(DataType::Float),
            Keywords::Boolean => Token::Type(DataType::Boolean),
            _ => panic!("Keyword not implemented"),
        }
    }
}

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
