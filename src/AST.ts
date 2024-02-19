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

    public getChildren(): ASTNode[] {
        return this.children;
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

    public getType(): Token {
        return this.type;
    }

    public getValue(): string {
        return this.value;
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

export class LParenASTNode extends ASTNode {
    constructor() {
        super(Token.LPAREN, '(', []);
    }
}

export class RParenASTNode extends ASTNode {
    constructor() {
        super(Token.RPAREN, ')', []);
    }
}

export class SemiASTNode extends ASTNode {
    constructor() {
        super(Token.SEMI, ';', []);
    }
}

export class DeclarationASTNode extends ASTNode {
    constructor() {
        super(Token.DECLERATION, 'let', []);
    }
}

export class IdentifierASTNode extends ASTNode {
    constructor(value: string) {
        super(Token.IDENTIFIER, value, []);
    }
}

export class AssignASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.ASSIGN, '=', [left, right]);
    }
}
