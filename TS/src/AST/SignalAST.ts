import { CompileScope } from '../Compilation/CompileScope';
import { Instruction } from '../Compilation/Instruction';
import Scope from '../Scope';
import { IChildrenEnumerable, RuntimeValue, Token } from '../types';
import { ASTNode, AssignASTNode, IdentifierASTNode } from './AST';

export class SignalAST extends ASTNode implements IChildrenEnumerable {
    constructor(name: string) {
        super(Token.SIGNAL);
        this.value = name.replace('#', '');
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getSignalValue(this.value);
    }

    public getChildren(): IChildrenEnumerable[] {
        return [];
    }

    compile(scope: CompileScope): never {
        throw new Error('Method not implemented.');
    }
}

export class SignalAssignmentAST extends AssignASTNode {
    public name: string;

    constructor(identifier: IdentifierASTNode, value: ASTNode) {
        super(identifier, value);
        this.type = Token.SIGNAL_ASSIGN;

        this.name = identifier.getValue().replace('#', '');
    }

    public evaluate(scope: Scope): RuntimeValue {
        const value = this.valueAST.evaluate(scope);

        scope.setSignal(this.name, value);
        return value;
    }
}

export class SignalComputeAST extends ASTNode {
    constructor(name: string) {
        super(Token.SIGNAL);
        this.value = name.replace('$', '');
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getSignalValue(this.value);
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        throw new Error('Method not implemented.');
    }
}

export class SignalComputeAssignmentAST extends AssignASTNode {
    public name: string;

    constructor(left: IdentifierASTNode, right: ASTNode) {
        super(left, right);
        this.type = Token.COMPUTE_ASSIGN;

        this.name = this.identifier.getValue();
        this.name = this.identifier.getValue().replace('$', '');
    }

    public evaluate(scope: Scope): RuntimeValue {
        scope.setSignalCompute(this.name, this);
    }
}
