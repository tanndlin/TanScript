import Scope from '../Scope';
import { TannerError } from '../errors';
import { Token } from '../types';
import { tokenToValue } from '../util';
import { ASTNode } from './AST';
import { INumberableAST } from './NumberAST';

export class BooleanOpASTNode extends ASTNode {
    constructor(type: Token, left: INumberableAST, right: INumberableAST) {
        super(type, tokenToValue(type), [left, right]);
    }

    evaluate(scope: Scope): boolean {
        const left = (this.getChildren()[0] as INumberableAST).evaluate(scope);
        const right = (this.getChildren()[1] as INumberableAST).evaluate(scope);

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
