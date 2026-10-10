use crate::{
    ast::{AtomType, Expression, ExpressionKind},
    type_check::{Type, TypeScope},
};

pub fn infer_type_from_expression(expression: &Expression, type_scope: &mut TypeScope) -> Type {
    match &expression.kind {
        ExpressionKind::Atom(atom_type) => {
            if let AtomType::Identifier(ident) = atom_type {
                type_scope.lookup(ident).unwrap().clone() // TODO: This shouldn't be falible because the parser checks for this i think maybe
            } else {
                Type::from(atom_type)
            }
        }
        ExpressionKind::Binary(binary_op, expression, expression1) => todo!(),
        ExpressionKind::Unary(unary_op, expression) => todo!(),
        ExpressionKind::Postfix(postfix_op, expression) => todo!(),
        ExpressionKind::FunctionCall(_, expressions) => todo!(),
        ExpressionKind::Index(expression, expression1) => todo!(),
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::StatementOrExpression,
        parser::parse,
        type_check::{Type, TypeScope, infer_type_from_expression},
    };

    #[test]
    fn infers_type_from_literal() {
        let program = parse("1;").unwrap();
        let Some(StatementOrExpression::Expression(expression)) = program.block.children.first()
        else {
            panic!("expected an expression");
        };
        assert_eq!(
            infer_type_from_expression(expression, &mut TypeScope::default()),
            Type::CoercibleNumber
        );
    }

    #[test]
    fn infers_type_from_variable() {
        let program = parse("let a: u32 = 1; a;").unwrap();
        let mut type_scope = TypeScope::default();

        let StatementOrExpression::Statement(declaration) = &program.block.children[0] else {
            panic!("expected a statement");
        };
        declaration.check_types(&mut type_scope).unwrap();

        let StatementOrExpression::Expression(expression) = &program.block.children[1] else {
            panic!("expected an expression");
        };
        assert_eq!(
            infer_type_from_expression(expression, &mut type_scope),
            Type::U32
        );
    }
}
