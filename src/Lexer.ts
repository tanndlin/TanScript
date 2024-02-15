import { LexerToken, Token } from './types';
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
            if (char === ' ' || char === '\n' || char === '\t') {
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

    getTokens(): LexerToken[] {
        return this.tokens;
    }
}

export default Lexer;
