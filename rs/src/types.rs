#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(i32),
    Add,
    Subtract,
    Multiply,
    Divide,
    Mod,
    Semi,
}
