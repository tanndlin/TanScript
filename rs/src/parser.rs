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
    consume_token(Token::Semi, tokens, position);
    node
}

fn parse_next(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let token = &tokens[*position];
    match token.token {
        Token::Add
        | Token::Subtract
        | Token::Multiply
        | Token::Divide
        | Token::Mod
        | Token::Identifier(_)
        | Token::Number(_) => parse_expression(tokens, position),
        Token::Declare => parse_declare(tokens, position),
        Token::Assign => parse_assignment(tokens, position),
        Token::LParen => parse_parentheses(tokens, position),
        Token::Function => parse_function(tokens, position),
        Token::LCurly => parse_block(tokens, position),
        Token::Return => parse_return(tokens, position),
        Token::If => parse_if(tokens, position),
        Token::Else => panic!("Unexpected else"),
        Token::RParen => panic!("Unexpected right parenthesis"),
        Token::Semi => panic!("Unexpected semicolon"),
        Token::Comma => panic!("Unexpected comma"),
        Token::RCurly => panic!("Unexpected RCurly"),
    }
}

fn parse_if(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    consume_token(Token::If, tokens, position);
    let condition = parse_expression(tokens, position);
    let block = parse_block(tokens, position);

    let mut children = vec![condition, block];

    if tokens[*position].token == Token::Else {
        consume_token(Token::Else, tokens, position);
        let else_block = parse_block(tokens, position);
        children.push(else_block);
    }

    ast::AstNode {
        node_type: ast::NodeType::If,
        children,
        value: None,
    }
}

fn parse_return(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    consume_token(Token::Return, tokens, position);
    let expression = parse_expression(tokens, position);

    ast::AstNode {
        node_type: ast::NodeType::Return,
        children: vec![expression],
        value: None,
    }
}

fn parse_function(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    consume_token(Token::Function, tokens, position);
    let name = parse_identifier(tokens, position);

    consume_token(Token::LParen, tokens, position);
    let mut args = vec![];
    while tokens[*position].token != Token::RParen {
        // type
        args.push(parse_identifier(tokens, position));

        // name
        args.push(parse_identifier(tokens, position));

        if tokens[*position].token == Token::RParen {
            break;
        }

        consume_token(Token::Comma, tokens, position);
    }

    consume_token(Token::RParen, tokens, position);

    let params = ast::AstNode {
        node_type: ast::NodeType::Parameters,
        children: args,
        value: None,
    };

    let block_ast = parse_block(tokens, position);
    ast::AstNode {
        node_type: ast::NodeType::FunctionDef,
        children: vec![params, block_ast],
        value: name.value,
    }
}

fn parse_block(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    consume_token(Token::LCurly, tokens, position);

    let mut children = vec![];
    while tokens[*position].token != Token::RCurly {
        children.push(parse_statement(tokens, position));
    }

    consume_token(Token::RCurly, tokens, position);

    ast::AstNode {
        node_type: ast::NodeType::Block,
        children,
        value: None,
    }
}

fn parse_parentheses(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    consume_token(Token::LParen, tokens, position);
    let ast = ast::AstNode {
        node_type: ast::NodeType::LParen,
        children: vec![parse_expression(tokens, position)],
        value: None,
    };

    consume_token(Token::RParen, tokens, position);
    ast
}

fn parse_declare(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    consume_token(Token::Declare, tokens, position);
    ast::AstNode {
        node_type: ast::NodeType::Declare,
        children: vec![parse_assignment(tokens, position)],
        value: None,
    }
}

fn parse_assignment(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let ident_ast = parse_identifier(tokens, position);

    consume_token(Token::Assign, tokens, position);
    let expression_ast = parse_expression(tokens, position);

    ast::AstNode {
        node_type: ast::NodeType::Assign,
        children: vec![ident_ast, expression_ast],
        value: None,
    }
}

fn parse_expression(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    parse_add_sub(tokens, position)
}

fn parse_mul_div(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let left = parse_factor(tokens, position);
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

fn parse_number(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let token = consume_token(Token::Number(0), tokens, position);

    ast::AstNode {
        node_type: ast::NodeType::Number,
        value: match token.token {
            types::Token::Number(value) => Some(value.to_string()),
            _ => panic!("Expected number"),
        },
        children: vec![],
    }
}

fn parse_identifier(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let tok = consume_token(Token::Identifier(String::new()), tokens, position);

    ast::AstNode {
        node_type: ast::NodeType::Identifier,
        value: match tok.token {
            types::Token::Identifier(value) => Some(value),
            _ => panic!("Expected identifier"),
        },
        children: vec![],
    }
}

fn parse_factor(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let token = &tokens[*position];

    match token.token {
        types::Token::Number(_) => parse_number(tokens, position),
        types::Token::Identifier(_) => parse_identifier_or_function_call(tokens, position),
        types::Token::LParen => parse_parentheses(tokens, position),
        _ => panic!("Expected number or identifier"),
    }
}

fn parse_identifier_or_function_call(
    tokens: &Vec<LexerToken>,
    position: &mut usize,
) -> ast::AstNode {
    match tokens.get(*position + 1) {
        Some(token) => match token.token {
            Token::LParen => parse_function_call(tokens, position),
            _ => parse_identifier(tokens, position),
        },
        None => parse_identifier(tokens, position),
    }
}

fn parse_function_call(tokens: &Vec<LexerToken>, position: &mut usize) -> ast::AstNode {
    let ident_ast = parse_identifier(tokens, position);
    consume_token(Token::LParen, tokens, position);

    let mut args = vec![];
    while tokens[*position].token != Token::RParen {
        args.push(parse_expression(tokens, position));
        if tokens[*position].token == Token::RParen {
            break;
        }
        consume_token(Token::Comma, tokens, position);
    }

    consume_token(Token::RParen, tokens, position);

    ast::AstNode {
        node_type: ast::NodeType::FunctionCall,
        children: args,
        value: ident_ast.value,
    }
}

fn consume_token(
    expected_token: Token,
    tokens: &Vec<LexerToken>,
    position: &mut usize,
) -> LexerToken {
    if tokens.len() <= *position {
        panic!("Expected token {:?}, got end of input", expected_token);
    }

    let token = &tokens[*position];
    if !types::variant_eq(&expected_token, &token.token) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn parser_parse_addition() {
        let script = "1 + 2;";
        let tokens = tokenize(script);
        let ast = parse(tokens);

        let add = &ast.children[0];
        assert_eq!(add.node_type, ast::NodeType::Add);
        assert_eq!(add.children.len(), 2);

        assert_eq!(add.children[0].node_type, ast::NodeType::Number);
        assert_eq!(add.children[0].value, Some("1".to_string()));

        assert_eq!(add.children[1].node_type, ast::NodeType::Number);
        assert_eq!(add.children[1].value, Some("2".to_string()));
    }

    #[test]
    fn parser_parse_assignment() {
        let script = "a = 1;";
        let tokens = tokenize(script);
        let assign_ast = parse_assignment(&tokens, &mut 0);

        assert_eq!(assign_ast.node_type, ast::NodeType::Assign);
        assert_eq!(assign_ast.children.len(), 2);

        assert_eq!(assign_ast.children[0].node_type, ast::NodeType::Identifier);
        assert_eq!(assign_ast.children[0].value, Some("a".to_string()));

        assert_eq!(assign_ast.children[1].node_type, ast::NodeType::Number);
        assert_eq!(assign_ast.children[1].value, Some("1".to_string()));
    }
}
