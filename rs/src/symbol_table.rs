use std::collections::{HashMap, HashSet};

use crate::ast::{AtomType, Block, Declaration, Expression, Statement, StatementOrExpression};

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

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: String,
    pub args: Vec<String>,
    pub body: Block,
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

impl FunctionDefinition {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        symbols.functions.insert(self.name.clone(), self.clone());
        self.body.discover(symbols);
    }
}

impl Declaration {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        self.assign.expression.discover(symbols);
    }
}

impl Statement {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        match self {
            Statement::FunctionDefintion(function_definition) => {
                function_definition.discover(symbols);
            }
            Statement::IfStatement(if_statement) => {
                if_statement.condition.discover(symbols);
                if_statement.block.discover(symbols);
                if let Some(else_block) = &if_statement.else_block {
                    else_block.discover(symbols);
                }
            }
            Statement::Declaration(declaration) => {
                declaration.discover(symbols);
            }
            Statement::Assign(assign) => {
                assign.expression.discover(symbols);
            }
            Statement::WhileLoop(while_loop) => {
                while_loop.condition.discover(symbols);
                while_loop.block.discover(symbols);
            }
            Statement::Return(expression) => {
                expression.discover(symbols);
            }
        }
    }
}

impl Expression {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        match self {
            Expression::FunctionCall(function_call, args) => {
                if !symbols.functions.contains_key(function_call) {
                    symbols.externs.insert(function_call.clone());
                }

                for arg in args {
                    arg.discover(symbols);
                }
            }
            Expression::Atom(atom_type) => match atom_type {
                AtomType::Identifier(_) | AtomType::Number(_) | AtomType::String(_) => {}
            },
            Expression::Operation(_, expressions) => {
                for expr in expressions {
                    expr.discover(symbols);
                }
            }
        }
    }
}
