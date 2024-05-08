#[derive(Debug, PartialEq)]
pub enum NodeType {
    // Factors
    Number,
    Identifier,

    // Flow control
    Block,
    FunctionDef,
    Parameters,
    FunctionCall,
    Return,
    If,
    While,

    // Expressions
    Add,
    Subtract,
    Multiply,
    Divide,
    LParen,
    Mod,
    Declare,
    Assign,
    Eq,
    NotEq,
    LessThan,
    GreaterThan,
    Leq,
    Geq,
}

#[derive(Debug)]
pub struct AstNode {
    pub node_type: NodeType,
    pub children: Vec<AstNode>,
    pub value: Option<String>,
}

impl AstNode {
    pub fn evaluate(&self) -> i32 {
        evaluate_node(self).unwrap()
    }
}

fn evaluate_node(node: &AstNode) -> Option<i32> {
    macro_rules! eval_operator {
        ($op:tt) => {
            match (evaluate_node(&node.children[0]), evaluate_node(&node.children[1])) {
                (Some(l), Some(r)) => Some(l $op r),
                _ => None,
            }
        };
    }

    match node.node_type {
        NodeType::Number => node.value.clone().unwrap().parse::<i32>().ok(),
        NodeType::Add => eval_operator!(+),
        NodeType::Subtract => eval_operator!(-),
        NodeType::Multiply => eval_operator!(*),
        NodeType::Divide => eval_operator!(/),
        NodeType::Mod => eval_operator!(%),
        NodeType::Block => {
            let mut result = None;
            for child in &node.children {
                result = evaluate_node(child);
            }
            result
        }
        NodeType::Declare => {
            let assign = &node.children[0];
            let expr = &assign.children[1];

            let value = evaluate_node(expr).unwrap();
            Some(value)
        }
        NodeType::LParen => evaluate_node(&node.children[0]),
        _ => panic!("Not implemented for {:?}", node.node_type),
    }
}
