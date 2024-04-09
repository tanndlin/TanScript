export enum Token {
    FOR = 'FOR',
    WHILE = 'WHILE',
    IF = 'IF',
    ELSE = 'ELSE',

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
    LESS = 'LESS',
    LEQ = 'LEQ',
    GREATER = 'GREATER',
    GEQ = 'GEQ',
    FUNCTION = 'FUNCTION',
    COMMA = 'COMMA',
}

export const OPERATORS = new Set([
    Token.PLUS,
    Token.MINUS,
    Token.MULTIPLY,
    Token.DIVIDE,
    Token.LESS,
    Token.LEQ,
    Token.GREATER,
    Token.GEQ,
]);

export const RESERVED_WORDS = {
    let: Token.DECLERATION,
    while: Token.WHILE,
    for: Token.FOR,
    if: Token.IF,
    else: Token.ELSE,
    def: Token.FUNCTION,
};

export type ReservedWordsKey = keyof typeof RESERVED_WORDS;

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
export type RuntimeValue = Maybe<number | string | boolean | void>;
