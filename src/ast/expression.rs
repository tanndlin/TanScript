use std::fmt;

use crate::{
    ast::{AtomType, BinaryOp, PostfixOp, UnaryOp},
    type_check::Type,
    types::Span,
};

#[derive(Debug, Clone)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
    pub etype: Box<Type>,
}

impl Expression {
    pub fn new(kind: ExpressionKind, span: Span) -> Self {
        Self {
            kind,
            span,
            etype: Box::new(Type::Unknown),
        }
    }

    pub fn is_place(&self) -> bool {
        matches!(
            self.kind,
            ExpressionKind::Atom(AtomType::Identifier(_))
                | ExpressionKind::Index(_, _)
                | ExpressionKind::StructAccess { .. }
                | ExpressionKind::Unary(UnaryOp::Deref, _)
        )
    }
}

impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

#[derive(Debug, Clone)]
pub enum ExpressionKind {
    Atom(AtomType),
    Binary(BinaryOp, Box<Expression>, Box<Expression>),
    Unary(UnaryOp, Box<Expression>),
    Postfix(PostfixOp, Box<Expression>),
    FunctionCall(String, Vec<Expression>),
    Index(Box<Expression>, Box<Expression>),
    Cast(Box<Expression>, Type),
    SizeOf(Type),
    StructInstantiation {
        struct_type_name: String,
        fields: Vec<(String, Expression)>,
    },
    StructAccess {
        base: Box<Expression>,
        field_name: String,
    },
    EnumVariant {
        enum_name: String,
        variant_name: String,
    },
}

impl ExpressionKind {
    pub fn binary(op: BinaryOp, lhs: Expression, rhs: Expression) -> Self {
        Self::Binary(op, Box::new(lhs), Box::new(rhs))
    }

    pub fn unary(op: UnaryOp, operand: Expression) -> Self {
        Self::Unary(op, Box::new(operand))
    }

    pub fn postfix(op: PostfixOp, operand: Expression) -> Self {
        Self::Postfix(op, Box::new(operand))
    }

    pub fn index(base: Expression, index: Expression) -> Self {
        Self::Index(Box::new(base), Box::new(index))
    }

    pub fn cast(expr: Expression, target: Type) -> Self {
        Self::Cast(Box::new(expr), target)
    }

    pub fn struct_access(base: Expression, field_name: String) -> Self {
        Self::StructAccess {
            base: Box::new(base),
            field_name,
        }
    }
}

impl fmt::Display for ExpressionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExpressionKind::Atom(i) => write!(f, "{i}"),
            ExpressionKind::Binary(op, left, right) => write!(f, "({op} {left} {right})"),
            ExpressionKind::Unary(op, child) => write!(f, "({op} {child})"),
            ExpressionKind::Postfix(op, identifier) => write!(f, "({op} {identifier})"),
            ExpressionKind::FunctionCall(name, expressions) => write!(
                f,
                "{}({})",
                name,
                expressions
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            ExpressionKind::Index(lhs, index) => write!(f, "({lhs}[{index}])"),
            ExpressionKind::Cast(expression, cast_type) => {
                write!(f, "({expression} as {cast_type})")
            }
            ExpressionKind::SizeOf(t) => write!(f, "sizeof({t})"),
            ExpressionKind::StructInstantiation {
                struct_type_name,
                fields,
            } => {
                write!(
                    f,
                    "{struct_type_name} {{ {} }}",
                    fields
                        .iter()
                        .map(|(field_name, field_expr)| format!("{field_name}: {field_expr}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            ExpressionKind::StructAccess { base, field_name } => {
                write!(f, "{base}.{field_name}")
            }
            ExpressionKind::EnumVariant {
                enum_name,
                variant_name,
            } => write!(f, "{enum_name}::{variant_name}"),
        }
    }
}
