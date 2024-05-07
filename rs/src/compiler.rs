use core::panic;

use crate::ast::AstNode;
use crate::ast::NodeType;

pub fn compile(ast: &AstNode) -> String {
    // Find all function definitions and put them at the top
    // Then compile the rest of the code
    let mut function_defs = vec![];
    let mut rest = vec![];

    for child in &ast.children {
        match child.node_type {
            NodeType::FunctionDef => function_defs.push(child),
            _ => rest.push(child),
        }
    }

    let function_def_str = function_defs
        .iter()
        .map(|node| compile_node(&node))
        .collect::<Vec<String>>()
        .join("\n");

    let header = "#include <stdio.h>\n";
    let main = "int main() {\n";
    let body = rest
        .iter()
        .map(|node| compile_node(&node))
        .map(|s| "\t".to_string() + &s)
        .collect::<Vec<String>>()
        .join("\n");

    let footer = "\n\treturn 0;\n}";

    format!(
        "{}\n{}\n\n{}{}{}",
        header, function_def_str, main, body, footer
    )
}

fn compile_node(node: &AstNode) -> String {
    match node.node_type {
        NodeType::Add
        | NodeType::Subtract
        | NodeType::Multiply
        | NodeType::Divide
        | NodeType::Mod
        | NodeType::Eq
        | NodeType::NotEq
        | NodeType::LessThan
        | NodeType::GreaterThan
        | NodeType::Leq
        | NodeType::Geq => compile_expression(node),
        NodeType::Number => node.value.clone().unwrap(),
        NodeType::Identifier => node.value.clone().unwrap(),
        NodeType::Block => compile_block(node),
        NodeType::Declare => compile_declare(node),
        NodeType::Assign => compile_assign(node),
        NodeType::LParen => compile_parentheses(&node.children[0]),
        NodeType::FunctionDef => compile_function_def(node),
        NodeType::FunctionCall => compile_function_call(node),
        NodeType::Return => format!("return {};", compile_node(&node.children[0])),
        NodeType::If => compile_if(node),
        NodeType::Parameters => panic!("Unexpected Parameters node"),
    }
}

fn compile_if(node: &AstNode) -> String {
    let cond_ast = &node.children[0];
    let body_ast = &node.children[1];

    if node.children.len() == 2 {
        return format!(
            "if ({}) {{\n{}\n}}",
            compile_expression(cond_ast),
            compile_block(body_ast)
        );
    }

    let cond = compile_expression(cond_ast);
    let body = compile_block(body_ast);
    let else_ast = &node.children[2];

    format!(
        "if ({}) {{\n{}\n}} else {{\n{}\n}}",
        cond,
        body,
        compile_block(else_ast)
    )
}

fn compile_block(node: &AstNode) -> String {
    node.children
        .iter()
        .map(compile_node)
        .map(|s| "\t".to_string() + &s)
        .collect::<Vec<String>>()
        .join("\n")
}

fn compile_function_def(node: &AstNode) -> String {
    let name = &node.value.clone().unwrap();
    let params = &node.children[0];
    let body = &node.children[1];

    // Params.children.len is a multiple of 2
    // The first one is the type and the second one is the name
    // Join each pair with a comma
    let param_str = params
        .children
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if i % 2 == 0 {
                format!(
                    "{} {}",
                    p.value.clone().unwrap(),
                    params.children[i + 1].value.clone().unwrap()
                )
            } else {
                "".to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(", ");

    // Remove last comma and space
    let param_str = param_str.trim_end_matches(", ").to_string();

    format!(
        "int {}({}) {{\n{}\n}}",
        name,
        param_str,
        compile_block(body)
    )
}

fn compile_function_call(node: &AstNode) -> String {
    // Check if the function is a built-in function
    if node.value.clone().unwrap().as_str() == "print" {
        return compile_print(node);
    }

    format!(
        "{}({})",
        node.value.clone().unwrap(),
        node.children
            .iter()
            .map(compile_node)
            .collect::<Vec<String>>()
            .join(", ")
    )
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
        NodeType::Eq => compile_operator!("=="),
        NodeType::NotEq => compile_operator!("!="),
        NodeType::LessThan => compile_operator!("<"),
        NodeType::GreaterThan => compile_operator!(">"),
        NodeType::Leq => compile_operator!("<="),
        NodeType::Geq => compile_operator!(">="),
        _ => compile_node(node),
    }
}

// ---------------------------- Built-in functions ----------------------------
fn compile_print(node: &AstNode) -> String {
    let args = node
        .children
        .iter()
        .map(compile_node)
        .collect::<Vec<String>>();

    let format = args.iter().map(|_| "%d").collect::<Vec<&str>>().join(", ");
    format!("printf(\"{}\\n\", {});", format, args.join(", "))
}
