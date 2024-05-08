#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i32),
    Add,
    Subtract,
    Multiply,
    Divide,
    LParen,
    RParen,
    Mod,
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
    ShortAddAssign,
    ShortSubAssign,
    ShortMulAssign,
    ShortDivAssign,
    ShortModAssign,
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
            // Keywords::True => Token::True,
            // Keywords::False => Token::False,
            _ => panic!("Keyword not implemented"),
        }
    }
}

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
