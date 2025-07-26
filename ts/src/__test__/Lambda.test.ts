import * as AST from '../AST';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { LexerToken, Token } from '../types';
import { getTokens } from './Lexer.test';

describe('Lambda Tests', () => {
    it('should lex a lambda properly', () => {
        const script = 'let f = () => {1 + 1;}';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);

        expect(tokens).toStrictEqual([
            new LexerToken(Token.DECLARATION, 'let', 1),
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
        const parser = new Parser(getTokens(lexer));
        const ast = parser.parse();

        const root = ast.getRoot();
        const [decl] = root.children;

        expect(decl).toBeInstanceOf(AST.ASTDeclaration);
        if (!(decl.type === Token.DECLARATION)) {
            throw new Error('Expected declaration');
        }

        const { child: assign } = decl;
        if (!(assign.type === Token.ASSIGN)) {
            throw new Error('Expected assignment');
        }

        const { valueAST: lambda } = assign;

        expect(lambda).toBeInstanceOf(AST.ASTFunctionDef);
        if (!(lambda instanceof AST.ASTFunctionDef)) {
            throw new Error('Expected lambda');
        }

        expect(lambda.getParamList()).toHaveLength(0);

        const { block } = lambda;
        expect(block).toBeInstanceOf(AST.ASTBlock);

        const [add] = block.children;
        expect(add).toBeInstanceOf(AST.ASTAdd);
    });

    it('should parse a lambda with params', () => {
        const script = 'let f = (a, b) => {a + b;}';
        const lexer = new Lexer(script);
        const parser = new Parser(getTokens(lexer));
        const ast = parser.parse();

        const root = ast.getRoot();
        const [decl] = root.children;

        expect(decl).toBeInstanceOf(AST.ASTDeclaration);
        if (!(decl.type === Token.DECLARATION)) {
            throw new Error('Expected declaration');
        }

        const { child: assign } = decl;
        expect(assign).toBeInstanceOf(AST.ASTAssign);
        if (!(assign.type === Token.ASSIGN)) {
            return;
        }
        expect(assign.valueAST).toBeInstanceOf(AST.ASTFunctionDef);
        const lambda = assign.valueAST as AST.ASTFunctionDef;

        const params = lambda.getParamList();
        expect(params.length).toBe(2);
        expect(params[0].getName()).toBe('a');
        expect(params[1].getName()).toBe('b');

        const { block } = lambda;
        expect(block).toBeInstanceOf(AST.ASTBlock);

        const [add] = block.children;
        expect(add).toBeInstanceOf(AST.ASTAdd);
    });
});
