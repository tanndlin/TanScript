import { LexerError } from './errors';
import {
    LexerToken,
    Maybe,
    RESERVED_WORDS,
    ReservedWordsKey,
    Token,
} from './types';
import {
    LETTERS,
    LOWERCASE_LETTERS,
    NUMBERS,
    UPPERCASE_LETTERS,
    valueToToken,
} from './util';

class Lexer {
    private tokens: LexerToken[];

    private pos = 0;

    private lineNumber = 1;

    private readonly validChars = new Set([
        '_',
        ...LOWERCASE_LETTERS,
        ...UPPERCASE_LETTERS,
        ...NUMBERS,
    ]);

    constructor(private script: string) {
        this.tokens = [];
        this.tokenize();
    }

    tokenize() {
        while (this.pos < this.script.length) {
            const token = this.getNextToken();
            if (token) {
                this.tokens.push(token);
            }
        }

        this.tokens.push(this.createToken(Token.EOF, ''));
    }

    private getNextToken(): Maybe<LexerToken> {
        if (this.pos >= this.script.length) {
            return null;
        }

        const char = this.script[this.pos];
        // If char is whitespace, skip
        if (/\s/.test(char)) {
            if (char === '\n') {
                this.lineNumber++;
            }

            this.pos++;
            return this.getNextToken();
        }

        const tokenType = valueToToken(char);
        switch (tokenType) {
            case Token.NUMBER:
                const number = this.readNumber();
                return this.createToken(tokenType, number.toString());
            case Token.IDENTIFIER:
                const identifier = this.readIdentifier();
                // if is a reserved word
                if (Object.keys(RESERVED_WORDS).includes(identifier)) {
                    return this.createToken(
                        RESERVED_WORDS[identifier as ReservedWordsKey],
                        identifier,
                    );
                }
                return this.createToken(Token.IDENTIFIER, identifier);

            case Token.ASSIGN:
                const nextToken = this.script[this.pos + 1];
                switch (nextToken) {
                    case Token.ASSIGN:
                        this.pos++;
                        return this.createToken(Token.EQUAL, '==');
                    case Token.GREATER:
                        this.pos++;
                        return this.createToken(Token.LAMBDA, '=>');
                    default:
                        return this.createToken(tokenType, char);
                }

            case Token.SIGNAL: {
                // If next char is a letter this is a signal
                const nextChar = this.script[this.pos + 1];
                if (LETTERS.has(nextChar)) {
                    this.pos++;
                    const identifier = this.readIdentifier();
                    return this.createToken(Token.SIGNAL, `#${identifier}`);
                }

                // Otherwise it must be a signal assignment
                if (nextChar !== '=') {
                    throw new LexerError(
                        `Unexpected token after #: ${nextChar}. Expected an assignment`,
                    );
                }

                this.pos++;
                return this.createToken(Token.SIGNAL_ASSIGN, '#=');
            }

            case Token.COMPUTE: {
                // If next char is a letter this is a signal
                const nextChar = this.script[this.pos + 1];
                if (LETTERS.has(nextChar)) {
                    const identifier = this.readIdentifier();
                    return this.createToken(Token.COMPUTE, `$${identifier}`);
                }

                // Otherwise it must be a COMPUTE assignment
                if (nextChar !== '=') {
                    throw new LexerError(
                        `Unexpected token after $: ${nextChar}. Expected an assignment`,
                    );
                }

                this.pos++;
                return this.createToken(Token.COMPUTE_ASSIGN, '$=');
            }

            case Token.STRING:
                const string = this.readString();
                return this.createToken(Token.STRING, string);

            case Token.OR:
                this.pos++;
                return this.createToken(Token.OR, '||');
            case Token.AND:
                this.pos++;
                return this.createToken(Token.AND, '&&');

            case Token.GREATER:
                return this.tryParsePair(tokenType, '=', Token.GEQ);
            case Token.LESS:
                return this.tryParsePair(tokenType, '=', Token.LEQ);
            case Token.NOT:
                return this.tryParsePair(tokenType, '=', Token.NEQ);
            case Token.PLUS:
                return this.tryParsePair(tokenType, '+', Token.INCREMENT);
            case Token.MINUS:
                return this.tryParsePair(tokenType, '-', Token.DECREMENT);
            case Token.DIVIDE:
                return this.tryParsePair(tokenType, '/', Token.INT_DIVIDE);
            default:
                return this.createToken(tokenType, char);
        }
    }

    tryParsePair(
        curToken: Token,
        secondChar: string,
        secondToken: Token,
    ): LexerToken {
        if (this.script[this.pos + 1] === secondChar) {
            this.pos++;
            return this.createToken(secondToken, secondToken);
        } else {
            return this.createToken(curToken, curToken);
        }
    }

    // Reads a whole number and leaves pos at the next char
    readNumber(): number {
        const start = this.pos;
        while (this.script[this.pos] >= '0' && this.script[this.pos] <= '9') {
            this.pos++;
        }

        this.pos--;
        return parseInt(this.script.substring(start, this.pos + 1));
    }

    readIdentifier() {
        const start = this.pos;
        while (
            this.validChars.has(this.script[this.pos]) &&
            this.pos < this.script.length
        ) {
            this.pos++;
        }

        this.pos--;
        return this.script.substring(start, this.pos + 1);
    }

    readString() {
        const start = this.pos;
        this.pos++;
        while (this.script[this.pos] !== '"') {
            this.pos++;
        }

        return this.script.substring(start + 1, this.pos);
    }

    getTokens(): LexerToken[] {
        return this.tokens;
    }

    createToken(type: Token, value: string): LexerToken {
        this.pos++;
        return new LexerToken(type, value, this.lineNumber);
    }
}

export default Lexer;
