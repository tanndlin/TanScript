import {
    AST,
    AssignASTNode,
    BlockASTNode,
    DeclarationASTNode,
    IdentifierASTNode,
    LParenASTNode,
    RParenASTNode,
    StringASTNode,
} from '../AST/AST';
import {
    GreaterEqASTNode,
    GreaterThanASTNode,
    LessEqASTNode,
    LessThanASTNode,
} from '../AST/BoolAST';
import {
    ForASTNode,
    FunctionCallASTNode,
    FunctionDefASTNode,
    IfASTNode,
    WhileASTNode,
} from '../AST/ControlAST';
import {
    AddASTNode,
    DivideASTNode,
    MultiplyASTNode,
    NumberASTNode,
    SubtractASTNode,
} from '../AST/NumberAST';
import Parser from '../Parser';
import { ParserError } from '../errors';
import { LexerToken, Token } from '../types';

describe('Parser Math Operators', () => {
    it('should parse a simple expression', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.getChildren();
        expect(children).toHaveLength(1);
        expect(children[0]).toBeInstanceOf(AddASTNode);

        // Left then right
        const [left, right] = children[0].getChildren();
        expect(left.getType()).toBe(Token.NUMBER);
        expect(left.getValue()).toBe('1');
        expect(right.getType()).toBe(Token.NUMBER);
        expect(right.getValue()).toBe('2');
    });

    it('should respect PEMDAS', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.MULTIPLY, '*'),
            new LexerToken(Token.NUMBER, '3'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.getChildren();
        expect(children).toHaveLength(1);
        expect(children[0]).toBeInstanceOf(AddASTNode);

        // Left then right
        const [left, right] = children[0].getChildren();
        expect(left.getType()).toBe(Token.NUMBER);
        expect(left.getValue()).toBe('1');
        expect(right.getType()).toBe(Token.MULTIPLY);
        expect(right.getValue()).toBe('*');

        // Left then right
        const [leftRight, rightRight] = right.getChildren();
        expect(leftRight.getType()).toBe(Token.NUMBER);
        expect(leftRight.getValue()).toBe('2');
        expect(rightRight.getType()).toBe(Token.NUMBER);
        expect(rightRight.getValue()).toBe('3');
    });

    it('Parentheses should be respected', () => {
        const tokens = [
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.MULTIPLY, '*'),
            new LexerToken(Token.NUMBER, '3'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.getChildren();
        expect(children).toHaveLength(1);
        expect(children[0]).toBeInstanceOf(MultiplyASTNode);

        // Left should be an the parentheses
        const [left, right] = children[0].getChildren();
        expect(left).toBeInstanceOf(LParenASTNode);

        const [expression, rightParen] = left.getChildren();
        expect(expression).toBeInstanceOf(AddASTNode);
        expect(rightParen).toBeInstanceOf(RParenASTNode);

        // Right should be a number (3)
        expect(right.getType()).toBe(Token.NUMBER);
        expect(right.getValue()).toBe('3');
    });

    it('should parse a basic conditional', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.getChildren();
        expect(children).toHaveLength(1);
        expect(children[0]).toBeInstanceOf(LessThanASTNode);

        // Left then right
        const [left, right] = children[0].getChildren();
        expect(left.getType()).toBe(Token.NUMBER);
        expect(left.getValue()).toBe('1');
        expect(right.getType()).toBe(Token.NUMBER);
        expect(right.getValue()).toBe('2');
    });
});

describe.each([
    [Token.LEQ, LessEqASTNode],
    [Token.GEQ, GreaterEqASTNode],
    [Token.LESS, LessThanASTNode],
    [Token.GREATER, GreaterThanASTNode],
])('should parse boolean operator %s', (tokenType, expectedNode) => {
    it('should parse correctly', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(tokenType, '<='),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.getChildren();
        expect(children).toHaveLength(1);
        expect(children[0]).toBeInstanceOf(expectedNode);

        // Left then right
        const [left, right] = children[0].getChildren();
        expect(left.getType()).toBe(Token.NUMBER);
        expect(left.getValue()).toBe('1');
        expect(right.getType()).toBe(Token.NUMBER);
        expect(right.getValue()).toBe('2');
    });
});

describe('Parser Assignment', () => {
    it('should parse a simple assignment', () => {
        const tokens = [
            new LexerToken(Token.DECLERATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.getChildren();
        expect(children).toHaveLength(1);

        const [declAST] = children;
        expect(declAST).toBeInstanceOf(DeclarationASTNode);

        const [assignAST] = children[0].getChildren();
        expect(assignAST).toBeInstanceOf(AssignASTNode);

        const [identifier, value] = assignAST.getChildren();
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');

        expect(value).toBeInstanceOf(NumberASTNode);
        expect(value.getValue()).toBe('1');
    });

    it('should parse a simple assignment with an expression', () => {
        const tokens = [
            new LexerToken(Token.DECLERATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.getChildren();
        expect(children).toHaveLength(1);

        const [declAST] = children;
        expect(declAST).toBeInstanceOf(DeclarationASTNode);

        const [assignAST] = children[0].getChildren();
        expect(assignAST).toBeInstanceOf(AssignASTNode);

        const [identifier, expression] = assignAST.getChildren();
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');

        expect(expression).toBeInstanceOf(AddASTNode);
    });

    it('should allow shorthand add assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.getChildren();
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AssignASTNode);

        const [identifier, expression] = assignAST.getChildren();
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');

        expect(expression).toBeInstanceOf(AddASTNode);
    });

    it('should allow shorthand minus assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.MINUS, '-'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.getChildren();
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AssignASTNode);

        const [_, expression] = assignAST.getChildren();
        expect(expression).toBeInstanceOf(SubtractASTNode);
    });

    it('should allow shorthand multiply assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.MULTIPLY, '*'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.getChildren();
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AssignASTNode);

        const [_, expression] = assignAST.getChildren();
        expect(expression).toBeInstanceOf(MultiplyASTNode);
    });

    it('should allow shorthand divide assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.DIVIDE, '/'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.getChildren();
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AssignASTNode);

        const [_, expression] = assignAST.getChildren();
        expect(expression).toBeInstanceOf(DivideASTNode);
    });

    it('should allow using var without assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.getChildren();
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AddASTNode);
    });

    it('should not allow shorthand assignment during declaration', () => {
        const tokens = [
            new LexerToken(Token.DECLERATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.EOF, ''),
        ];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });
});

describe('Parser Error Cases', () => {
    it('should throw error for unexpected token', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.PLUS, '+'),
        ];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });

    it('should throw error for unexpected token in parseExpressionOrNumber', () => {
        const tokens = [new LexerToken(Token.PLUS, '+')];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });

    it('should throw error for missing right paren', () => {
        const tokens = [
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.EOF, ''),
        ];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });

    it('should throw error for unexpected EOF in parseAssignment', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.EOF, ''),
        ];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });
});

describe('Parser Curly Braces', () => {
    it('should parse a simple block', () => {
        const tokens = [
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [blockAST] = root.getChildren();
        expect(blockAST).toBeInstanceOf(BlockASTNode);

        const blockChildren = blockAST.getChildren();
        expect(blockChildren).toHaveLength(1);

        const [numberAST] = blockChildren;
        expect(numberAST).toBeInstanceOf(NumberASTNode);
    });

    it('should parse a block with multiple children', () => {
        const tokens = [
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [blockAST] = root.getChildren();
        expect(blockAST).toBeInstanceOf(BlockASTNode);

        const blockChildren = blockAST.getChildren();
        expect(blockChildren).toHaveLength(2);

        const [numberAST1, numberAST2] = blockChildren;
        expect(numberAST1).toBeInstanceOf(NumberASTNode);
        expect(numberAST2).toBeInstanceOf(NumberASTNode);
    });
});

describe('Control Structures', () => {
    it('should parse a while loop', () => {
        const tokens = [
            new LexerToken(Token.WHILE, 'while'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [whileAST] = root.getChildren();
        expect(whileAST).toBeInstanceOf(WhileASTNode);

        const [condition, body] = whileAST.getChildren();
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(body).toBeInstanceOf(BlockASTNode);
    });

    it('should parse a for loop', () => {
        const tokens = [
            new LexerToken(Token.FOR, 'for'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '0'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [forAST] = root.getChildren();
        expect(forAST).toBeInstanceOf(ForASTNode);

        const [init, condition, update, body] = forAST.getChildren();
        expect(init).toBeInstanceOf(AssignASTNode);
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(update).toBeInstanceOf(AssignASTNode);
        expect(body).toBeInstanceOf(BlockASTNode);
    });

    it('should parser a for loop with a declaration', () => {
        const tokens = [
            new LexerToken(Token.FOR, 'for'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.DECLERATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '0'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [forAST] = root.getChildren();
        expect(forAST).toBeInstanceOf(ForASTNode);

        const [init, condition, update, body] = forAST.getChildren();
        expect(init).toBeInstanceOf(DeclarationASTNode);
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(update).toBeInstanceOf(AssignASTNode);
        expect(body).toBeInstanceOf(BlockASTNode);
    });

    it('should parse a simple if statement', () => {
        const tokens = [
            new LexerToken(Token.IF, 'if'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [ifAST] = root.getChildren();
        expect(ifAST).toBeInstanceOf(IfASTNode);

        const [condition, body, elseBlock] = ifAST.getChildren();
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(body).toBeInstanceOf(BlockASTNode);
        expect(elseBlock).toBeUndefined();
    });

    it('should parse an if statement with an else block', () => {
        const tokens = [
            new LexerToken(Token.IF, 'if'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.ELSE, 'else'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [ifAST] = root.getChildren();
        expect(ifAST).toBeInstanceOf(IfASTNode);

        const [condition, body, elseBlock] = ifAST.getChildren();
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(body).toBeInstanceOf(BlockASTNode);
        expect(elseBlock).toBeInstanceOf(BlockASTNode);
    });

    it('should parse a function with no params', () => {
        const tokens = [
            new LexerToken(Token.FUNCTION, 'def'),
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionAST] = root.getChildren();
        expect(functionAST).toBeInstanceOf(FunctionDefASTNode);

        const [block] = functionAST.getChildren();
        expect(block).toBeInstanceOf(BlockASTNode);
    });

    it('should parse a function with params', () => {
        const tokens = [
            new LexerToken(Token.FUNCTION, 'def'),
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.COMMA, ','),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionDef] = root.getChildren();
        expect(functionDef).toBeInstanceOf(FunctionDefASTNode);

        const [block] = functionDef.getChildren();
        expect(block).toBeInstanceOf(BlockASTNode);

        const params = (functionDef as FunctionDefASTNode).getParamList();
        expect(params).toHaveLength(2);

        const [param] = params;
        expect(param).toBeInstanceOf(IdentifierASTNode);
    });

    it('should parse function calls', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.COMMA, ','),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionCall] = root.getChildren();
        expect(functionCall).toBeInstanceOf(FunctionCallASTNode);
        expect(functionCall.getChildren()).toHaveLength(2);

        const [arg1, arg2] = functionCall.getChildren();
        expect(arg1).toBeInstanceOf(NumberASTNode);
        expect(arg2).toBeInstanceOf(NumberASTNode);
    });

    it('should parse function calls with expressions', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionCall] = root.getChildren();
        expect(functionCall).toBeInstanceOf(FunctionCallASTNode);
        expect(functionCall.getChildren()).toHaveLength(1);

        const [expression] = functionCall.getChildren();
        expect(expression).toBeInstanceOf(AddASTNode);
    });

    it('should parse function calls with identifiers', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionCall] = root.getChildren();
        expect(functionCall).toBeInstanceOf(FunctionCallASTNode);
        expect(functionCall.getChildren()).toHaveLength(1);

        const [identifier] = functionCall.getChildren();
        expect(identifier).toBeInstanceOf(IdentifierASTNode);
    });

    it('should parse function calls with no args', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionCall] = root.getChildren();
        expect(functionCall).toBeInstanceOf(FunctionCallASTNode);
        expect(functionCall.getChildren()).toHaveLength(0);
    });

    it('should be able to add function calls', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.COMMA, ','),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.COMMA, ','),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [addAST] = root.getChildren();
        expect(addAST).toBeInstanceOf(AddASTNode);

        const [functionCall1, functionCall2] = addAST.getChildren();
        expect(functionCall1).toBeInstanceOf(FunctionCallASTNode);
        expect(functionCall2).toBeInstanceOf(FunctionCallASTNode);
    });
});

describe('String Parsing', () => {
    it('should parse a simgple string', () => {
        const tokens = [
            new LexerToken(Token.DECLERATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.STRING, 'Hello world!'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [declAST] = root.getChildren();
        expect(declAST).toBeInstanceOf(DeclarationASTNode);

        const [assignAST] = declAST.getChildren();
        expect(assignAST).toBeInstanceOf(AssignASTNode);

        const [identifier, value] = assignAST.getChildren();
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');

        expect(value).toBeInstanceOf(StringASTNode);
        expect(value.getValue()).toBe('Hello world!');
    });

    it('should parse string concatenation', () => {
        const tokens = [
            new LexerToken(Token.STRING, 'hello'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.STRING, 'world'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [addAST] = root.getChildren();
        expect(addAST).toBeInstanceOf(AddASTNode);

        const [string1, string2] = addAST.getChildren();
        expect(string1).toBeInstanceOf(StringASTNode);
        expect(string2).toBeInstanceOf(StringASTNode);
    });
});
