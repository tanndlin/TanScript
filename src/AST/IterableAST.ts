import Scope from '../Scope';
import { RuntimeValue, Token } from '../types';
import { ASTNode } from './AST';

export class IterableASTNode extends ASTNode {
    constructor(items: ASTNode[]) {
        super(Token.LBRACKET, items);
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.children.map((child) => child.evaluate(scope));
    }

    createIterator(): Iterator {
        return new Iterator(this.children);
    }
}

export class ListASTNode extends IterableASTNode {
    constructor(items: ASTNode[]) {
        super(items);
    }
}

class Iterator {
    private items: ASTNode[];
    private index: number;

    constructor(items: ASTNode[]) {
        this.items = items;
        this.index = 0;
    }

    hasNext(): boolean {
        return this.index < this.items.length;
    }

    next(): ASTNode {
        return this.items[this.index++];
    }

    reset(): void {
        this.index = 0;
    }

    length(): number {
        return this.items.length;
    }
}
