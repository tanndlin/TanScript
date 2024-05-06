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
    let left = parse_factor(&tokens[*position]);
    *position += 1;

    // If the next token is not an operator, return the left node
    if *position == tokens.len() {
        return left;
    }

    match tokens[*position].token {
        Token::Add | Token::Subtract | Token::Multiply | Token::Divide => {
            let mut operator = parse_operator(&tokens[*position]);
            operator.children.push(left);

            *position += 1;
            let right = parse_expression(tokens, position);
            operator.children.push(right);

            operator
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

fn parse_factor(token: &LexerToken) -> ast::AstNode {
    match token.token {
        types::Token::Number(_) => parse_number(&token),
        types::Token::Identifier(_) => parse_identifier(&token),
        _ => panic!("Expected number or identifier"),
    }
}

fn parse_operator(token: &LexerToken) -> ast::AstNode {
    let op_type = match token.token {
        types::Token::Add => ast::NodeType::Add,
        types::Token::Subtract => ast::NodeType::Subtract,
        types::Token::Multiply => ast::NodeType::Multiply,
        types::Token::Divide => ast::NodeType::Divide,
        _ => panic!("Expected operator"),
    };

    ast::AstNode {
        node_type: op_type,
        children: vec![],
        value: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let tokens = vec![LexerToken {
            token: Token::Number(42),
            position: 0,
            line: 0,
        }];

        let ast = parse(tokens);
        assert_eq!(ast.node_type, ast::NodeType::Block);
        assert_eq!(ast.children.len(), 1);

        let number = &ast.children[0];
        assert_eq!(number.node_type, ast::NodeType::Number);
        assert_eq!(number.value, Some("42".to_string()));
    }

    #[test]
    fn test_parse_identifier() {
        let tokens = vec![LexerToken {
            token: Token::Identifier("foo".to_string()),
            position: 0,
            line: 0,
        }];

        let ast = parse(tokens);
        assert_eq!(ast.node_type, ast::NodeType::Block);
        assert_eq!(ast.children.len(), 1);

        let identifier = &ast.children[0];
        assert_eq!(identifier.node_type, ast::NodeType::Identifier);
        assert_eq!(identifier.value, Some("foo".to_string()));
    }

    #[test]
    fn test_parse_addition() {
        let tokens = vec![
            LexerToken {
                token: Token::Number(42),
                position: 0,
                line: 0,
            },
            LexerToken {
                token: Token::Add,
                position: 0,
                line: 0,
            },
            LexerToken {
                token: Token::Number(42),
                position: 0,
                line: 0,
            },
        ];

        let ast = parse(tokens);
        assert_eq!(ast.node_type, ast::NodeType::Block);
        assert_eq!(ast.children.len(), 1);

        let add = &ast.children[0];
        assert_eq!(add.node_type, ast::NodeType::Add);
        assert_eq!(add.children.len(), 2);

        let left = &add.children[0];
        assert_eq!(left.node_type, ast::NodeType::Number);
        assert_eq!(left.value, Some("42".to_string()));

        let right = &add.children[1];
        assert_eq!(right.node_type, ast::NodeType::Number);
        assert_eq!(right.value, Some("42".to_string()));
    }
}
