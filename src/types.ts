import { IdentifierASTNode } from './AST/AST';
import { IterableASTNode } from './AST/IterableAST';

export enum Token {
    FOR = 'for',
    FOREACH = 'foreach',
    WHILE = 'while',
    IF = 'if',
    ELSE = 'else',
    FUNCTION = 'def',
    DECLERATION = 'let',
    TRUE = 'true',
    FALSE = 'false',
    IN = 'in',

    NUMBER = 'number',
    STRING = 'string',
    PLUS = '+',
    MINUS = '-',
    MULTIPLY = '*',
    DIVIDE = '/',
    LPAREN = '(',
    RPAREN = ')',
    EOF = 'EOF',
    ROOT = 'ROOT',
    SEMI = ';',
    IDENTIFIER = 'IDENTIFIER',
    ASSIGN = '=',
    LCURLY = '{',
    RCURLY = '}',
    LESS = '<',
    LEQ = '<=',
    GREATER = '>',
    GEQ = '>=',
    EQUAL = '==',
    LAMBDA = '=>',
    NOT = '!',
    NEQ = '!=',
    AND = '&&',
    OR = '||',
    COMMA = ',',
    COLON = ':',
    SIGNAL = '#',
    COMPUTE = '$',
    COMPUTE_ASSIGN = '$=',
    SIGNAL_ASSIGN = 'SIGNAL_ASSIGN',
    LBRACKET = '[',
    RBRACKET = ']',
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
    Token.EQUAL,
    Token.NEQ,
    Token.AND,
    Token.OR,
]);

export const PrimitiveValues = new Set([
    Token.NUMBER,
    Token.STRING,
    Token.TRUE,
    Token.FALSE,
]);

export const SIGNAL_OPERATORS = new Set([Token.SIGNAL, Token.COMPUTE]);

export const RESERVED_WORDS = {
    let: Token.DECLERATION,
    while: Token.WHILE,
    for: Token.FOR,
    foreach: Token.FOREACH,
    if: Token.IF,
    else: Token.ELSE,
    def: Token.FUNCTION,
    true: Token.TRUE,
    false: Token.FALSE,
    in: Token.IN,
};

export type ReservedWordsKey = keyof typeof RESERVED_WORDS;

export class LexerToken {
    private type: Token;
    private value: string;
    private lineNumber: number;

    constructor(type: Token, value: string, lineNumber: number = -1) {
        this.type = type;
        this.value = value;
        this.lineNumber = lineNumber;
    }

    getType(): Token {
        return this.type;
    }

    getValue(): string {
        return this.value;
    }

    getLineNumber(): number {
        return this.lineNumber;
    }
}

export type Maybe<T> = T | null | undefined;
export type Iterable = RuntimeValue[];
export type RuntimeValue = Maybe<number | string | boolean | Iterable | void>;
export type BooleanToken = Token.TRUE | Token.FALSE;
export type IterableResolvable = IterableASTNode | IdentifierASTNode;
