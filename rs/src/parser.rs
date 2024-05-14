use crate::ast::*;
use crate::lexer::LexerToken;
use crate::types::*;

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

pub fn parse(tokens: Vec<LexerToken>) -> AstNode {
    let mut parser = Parser {
        tokens,
        position: 0,
    };

    let mut root = AstNode {
        node_type: NodeType::Block,
        children: vec![],
        value: None,
    };

    while parser.position < parser.tokens.len() {
        let node = parse_statement(&mut parser);
        root.children.push(node);
    }

    root
}

fn parse_statement(parser: &mut Parser) -> AstNode {
    let node = parse_next(parser);
    consume_token(parser, Token::Semi);
    node
}

fn parse_next(parser: &mut Parser) -> AstNode {
    let token = parser.get_current_token();
    match token.token {
        Token::Operator(_)
        | Token::Not
        | Token::BitwiseOp(BitwiseOp::Not)
        | Token::Number(_)
        | Token::Boolean(_) => parse_expression(parser),
        Token::Identifier(_) => parse_expression_or_assignment(parser),
        Token::Type(_) => parse_declare(parser),
        Token::LParen => parse_parentheses(parser),
        Token::Function => parse_function(parser),
        Token::LCurly => parse_block(parser),
        Token::Return => parse_return(parser),
        Token::If => parse_if(parser),
        Token::While => parse_while(parser),
        Token::Assign => panic!("Unexpected assign"),
        Token::ShortAssign(_) => panic!("Unexpected short assign"),
        Token::Increment => panic!("Unexpected increment"),
        Token::Decrement => panic!("Unexpected decrement"),
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
        Token::And => panic!("Unexpected And"),
        Token::Or => panic!("Unexpected Or"),
        Token::BitwiseOp(_) => panic!("Unexpected BitwiseOp"),
        Token::Colon => panic!("Unexpected colon"),
    }
}

fn parse_expression_or_assignment(parser: &mut Parser) -> AstNode {
    let next_token = parser.get_next(1).unwrap();

    match next_token.token {
        Token::Assign => parse_assignment(parser),
        Token::ShortAssign(_) => parse_short_assign(parser),
        Token::Increment | Token::Decrement => parse_increment_decrement(parser),
        _ => parse_expression(parser),
    }
}

fn parse_short_assign(parser: &mut Parser) -> AstNode {
    let ident_ast = parse_identifier(parser);
    let op = consume_token(parser, Token::ShortAssign(Operator::Add));

    let expression_ast = parse_expression(parser);

    AstNode {
        node_type: match op.token {
            Token::ShortAssign(op) => NodeType::ShortAssign(op),
            _ => panic!("expected short assign, got {:?}", op),
        },
        children: vec![ident_ast, expression_ast],
        value: None,
    }
}

fn parse_increment_decrement(parser: &mut Parser) -> AstNode {
    let ident_ast = parse_identifier(parser);
    let op = consume_one_of(parser, vec![Token::Increment, Token::Decrement]);

    let one_ast = AstNode {
        node_type: NodeType::Number,
        children: vec![],
        value: Some("1".to_string()),
    };

    AstNode {
        node_type: match op.token {
            Token::Increment => NodeType::ShortAssign(Operator::Add),
            Token::Decrement => NodeType::ShortAssign(Operator::Subtract),
            _ => panic!("Unexpected increment or decrement"),
        },
        children: vec![ident_ast, one_ast],
        value: None,
    }
}

fn parse_while(parser: &mut Parser) -> AstNode {
    consume_token(parser, Token::While);
    let condition = parse_expression(parser);
    let block = parse_block(parser);

    AstNode {
        node_type: NodeType::While,
        children: vec![condition, block],
        value: None,
    }
}

fn parse_if(parser: &mut Parser) -> AstNode {
    consume_token(parser, Token::If);
    let condition = parse_expression(parser);
    let block = parse_block(parser);

    let mut children = vec![condition, block];

    if parser.get_current_token().token == Token::Else {
        consume_token(parser, Token::Else);
        let else_block = parse_block(parser);
        children.push(else_block);
    }

    AstNode {
        node_type: NodeType::If,
        children,
        value: None,
    }
}

fn parse_return(parser: &mut Parser) -> AstNode {
    consume_token(parser, Token::Return);
    let expression = parse_expression(parser);

    AstNode {
        node_type: NodeType::Return,
        children: vec![expression],
        value: None,
    }
}

fn parse_function(parser: &mut Parser) -> AstNode {
    consume_token(parser, Token::Function);
    let name = parse_identifier(parser);

    consume_token(parser, Token::LParen);
    let mut args = vec![];
    while parser.get_current_token().token != Token::RParen {
        args.push(parse_parameter(parser));

        if parser.get_current_token().token == Token::RParen {
            break;
        }

        consume_token(parser, Token::Comma);
    }

    consume_token(parser, Token::RParen);

    let params = AstNode {
        node_type: NodeType::Parameters,
        children: args,
        value: None,
    };

    let token = consume_token(parser, Token::Type(DataType::Integer));
    let explicit_type = match token.token.clone() {
        Token::Type(t @ DataType::Integer)
        | Token::Type(t @ DataType::Boolean)
        | Token::Type(t @ DataType::Float) => t,
        _ => panic!("Expected type specifier"),
    };

    let type_ast = AstNode {
        node_type: NodeType::Type(explicit_type),
        children: vec![],
        value: None,
    };

    let block_ast = parse_block(parser);
    AstNode {
        node_type: NodeType::FunctionDef,
        children: vec![params, type_ast, block_ast],
        value: name.value,
    }
}

fn parse_parameter(parser: &mut Parser) -> AstNode {
    let token = consume_token(parser, Token::Type(DataType::Integer));

    let explicit_type = match token.token.clone() {
        Token::Type(t @ DataType::Integer)
        | Token::Type(t @ DataType::Boolean)
        | Token::Type(t @ DataType::Float) => t,
        _ => panic!("Expected type specifier"),
    };

    let type_ast = AstNode {
        node_type: NodeType::Type(explicit_type.clone()),
        children: vec![],
        value: Some(explicit_type.to_string()),
    };

    let ident_ast = parse_identifier(parser);
    AstNode {
        node_type: NodeType::Parameter,
        children: vec![type_ast, ident_ast],
        value: None,
    }
}

fn parse_block(parser: &mut Parser) -> AstNode {
    consume_token(parser, Token::LCurly);

    let mut children = vec![];
    while parser.get_current_token().token != Token::RCurly {
        children.push(parse_statement(parser));
    }

    consume_token(parser, Token::RCurly);

    AstNode {
        node_type: NodeType::Block,
        children,
        value: None,
    }
}

fn parse_parentheses(parser: &mut Parser) -> AstNode {
    consume_token(parser, Token::LParen);
    let ast = AstNode {
        node_type: NodeType::LParen,
        children: vec![parse_expression(parser)],
        value: None,
    };

    consume_token(parser, Token::RParen);
    ast
}

fn parse_declare(parser: &mut Parser) -> AstNode {
    let token = consume_token(parser, Token::Type(DataType::Integer));
    let type_ast = match token.token.clone() {
        Token::Type(t @ DataType::Integer)
        | Token::Type(t @ DataType::Boolean)
        | Token::Type(t @ DataType::Float)
        | Token::Type(t @ DataType::Pointer(_)) => AstNode {
            node_type: NodeType::Type(t.clone()),
            children: vec![],
            value: Some(t.to_string()),
        },
        _ => panic!("Expected type specifier"),
    };

    let next_token = parser.get_next(1).unwrap();
    if next_token.token == Token::Assign {
        return AstNode {
            node_type: NodeType::Declare,
            children: vec![type_ast, parse_assignment(parser)],
            value: None,
        };
    }

    // Allow uninitialized variables
    AstNode {
        node_type: NodeType::Declare,
        children: vec![type_ast, parse_identifier(parser)],
        value: None,
    }
}

fn parse_assignment(parser: &mut Parser) -> AstNode {
    let ident_ast = parse_identifier(parser);

    consume_token(parser, Token::Assign);
    let expression_ast = parse_expression(parser);

    AstNode {
        node_type: NodeType::Assign,
        children: vec![ident_ast, expression_ast],
        value: None,
    }
}

fn parse_expression(parser: &mut Parser) -> AstNode {
    parse_and_or(parser)
}

fn parse_prefix_op(parser: &mut Parser) -> AstNode {
    let token = parser.get_current_token().clone();
    match token.token {
        Token::Not | Token::BitwiseOp(BitwiseOp::Not) => {
            parser.position += 1;
            AstNode {
                node_type: match token.token {
                    Token::Not => NodeType::Not,
                    Token::BitwiseOp(BitwiseOp::Not) => NodeType::BitwiseOp(BitwiseOp::Not),
                    _ => panic!("Expected not or bitwise not"),
                },
                children: vec![parse_prefix_op(parser)],
                value: None,
            }
        }
        _ => parse_factor(parser),
    }
}

fn parse_mul_div(parser: &mut Parser) -> AstNode {
    let left = parse_prefix_op(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token.clone() {
        Token::Operator(op @ Operator::Multiply)
        | Token::Operator(op @ Operator::Divide)
        | Token::Operator(op @ Operator::Mod) => {
            parser.position += 1;
            AstNode {
                node_type: operator_to_node_type(op),
                children: vec![left, parse_mul_div(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_add_sub(parser: &mut Parser) -> AstNode {
    let left = parse_mul_div(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token.clone() {
        Token::Operator(op @ Operator::Add) | Token::Operator(op @ Operator::Subtract) => {
            parser.position += 1;
            AstNode {
                node_type: operator_to_node_type(op),
                children: vec![left, parse_add_sub(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_relational(parser: &mut Parser) -> AstNode {
    let left = parse_add_sub(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token {
        Token::LessThan | Token::GreaterThan | Token::Leq | Token::Geq => {
            parser.position += 1;
            AstNode {
                node_type: match parser.get_current_token().token {
                    Token::LessThan => NodeType::LessThan,
                    Token::GreaterThan => NodeType::GreaterThan,
                    Token::Leq => NodeType::Leq,
                    Token::Geq => NodeType::Geq,
                    _ => panic!("Expected relational operator"),
                },
                children: vec![left, parse_relational(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_equality(parser: &mut Parser) -> AstNode {
    let left = parse_relational(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token {
        Token::Eq | Token::NotEq => {
            parser.position += 1;
            AstNode {
                node_type: match parser.get_current_token().token {
                    Token::Eq => NodeType::Eq,
                    Token::NotEq => NodeType::NotEq,
                    _ => panic!("Expected equality operator"),
                },
                children: vec![left, parse_equality(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_bitwise_op(parser: &mut Parser) -> AstNode {
    let left = parse_equality(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token.clone() {
        Token::BitwiseOp(op @ BitwiseOp::And)
        | Token::BitwiseOp(op @ BitwiseOp::Or)
        | Token::BitwiseOp(op @ BitwiseOp::Xor) => {
            parser.position += 1;
            AstNode {
                node_type: bitwise_op_to_node_type(op),
                children: vec![left, parse_bitwise_op(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_and_or(parser: &mut Parser) -> AstNode {
    let left = parse_bitwise_op(parser);
    if parser.is_end() {
        return left;
    }

    match parser.get_current_token().token {
        Token::And | Token::Or => {
            parser.position += 1;
            AstNode {
                node_type: match parser.get_current_token().token {
                    Token::And => NodeType::And,
                    Token::Or => NodeType::Or,
                    _ => panic!("Expected and or or"),
                },
                children: vec![left, parse_and_or(parser)],
                value: None,
            }
        }
        _ => left,
    }
}

fn parse_number(parser: &mut Parser) -> AstNode {
    let token = consume_token(parser, Token::Number(0));

    AstNode {
        node_type: NodeType::Number,
        value: match token.token {
            Token::Number(value) => Some(value.to_string()),
            _ => panic!("Expected number"),
        },
        children: vec![],
    }
}

fn parse_identifier(parser: &mut Parser) -> AstNode {
    let tok = consume_token(parser, Token::Identifier(String::new()));

    AstNode {
        node_type: NodeType::Identifier,
        value: match tok.token {
            Token::Identifier(value) => Some(value),
            _ => panic!("Expected identifier"),
        },
        children: vec![],
    }
}

fn parse_factor(parser: &mut Parser) -> AstNode {
    let token = &parser.get_current_token();

    match token.token {
        Token::Number(_) => parse_number(parser),
        Token::Identifier(_) => parse_identifier_or_function_call(parser),
        Token::LParen => parse_parentheses(parser),
        Token::Boolean(_) => parse_boolean(parser),
        _ => panic!("Expected number or identifier"),
    }
}

fn parse_boolean(parser: &mut Parser) -> AstNode {
    let token = consume_token(parser, Token::Boolean(false));

    AstNode {
        node_type: NodeType::Boolean,
        value: match token.token {
            Token::Boolean(value) => Some(value.to_string()),
            _ => panic!("Expected boolean"),
        },
        children: vec![],
    }
}

fn parse_identifier_or_function_call(parser: &mut Parser) -> AstNode {
    match parser.get_next(1) {
        Some(token) => match token.token {
            Token::LParen => parse_function_call(parser),
            _ => parse_identifier(parser),
        },
        None => parse_identifier(parser),
    }
}

fn parse_function_call(parser: &mut Parser) -> AstNode {
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

    AstNode {
        node_type: NodeType::FunctionCall,
        children: args,
        value: ident_ast.value,
    }
}

fn consume_token(parser: &mut Parser, expected_token: Token) -> LexerToken {
    if parser.is_end() {
        panic!("Expected token {:?}, got end of input", expected_token);
    }

    let token = parser.get_current_token().clone();
    if !variant_eq(&expected_token, &token.token) {
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
        assert_eq!(add.node_type, NodeType::Operator(Operator::Add));
        assert_eq!(add.children.len(), 2);

        assert_eq!(add.children[0].node_type, NodeType::Number);
        assert_eq!(add.children[0].value, Some("1".to_string()));

        assert_eq!(add.children[1].node_type, NodeType::Number);
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

        assert_eq!(assign_ast.node_type, NodeType::Assign);
        assert_eq!(assign_ast.children.len(), 2);

        assert_eq!(assign_ast.children[0].node_type, NodeType::Identifier);
        assert_eq!(assign_ast.children[0].value, Some("a".to_string()));

        assert_eq!(assign_ast.children[1].node_type, NodeType::Number);
        assert_eq!(assign_ast.children[1].value, Some("1".to_string()));
    }
}
