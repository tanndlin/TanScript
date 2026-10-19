use crate::ast::{AtomType, Block, Expression, ExpressionKind, Statement, StatementOrExpression};

impl Block {
    pub fn references(&self, names: &mut Vec<String>) {
        self.children.iter().for_each(|c| c.references(names));
    }
}

impl StatementOrExpression {
    pub fn references(&self, names: &mut Vec<String>) {
        match self {
            StatementOrExpression::Statement(statement) => statement.references(names),
            StatementOrExpression::Expression(expression) => expression.references(names),
        }
    }
}

impl Statement {
    pub fn references(&self, names: &mut Vec<String>) {
        match self {
            Statement::Assignment(s) => {
                s.lhs.references(names);
                s.expression.references(names);
            }
            Statement::Declaration(s) => s.assign.expression.references(names),
            Statement::ForLoop(s) => {
                s.init.references(names);
                s.condition.references(names);
                s.update.references(names);
                s.block.references(names);
            }
            Statement::FunctionDefinition(s) => s.body.references(names),
            Statement::IfStatement(s) => {
                s.condition.references(names);
                s.block.references(names);
                if let Some(else_block) = &s.else_block {
                    else_block.references(names);
                }
            }
            Statement::Return(s) => {
                if let Some(expr) = &s.expr {
                    expr.references(names);
                }
            }
            Statement::WhileLoop(s) => {
                s.condition.references(names);
                s.block.references(names);
            }
            Statement::StructDefinition(_)
            | Statement::EnumDefinition(_)
            | Statement::Break(_)
            | Statement::Continue(_) => {}
        }
    }
}

impl Expression {
    pub fn references(&self, names: &mut Vec<String>) {
        match &self.kind {
            ExpressionKind::Atom(AtomType::Identifier(name)) => names.push(name.clone()),
            ExpressionKind::StructAccess { base, .. } => base.references(names),
            ExpressionKind::Atom(_) | ExpressionKind::EnumVariant { .. } => {}
            ExpressionKind::Binary(_, lhs, rhs) | ExpressionKind::Index(lhs, rhs) => {
                lhs.references(names);
                rhs.references(names);
            }
            ExpressionKind::Unary(_, inner)
            | ExpressionKind::Postfix(_, inner)
            | ExpressionKind::Cast(inner, _) => inner.references(names),
            ExpressionKind::FunctionCall(name, args) => {
                names.push(name.clone());
                for arg in args {
                    arg.references(names);
                }
            }
            ExpressionKind::StructInstantiation {
                struct_type_name,
                fields,
            } => {
                names.push(struct_type_name.clone());
                for (_, field) in fields {
                    field.references(names);
                }
            }
        }
    }
}
