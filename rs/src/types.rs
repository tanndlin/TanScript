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
    Semi,
    Assign,
    LCurly,
    RCurly,

    Declare,
    Function,
    Comma,
    Return,
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
            // Keywords::If => Token::Identifier("if".to_string()),
            // Keywords::Else => Token::Identifier("else".to_string()),
            // Keywords::While => Token::Identifier("while".to_string()),
            Keywords::Function => Token::Function,
            // Keywords::True => Token::Identifier("true".to_string()),
            // Keywords::False => Token::Identifier("false".to_string()),
            _ => panic!("Keyword not implemented"),
        }
    }
}

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}
