use crate::{
    ast::{ASTNode, ASTType},
    types::LexerToken,
};

pub struct Parser {
    pub index: usize,
    pub tokens: Vec<LexerToken>,
}

impl Parser {
    pub fn new(tokens: Vec<LexerToken>) -> Parser {
        Parser { index: 0, tokens }
    }

    pub fn parse(&mut self) -> ASTNode {
        let mut prog = ASTNode::new(ASTType::Program);

        match self.parse_next() {
            Some(mut ptr) => {
                prog.set_left(ptr);

                while let Some(node) = self.parse_next() {
                    ptr.set_right(node);
                    ptr = node;
                }
            }
            None => (),
        }
        prog
    }

    fn parse_next(&mut self) -> Option<ASTNode> {}

    fn peek(&self) -> Option<&LexerToken> {
        if self.index < self.tokens.len() {
            Some(&self.tokens[self.index])
        } else {
            None
        }
    }
}
