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

export class CompilerError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'CompilerError';
    }
}

export class RuntimeError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'RuntimeError';
    }
}

export class UseBeforeDeclarationError extends RuntimeError {
    constructor(message: string) {
        super(message);
        this.name = 'UseBeforeDeclarationError';
    }
}

export class UndeclaredVariableError extends RuntimeError {
    constructor(message: string) {
        super(message);
        this.name = 'UndeclaredVariableError';
    }
}

export class UndeclaredFunctionError extends RuntimeError {
    constructor(message: string) {
        super(message);
        this.name = 'UndeclaredFunctionError';
    }
}

export class TannerError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'TannerError';
    }
}
