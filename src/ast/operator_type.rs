use std::fmt;

macro_rules! operators {
    ($($variant:ident => $symbol:literal),* $(,)?) => {
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub enum OperatorType {
            $($variant),*
        }

        /// Every operator's spelling. The lexer takes the longest match, so order doesn't matter
        pub const OPERATORS: &[(&str, OperatorType)] = &[
            $(($symbol, OperatorType::$variant)),*
        ];

        impl OperatorType {
            pub fn symbol(&self) -> &'static str {
                match self {
                    $(OperatorType::$variant => $symbol),*
                }
            }
        }
    };
}

operators! {
    Add => "+",
    Subtract => "-",
    Multiply => "*",
    Divide => "/",
    Modulo => "%",
    LessThan => "<",
    LessOrEqual => "<=",
    GreaterThan => ">",
    GreaterOrEqual => ">=",
    Equal => "==",
    NotEqual => "!=",
    Or => "||",
    And => "&&",
    Assign => "=",
    OpenCurly => "{",
    CloseCurly => "}",
    OpenParen => "(",
    CloseParen => ")",
    OpenBracket => "[",
    CloseBracket => "]",
    Not => "!",
    AddAssign => "+=",
    SubAssign => "-=",
    MulAssign => "*=",
    DivAssign => "/=",
    ModAssign => "%=",
    Increment => "++",
    Decrement => "--",
    Dot => ".",
    PathSep => "::",
    Cast => "as",
}

impl fmt::Display for OperatorType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.symbol())
    }
}
