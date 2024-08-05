import { BuiltInFuncName, allFunctions } from '../BuiltInFunctions';
import Scope from '../Scope';
import { RuntimeError } from '../errors';
import { RuntimeValue, Token } from '../types';
import { ASTNode, BlockASTNode, IdentifierASTNode } from './AST';
import { BooleanOpASTNode } from './BoolAST';

export abstract class ControlStructureASTNode extends ASTNode {
    constructor(type: Token, children: ASTNode[]) {
        super(type, children);
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
        public init: ASTNode,
        public condition: ASTNode,
        public update: ASTNode,
        public block: ASTNode
    ) {
        super(Token.FOR, []);
    }

    evaluate(scope: Scope): RuntimeValue {
        // Make a new scope for the looping variable
        const newScope = new Scope(scope.globalScope, scope);
        this.init.evaluate(newScope);

        let ret;
        while ((this.condition as BooleanOpASTNode).evaluate(newScope)) {
            ret = this.block.evaluate(newScope);
            this.update.evaluate(newScope);
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
            const ret = block.evaluate(scope);
            return ret;
        } else if (elseBlock) {
            return elseBlock.evaluate(scope);
        }
    }
}

export class FunctionDefASTNode extends ASTNode {
    constructor(
        name: string,
        private paramList: IdentifierASTNode[],
        block: BlockASTNode
    ) {
        super(Token.FUNCTION, [block]);
        this.value = name;
    }

    evaluate(scope: Scope): RuntimeValue {
        scope.addFunction(this.value, this);
        return null;
    }

    callFunction(
        callersScope: Scope,
        params: ASTNode[],
        funcDef: FunctionDefASTNode
    ): RuntimeValue {
        // Make sure the number of params line up
        if (params.length !== this.paramList.length) {
            throw new RuntimeError(
                `Function ${this.getValue()} expected ${
                    this.paramList.length
                } params, got ${params.length}`
            );
        }

        const newScope = new Scope(callersScope.globalScope, null);
        newScope.addFunction(this.value, funcDef);
        const [block] = this.getChildren();

        params.forEach((param, i) => {
            const expectedParam = this.paramList[i];
            const name = expectedParam.getValue();

            newScope.addVariable(name, param.evaluate(callersScope));
        });

        return block.evaluate(newScope);
    }

    getParamList() {
        return this.paramList;
    }
}

export class FunctionCallASTNode extends ASTNode {
    constructor(name: string, args: ASTNode[]) {
        super(Token.IDENTIFIER, args);
        this.value = name;
    }

    evaluate(scope: Scope): RuntimeValue {
        if (this.value in allFunctions) {
            const args = this.getChildren().map((arg) =>
                arg.evaluate(scope)
            ) as RuntimeValue[];
            return allFunctions[this.value as BuiltInFuncName](...args);
        }

        const funcDef = scope.getFunction(this.value);
        return funcDef.callFunction(scope, this.getChildren(), funcDef);
    }
}

export class ReturnASTNode extends ControlStructureASTNode {
    constructor(value: ASTNode) {
        super(Token.RETURN, [value]);
    }

    evaluate(scope: Scope): RuntimeValue {
        const [value] = this.getChildren();
        return scope.setReturnValue(value.evaluate(scope));
    }
}
