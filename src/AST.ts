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

export class BlockASTNode extends ASTNode {
    constructor(children: ASTNode[]) {
        super(Token.LCURLY, '{', children);
    }
}

export class WhileASTNode extends ASTNode {
    constructor(condition: ASTNode, block: ASTNode) {
        super(Token.WHILE, 'while', [condition, block]);
    }
}

export class ForASTNode extends ASTNode {
    constructor(
        init: ASTNode,
        condition: ASTNode,
        update: ASTNode,
        block: ASTNode
    ) {
        super(Token.FOR, 'for', [init, condition, update, block]);
    }
}

export class LessThanASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.LESS, '<', [left, right]);
    }
}

export class LessEqASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.LEQ, '<=', [left, right]);
    }
}

export class GreaterThanASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.GREATER, '>', [left, right]);
    }
}

export class GreaterEqASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.GEQ, '>=', [left, right]);
    }
}
