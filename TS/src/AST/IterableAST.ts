import { CompileScope } from '../Compilation/CompileScope';
import Scope from '../Scope';
import { Iterable, IterableResolvable, RuntimeValue, Token } from '../types';
import {
    ASTNode,
    BlockASTNode,
    DeclarationASTNode,
    IdentifierASTNode,
} from './AST';

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

export class IterableASTNode extends ASTNode {
    public items: ASTNode[];

    constructor(items: ASTNode[]) {
        super(Token.LBRACKET);
        this.items = items;
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.items.map((child) => child.evaluate(scope));
    }

    createIterator(scope: Scope): Iterator {
        return new Iterator(this.items.map((child) => child.evaluate(scope)));
    }

    compile(scope: CompileScope): never {
        throw new Error('Method not implemented.');
    }
}

export class ListASTNode extends IterableASTNode {}

export class ForEachASTNode extends ASTNode {
    public init: DeclarationASTNode | IdentifierASTNode;

    public iterable: IterableResolvable;

    public block: BlockASTNode;

    constructor(
        init: DeclarationASTNode | IdentifierASTNode,
        iterable: IterableResolvable,
        block: BlockASTNode,
    ) {
        super(Token.FOREACH);
        this.init = init;
        this.iterable = iterable;
        this.block = block;
    }

    evaluate(scope: Scope): RuntimeValue {
        let iterator: Iterator;
        if (this.iterable instanceof IdentifierASTNode) {
            const items = scope.getVariable(
                this.iterable.getValue(),
            ) as Iterable;
            iterator = new Iterator(items);
        } else {
            iterator = (this.iterable as IterableASTNode).createIterator(scope);
        }

        let ret;
        while (iterator.hasNext() && !scope.isReturning()) {
            const curItem = iterator.next();

            // Add the current item to the scope
            const newScope = new Scope(scope.globalScope, scope);
            newScope.addVariable(this.init.getValue(), curItem);
            ret = this.block.evaluate(newScope);
        }

        return ret;
    }

    compile(scope: CompileScope): never {
        throw new Error('Method not implemented.');
    }
}
