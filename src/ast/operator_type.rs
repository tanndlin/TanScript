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
    AddAssign,
    SubAssign,
    Increment,
    Decrement,
    OpenBracket,
    CloseBracket,
}

/// Every operator's spelling. The lexer takes the longest match, so order doesn't matter
pub const OPERATORS: &[(&str, OperatorType)] = &[
    ("+", OperatorType::Add),
    ("-", OperatorType::Subtract),
    ("*", OperatorType::Multiply),
    ("/", OperatorType::Divide),
    ("%", OperatorType::Modulo),
    ("<", OperatorType::LessThan),
    ("<=", OperatorType::LessOrEqual),
    (">", OperatorType::GreaterThan),
    (">=", OperatorType::GreaterOrEqual),
    ("==", OperatorType::Equal),
    ("!=", OperatorType::NotEqual),
    ("||", OperatorType::Or),
    ("&&", OperatorType::And),
    ("=", OperatorType::Assign),
    ("{", OperatorType::OpenCurly),
    ("}", OperatorType::CloseCurly),
    ("(", OperatorType::OpenParen),
    (")", OperatorType::CloseParen),
    ("[", OperatorType::OpenBracket),
    ("]", OperatorType::CloseBracket),
    ("!", OperatorType::Not),
    (",", OperatorType::Comma),
    ("+=", OperatorType::AddAssign),
    ("-=", OperatorType::SubAssign),
    ("++", OperatorType::Increment),
    ("--", OperatorType::Decrement),
];

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (symbol, _) = OPERATORS
            .iter()
            .find(|(_, op)| op == self)
            .expect("Every operator is in OPERATORS");
        write!(f, "{symbol}")
    }
}
