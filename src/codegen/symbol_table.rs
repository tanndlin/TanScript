use std::collections::{HashMap, HashSet};

use crate::ast::{
    AtomType, Block, Expression, ExpressionKind, FunctionDefinition, StatementOrExpression,
};

pub struct SymbolTable {
    pub functions: HashMap<String, FunctionDefinition>,
    pub externs: HashSet<String>,
}
impl SymbolTable {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
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
            ExpressionKind::Atom(atom_type) => match atom_type {
                AtomType::Identifier(_)
                | AtomType::Number(_)
                | AtomType::String(_)
                | AtomType::Boolean(_)
                | AtomType::Colon => {}
            },
            ExpressionKind::Binary(_, left, right) => {
                left.discover(symbols);
                right.discover(symbols);
            }
            ExpressionKind::Unary(_, child) => child.discover(symbols),
            ExpressionKind::Postfix(_, _) => {}
            ExpressionKind::Index(lhs, index) => {
                lhs.discover(symbols);
                index.discover(symbols);
            }
        }
    }
}
