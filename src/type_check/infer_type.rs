use crate::{
    ast::{Expression, ExpressionKind},
    type_check::Type,
};

pub fn infer_type_from_expression(expression: &Expression) -> Type {
    match &expression.kind {
        ExpressionKind::Atom(atom_type) => Type::from(atom_type),
        ExpressionKind::Binary(binary_op, expression, expression1) => todo!(),
        ExpressionKind::Unary(unary_op, expression) => todo!(),
        ExpressionKind::Postfix(postfix_op, expression) => todo!(),
        ExpressionKind::FunctionCall(_, expressions) => todo!(),
        ExpressionKind::Index(expression, expression1) => todo!(),
    }
}
