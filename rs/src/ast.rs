pub enum ASTType {
    Program,
    Number(i32),
    Add,
    Subtract,
    Multiply,
    Divide,
}

pub struct ASTNode<'a> {
    pub ast_type: ASTType,
    pub left: Option<&'a ASTNode<'a>>,
    pub right: Option<&'a ASTNode<'a>>,
}

impl ASTNode {
    pub fn new(ast_type: ASTType) -> ASTNode {
        ASTNode {
            ast_type,
            left: None,
            right: None,
        }
    }

    pub fn set_left(&mut self, left: &ASTNode) {
        self.left = Some(Box::new(left));
    }

    pub fn set_right(&mut self, right: &ASTNode) {
        self.right = Some(Box::new(right));
    }
}
