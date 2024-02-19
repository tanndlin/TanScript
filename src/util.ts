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
        case '=':
            return Token.ASSIGN;
        default:
            // Check for numbers and identifiers
            if (!isNaN(Number(value))) return Token.NUMBER;
            return Token.IDENTIFIER;
    }
};
