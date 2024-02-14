import { Token } from './types';

export class ASTNode {
    protected type: Token;
    protected value: string;
    protected children: ASTNode[];

    constructor(type: Token, value: string, children: ASTNode[]) {
        this.type = type;
        this.value = value;
        this.children = children;
    }
}

export class AST {
    constructor(private root: ASTNode) {}

    getRoot() {
        return this.root;
    }
}

export class AddASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.PLUS, '+', [left, right]);
    }
}

export class SubtractASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.MINUS, '-', [left, right]);
    }
}

export class MultiplyASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.MULTIPLY, '*', [left, right]);
    }
}

export class DivideASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.DIVIDE, '/', [left, right]);
    }
}

export class NumberASTNode extends ASTNode {
    constructor(value: string) {
        super(Token.NUMBER, value, []);
    }
}

export class EOFASTNode extends ASTNode {
    constructor() {
        super(Token.EOF, '', []);
    }
}
