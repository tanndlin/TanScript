import { ASTNode } from './AST/AST';
import { LexerError } from './errors';
import { Token } from './types';

export const valueToToken = (value: string): Token => {
    // See if the value is an enum as its string component
    if (Object.values(Token).includes(value as Token)) return value as Token;

    switch (value) {
        case '&':
            return Token.AND;
        case '|':
            return Token.OR;
        case '"':
            return Token.STRING;

        default:
            // Check for numbers and identifiers
            if (!isNaN(Number(value))) return Token.NUMBER;
            if (
                (value.charCodeAt(0) >= 65 && value.charCodeAt(0) <= 90) ||
                (value.charCodeAt(0) >= 97 && value.charCodeAt(0) <= 122)
            )
                return Token.IDENTIFIER;

            throw new LexerError(`Unexpected token: ${value}`);
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
        if (c.isOneOf(Token.SIGNAL, Token.COMPUTE)) ret.push(c.getValue());
        else findSignals(c).forEach((s) => ret.push(s));
    });

    return ret;
};
