import { Err, Ok, Result } from './Result';
import { LexerToken, RESERVED_WORDS, ReservedWordsKey, Token } from './types';
import {
    LOWERCASE_LETTERS,
    NUMBERS,
    UPPERCASE_LETTERS,
    valueToToken,
} from './util';

class Lexer {
    private pos = 0;
    private lineNumber = 1;

    private readonly validChars = new Set([
        '_',
        ...LOWERCASE_LETTERS,
        ...UPPERCASE_LETTERS,
        ...NUMBERS,
    ]);

    constructor(private script: string) {}

    public tokenize(): Result<LexerToken[], string> {
        const tokens: LexerToken[] = [];
        while (this.pos < this.script.length) {
            const result = this.getNextToken();
            if (!result.ok) {
                return result;
            }

            tokens.push(result.val);
        }

        if (tokens[tokens.length - 1].getType() !== Token.EOF) {
            tokens.push(this.createToken(Token.EOF, ''));
        }

        return new Ok(tokens);
    }

    private getNextToken(): Result<LexerToken, string> {
        if (this.pos >= this.script.length) {
            return new Ok(this.createToken(Token.EOF, ''));
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

        if (char === '#') {
            // Skip comments
            while (
                this.pos < this.script.length &&
                this.script[this.pos] !== '\n'
            ) {
                this.pos++;
            }

            return this.getNextToken();
        }

        const tokenType = valueToToken(char);
        switch (tokenType) {
            case Token.NUMBER:
                const number = this.readNumber();
                return new Ok(this.createToken(tokenType, number.toString()));
            case Token.IDENTIFIER:
                const identifier = this.readIdentifier();
                // if is a reserved word
                if (Object.keys(RESERVED_WORDS).includes(identifier)) {
                    return new Ok(
                        this.createToken(
                            RESERVED_WORDS[identifier as ReservedWordsKey],
                            identifier,
                        ),
                    );
                }
                return new Ok(this.createToken(Token.IDENTIFIER, identifier));

            case Token.ASSIGN:
                const nextToken = this.script[this.pos + 1];
                switch (nextToken) {
                    case Token.ASSIGN:
                        this.pos++;
                        return new Ok(this.createToken(Token.EQUAL, '=='));
                    case Token.GREATER:
                        this.pos++;
                        return new Ok(this.createToken(Token.LAMBDA, '=>'));
                    default:
                        return new Ok(this.createToken(tokenType, char));
                }

            case Token.STRING:
                const result = this.readString();
                if (result.ok) {
                    return new Ok(this.createToken(Token.STRING, result.val));
                }

                return result;

            case Token.OR:
                this.pos++;
                return new Ok(this.createToken(Token.OR, '||'));
            case Token.AND:
                this.pos++;
                return new Ok(this.createToken(Token.AND, '&&'));

            case Token.GREATER:
                return new Ok(this.tryParsePair(tokenType, '=', Token.GEQ));
            case Token.LESS:
                return new Ok(this.tryParsePair(tokenType, '=', Token.LEQ));
            case Token.NOT:
                return new Ok(this.tryParsePair(tokenType, '=', Token.NEQ));
            case Token.PLUS:
                return new Ok(
                    this.tryParsePair(tokenType, '+', Token.INCREMENT),
                );
            case Token.MINUS:
                return new Ok(
                    this.tryParsePair(tokenType, '-', Token.DECREMENT),
                );
            case Token.DIVIDE:
                return new Ok(
                    this.tryParsePair(tokenType, '/', Token.INT_DIVIDE),
                );
            default:
                return new Ok(this.createToken(tokenType, char));
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

    readString(): Result<string, string> {
        const start = this.pos;
        this.pos++;
        while (this.script[this.pos] !== '"') {
            this.pos++;

            if (this.pos >= this.script.length) {
                return new Err('No closing quote for string found');
            }
        }

        return new Ok(this.script.substring(start + 1, this.pos));
    }

    createToken(type: Token, value: string): LexerToken {
        this.pos++;
        return new LexerToken(type, value, this.lineNumber);
    }
}

export default Lexer;
