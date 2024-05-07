use crate::ast::AstNode;
use crate::ast::NodeType;

pub fn compile(ast: &AstNode) -> String {
    let s = "#include <stdio.h>\nint main() {\n";
    s.to_string() + &compile_node(ast) + "\n}"
}

fn compile_node(node: &AstNode) -> String {
    match node.node_type {
        NodeType::Add
        | NodeType::Subtract
        | NodeType::Multiply
        | NodeType::Divide
        | NodeType::Mod => compile_expression(node),
        NodeType::Number => node.value.clone().unwrap(),
        NodeType::Identifier => node.value.clone().unwrap(),
        NodeType::Block => node.children.iter().map(compile_node).collect(),
        NodeType::Declare => compile_declare(node),
        NodeType::Assign => compile_assign(node),
        NodeType::LParen => compile_parentheses(&node.children[0]),
    }
}

fn compile_parentheses(node: &AstNode) -> String {
    format!("({})", compile_expression(node))
}

fn compile_declare(node: &AstNode) -> String {
    format!("int {}", compile_assign(&node.children[0]))
}

fn compile_assign(node: &AstNode) -> String {
    let ident = &node.children[0];
    let expression = &node.children[1];

    format!(
        "{} = {};",
        ident.value.clone().unwrap(),
        compile_expression(expression)
    )
}

fn compile_expression(node: &AstNode) -> String {
    macro_rules! compile_operator {
        ($op:tt) => {
            format!(
                "{} {} {}",
                compile_expression(&node.children[0]),
                $op,
                compile_expression(&node.children[1])
            )
        };
    }

    match node.node_type {
        NodeType::Add => compile_operator!("+"),
        NodeType::Subtract => compile_operator!("-"),
        NodeType::Multiply => compile_operator!("*"),
        NodeType::Divide => compile_operator!("/"),
        NodeType::Mod => compile_operator!("%"),
        _ => compile_node(node),
    }
}
