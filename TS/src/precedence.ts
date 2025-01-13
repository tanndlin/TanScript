import * as AST from './AST';
import { ITokenConstructorPair, Token } from './types';

export const PRECEDENCE: ITokenConstructorPair[][] = [
    [
        { token: Token.MULTIPLY, ast: AST.MultiplyASTNode },
        { token: Token.DIVIDE, ast: AST.DivideASTNode },
        { token: Token.INT_DIVIDE, ast: AST.IntegerDivideASTNode },
        { token: Token.MOD, ast: AST.ModASTNode },
    ],
    [
        { token: Token.PLUS, ast: AST.AddASTNode },
        { token: Token.MINUS, ast: AST.SubtractASTNode },
    ],
    [
        { token: Token.LESS, ast: AST.LessThanASTNode },
        { token: Token.GREATER, ast: AST.GreaterThanASTNode },
        { token: Token.LEQ, ast: AST.LessEqASTNode },
        { token: Token.GEQ, ast: AST.GreaterEqASTNode },
    ],
    [
        { token: Token.EQUAL, ast: AST.EqualASTNode },
        { token: Token.NEQ, ast: AST.NotEqualASTNode },
    ],
    [
        { token: Token.AND, ast: AST.AndASTNode },
        { token: Token.OR, ast: AST.OrASTNode },
    ],
];
