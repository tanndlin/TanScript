use crate::{
    ast::{AtomType, Block, Expression, ExpressionKind, Statement, StatementOrExpression},
    type_check::Type,
};

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
            Statement::Declaration(s) => {
                if let Some(dtype) = &s.dtype {
                    dtype.references(names);
                }
                s.assign.expression.references(names);
            }
            Statement::ForLoop(s) => {
                s.init.references(names);
                s.condition.references(names);
                s.update.references(names);
                s.block.references(names);
            }
            Statement::FunctionDefinition(s) => {
                s.args.iter().for_each(|a| a.atype.references(names));
                s.return_type.references(names);
                s.body.references(names);
            }
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
            Statement::StructDefinition(s) => {
                s.fields.iter().for_each(|f| f.ftype.references(names));
            }
            Statement::EnumDefinition(_) | Statement::Break(_) | Statement::Continue(_) => {}
        }
    }
}

impl Expression {
    pub fn references(&self, names: &mut Vec<String>) {
        match &self.kind {
            ExpressionKind::Atom(AtomType::Identifier(name)) => names.push(name.clone()),
            ExpressionKind::StructAccess { base, .. } => base.references(names),
            ExpressionKind::Atom(_) => {}
            ExpressionKind::EnumVariant { enum_name, .. } => names.push(enum_name.clone()),
            ExpressionKind::Binary(_, lhs, rhs) | ExpressionKind::Index(lhs, rhs) => {
                lhs.references(names);
                rhs.references(names);
            }
            ExpressionKind::Unary(_, inner) | ExpressionKind::Postfix(_, inner) => {
                inner.references(names);
            }
            ExpressionKind::Cast(inner, ctype) => {
                ctype.references(names);
                inner.references(names);
            }
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

impl Type {
    pub fn references(&self, names: &mut Vec<String>) {
        match self {
            Type::Named {
                name,
                concrete_generics,
                ..
            } => {
                names.push(name.clone());
                for g in concrete_generics {
                    g.references(names);
                }
            }
            Type::Struct(s) => names.push(s.name.clone()),
            Type::Enum(e) => names.push(e.name.clone()),
            Type::Pointer(inner) => inner.references(names),
            Type::Function(f) => {
                f.args.iter().for_each(|a| a.references(names));
                f.return_type.references(names);
            }
            Type::Int(_)
            | Type::Char
            | Type::Boolean
            | Type::Unit
            | Type::Null
            | Type::Unknown
            | Type::Generic(_) => {}
        }
    }
}
