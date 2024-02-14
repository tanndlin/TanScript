export enum Token {
    NUMBER = 'NUMBER',
    PLUS = 'PLUS',
    MINUS = 'MINUS',
    MULTIPLY = 'MULTIPLY',
    DIVIDE = 'DIVIDE',
    LPAREN = 'LPAREN',
    RPAREN = 'RPAREN',
    EOF = 'EOF',
}

export class LexerToken {
    private type: Token;
    private value: string;

    constructor(type: Token, value: string) {
        this.type = type;
        this.value = value;
    }

    getType(): Token {
        return this.type;
    }

    getValue(): string {
        return this.value;
    }
}
