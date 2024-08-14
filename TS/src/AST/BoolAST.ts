import { Instruction, PushInstruction } from '../Compilation/Instruction';
import Scope from '../Scope';
import { TannerError } from '../errors';
import {
    IBooleanableAST,
    IChildrenEnumerable,
    INumberableAST,
    Token,
} from '../types';
import { ASTNode } from './AST';

export class BooleanOpASTNode extends ASTNode implements IBooleanableAST {
    public left: INumberableAST;

    public right: INumberableAST;

    constructor(type: Token, left: INumberableAST, right: INumberableAST) {
        super(type);

        this.left = left;
        this.right = right;
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);
        const right = this.right.evaluate(scope);

        switch (this.getType()) {
            case Token.LESS:
                return left < right;
            case Token.LEQ:
                return left <= right;
            case Token.GREATER:
                return left > right;
            case Token.GEQ:
                return left >= right;
            default:
                throw new TannerError(`Unexpected token: ${this.getType()}`);
        }
    }

    getChildren(): IChildrenEnumerable[] {
        return [];
    }

    compile(): Instruction | Instruction[] {
        throw new TannerError('Unexpected call to BooleanOpASTNode.compile');
    }
}

export class BooleanASTNode extends ASTNode {
    evaluate(): boolean {
        return this.isType(Token.TRUE);
    }

    compile(): Instruction | Instruction[] {
        return new PushInstruction(this.isType(Token.TRUE) ? 1 : 0);
    }
}

export class LessThanASTNode extends BooleanOpASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.LESS, left, right);
    }
}

export class LessEqASTNode extends BooleanOpASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.LEQ, left, right);
    }
}

export class GreaterThanASTNode extends BooleanOpASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.GREATER, left, right);
    }
}

export class GreaterEqASTNode extends BooleanOpASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.GEQ, left, right);
    }
}

export class NotEqualASTNode extends BooleanOpASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.NEQ, left as INumberableAST, right as INumberableAST);
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);
        const right = this.right.evaluate(scope);

        return left !== right;
    }
}

export class EqualASTNode extends BooleanOpASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.EQUAL, left, right);
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);
        const right = this.right.evaluate(scope);

        return left === right;
    }
}

export class AndASTNode extends BooleanOpASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.AND, left, right);
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);

        // Short circuit
        if (!left) {
            return false;
        }

        const right = this.right.evaluate(scope);

        return !!left && !!right;
    }
}

export class OrASTNode extends BooleanOpASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.OR, left, right);
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);

        // Short circuit
        if (left) {
            return true;
        }

        const right = this.right.evaluate(scope);

        return !!left || !!right;
    }
}

export class NotASTNode extends ASTNode {
    public child: ASTNode;

    constructor(valueAST: ASTNode) {
        super(Token.NOT);
        this.child = valueAST;
    }

    evaluate(scope: Scope): boolean {
        return !this.child.evaluate(scope);
    }

    compile(): Instruction | Instruction[] {
        throw new TannerError('NotASTNode.compile not implemented');
    }
}
