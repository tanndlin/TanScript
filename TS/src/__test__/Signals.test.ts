import {
    AssignASTNode,
    DeclarationASTNode,
    IdentifierASTNode,
} from '../AST/AST';
import { NumberASTNode } from '../AST/NumberAST';
import { SignalAssignmentAST } from '../AST/SignalAST';
import Engine from '../Engine';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { ComputedSignal } from '../Signal';
import { LexerToken, Token } from '../types';

describe('Signals Tests', () => {
    it('should lex signal operators', () => {
        const script = 'let x #= 10;\nlet y $= #x + 2';
        const lexer = new Lexer(script);

        lexer.tokenize();
        const tokens = lexer.getTokens();

        expect(tokens).toStrictEqual([
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

        const [assignAST] = signalAssign.getChildren();
        expect(assignAST).toBeInstanceOf(AssignASTNode);

        const [ident, value] = assignAST.getChildren();
        expect(ident).toBeInstanceOf(IdentifierASTNode);
        expect(value).toBeInstanceOf(NumberASTNode);
    });

    it('should not update computed signals until they are accessed', () => {
        const script = 'let x #= 1;\nlet y $= #x + 2;\n x #= 4;';
        const engine = new Engine(script);

        engine.run();

        const env = engine.getEnvironment();
        const scope = env.getGlobalScope();
        const x = scope.getSignal('x');
        expect(x.getValue()).toBe(4);

        const y = scope.getSignal('y') as ComputedSignal;
        expect(y.value).toBe(3);
        expect(y.isDirty).toBe(true);

        expect(y.getValue()).toBe(6);
        expect(y.isDirty).toBe(false);
    });

    it('should work for chained computed signals', () => {
        const script = 'let x #= 1;\nlet y $= #x + 2;\nlet z $= #y + 3;';
        const engine = new Engine(script);

        engine.run();

        const env = engine.getEnvironment();
        const scope = env.getGlobalScope();
        const x = scope.getSignal('x');
        expect(x.getValue()).toBe(1);

        const y = scope.getSignal('y') as ComputedSignal;
        expect(y.getValue()).toBe(3);

        const z = scope.getSignal('z') as ComputedSignal;
        expect(z.getValue()).toBe(6);
    });
});
