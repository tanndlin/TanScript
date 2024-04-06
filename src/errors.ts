export class LexerError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'LexerError';
    }
}

export class ParserError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'ParserError';
    }
}

export class RuntimeError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'RuntimeError';
    }
}
