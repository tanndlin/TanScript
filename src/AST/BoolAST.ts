import Scope from '../Scope';
import { TannerError } from '../errors';
import { RuntimeValue, Token } from '../types';
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
            case Token.EQUAL:
                return left === right;
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

export class NotEqualASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.NEQ, '!=', [left, right]);
    }

    evaluate(scope: Scope): RuntimeValue {
        const left = this.getChildren()[0].evaluate(scope);
        const right = this.getChildren()[1].evaluate(scope);

        return left !== right;
    }
}

export class EqualASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.EQUAL, '==', [left, right]);
    }

    evaluate(scope: Scope): RuntimeValue {
        const left = this.getChildren()[0].evaluate(scope);
        const right = this.getChildren()[1].evaluate(scope);

        return left === right;
    }
}

export class AndASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.AND, '&&', [left, right]);
    }

    evaluate(scope: Scope): boolean {
        const left = this.getChildren()[0].evaluate(scope);
        const right = this.getChildren()[1].evaluate(scope);

        return !!left && !!right;
    }
}

export class OrASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.OR, '||', [left, right]);
    }

    evaluate(scope: Scope): boolean {
        const left = this.getChildren()[0].evaluate(scope);
        const right = this.getChildren()[1].evaluate(scope);

        return !!left || !!right;
    }
}

export class NotASTNode extends ASTNode {
    constructor(value: ASTNode) {
        super(Token.NOT, '!', [value]);
    }

    evaluate(scope: Scope): boolean {
        return !this.getChildren()[0].evaluate(scope);
    }
}
