use std::collections::{HashMap, HashSet};

use crate::ast::{
    Block, Expression, ExpressionKind, FunctionDefinition, Statement, StatementOrExpression,
    StructDefinition,
};

pub struct SymbolTable {
    pub functions: HashMap<String, FunctionDefinition>,
    pub structs: HashMap<String, StructDefinition>,
    pub externs: HashSet<String>,
}
impl SymbolTable {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
            structs: HashMap::new(),
            externs: HashSet::new(),
        }
    }
}

impl Block {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        self.children.iter().for_each(|c| c.discover(symbols));
    }
}

impl StatementOrExpression {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        match self {
            StatementOrExpression::Statement(statement) => {
                statement.discover(symbols);
            }
            StatementOrExpression::Expression(expression) => {
                expression.discover(symbols);
            }
        }
    }
}

impl Statement {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        match self {
            Statement::Assignment(s) => s.expression.discover(symbols),
            Statement::Declaration(s) => s.assign.expression.discover(symbols),
            Statement::ForLoop(s) => {
                s.condition.discover(symbols);
                s.block.discover(symbols);
            }
            Statement::FunctionDefinition(s) => {
                symbols.functions.insert(s.name.clone(), s.clone());
                s.body.discover(symbols);
            }
            Statement::IfStatement(s) => {
                s.condition.discover(symbols);
                s.block.discover(symbols);
                if let Some(else_block) = &s.else_block {
                    else_block.discover(symbols);
                }
            }
            Statement::Return(s) => {
                if let Some(expr) = &s.expr {
                    expr.discover(symbols);
                }
            }
            Statement::WhileLoop(s) => {
                s.condition.discover(symbols);
                s.block.discover(symbols);
            }
            Statement::StructDefinition(s) => {
                symbols.structs.insert(s.name.clone(), s.clone());
            }
            Statement::Continue(_) | Statement::Break(_) => {}
        }
    }
}

impl Expression {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        match &self.kind {
            ExpressionKind::FunctionCall(function_call, args) => {
                if !symbols.functions.contains_key(function_call) {
                    symbols.externs.insert(function_call.clone());
                }

                for arg in args {
                    arg.discover(symbols);
                }
            }
            ExpressionKind::Atom(_) | ExpressionKind::Postfix(_, _) => {}
            ExpressionKind::StructAccess { base, .. } => base.discover(symbols),
            ExpressionKind::Binary(_, left, right) => {
                left.discover(symbols);
                right.discover(symbols);
            }
            ExpressionKind::Unary(_, child) => child.discover(symbols),
            ExpressionKind::Index(lhs, index) => {
                lhs.discover(symbols);
                index.discover(symbols);
            }
            ExpressionKind::Cast(expression, _) => expression.discover(symbols),
            ExpressionKind::StructInstantiation { fields, .. } => {
                for (_, expr) in fields {
                    expr.discover(symbols);
                }
            }
        }
    }
}
