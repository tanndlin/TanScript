import { BlockASTNode, DeclarationASTNode } from '../AST/AST';
import { FunctionDefASTNode } from '../AST/ControlAST';
import { AddASTNode } from '../AST/NumberAST';
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

        const [assign] = decl.getChildren();
        const [ident, lambda] = assign.getChildren();

        expect(lambda).toBeInstanceOf(FunctionDefASTNode);
        expect(lambda.getChildren().length).toBe(1);

        const [block] = lambda.getChildren();
        expect(block).toBeInstanceOf(BlockASTNode);

        const [add] = block.getChildren();
        expect(add).toBeInstanceOf(AddASTNode);
    });
});
