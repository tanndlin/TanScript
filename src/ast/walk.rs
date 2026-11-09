use crate::ast::{Block, Expression, ExpressionKind, Statement, StatementOrExpression};
use derive_more::From;

#[derive(From)]
pub enum Child<'a> {
    Statement(&'a Statement),
    Expression(&'a Expression),
    Block(&'a Block),
}

impl<'a> From<&'a StatementOrExpression> for Child<'a> {
    fn from(child: &'a StatementOrExpression) -> Self {
        match child {
            StatementOrExpression::Statement(statement) => Child::Statement(statement),
            StatementOrExpression::Expression(expression) => Child::Expression(expression),
        }
    }
}

pub trait Walk {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool);

    fn walk(&self, f: &mut impl FnMut(&Expression)) {
        self.visit(&mut |c| match c {
            Child::Statement(s) => !matches!(s, Statement::FunctionDefinition(_)),
            Child::Expression(e) => {
                f(e);
                true
            }
            Child::Block(_) => true,
        });
    }

    fn any_expr(&self, pred: impl Fn(&Expression) -> bool) -> bool {
        let mut found = false;
        self.walk(&mut |e| found |= pred(e));
        found
    }

    fn calls_function(&self) -> bool {
        self.any_expr(|e| matches!(e.kind, ExpressionKind::FunctionCall(..)))
    }
}

impl<T: Walk> Walk for Option<T> {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool) {
        if let Some(t) = self {
            t.visit(f);
        }
    }
}

impl<T: Walk> Walk for Vec<T> {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool) {
        for t in self {
            t.visit(f);
        }
    }
}

impl<T: Walk> Walk for Box<T> {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool) {
        self.as_ref().visit(f);
    }
}

impl Walk for Expression {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool) {
        if !f(Child::Expression(self)) {
            return;
        }
        match &self.kind {
            ExpressionKind::Atom(_)
            | ExpressionKind::EnumVariant { .. }
            | ExpressionKind::SizeOf(_) => {}
            ExpressionKind::Binary(_, l, r) | ExpressionKind::Index(l, r) => {
                l.visit(f);
                r.visit(f);
            }
            ExpressionKind::Unary(_, c)
            | ExpressionKind::Postfix(_, c)
            | ExpressionKind::Cast(c, _)
            | ExpressionKind::StructAccess { base: c, .. } => c.visit(f),
            ExpressionKind::FunctionCall(_, args) => args.visit(f),
            ExpressionKind::StructInstantiation { fields, .. } => {
                for (_, e) in fields {
                    e.visit(f);
                }
            }
        }
    }
}

impl Walk for Block {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool) {
        if f(Child::Block(self)) {
            self.children.visit(f);
        }
    }
}

impl Walk for StatementOrExpression {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool) {
        match self {
            StatementOrExpression::Statement(s) => s.visit(f),
            StatementOrExpression::Expression(e) => e.visit(f),
        }
    }
}

impl Walk for Statement {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool) {
        if f(Child::Statement(self)) {
            self.children().for_each(|child| child.visit(f));
        }
    }
}

impl Walk for Child<'_> {
    fn visit(&self, f: &mut impl FnMut(Child) -> bool) {
        match self {
            Child::Statement(s) => s.visit(f),
            Child::Expression(e) => e.visit(f),
            Child::Block(b) => b.visit(f),
        }
    }
}
