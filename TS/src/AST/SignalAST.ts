import Scope from '../Scope';
import { RuntimeValue, Token } from '../types';
import { ASTNode, AssignASTNode } from './AST';

export class SignalAST extends ASTNode {
    constructor(name: string) {
        super(Token.SIGNAL, []);
        this.value = name.replace('#', '');
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getSignalValue(this.value);
    }
}

export class SignalAssignmentAST extends ASTNode {
    public identifier: string;

    constructor(assign: AssignASTNode) {
        super(Token.SIGNAL_ASSIGN, [assign]);

        const [identAST, valueAST] = assign.getChildren();
        this.identifier = identAST.getValue().replace('#', '');
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
    constructor(name: string) {
        super(Token.SIGNAL, []);
        this.value = name.replace('$', '');
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getSignalValue(this.value);
    }
}

export class SignalComputeAssignmentAST extends ASTNode {
    public identifier: string;

    constructor(assign: AssignASTNode) {
        super(Token.COMPUTE_ASSIGN, [assign]);

        const [identAST, valueAST] = assign.getChildren();
        this.identifier = identAST.getValue();
        this.identifier = identAST.getValue().replace('$', '');
    }

    public evaluate(scope: Scope): RuntimeValue {
        const [assignAST] = this.getChildren();

        const [identAST, valueAST] = assignAST.getChildren();
        const callback = () => valueAST.evaluate(scope);

        scope.setSignalCompute(this.identifier, assignAST);
    }
}
