use crate::ast;
use crate::lexer::LexerToken;
use crate::types;
use crate::types::Token;

pub fn parse(tokens: Vec<LexerToken>) -> ast::AstNode {
    let mut root = ast::AstNode {
        node_type: ast::NodeType::Block,
        children: vec![],
        value: None,
    };

    let mut position = 0;
    while position < tokens.len() {
        let node = parse_statement(&tokens, &mut position);
        root.children.push(node);
    }

    root
}

fn parse_statement(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let node = parse_next(tokens, position);
    *position += 1; // Skip the semicolon
    node
}

fn parse_next(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let token = &tokens[*position];
    match token.token {
        Token::Add
        | Token::Subtract
        | Token::Multiply
        | Token::Divide
        | Token::Identifier(_)
        | Token::Number(_) => parse_expression(tokens, position),
        _ => panic!("Unexpected token, got {:?}", token.token),
    }
}

fn parse_expression(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    parse_add_sub(tokens, position)
}

fn parse_mul_div(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let left = parse_factor(&tokens[*position], position);
    if *position == tokens.len() {
        return left;
    }

    match tokens[*position].token {
        Token::Multiply | Token::Divide | Token::Mod => {
            let op = consume_one_of(
                vec![Token::Multiply, Token::Divide, Token::Mod],
                tokens,
                position,
            );
            let mut ast = ast::AstNode {
                node_type: match op.token {
                    Token::Multiply => ast::NodeType::Multiply,
                    Token::Divide => ast::NodeType::Divide,
                    Token::Mod => ast::NodeType::Mod,
                    _ => panic!("Expected multiply or divide"),
                },
                children: vec![left],
                value: None,
            };

            let right = parse_mul_div(tokens, position);
            ast.children.push(right);
            ast
        }
        _ => left,
    }
}

fn parse_add_sub(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let left = parse_mul_div(&tokens, position);
    if *position == tokens.len() {
        return left;
    }

    match tokens[*position].token {
        Token::Add | Token::Subtract => {
            let op = consume_one_of(vec![Token::Add, Token::Subtract], tokens, position);
            let mut ast = ast::AstNode {
                node_type: match op.token {
                    Token::Add => ast::NodeType::Add,
                    Token::Subtract => ast::NodeType::Subtract,
                    _ => panic!("Expected add or subtract"),
                },
                children: vec![left],
                value: None,
            };

            let right = parse_add_sub(tokens, position);
            ast.children.push(right);
            ast
        }
        _ => left,
    }
}

fn parse_number(token: &LexerToken) -> ast::AstNode {
    ast::AstNode {
        node_type: ast::NodeType::Number,
        value: match token.token {
            types::Token::Number(value) => Some(value.to_string()),
            _ => panic!("Expected number"),
        },
        children: vec![],
    }
}

fn parse_identifier(token: &LexerToken) -> ast::AstNode {
    ast::AstNode {
        node_type: ast::NodeType::Identifier,
        value: match &token.token {
            types::Token::Identifier(name) => Some(name.clone()),
            _ => panic!("Expected identifier"),
        },
        children: vec![],
    }
}

fn parse_factor(token: &LexerToken, position: &mut usize) -> ast::AstNode {
    let node = match token.token {
        types::Token::Number(_) => parse_number(&token),
        types::Token::Identifier(_) => parse_identifier(&token),
        _ => panic!("Expected number or identifier"),
    };

    *position += 1;
    return node;
}

fn consume_token(
    expected_token: Token,
    tokens: &Vec<LexerToken>,
    position: &mut usize,
) -> LexerToken {
    let token = &tokens[*position];
    if token.token != expected_token {
        panic!("Expected token {:?}, got {:?}", expected_token, token.token);
    }

    *position += 1;
    return token.clone();
}

fn consume_one_of(
    expected_tokens: Vec<Token>,
    tokens: &Vec<LexerToken>,
    position: &mut usize,
) -> LexerToken {
    let token = &tokens[*position];
    if !expected_tokens.contains(&token.token) {
        panic!(
            "Expected one of {:?}, got {:?}",
            expected_tokens, token.token
        );
    }

    *position += 1;
    return token.clone();
}
