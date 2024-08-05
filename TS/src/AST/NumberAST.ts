import Scope from '../Scope';
import { TannerError } from '../errors';
import { IChildrenEnumerable, INumberableAST, Token } from '../types';
import { ASTNode } from './AST';

export class MathASTNode
    extends ASTNode
    implements INumberableAST, IChildrenEnumerable
{
    public left: INumberableAST;
    public right: INumberableAST;

    constructor(type: Token, left: INumberableAST, right: INumberableAST) {
        super(type);
        this.left = left;
        this.right = right;
    }

    evaluate(scope: Scope): number {
        const left = (this.left as INumberableAST).evaluate(scope);
        const right = (this.right as INumberableAST).evaluate(scope);

        switch (this.getType()) {
            case Token.PLUS:
                return this.handleAddition(left, right);
            case Token.MINUS:
                return left - right;
            case Token.MULTIPLY:
                return left * right;
            case Token.DIVIDE:
                return left / right;
            case Token.INT_DIVIDE:
                return Math.floor(left / right);
            case Token.MOD:
                return left % right;
            default:
                throw new TannerError(`Unexpected token: ${this.getType()}`);
        }
    }

    handleAddition(left: any, right: any): any {
        // Special case to handle array concatenation
        if (left instanceof Array && right instanceof Array) {
            return left.concat(right);
        }

        return left + right;
    }

    getChildren(): IChildrenEnumerable[] {
        return [this.left, this.right];
    }
}

export class AddASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.PLUS, left, right);
    }
}

export class SubtractASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.MINUS, left, right);
    }
}

export class MultiplyASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.MULTIPLY, left, right);
    }
}

export class DivideASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.DIVIDE, left, right);
    }
}

export class IntegerDivideASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.INT_DIVIDE, left, right);
    }
}

export class ModASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.MOD, left, right);
    }
}

export class NumberASTNode extends ASTNode implements INumberableAST {
    constructor(value: string) {
        super(Token.NUMBER);
        this.value = value;
    }

    getChildren(): IChildrenEnumerable[] {
        return [];
    }

    evaluate(sope: Scope): number {
        return +this.value;
    }
}
