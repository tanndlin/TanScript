use crate::ast::{
    Assignment, BreakStatement, Child, ContinueStatement, Declaration, EnumDefinition, ForLoop,
    FunctionDefinition, IfStatement, MatchStatement, ReturnStatement, StructDefinition, WhileLoop,
};
use derive_more::{Display, From};

#[derive(Debug, Clone, From, Display)]
pub enum Statement {
    Assignment(Assignment),
    Declaration(Declaration),
    ForLoop(ForLoop),
    FunctionDefinition(FunctionDefinition),
    StructDefinition(StructDefinition),
    EnumDefinition(EnumDefinition),
    MatchStatement(MatchStatement),
    IfStatement(IfStatement),
    Return(ReturnStatement),
    WhileLoop(WhileLoop),
    Break(BreakStatement),
    Continue(ContinueStatement),
}

impl Statement {
    // Block statements (while, if, def) are not terminated by a semicolon
    pub fn requires_semicolon(&self) -> bool {
        !matches!(
            self,
            Statement::ForLoop(_)
                | Statement::IfStatement(_)
                | Statement::WhileLoop(_)
                | Statement::FunctionDefinition(_)
                | Statement::StructDefinition(_)
                | Statement::EnumDefinition(_)
        )
    }

    pub fn children(&self) -> impl Iterator<Item = Child<'_>> {
        let children = match self {
            Statement::Assignment(Assignment {
                lhs, expression, ..
            })
            | Statement::Declaration(Declaration {
                assign: Assignment {
                    lhs, expression, ..
                },
                ..
            }) => vec![Child::Expression(lhs), Child::Expression(expression)],
            Statement::ForLoop(ForLoop {
                init,
                condition,
                update,
                block,
            }) => vec![
                init.into(),
                Child::Expression(condition),
                update.into(),
                Child::Block(block),
            ],
            Statement::WhileLoop(WhileLoop { condition, block }) => {
                vec![Child::Expression(condition), Child::Block(block)]
            }
            Statement::IfStatement(IfStatement {
                condition,
                block,
                else_block,
            }) => {
                let mut children = vec![Child::Expression(condition), Child::Block(block)];
                children.extend(else_block.as_ref().map(Child::Block));
                children
            }
            Statement::MatchStatement(MatchStatement {
                lhs,
                cases,
                default,
                ..
            }) => {
                let mut children = vec![Child::Expression(lhs)];
                for case in cases {
                    children.push(Child::Expression(&case.rhs));
                    children.push(Child::Block(&case.block));
                }
                children.extend(default.as_ref().map(Child::Block));
                children
            }
            Statement::Return(ReturnStatement { expr, .. }) => {
                expr.iter().map(Child::Expression).collect()
            }
            Statement::FunctionDefinition(FunctionDefinition { body, .. }) => {
                vec![Child::Block(body)]
            }
            Statement::StructDefinition(_)
            | Statement::EnumDefinition(_)
            | Statement::Break(_)
            | Statement::Continue(_) => vec![],
        };
        children.into_iter()
    }
}
