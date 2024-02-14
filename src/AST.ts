import { Token } from './types';

export class AST {
    constructor(private root: RootASTNode) {}

    getRoot() {
        return this.root;
    }
}

export class RootASTNode {
    protected children: ASTNode[];

    constructor(children?: ASTNode[]) {
        this.children = children ?? [];
    }

    addChild(node: ASTNode) {
        this.children.push(node);
    }
}

export class ASTNode extends RootASTNode {
    protected type: Token;
    protected value: string;

    constructor(type: Token, value: string, children: ASTNode[]) {
        super(children);
        this.type = type;
        this.value = value;
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
