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
        case '(':
            return Token.LPAREN;
        case ')':
            return Token.RPAREN;
        case ';':
            return Token.SEMI;
        default:
            return Token.NUMBER;
    }
};
