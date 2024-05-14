use crate::ast::*;
use crate::types::*;

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

    let header = "#include <stdio.h>\n#include <stdbool.h>\n";
    let main = "int main() {\n";
    let body = rest
        .iter()
        .map(|node| compile_node(&node))
        .map(|s| "\t".to_string() + &s + ";")
        .collect::<Vec<String>>()
        .join("\n");

    let footer = "\n\treturn 0;\n}";

    format!(
        "{}\n{}\n\n{}{}{}",
        header, function_def_str, main, body, footer
    )
}

pub fn compile_node(node: &AstNode) -> String {
    match node.node_type {
        NodeType::Operator(_)
        | NodeType::Eq
        | NodeType::NotEq
        | NodeType::LessThan
        | NodeType::GreaterThan
        | NodeType::Leq
        | NodeType::Geq
        | NodeType::And
        | NodeType::Or
        | NodeType::BitwiseOp(_) => compile_expression(node),
        NodeType::Not => format!("!{}", compile_expression(&node.children[0])),
        NodeType::Number | NodeType::Identifier | NodeType::Boolean => node.value.clone().unwrap(),
        NodeType::Block => compile_block(node),
        NodeType::Declare => compile_declare(node),
        NodeType::Assign => compile_assign(node),
        NodeType::ShortAssign(_) => compile_short_assign(node),
        NodeType::LParen => compile_parentheses(&node.children[0]),
        NodeType::FunctionDef => compile_function_def(node),
        NodeType::FunctionCall => compile_function_call(node),
        NodeType::Return => format!("return {}", compile_node(&node.children[0])),
        NodeType::If => compile_if(node),
        NodeType::While => compile_while(node),
        NodeType::Parameters => panic!("Unexpected Parameters node"),
        NodeType::Type(_) => panic!("Unexpected Type node"),
        NodeType::Parameter => panic!("Unexpected Parameter node"),
    }
}

pub fn compile_short_assign(node: &AstNode) -> String {
    let ident = &node.children[0];
    let expression = &node.children[1];

    let op = match node.node_type {
        NodeType::ShortAssign(Operator::Add) => "+",
        NodeType::ShortAssign(Operator::Subtract) => "-",
        NodeType::ShortAssign(Operator::Multiply) => "*",
        NodeType::ShortAssign(Operator::Divide) => "/",
        NodeType::ShortAssign(Operator::Mod) => "%",
        _ => panic!("Unexpected node type"),
    };

    format!(
        "{} {}= {}",
        ident.value.clone().unwrap(),
        op,
        compile_expression(expression)
    )
}

pub fn compile_while(node: &AstNode) -> String {
    let cond_ast = &node.children[0];
    let body_ast = &node.children[1];

    format!(
        "while ({}) {{\n{}\n}}",
        compile_expression(cond_ast),
        compile_block(body_ast)
    )
}

pub fn compile_if(node: &AstNode) -> String {
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

pub fn compile_block(node: &AstNode) -> String {
    node.children
        .iter()
        .map(compile_node)
        .map(|s| "\t".to_string() + &s + ";")
        .collect::<Vec<String>>()
        .join("\n")
}

pub fn compile_function_def(node: &AstNode) -> String {
    let name = &node.value.clone().unwrap();
    let params = &node.children[0];
    let explicit_type = &node.children[1];
    let body = &node.children[2];

    let param_str = compile_parameters(params);
    let return_type = match explicit_type.node_type {
        NodeType::Type(DataType::Integer) => "int",
        NodeType::Type(DataType::Float) => "float",
        NodeType::Type(DataType::Boolean) => "bool",
        _ => panic!("Unexpected type"),
    };

    format!(
        "{} {}({}) {{\n{}\n}}",
        return_type,
        name,
        param_str,
        compile_block(body)
    )
}

pub fn compile_parameters(node: &AstNode) -> String {
    node.children
        .iter()
        .map(compile_parameter)
        .collect::<Vec<String>>()
        .join(", ")
}

pub fn compile_parameter(node: &AstNode) -> String {
    format!(
        "{} {}",
        node.children[0].value.clone().unwrap(),
        node.children[1].value.clone().unwrap()
    )
}

pub fn compile_function_call(node: &AstNode) -> String {
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

pub fn compile_parentheses(node: &AstNode) -> String {
    format!("({})", compile_expression(node))
}

pub fn compile_declare(node: &AstNode) -> String {
    let value = match node.children[1].node_type {
        NodeType::Assign => compile_assign(&node.children[1]),
        NodeType::Identifier => node.children[1].value.clone().unwrap(),
        _ => panic!("Unexpected node type inside declare"),
    };

    format!(
        "{} {}",
        compile_data_type(&node.children[0].node_type),
        value
    )
}

pub fn compile_data_type(node_type: &NodeType) -> String {
    match node_type {
        NodeType::Type(DataType::Integer) => "int".to_string(),
        NodeType::Type(DataType::Float) => "float".to_string(),
        NodeType::Type(DataType::Boolean) => "bool".to_string(),
        NodeType::Type(DataType::Pointer(pointer_base)) => {
            format!("{}*", compile_pointer_type(pointer_base))
        }
        _ => panic!("Unexpected data type"),
    }
}

pub fn compile_pointer_type(node_type: &Box<DataType>) -> String {
    match **node_type {
        DataType::Integer => "int".to_string(),
        DataType::Float => "float".to_string(),
        DataType::Boolean => "bool".to_string(),
        DataType::Pointer(ref pointer_base) => {
            format!("{}*", compile_pointer_type(pointer_base))
        }
    }
}

pub fn compile_assign(node: &AstNode) -> String {
    let ident = &node.children[0];
    let expression = &node.children[1];

    format!(
        "{} = {}",
        ident.value.clone().unwrap(),
        compile_expression(expression)
    )
}

pub fn compile_expression(node: &AstNode) -> String {
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
        NodeType::Operator(Operator::Add) => compile_operator!("+"),
        NodeType::Operator(Operator::Subtract) => compile_operator!("-"),
        NodeType::Operator(Operator::Multiply) => compile_operator!("*"),
        NodeType::Operator(Operator::Divide) => compile_operator!("/"),
        NodeType::Operator(Operator::Mod) => compile_operator!("%"),
        NodeType::Eq => compile_operator!("=="),
        NodeType::NotEq => compile_operator!("!="),
        NodeType::LessThan => compile_operator!("<"),
        NodeType::GreaterThan => compile_operator!(">"),
        NodeType::Leq => compile_operator!("<="),
        NodeType::Geq => compile_operator!(">="),
        NodeType::And => compile_operator!("&&"),
        NodeType::Or => compile_operator!("||"),
        NodeType::BitwiseOp(BitwiseOp::And) => compile_operator!("&"),
        NodeType::BitwiseOp(BitwiseOp::Or) => compile_operator!("|"),
        NodeType::BitwiseOp(BitwiseOp::Not) => {
            format!("~{}", compile_expression(&node.children[0]))
        }
        NodeType::BitwiseOp(BitwiseOp::Xor) => compile_operator!("^"),
        _ => compile_node(node),
    }
}

// ---------------------------- Built-in functions ----------------------------
pub fn compile_print(node: &AstNode) -> String {
    let args = node
        .children
        .iter()
        .map(compile_node)
        .collect::<Vec<String>>();

    let format = args.iter().map(|_| "%d").collect::<Vec<&str>>().join(", ");
    format!("printf(\"{}\\n\", {})", format, args.join(", "))
}
