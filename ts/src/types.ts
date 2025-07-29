import * as AST from './AST';

export enum Token {
    FOR = 'for',
    FOREACH = 'foreach',
    WHILE = 'while',
    IF = 'if',
    ELSE = 'else',
    FUNCTION = 'def',
    DECLARATION = 'let',
    TRUE = 'true',
    FALSE = 'false',
    IN = 'in',
    RETURN = 'return',
    INT = 'int',

    NUMBER = 'number',
    STRING = 'string',
    PLUS = '+',
    MINUS = '-',
    MULTIPLY = '*',
    DIVIDE = '/',
    MOD = '%',
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
    LBRACKET = '[',
    RBRACKET = ']',
    INT_DIVIDE = '//',
    INCREMENT = '++',
    DECREMENT = '--',
    PERIOD = '.',
    PROGRAM = 'PROGRAM',
    Other = 'Other',
}

export type MathToken =
    | Token.PLUS
    | Token.MINUS
    | Token.MULTIPLY
    | Token.DIVIDE
    | Token.INT_DIVIDE
    | Token.MOD;

export type ComparisonToken =
    | Token.LESS
    | Token.GREATER
    | Token.LEQ
    | Token.GEQ
    | Token.EQUAL
    | Token.NEQ
    | Token.AND
    | Token.OR;

export type BooleanToken = Token.TRUE | Token.FALSE;

export const OPERATORS = new Set([
    Token.PLUS,
    Token.MINUS,
    Token.MULTIPLY,
    Token.DIVIDE,
    Token.INT_DIVIDE,
    Token.MOD,
    Token.LESS,
    Token.LEQ,
    Token.GREATER,
    Token.GEQ,
    Token.EQUAL,
    Token.NEQ,
    Token.AND,
    Token.OR,
    Token.INCREMENT,
    Token.DECREMENT,
]);

export const PrimitiveValues = new Set([
    Token.NUMBER,
    Token.STRING,
    Token.TRUE,
    Token.FALSE,
]);

export const RESERVED_WORDS = {
    let: Token.DECLARATION,
    while: Token.WHILE,
    for: Token.FOR,
    foreach: Token.FOREACH,
    if: Token.IF,
    else: Token.ELSE,
    def: Token.FUNCTION,
    true: Token.TRUE,
    false: Token.FALSE,
    in: Token.IN,
    return: Token.RETURN,
};

export type ReservedWordsKey = keyof typeof RESERVED_WORDS;

export enum Register {
    RAX = 'rax',
    RBX = 'rbx',
    RCX = 'rcx',
    RDX = 'rdx',
    R8 = 'r8',
    R9 = 'r9',
    R10 = 'r10',
    R11 = 'r11',
    R12 = 'r12',
    R13 = 'r13',
    R14 = 'r14',
    R15 = 'r15',
    EAX = 'eax',
}

export interface TokenTypeable {
    isType(type: Token): boolean;
    isOneOf(...types: Token[]): boolean;
}

export class LexerToken implements TokenTypeable {
    constructor(
        private type: Token,
        private value: string,
        private lineNumber: number = -1,
    ) {}

    getType(): Token {
        return this.type;
    }

    getValue(): string {
        return this.value;
    }

    getLineNumber(): number {
        return this.lineNumber;
    }

    public isType(type: Token) {
        return this.type === type;
    }

    public isOneOf(...types: Token[]): boolean {
        return types.includes(this.type);
    }
}

export type Maybe<T> = T | null | undefined;
export type IterableResolvable = AST.ASTIterable | AST.ASTIdentifier;
