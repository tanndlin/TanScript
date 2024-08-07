import { LexerError } from './errors';
import { LexerToken, RESERVED_WORDS, ReservedWordsKey, Token } from './types';
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
            const char = this.script[this.pos];
            // If char is whitespace, skip
            if (/\s/.test(char)) {
                if (char === '\n') {
                    this.lineNumber++;
                }
                this.pos++;
                continue;
            }

            const tokenType = valueToToken(char);
            switch (tokenType) {
                case Token.NUMBER:
                    const number = this.readNumber();
                    this.tokens.push(
                        this.createToken(tokenType, number.toString()),
                    );
                    break;
                case Token.IDENTIFIER:
                    const identifier = this.readIdentifier();
                    // if is a reserved word
                    if (Object.keys(RESERVED_WORDS).includes(identifier)) {
                        this.tokens.push(
                            this.createToken(
                                RESERVED_WORDS[identifier as ReservedWordsKey],
                                identifier,
                            ),
                        );
                    } else {
                        this.tokens.push(
                            this.createToken(Token.IDENTIFIER, identifier),
                        );
                    }
                    break;

                case Token.LESS:
                    this.tokens.push(
                        this.tryParsePair(tokenType, '=', Token.LEQ),
                    );
                    break;
                case Token.GREATER:
                    this.tokens.push(
                        this.tryParsePair(tokenType, '=', Token.GEQ),
                    );
                    break;
                case Token.ASSIGN:
                    const nextToken = this.script[this.pos + 1];
                    switch (nextToken) {
                        case Token.ASSIGN:
                            this.pos++;
                            this.tokens.push(
                                this.createToken(Token.EQUAL, '=='),
                            );
                            break;
                        case Token.GREATER:
                            this.pos++;
                            this.tokens.push(
                                this.createToken(Token.LAMBDA, '=>'),
                            );
                            break;
                        default:
                            this.tokens.push(
                                this.createToken(Token.ASSIGN, '='),
                            );
                            break;
                    }

                    break;
                case Token.NOT:
                    this.tokens.push(
                        this.tryParsePair(tokenType, '=', Token.NEQ),
                    );
                    break;
                case Token.SIGNAL: {
                    // If next char is a letter this is a signal
                    const nextChar = this.script[this.pos + 1];
                    if (LETTERS.has(nextChar)) {
                        this.pos++;
                        const identifier = this.readIdentifier();
                        this.tokens.push(
                            this.createToken(Token.SIGNAL, `#${identifier}`),
                        );
                        break;
                    }

                    // Otherwise it must be a signal assignment
                    if (nextChar !== '=') {
                        throw new LexerError(
                            `Unexpected token aftet #: ${nextChar}. Expected an assignment`,
                        );
                    }

                    this.pos++;
                    this.tokens.push(
                        this.createToken(Token.SIGNAL_ASSIGN, '#='),
                    );
                    break;
                }
                case Token.COMPUTE: {
                    // If next char is a letter this is a signal
                    const nextChar = this.script[this.pos + 1];
                    if (LETTERS.has(nextChar)) {
                        this.pos++;
                        const identifier = this.readIdentifier();
                        this.tokens.push(
                            this.createToken(Token.COMPUTE, `$${identifier}`),
                        );
                        break;
                    }

                    // Otherwise it must be a COMPUTE assignment
                    if (nextChar !== '=') {
                        throw new LexerError(
                            `Unexpected token aftet $: ${nextChar}. Expected an assignment`,
                        );
                    }

                    this.pos++;
                    this.tokens.push(
                        this.createToken(Token.COMPUTE_ASSIGN, '$='),
                    );
                    break;
                }

                case Token.PLUS:
                    this.tokens.push(
                        this.tryParsePair(tokenType, '+', Token.INCREMENT),
                    );
                    break;

                case Token.MINUS:
                    this.tokens.push(
                        this.tryParsePair(tokenType, '-', Token.DECREMENT),
                    );
                    break;

                case Token.DIVIDE:
                    this.tokens.push(
                        this.tryParsePair(tokenType, '/', Token.INT_DIVIDE),
                    );
                    break;

                // Grab the second one for now, since bitwise is not implemented
                case Token.AND:
                    this.pos++;
                    this.tokens.push(this.createToken(Token.AND, '&&'));
                    break;
                case Token.OR:
                    this.pos++;
                    this.tokens.push(this.createToken(Token.OR, '||'));
                    break;

                case Token.STRING:
                    const string = this.readString();
                    this.tokens.push(this.createToken(Token.STRING, string));
                    break;

                default:
                    this.tokens.push(this.createToken(tokenType, char));
            }

            this.pos++;
        }

        this.tokens.push(this.createToken(Token.EOF, ''));
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
        return new LexerToken(type, value, this.lineNumber);
    }
}

export default Lexer;
