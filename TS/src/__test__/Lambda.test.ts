import { BlockASTNode, DeclarationASTNode } from '../AST/AST';
import { FunctionDefASTNode } from '../AST/ControlAST';
import { AddASTNode } from '../AST/NumberAST';
import Environment from '../Environment';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { LexerToken, Token } from '../types';

describe('Lambda Tests', () => {
    it('should lex a lambda properly', () => {
        const script = 'let f = () => {1 + 1;}';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();

        expect(tokens).toStrictEqual([
            new LexerToken(Token.DECLERATION, 'let', 1),
            new LexerToken(Token.IDENTIFIER, 'f', 1),
            new LexerToken(Token.ASSIGN, '=', 1),
            new LexerToken(Token.LPAREN, '(', 1),
            new LexerToken(Token.RPAREN, ')', 1),
            new LexerToken(Token.LAMBDA, '=>', 1),
            new LexerToken(Token.LCURLY, '{', 1),
            new LexerToken(Token.NUMBER, '1', 1),
            new LexerToken(Token.PLUS, '+', 1),
            new LexerToken(Token.NUMBER, '1', 1),
            new LexerToken(Token.SEMI, ';', 1),
            new LexerToken(Token.RCURLY, '}', 1),
            new LexerToken(Token.EOF, '', 1),
        ]);
    });

    it('should parse a lambda with no params', () => {
        const script = 'let f = () => {1 + 1;}';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();

        const root = ast.getRoot();
        const [decl] = root.getChildren();

        expect(decl).toBeInstanceOf(DeclarationASTNode);
        if (!(decl instanceof DeclarationASTNode)) {
            throw new Error('Expected declaration');
        }

        const { child: assign } = decl;
        const [ident, lambda] = assign.getChildren();

        expect(lambda).toBeInstanceOf(FunctionDefASTNode);
        if (!(lambda instanceof FunctionDefASTNode)) {
            throw new Error('Expected lambda');
        }

        expect(lambda.getParamList()).toHaveLength(0);

        const { block } = lambda;
        expect(block).toBeInstanceOf(BlockASTNode);

        const [add] = block.getChildren();
        expect(add).toBeInstanceOf(AddASTNode);
    });

    it('should parse a lambda with params', () => {
        const script = 'let f = (a, b) => {a + b;}';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();

        const root = ast.getRoot();
        const [decl] = root.getChildren();

        expect(decl).toBeInstanceOf(DeclarationASTNode);
        if (!(decl instanceof DeclarationASTNode)) {
            throw new Error('Expected declaration');
        }

        const { child: assign } = decl;
        expect(assign.getChildren().length).toBe(2);
        expect(assign.getChildren()[1]).toBeInstanceOf(FunctionDefASTNode);
        const lambda = assign.getChildren()[1] as FunctionDefASTNode;

        const params = lambda.getParamList();
        expect(params.length).toBe(2);
        expect(params[0].getValue()).toBe('a');
        expect(params[1].getValue()).toBe('b');

        const { block } = lambda;
        expect(block).toBeInstanceOf(BlockASTNode);

        const [add] = block.getChildren();
        expect(add).toBeInstanceOf(AddASTNode);
    });

    it('should run a lambda successfully', () => {
        const script = 'let f = (a, b) => {a + b;}; f(1, 2);';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();
        const env = new Environment(ast, true);
        const result = env.evaluate();
        expect(result).toBe(3);

        const scope = env.getGlobalScope();
        const f = scope.getFunction('f');

        expect(f).toBeInstanceOf(FunctionDefASTNode);
    });

    it('lambda should have access to outer scope', () => {
        const script = 'let a = 1; let f = () => {a + 1;}; f();';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();
        const env = new Environment(ast, true);
        const result = env.evaluate();
        expect(result).toBe(2);
    });

    it('lambda can be used as a param', () => {
        const script =
            'let double = (a) => {a * 2;}; let do = (f, a) => {f(a);}; do(double, 5);';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();
        const env = new Environment(ast, true);
        const result = env.evaluate();
        expect(result).toBe(10);
    });
});
