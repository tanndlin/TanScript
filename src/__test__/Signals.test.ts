import { DeclarationASTNode, IdentifierASTNode } from '../AST/AST';
import { NumberASTNode } from '../AST/NumberAST';
import { SignalAssignmentAST } from '../AST/SignalAST';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { LexerToken, Token } from '../types';

describe('Signals Tests', () => {
    it('should lex signal operators', () => {
        const script = 'let x #= 10;\nlet y $= #x + 2';
        const lexer = new Lexer(script);

        lexer.tokenize();
        const tokens = lexer.getTokens();

        expect(tokens).toBe([
            new LexerToken(Token.DECLERATION, 'let', 1),
            new LexerToken(Token.IDENTIFIER, 'x', 1),
            new LexerToken(Token.SIGNAL_ASSIGN, '#=', 1),
            new LexerToken(Token.NUMBER, '10', 1),
            new LexerToken(Token.SEMI, ';', 1),
            new LexerToken(Token.DECLERATION, 'let', 2),
            new LexerToken(Token.IDENTIFIER, 'y', 2),
            new LexerToken(Token.COMPUTE_ASSIGN, '$=', 2),
            new LexerToken(Token.SIGNAL, '#x', 2),
            new LexerToken(Token.PLUS, '+', 2),
            new LexerToken(Token.NUMBER, '2', 2),
            new LexerToken(Token.EOF, '', 2),
        ]);
    });

    it('should parse a signal assignment', () => {
        const script = 'let x #= 1;';
        const lexer = new Lexer(script);
        lexer.tokenize();

        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();

        const root = ast.getRoot();
        const [decl] = root.getChildren();
        expect(decl).toBeInstanceOf(DeclarationASTNode);

        const [signalAssign] = decl.getChildren();
        expect(signalAssign).toBeInstanceOf(SignalAssignmentAST);

        const [ident, value] = signalAssign.getChildren();
        expect(ident).toBeInstanceOf(IdentifierASTNode);
        expect(value).toBeInstanceOf(NumberASTNode);
    });
});
