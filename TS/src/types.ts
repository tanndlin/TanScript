import { ASTNode, IdentifierASTNode } from './AST/AST';
import * as BoolAST from './AST/BoolAST';
import { IterableASTNode } from './AST/IterableAST';
import * as NumberAST from './AST/NumberAST';
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
    SIGNAL = '#',
    COMPUTE = '$',
    COMPUTE_ASSIGN = '$=',
    SIGNAL_ASSIGN = 'SIGNAL_ASSIGN',
    LBRACKET = '[',
    RBRACKET = ']',
    INT_DIVIDE = '//',
    INCREMENT = '++',
    DECREMENT = '--',
    PERIOD = '.',
}

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
    return: Token.RETURN,
};

export type ReservedWordsKey = keyof typeof RESERVED_WORDS;

export abstract class TokenTypeable {
    constructor(protected type: Token) {}

    public isType(type: Token): boolean {
        return this.type === type;
    }

    public isOneOf(...types: Token[]): boolean {
        return types.includes(this.type);
    }
}

export class LexerToken extends TokenTypeable {
    private value: string;

    private lineNumber: number;

    constructor(type: Token, value: string, lineNumber: number = -1) {
        super(type);
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
export type Object = {
    attributes: Record<string, RuntimeValue>;
    methods: Record<string, Function>;
};
export type RuntimeValue = Maybe<
    number | string | boolean | Iterable | void | Object
>;
export type BooleanToken = Token.TRUE | Token.FALSE;
export type IterableResolvable = IterableASTNode | IdentifierASTNode;

export interface IChildrenEnumerable extends ASTNode {
    getChildren(): IChildrenEnumerable[];
}

export interface INumberableAST extends IChildrenEnumerable {
    evaluate(scope: Scope): number;
}

export interface IBooleanableAST extends IChildrenEnumerable {
    evaluate(scope: Scope): boolean;
}

export type ExpressionableAST = INumberableAST | IBooleanableAST;

export interface ITokenConstructorPair {
    token: Token;
    ast: AnyOperatorConstructor;
}

export interface IMathOperatorConstructor {
    new (left: INumberableAST, right: INumberableAST): NumberAST.MathASTNode;
}
export interface IRelationalOperatorConstructor {
    new (left: INumberableAST, right: INumberableAST): BoolAST.BooleanOpASTNode;
}
export interface IEqualityOperatorConstructor {
    new (left: ASTNode, right: ASTNode): BoolAST.BooleanOpASTNode;
}

export type AnyOperatorConstructor =
    | IMathOperatorConstructor
    | IRelationalOperatorConstructor
    | IEqualityOperatorConstructor;
