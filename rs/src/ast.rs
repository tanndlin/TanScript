#[derive(Debug)]
pub enum ASTType {
    Program,
    Number(i32),
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug)]
pub struct ASTNode {
    pub ast_type: ASTType,
    pub children: Vec<ASTNode>,
}

impl ASTNode {
    pub fn new(ast_type: ASTType) -> ASTNode {
        ASTNode {
            ast_type,
            children: vec![],
        }
    }

    pub fn add_child(&mut self, node: ASTNode) {
        self.children.push(node);
    }
}

use std::fmt;

#[derive(Debug)]
pub enum S {
    Atom(char),
    Cons(char, Vec<S>),
}

impl fmt::Display for S {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            S::Atom(i) => write!(f, "{}", i),
            S::Cons(head, rest) => {
                write!(f, "({}", head)?;
                for s in rest {
                    write!(f, " {}", s)?
                }
                write!(f, ")")
            }
        }
    }
}
