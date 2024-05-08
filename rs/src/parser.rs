use crate::ast;
use crate::lexer::LexerToken;
use crate::types;
use crate::types::Token;

struct Parser {
    tokens: Vec<LexerToken>,
    position: usize,
}

impl Parser {
    fn get_current_token(&self) -> &LexerToken {
        self.tokens.get(self.position).unwrap()
    }

    fn get_next(&self, n: usize) -> Option<&LexerToken> {
        self.tokens.get(self.position + n)
    }

    fn is_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}

pub fn parse(tokens: Vec<LexerToken>) -> ast::AstNode {
    let mut parser = Parser {
        tokens,
        position: 0,
    };

    let mut root = ast::AstNode {
        node_type: ast::NodeType::Block,
        children: vec![],
        value: None,
    };

    while parser.position < parser.tokens.len() {
        let node = parse_statement(&mut parser);
        root.children.push(node);
    }

    root
}

fn parse_statement(parser: &mut Parser) -> ast::AstNode {
    let node = parse_next(parser);
    consume_token(parser, Token::Semi);
    node
}

fn parse_next(parser: &mut Parser) -> ast::AstNode {
    let token = parser.get_current_token();
    match token.token {
        Token::Add
        | Token::Subtract
        | Token::Multiply
        | Token::Divide
        | Token::Mod
        | Token::Not
        | Token::Identifier(_)
        | Token::Number(_) => parse_expression(parser),
        Token::Declare => parse_declare(parser),
        Token::Assign => parse_assignment(parser),
        Token::LParen => parse_parentheses(parser),
        Token::Function => parse_function(parser),
        Token::LCurly => parse_block(parser),
        Token::Return => parse_return(parser),
        Token::If => parse_if(parser),
        Token::Else => panic!("Unexpected else"),
        Token::RParen => panic!("Unexpected right parenthesis"),
        Token::Semi => panic!("Unexpected semicolon"),
        Token::Comma => panic!("Unexpected comma"),
        Token::RCurly => panic!("Unexpected RCurly"),
        Token::Eq => panic!("Unexpected Eq"),
        Token::NotEq => panic!("Unexpected NotEq"),
        Token::LessThan => panic!("Unexpected LessThan"),
        Token::GreaterThan => panic!("Unexpected GreaterThan"),
        Token::Leq => panic!("Unexpected Leq"),
        Token::Geq => panic!("Unexpected Geq"),
    }
}

fn parse_if(parser: &mut Parser) -> ast::AstNode {
    consume_token(parser, Token::If);
    let condition = parse_expression(parser);
    let block = parse_block(parser);

    let mut children = vec![condition, block];

    if parser.get_current_token().token == Token::Else {
        consume_token(parser, Token::Else);
        let else_block = parse_block(parser);
        children.push(else_block);
    }

    ast::AstNode {
        node_type: ast::NodeType::If,
        children,
        value: None,
    }
}

fn parse_return(parser: &mut Parser) -> ast::AstNode {
    consume_token(parser, Token::Return);
    let expression = parse_expression(parser);

    ast::AstNode {
        node_type: ast::NodeType::Return,
        children: vec![expression],
        value: None,
    }
}

fn parse_function(parser: &mut Parser) -> ast::AstNode {
    consume_token(parser, Token::Function);
    let name = parse_identifier(parser);

    consume_token(parser, Token::LParen);
    let mut args = vec![];
    while parser.get_current_token().token != Token::RParen {
        // type
        args.push(parse_identifier(parser));

        // name
        args.push(parse_identifier(parser));

        if parser.get_current_token().token == Token::RParen {
            break;
        }

        consume_token(parser, Token::Comma);
    }

    consume_token(parser, Token::RParen);

    let params = ast::AstNode {
        node_type: ast::NodeType::Parameters,
        children: args,
        value: None,
    };

    let block_ast = parse_block(parser);
    ast::AstNode {
        node_type: ast::NodeType::FunctionDef,
        children: vec![params, block_ast],
        value: name.value,
    }
}

fn parse_block(parser: &mut Parser) -> ast::AstNode {
    consume_token(parser, Token::LCurly);

    let mut children = vec![];
    while parser.get_current_token().token != Token::RCurly {
        children.push(parse_statement(parser));
    }

    consume_token(parser, Token::RCurly);

    ast::AstNode {
        node_type: ast::NodeType::Block,
        children,
        value: None,
    }
}

fn parse_parentheses(parser: &mut Parser) -> ast::AstNode {
    consume_token(parser, Token::LParen);
    let ast = ast::AstNode {
        node_type: ast::NodeType::LParen,
        children: vec![parse_expression(parser)],
        value: None,
    };

    consume_token(parser, Token::RParen);
    ast
}

fn parse_declare(parser: &mut Parser) -> ast::AstNode {
    consume_token(parser, Token::Declare);
    ast::AstNode {
        node_type: ast::NodeType::Declare,
        children: vec![parse_assignment(parser)],
        value: None,
    }
}

fn parse_assignment(parser: &mut Parser) -> ast::AstNode {
    let ident_ast = parse_identifier(parser);

    consume_token(parser, Token::Assign);
    let expression_ast = parse_expression(parser);

    ast::AstNode {
        node_type: ast::NodeType::Assign,
        children: vec![ident_ast, expression_ast],
        value: None,
    }
}

fn parse_expression(parser: &mut Parser) -> ast::AstNode {
    parse_equality(parser)
}

fn parse_mul_div(parser: &mut Parser) -> ast::AstNode {
    let left = parse_factor(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token {
        Token::Multiply | Token::Divide | Token::Mod => {
            let op = consume_one_of(parser, vec![Token::Multiply, Token::Divide, Token::Mod]);
            ast::AstNode {
                node_type: match op.token {
                    Token::Multiply => ast::NodeType::Multiply,
                    Token::Divide => ast::NodeType::Divide,
                    Token::Mod => ast::NodeType::Mod,
                    _ => panic!("Expected multiply or divide"),
                },
                children: vec![left, parse_mul_div(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_add_sub(parser: &mut Parser) -> ast::AstNode {
    let left = parse_mul_div(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token {
        Token::Add | Token::Subtract => {
            let op = consume_one_of(parser, vec![Token::Add, Token::Subtract]);
            ast::AstNode {
                node_type: match op.token {
                    Token::Add => ast::NodeType::Add,
                    Token::Subtract => ast::NodeType::Subtract,
                    _ => panic!("Expected add or subtract"),
                },
                children: vec![left, parse_add_sub(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_relational(parser: &mut Parser) -> ast::AstNode {
    let left = parse_add_sub(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token {
        Token::LessThan | Token::GreaterThan | Token::Leq | Token::Geq => {
            let op = consume_one_of(
                parser,
                vec![Token::LessThan, Token::GreaterThan, Token::Leq, Token::Geq],
            );
            ast::AstNode {
                node_type: match op.token {
                    Token::LessThan => ast::NodeType::LessThan,
                    Token::GreaterThan => ast::NodeType::GreaterThan,
                    Token::Leq => ast::NodeType::Leq,
                    Token::Geq => ast::NodeType::Geq,
                    _ => panic!("Expected relational operator"),
                },
                children: vec![left, parse_relational(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_equality(parser: &mut Parser) -> ast::AstNode {
    let left = parse_relational(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token {
        Token::Eq | Token::NotEq => {
            let op = consume_one_of(parser, vec![Token::Eq, Token::NotEq]);
            let mut ast = ast::AstNode {
                node_type: match op.token {
                    Token::Eq => ast::NodeType::Eq,
                    Token::NotEq => ast::NodeType::NotEq,
                    _ => panic!("Expected equality operator"),
                },
                children: vec![left],
                value: None,
            };

            let right = parse_equality(parser);
            ast.children.push(right);
            ast
        }
        _ => left,
    }
}

fn parse_number(parser: &mut Parser) -> ast::AstNode {
    let token = consume_token(parser, Token::Number(0));

    ast::AstNode {
        node_type: ast::NodeType::Number,
        value: match token.token {
            types::Token::Number(value) => Some(value.to_string()),
            _ => panic!("Expected number"),
        },
        children: vec![],
    }
}

fn parse_identifier(parser: &mut Parser) -> ast::AstNode {
    let tok = consume_token(parser, Token::Identifier(String::new()));

    ast::AstNode {
        node_type: ast::NodeType::Identifier,
        value: match tok.token {
            types::Token::Identifier(value) => Some(value),
            _ => panic!("Expected identifier"),
        },
        children: vec![],
    }
}

fn parse_factor(parser: &mut Parser) -> ast::AstNode {
    let token = &parser.get_current_token();

    match token.token {
        types::Token::Number(_) => parse_number(parser),
        types::Token::Identifier(_) => parse_identifier_or_function_call(parser),
        types::Token::LParen => parse_parentheses(parser),
        _ => panic!("Expected number or identifier"),
    }
}

fn parse_identifier_or_function_call(parser: &mut Parser) -> ast::AstNode {
    match parser.get_next(1) {
        Some(token) => match token.token {
            Token::LParen => parse_function_call(parser),
            _ => parse_identifier(parser),
        },
        None => parse_identifier(parser),
    }
}

fn parse_function_call(parser: &mut Parser) -> ast::AstNode {
    let ident_ast = parse_identifier(parser);
    consume_token(parser, Token::LParen);

    let mut args = vec![];
    while parser.get_current_token().token != Token::RParen {
        args.push(parse_expression(parser));
        if parser.get_current_token().token == Token::RParen {
            break;
        }
        consume_token(parser, Token::Comma);
    }

    consume_token(parser, Token::RParen);

    ast::AstNode {
        node_type: ast::NodeType::FunctionCall,
        children: args,
        value: ident_ast.value,
    }
}

fn consume_token(parser: &mut Parser, expected_token: Token) -> LexerToken {
    if parser.is_end() {
        panic!("Expected token {:?}, got end of input", expected_token);
    }

    let token = parser.get_current_token().clone();
    if !types::variant_eq(&expected_token, &token.token) {
        panic!("Expected token {:?}, got {:?}", expected_token, token.token);
    }

    parser.position += 1;
    return token.clone();
}

fn consume_one_of(parser: &mut Parser, expected_tokens: Vec<Token>) -> LexerToken {
    let token = parser.get_current_token().clone();
    if !expected_tokens.contains(&token.token) {
        panic!(
            "Expected one of {:?}, got {:?}",
            expected_tokens, token.token
        );
    }

    parser.position += 1;
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
        let assign_ast = parse_assignment(&mut Parser {
            tokens,
            position: 0,
        });

        assert_eq!(assign_ast.node_type, ast::NodeType::Assign);
        assert_eq!(assign_ast.children.len(), 2);

        assert_eq!(assign_ast.children[0].node_type, ast::NodeType::Identifier);
        assert_eq!(assign_ast.children[0].value, Some("a".to_string()));

        assert_eq!(assign_ast.children[1].node_type, ast::NodeType::Number);
        assert_eq!(assign_ast.children[1].value, Some("1".to_string()));
    }
}
