import * as AST from './AST';
import { Token } from './types';

export const PRECEDENCE = [
    [
        { token: Token.MULTIPLY, ast: AST.ASTMultiply },
        { token: Token.DIVIDE, ast: AST.ASTDivide },
        { token: Token.INT_DIVIDE, ast: AST.ASTIntegerDivide },
        { token: Token.MOD, ast: AST.ASTMod },
    ],
    [
        { token: Token.PLUS, ast: AST.ASTAdd },
        { token: Token.MINUS, ast: AST.ASTSubtract },
    ],
    [
        { token: Token.LESS, ast: AST.ASTLessThan },
        { token: Token.GREATER, ast: AST.ASTGreaterThan },
        { token: Token.LEQ, ast: AST.ASTLessEq },
        { token: Token.GEQ, ast: AST.ASTGreaterEq },
    ],
    [
        { token: Token.EQUAL, ast: AST.ASTEqual },
        { token: Token.NEQ, ast: AST.ASTNotEqual },
    ],
    [
        { token: Token.AND, ast: AST.ASTAnd },
        { token: Token.OR, ast: AST.ASTOr },
    ],
];
