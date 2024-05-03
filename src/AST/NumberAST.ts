import Scope from '../Scope';
import { TannerError } from '../errors';
import { INumberableAST, Token } from '../types';
import { ASTNode } from './AST';

export class MathASTNode extends ASTNode implements INumberableAST {
    constructor(type: Token, left: INumberableAST, right: INumberableAST) {
        super(type, [left, right]);
    }

    evaluate(scope: Scope): number {
        const left = (this.getChildren()[0] as INumberableAST).evaluate(scope);
        const right = (this.getChildren()[1] as INumberableAST).evaluate(scope);

        switch (this.getType()) {
            case Token.PLUS:
                return left + right;
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

export class NumberASTNode extends ASTNode {
    constructor(value: string) {
        super(Token.NUMBER, []);
        this.value = value;
    }

    evaluate(): number {
        return +this.value;
    }
}
