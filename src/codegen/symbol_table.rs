use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    rc::Rc,
};

use crate::ast::{
    Block, Child, EnumDefinition, Expression, ExpressionKind, ExternDeclaration,
    FunctionDefinition, Statement, StructDefinition, Walk,
};

#[derive(Default)]
pub struct SymbolTable {
    pub functions: BTreeMap<String, Rc<FunctionDefinition>>,
    pub structs: HashMap<String, Rc<StructDefinition>>,
    pub externs: BTreeSet<String>,
    pub extern_declarations: BTreeMap<String, Rc<ExternDeclaration>>,
    pub enums: HashMap<String, Rc<EnumDefinition>>,
}

impl Block {
    pub fn discover(&self, symbols: &mut SymbolTable) {
        self.visit(&mut |c| {
            match c {
                Child::Statement(Statement::FunctionDefinition(s))
                    if !s.signature.generics.is_empty() =>
                {
                    // Skip generic templates; their instances are discovered instead.
                    return false;
                }
                Child::Statement(Statement::FunctionDefinition(s)) => {
                    symbols
                        .functions
                        .insert(s.signature.name.clone(), s.clone());
                }
                Child::Statement(Statement::ExternDeclaration(s)) => {
                    symbols
                        .extern_declarations
                        .insert(s.signature.name.clone(), s.clone());
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
                }) => {
                    symbols.externs.insert(name.clone());
                }
                _ => {}
            }
            true
        });
        symbols
            .externs
            .retain(|name| !symbols.functions.contains_key(name));
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

    #[test]
    fn discovers_extern_declarations() {
        let program = parse(
            lex(
                "extern def puts(s: *char): i32; if (true) { extern def abs(n: i32): i32; }",
                0,
            )
            .unwrap(),
        )
        .unwrap();
        let mut symbols = SymbolTable::default();
        program.block.discover(&mut symbols);
        let names: Vec<&String> = symbols.extern_declarations.keys().collect();
        assert_eq!(names, vec!["abs", "puts"]);
    }
}
