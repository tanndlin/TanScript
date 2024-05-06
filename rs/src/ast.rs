#[derive(Debug, PartialEq)]
pub enum NodeType {
    // Factors
    Number,
    Identifier,

    // Flow control
    Block,

    // Expressions
    Add,
    Subtract,
    Multiply,
    Divide,
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
    match node.node_type {
        NodeType::Number => node.value.clone().unwrap().parse::<i32>().ok(),
        NodeType::Add => {
            let left = evaluate_node(&node.children[0]);
            let right = evaluate_node(&node.children[1]);
            match (left, right) {
                (Some(l), Some(r)) => Some(l + r),
                _ => None,
            }
        }
        NodeType::Subtract => {
            let left = evaluate_node(&node.children[0]);
            let right = evaluate_node(&node.children[1]);
            match (left, right) {
                (Some(l), Some(r)) => Some(l - r),
                _ => None,
            }
        }
        NodeType::Multiply => {
            let left = evaluate_node(&node.children[0]);
            let right = evaluate_node(&node.children[1]);
            match (left, right) {
                (Some(l), Some(r)) => Some(l * r),
                _ => None,
            }
        }
        NodeType::Divide => {
            let left = evaluate_node(&node.children[0]);
            let right = evaluate_node(&node.children[1]);
            match (left, right) {
                (Some(l), Some(r)) => Some(l / r),
                _ => None,
            }
        }
        NodeType::Block => {
            let mut result = None;
            for child in &node.children {
                result = evaluate_node(child);
            }
            result
        }
        _ => None,
    }
}
