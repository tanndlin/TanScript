import * as AST from './AST';
import Scope from './Scope';

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
    RETURN = 'return',

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
}

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
    return: Token.RETURN,
};

export type ReservedWordsKey = keyof typeof RESERVED_WORDS;

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
export type Iterable = RuntimeValue[];
export type Object = {
    attributes: Record<string, RuntimeValue>;
    methods: Record<string, Function>;
};
export type RuntimeValue = Maybe<
    number | string | boolean | Iterable | void | Object
>;
export type IterableResolvable = AST.IterableASTNode | AST.IdentifierASTNode;

export interface INumberableAST extends AST.Expr {
    evaluate(scope: Scope): number;
}

export interface IBooleanableAST extends AST.Expr {
    evaluate(scope: Scope): boolean;
}

export type ExpressionableAST = INumberableAST | IBooleanableAST;

export interface ITokenConstructorPair {
    token: Token;
    ast: AnyOperatorConstructor;
}

export interface IMathOperatorConstructor {
    new (left: INumberableAST, right: INumberableAST): AST.MathASTNode;
}
export interface IRelationalOperatorConstructor {
    new (left: INumberableAST, right: INumberableAST): AST.ComparisonASTNode;
}
export interface IEqualityOperatorConstructor {
    new (left: AST.Expr, right: AST.Expr): AST.ComparisonASTNode;
}

export type AnyOperatorConstructor =
    | IMathOperatorConstructor
    | IRelationalOperatorConstructor
    | IEqualityOperatorConstructor;
