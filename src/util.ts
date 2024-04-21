import { ASTNode } from './AST/AST';
import { Token } from './types';

export const valueToToken = (value: string): Token => {
    switch (value) {
        case '+':
            return Token.PLUS;
        case '-':
            return Token.MINUS;
        case '*':
            return Token.MULTIPLY;
        case '/':
            return Token.DIVIDE;
        case '<':
            return Token.LESS;
        case '>':
            return Token.GREATER;
        case '(':
            return Token.LPAREN;
        case ')':
            return Token.RPAREN;
        case ';':
            return Token.SEMI;
        case '=':
            return Token.ASSIGN;
        case '{':
            return Token.LCURLY;
        case '}':
            return Token.RCURLY;
        case ',':
            return Token.COMMA;
        case '&':
            return Token.AND;
        case '|':
            return Token.OR;
        case '!':
            return Token.NOT;
        case '"':
            return Token.STRING;
        case ':':
            return Token.COLON;
        case '#':
            return Token.SIGNAL;
        case '$':
            return Token.COMPUTE;

        default:
            // Check for numbers and identifiers
            if (!isNaN(Number(value))) return Token.NUMBER;
            return Token.IDENTIFIER;
    }
};

export const tokenToValue = (token: Token): string => {
    switch (token) {
        case Token.PLUS:
            return '+';
        case Token.MINUS:
            return '-';
        case Token.MULTIPLY:
            return '*';
        case Token.DIVIDE:
            return '/';
        case Token.LESS:
            return '<';
        case Token.LEQ:
            return '<=';
        case Token.GREATER:
            return '>';
        case Token.GEQ:
            return '>=';
        case Token.LPAREN:
            return '(';
        case Token.RPAREN:
            return ')';
        case Token.SEMI:
            return ';';
        case Token.ASSIGN:
            return '=';
        case Token.EQUAL:
            return '==';
        case Token.NEQ:
            return '!=';
        case Token.COMMA:
            return ',';
        case Token.LCURLY:
            return '{';
        case Token.RCURLY:
            return '}';
        case Token.WHILE:
            return 'while';
        case Token.FOR:
            return 'for';
        case Token.IF:
            return 'if';
        case Token.ELSE:
            return 'else';
        default:
            return token;
    }
};

export const LOWERCASE_LETTERS = Array.from({ length: 26 }, (_, i) =>
    String.fromCharCode(i + 97)
);

export const UPPERCASE_LETTERS = Array.from({ length: 26 }, (_, i) =>
    String.fromCharCode(i + 65)
);

export const NUMBERS = Array.from({ length: 10 }, (_, i) =>
    String.fromCharCode(i + 48)
);

export const findSignals = (ast: ASTNode): string[] => {
    const children = ast.getChildren();

    if (children.length == 0) {
        return [];
    }

    const ret: string[] = [];

    children.forEach((c) => {
        if (c.getType() === Token.SIGNAL || c.getType() === Token.COMPUTE) {
            ret.push(c.getValue());
        } else {
            const found = findSignals(c).forEach((s) => ret.push(s));
        }
    });

    return ret;
};
