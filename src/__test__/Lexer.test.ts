import Lexer from '../Lexer';
import { LexerToken, RESERVED_WORDS, Token } from '../types';

describe('Lexer Tests', () => {
    it('should tokenize a number', () => {
        const lexer = new Lexer('10');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.NUMBER, '10', 1),
            new LexerToken(Token.EOF, '', 1),
        ]);
    });

    const objectFlip = (obj) => {
        const ret = {};
        Object.keys(obj).forEach((key) => {
            ret[obj[key]] = key;
        });

        return Object.entries(ret) as string[][];
    };

    it.each(objectFlip(RESERVED_WORDS))(
        'should tokenize reserved word %s',
        (tokenType, word) => {
            const lexer = new Lexer(word);
            const tokens = lexer.getTokens();
            expect(tokens).toEqual([
                new LexerToken(tokenType as Token, word, 1),
                new LexerToken(Token.EOF, '', 1),
            ]);
        }
    );

    it('should tokenize an identifier', () => {
        const lexer = new Lexer('x');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.IDENTIFIER, 'x', 1),
            new LexerToken(Token.EOF, '', 1),
        ]);
    });

    it('should tokenize an identifier with a reserved keyword', () => {
        const lexer = new Lexer('letx');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.IDENTIFIER, 'letx', 1),
            new LexerToken(Token.EOF, '', 1),
        ]);
    });

    it('should tokenize an identifier with a number', () => {
        const lexer = new Lexer('x10');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.IDENTIFIER, 'x10', 1),
            new LexerToken(Token.EOF, '', 1),
        ]);
    });

    it('should tokenize math operators', () => {
        const lexer = new Lexer('+-*/');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.PLUS, '+', 1),
            new LexerToken(Token.MINUS, '-', 1),
            new LexerToken(Token.MULTIPLY, '*', 1),
            new LexerToken(Token.DIVIDE, '/', 1),
            new LexerToken(Token.EOF, '', 1),
        ]);
    });

    it('should tokenize leq', () => {
        const lexer = new Lexer('<=');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.LEQ, '<=', 1),
            new LexerToken(Token.EOF, '', 1),
        ]);
    });

    it('should tokenize geq', () => {
        const lexer = new Lexer('>=');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.GEQ, '>=', 1),
            new LexerToken(Token.EOF, '', 1),
        ]);
    });
});
