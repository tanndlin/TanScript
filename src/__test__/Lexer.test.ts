import Lexer from '../Lexer';
import { LexerToken, Token } from '../types';

describe('Lexer Tests', () => {
    it('should tokenize a number', () => {
        const lexer = new Lexer('10');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.EOF, ''),
        ]);
    });

    it('should tokenize reserved key words', () => {
        const lexer = new Lexer('let while');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.DECLERATION, 'let'),
            new LexerToken(Token.WHILE, 'while'),
            new LexerToken(Token.EOF, ''),
        ]);
    });

    it('should tokenize an identifier', () => {
        const lexer = new Lexer('x');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.EOF, ''),
        ]);
    });

    it('should tokenize an identifier with a reserved keyword', () => {
        const lexer = new Lexer('letx');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.IDENTIFIER, 'letx'),
            new LexerToken(Token.EOF, ''),
        ]);
    });

    it('should tokenize an identifier with a number', () => {
        const lexer = new Lexer('x10');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.IDENTIFIER, 'x10'),
            new LexerToken(Token.EOF, ''),
        ]);
    });

    it('should tokenize math operators', () => {
        const lexer = new Lexer('+-*/');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.MINUS, '-'),
            new LexerToken(Token.MULTIPLY, '*'),
            new LexerToken(Token.DIVIDE, '/'),
            new LexerToken(Token.EOF, ''),
        ]);
    });

    it('should tokenize leq', () => {
        const lexer = new Lexer('<=');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.LEQ, '<='),
            new LexerToken(Token.EOF, ''),
        ]);
    });

    it('should tokenize geq', () => {
        const lexer = new Lexer('>=');
        const tokens = lexer.getTokens();
        expect(tokens).toEqual([
            new LexerToken(Token.GEQ, '>='),
            new LexerToken(Token.EOF, ''),
        ]);
    });
});
