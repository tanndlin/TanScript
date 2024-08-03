import Scope from '../Scope';
import { Iterable, IterableResolvable, RuntimeValue, Token } from '../types';
import {
    ASTNode,
    BlockASTNode,
    DeclarationASTNode,
    IdentifierASTNode,
} from './AST';
import { ControlStructureASTNode } from './ControlAST';

export class IterableASTNode extends ASTNode {
    constructor(items: ASTNode[]) {
        super(Token.LBRACKET, items);
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.children.map((child) => child.evaluate(scope));
    }

    createIterator(scope: Scope): Iterator {
        return new Iterator(
            this.children.map((child) => child.evaluate(scope))
        );
    }
}

export class ListASTNode extends IterableASTNode {
    constructor(items: ASTNode[]) {
        super(items);
    }
}

export class ForEachASTNode extends ControlStructureASTNode {
    constructor(
        decl: DeclarationASTNode,
        iterable: IterableResolvable,
        block: ASTNode
    ) {
        super(Token.FOREACH, [decl, iterable, block]);
    }

    evaluate(scope: Scope): RuntimeValue {
        const ident = this.children[0] as IdentifierASTNode;
        const iterParam = this.children[1] as IterableResolvable;
        const block = this.children[2] as BlockASTNode;

        let iterator: Iterator;
        if (iterParam instanceof IdentifierASTNode) {
            const items = scope.getVariable(iterParam.getValue()) as Iterable;
            iterator = new Iterator(items);
        } else {
            iterator = (iterParam as IterableASTNode).createIterator(scope);
        }

        let ret;
        while (iterator.hasNext() && !scope.isReturning()) {
            const curItem = iterator.next();

            // Add the current item to the scope
            const newScope = new Scope(scope.globalScope, scope);
            newScope.addVariable(ident.getValue(), curItem);
            ret = block.evaluate(newScope);
        }

        return ret;
    }
}

class Iterator {
    protected items: RuntimeValue[];
    private index: number;

    constructor(items: RuntimeValue[]) {
        this.items = items;
        this.index = 0;
    }

    hasNext(): boolean {
        return this.index < this.items.length;
    }

    next(): RuntimeValue {
        return this.items[this.index++];
    }

    reset(): void {
        this.index = 0;
    }

    length(): number {
        return this.items.length;
    }
}

class SortedIterator extends Iterator {
    constructor(items: RuntimeValue[]) {
        super(items);
        this.items.sort();
    }
}
