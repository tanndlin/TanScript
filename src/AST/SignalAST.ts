import Scope from '../Scope';
import { RuntimeValue, Token } from '../types';
import { ASTNode, AssignASTNode } from './AST';

export class SignalAST extends ASTNode {
    constructor(public name: string) {
        super(Token.SIGNAL, []);
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getSignalValue(this.name);
    }
}

export class SignalAssignmentAST extends ASTNode {
    public identifier: string;

    constructor(assign: AssignASTNode) {
        super(Token.SIGNAL, [assign]);

        const [identAST, valueAST] = assign.getChildren();
        this.identifier = identAST.getValue();
    }

    public evaluate(scope: Scope): RuntimeValue {
        const [assign] = this.getChildren();
        const [identAST, valueAST] = assign.getChildren();
        const value = valueAST.evaluate(scope);

        scope.setSignal(this.identifier, value);
        return value;
    }
}

export class SignalComputeAST extends ASTNode {
    constructor(private name: string) {
        super(Token.SIGNAL, []);
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getSignalValue(this.name);
    }
}

export class SignalComputeAssignmentAST extends ASTNode {
    public identifier: string;

    constructor(assign: AssignASTNode) {
        super(Token.SIGNAL, [assign]);

        const [identAST, valueAST] = assign.getChildren();
        this.identifier = identAST.getValue();
    }

    public evaluate(scope: Scope): RuntimeValue {
        const [assignAST] = this.getChildren();

        const [identAST, valueAST] = assignAST.getChildren();
        const callback = () => valueAST.evaluate(scope);

        scope.setSignalCompute(this.identifier, assignAST);
    }
}
