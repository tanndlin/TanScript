import Scope from '../Scope';
import { RuntimeValue, Token } from '../types';
import { tokenToValue } from '../util';
import { ASTNode } from './AST';
import { BooleanOpASTNode } from './BoolAST';

export abstract class ControlStructureASTNode extends ASTNode {
    constructor(type: Token, children: ASTNode[]) {
        super(type, tokenToValue(type), children);
    }
}

export class WhileASTNode extends ControlStructureASTNode {
    constructor(condition: ASTNode, block: ASTNode) {
        super(Token.WHILE, [condition, block]);
    }

    evaluate(scope: Scope): RuntimeValue {
        let ret;

        const [condition, block] = this.getChildren();
        while ((condition as BooleanOpASTNode).evaluate(scope)) {
            ret = block.evaluate(scope);
        }

        return ret;
    }
}

export class ForASTNode extends ControlStructureASTNode {
    constructor(
        init: ASTNode,
        condition: ASTNode,
        update: ASTNode,
        block: ASTNode
    ) {
        super(Token.FOR, [init, condition, update, block]);
    }

    evaluate(scope: Scope): RuntimeValue {
        const [init, condition, update, block] = this.getChildren();

        // Make a new scope for the looping variable
        const newScope = new Scope(scope);
        init.evaluate(newScope);

        let ret;
        while ((condition as BooleanOpASTNode).evaluate(newScope)) {
            ret = block.evaluate(newScope);
            update.evaluate(newScope);
        }

        return ret;
    }
}

export class IfASTNode extends ControlStructureASTNode {
    constructor(condition: ASTNode, block: ASTNode, elseBlock?: ASTNode) {
        super(
            Token.IF,
            [condition, block, elseBlock].filter(Boolean) as ASTNode[]
        );
    }

    evaluate(scope: Scope): RuntimeValue {
        const [condition, block, elseBlock] = this.getChildren();

        if ((condition as BooleanOpASTNode).evaluate(scope)) {
            return block.evaluate(scope);
        } else if (elseBlock) {
            return elseBlock.evaluate(scope);
        }
    }
}
