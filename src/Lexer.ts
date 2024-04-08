import { LexerToken, RESERVED_WORDS, ReservedWordsKey, Token } from './types';
import { valueToToken } from './util';

class Lexer {
    private tokens: LexerToken[];
    private pos = 0;

    constructor(private script: string) {
        this.tokens = [];
        this.tokenize();
    }

    tokenize() {
        while (this.pos < this.script.length) {
            const char = this.script[this.pos];
            // If char is whitespace, skip
            if (/\s/.test(char)) {
                this.pos++;
                continue;
            }

            const tokenType = valueToToken(char);
            switch (tokenType) {
                case Token.NUMBER:
                    const number = this.readNumber();
                    this.tokens.push(
                        new LexerToken(tokenType, number.toString())
                    );
                    break;
                case Token.IDENTIFIER:
                    const identifier = this.readIdentifier();
                    // if is a reserved word
                    if (Object.keys(RESERVED_WORDS).includes(identifier))
                        this.tokens.push(
                            new LexerToken(
                                RESERVED_WORDS[identifier as ReservedWordsKey],
                                identifier
                            )
                        );
                    else
                        this.tokens.push(
                            new LexerToken(Token.IDENTIFIER, identifier)
                        );
                    break;
                default:
                    this.tokens.push(new LexerToken(tokenType, char));
            }

            this.pos++;
        }

        this.tokens.push(new LexerToken(Token.EOF, ''));
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
        const validChars = new Set([
            '_',
            ...Array.from({ length: 26 }, (_, i) =>
                String.fromCharCode(i + 97)
            ),
            ...Array.from({ length: 26 }, (_, i) =>
                String.fromCharCode(i + 65)
            ),
            ...Array.from({ length: 10 }, (_, i) =>
                String.fromCharCode(i + 48)
            ),
        ]);

        const start = this.pos;
        while (
            validChars.has(this.script[this.pos]) &&
            this.pos < this.script.length
        ) {
            this.pos++;
        }

        this.pos--;
        return this.script.substring(start, this.pos + 1);
    }

    getTokens(): LexerToken[] {
        return this.tokens;
    }
}

export default Lexer;
