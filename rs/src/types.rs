#[derive(Debug, Clone)]
pub enum Token {
    Identifier(String),
    Number(i32),
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub enum LexerValue {
    String(String),
    Int(i32),
}

// Set of all the operators
pub fn operators() -> Vec<Token> {
    [Token::Add, Token::Subtract, Token::Multiply, Token::Divide]
        .iter()
        .cloned()
        .collect()
}
