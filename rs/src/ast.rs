use crate::types::BitwiseOp;
use crate::types::DataType;
use crate::types::Operator;

#[derive(Debug, PartialEq)]
pub enum NodeType {
    // Factors
    Number,
    Identifier,
    Boolean,

    // Flow control
    Block,
    FunctionDef,
    Parameters,
    Parameter,
    Type(DataType),
    FunctionCall,
    Return,
    If,
    While,

    // Expressions
    Operator(Operator),
    LParen,
    Eq,
    NotEq,
    LessThan,
    GreaterThan,
    Leq,
    Geq,
    And,
    Or,
    Not,
    BitwiseOp(BitwiseOp),

    ShortAssign(Operator),
    Assign,
    Declare,
}

pub fn operator_to_node_type(op: Operator) -> NodeType {
    NodeType::Operator(op)
}

pub fn bitwise_op_to_node_type(op: BitwiseOp) -> NodeType {
    NodeType::BitwiseOp(op)
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
        NodeType::Operator(Operator::Add) => eval_operator!(+),
        NodeType::Operator(Operator::Subtract) => eval_operator!(-),
        NodeType::Operator(Operator::Multiply) => eval_operator!(*),
        NodeType::Operator(Operator::Divide) => eval_operator!(/),
        NodeType::Operator(Operator::Mod) => eval_operator!(%),
        NodeType::Block => {
            let mut result = None;
            for child in &node.children {
                result = evaluate_node(child);
            }
            result
        }
        NodeType::Declare => {
            let assign = &node.children[1];
            let expr = &assign.children[1];

            let value = evaluate_node(expr).unwrap();
            Some(value)
        }
        NodeType::LParen => evaluate_node(&node.children[0]),
        _ => panic!("Not implemented for {:?}", node.node_type),
    }
}
