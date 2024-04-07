export enum Token {
    NUMBER = 'NUMBER',
    PLUS = 'PLUS',
    MINUS = 'MINUS',
    MULTIPLY = 'MULTIPLY',
    DIVIDE = 'DIVIDE',
    LPAREN = 'LPAREN',
    RPAREN = 'RPAREN',
    EOF = 'EOF',
    ROOT = 'ROOT',
    SEMI = 'SEMI',
    DECLERATION = 'DECLERATION',
    IDENTIFIER = 'IDENTIFIER',
    ASSIGN = 'ASSIGN',
    LCURLY = 'LCURLY',
    RCURLY = 'RCURLY',
    WHILE = 'WHILE',
    LESS = 'LESS',
    GREATER = 'GREATER',
}

export const OPERATORS = new Set([
    Token.PLUS,
    Token.MINUS,
    Token.MULTIPLY,
    Token.DIVIDE,
]);

export class LexerToken {
    private type: Token;
    private value: string;

    constructor(type: Token, value: string) {
        this.type = type;
        this.value = value;
    }

    getType(): Token {
        return this.type;
    }

    getValue(): string {
        return this.value;
    }
}

export type Maybe<T> = T | null | undefined;
export type RuntimeValue = Maybe<number | string>;
