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
                case Token.PLUS:
                case Token.MINUS:
                case Token.MULTIPLY:
                case Token.DIVIDE:
                case Token.LPAREN:
                case Token.RPAREN:
                    this.tokens.push(new LexerToken(tokenType, char));
                    break;

                case Token.NUMBER:
                    const number = this.readNumber();
                    this.tokens.push(
                        new LexerToken(tokenType, number.toString())
                    );
                    break;
            }

            this.pos++;
        }
    }

    // Reads a whole number and leaves pos at the next char
    readNumber(): number {
        const start = this.pos;
        while (this.script[this.pos] >= '0' && this.script[this.pos] <= '9') {
            this.pos++;
        }

        return parseInt(this.script.substring(start, this.pos));
    }

    getTokens(): LexerToken[] {
        return this.tokens;
    }
}

export default Lexer;
