use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::ast::{
    Block, Child, EnumDefinition, Expression, ExpressionKind, FunctionDefinition, Statement,
    StructDefinition, Walk,
};

#[derive(Default)]
pub struct SymbolTable {
    pub functions: HashMap<String, Rc<FunctionDefinition>>,
    pub structs: HashMap<String, Rc<StructDefinition>>,
    pub externs: HashSet<String>,
    pub enums: HashMap<String, EnumDefinition>,
}

impl Block {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        self.visit(&mut |c| {
            match c {
                Child::Statement(Statement::FunctionDefinition(s)) => {
                    symbols.functions.insert(s.name.clone(), s.clone());
                }
                Child::Statement(Statement::StructDefinition(s)) => {
                    symbols.structs.insert(s.name.clone(), s.clone());
                }
                Child::Statement(Statement::EnumDefinition(s)) => {
                    symbols.enums.insert(s.name.clone(), s.clone());
                }
                Child::Expression(Expression {
                    kind: ExpressionKind::FunctionCall(name, _),
                    ..
                }) if !symbols.functions.contains_key(name) => {
                    symbols.externs.insert(name.clone());
                }
                _ => {}
            }
            true
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::{codegen::symbol_table::SymbolTable, lex::lex, parse::parse};

    fn externs(src: &str) -> Vec<String> {
        let program = parse(lex(src, 0).unwrap()).unwrap();
        let mut symbols = SymbolTable::default();
        program.block.discover(&mut symbols);
        let mut externs: Vec<String> = symbols.externs.into_iter().collect();
        externs.sort();
        externs
    }

    #[test]
    fn discovers_externs_in_for_loop_header() {
        assert_eq!(
            externs("for (let i = start(); i < 10; i = step(i)) {}"),
            vec!["start", "step"]
        );
    }
}
