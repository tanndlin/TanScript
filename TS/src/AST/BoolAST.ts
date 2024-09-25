import { CompileScope } from '../Compilation/CompileScope';
import Scope from '../Scope';
import { TannerError } from '../errors';
import {
    IBooleanableAST,
    IChildrenEnumerable,
    INumberableAST,
    Token,
} from '../types';
import {
    EqInstruction,
    GeqInstruction,
    GreaterInstruction,
    Instruction,
    LeqInstruction,
    LessInstruction,
    PushInstruction,
} from './../Compilation/Instruction';
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

    compile(scope: CompileScope): Instruction[] {
        const left = [this.left.compile(scope)].flat();
        const right = [this.right.compile(scope)].flat();

        const instructions = [...left, ...right];
        switch (this.getType()) {
            case Token.LESS:
                instructions.push(new LessInstruction());
                break;
            case Token.LEQ:
                instructions.push(new LeqInstruction());
                break;
            case Token.GREATER:
                instructions.push(new GreaterInstruction());
                break;
            case Token.GEQ:
                instructions.push(new GeqInstruction());
                break;
            case Token.EQUAL:
                instructions.push(new EqInstruction());
                break;
            default:
                throw new TannerError(`Unexpected token: ${this.getType()}`);
        }

        return instructions;
    }
}

export class BooleanASTNode extends ASTNode {
    evaluate(): boolean {
        return this.isType(Token.TRUE);
    }

    compile(scope: CompileScope): Instruction[] {
        return [new PushInstruction(this.isType(Token.TRUE) ? 1 : 0)];
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

    compile(scope: CompileScope): Instruction[] {
        throw new TannerError('NotASTNode.compile not implemented');
    }
}
