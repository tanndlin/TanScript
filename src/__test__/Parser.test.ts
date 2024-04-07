import {
    AST,
    AddASTNode,
    AssignASTNode,
    BlockASTNode,
    DeclarationASTNode,
    DivideASTNode,
    LParenASTNode,
    MultiplyASTNode,
    NumberASTNode,
    RParenASTNode,
    SubtractASTNode,
} from '../AST';
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
});

// describe('While Loop', () => {
