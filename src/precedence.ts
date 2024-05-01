import * as BoolAST from './AST/BoolAST';
import * as NumberAST from './AST/NumberAST';
import { ITokenConstructorPair, Token } from './types';

export const PRECEDENCE: ITokenConstructorPair[][] = [
    [
        { token: Token.MULTIPLY, ast: NumberAST.MultiplyASTNode },
        { token: Token.DIVIDE, ast: NumberAST.DivideASTNode },
        { token: Token.MOD, ast: NumberAST.ModASTNode },
    ],
    [
        { token: Token.PLUS, ast: NumberAST.AddASTNode },
        { token: Token.MINUS, ast: NumberAST.SubtractASTNode },
    ],
    [
        { token: Token.LESS, ast: BoolAST.LessThanASTNode },
        { token: Token.GREATER, ast: BoolAST.GreaterThanASTNode },
        { token: Token.LEQ, ast: BoolAST.LessEqASTNode },
        { token: Token.GEQ, ast: BoolAST.GreaterEqASTNode },
    ],
    [
        { token: Token.EQUAL, ast: BoolAST.EqualASTNode },
        { token: Token.NEQ, ast: BoolAST.NotEqualASTNode },
    ],
    [
        { token: Token.AND, ast: BoolAST.AndASTNode },
        { token: Token.OR, ast: BoolAST.OrASTNode },
    ],
];
