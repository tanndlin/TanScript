import { CompileScope } from '../Compilation/CompileScope';
import {
    AllocInstruction,
    Instruction,
    JumpInstruction,
    LoadInstruction,
    PushInstruction,
    StoreInstruction,
} from '../Compilation/Instruction';
import Scope from '../Scope';
import { TannerError } from '../errors';
import {
    IChildrenEnumerable,
    RuntimeValue,
    Token,
    TokenTypeable,
} from '../types';

export class AST {
    constructor(private root: BlockASTNode) {}

    getRoot() {
        return this.root;
    }

    compile(): Instruction[] {
        const globalScope = new CompileScope();
        return [this.root.compile(globalScope)].flat();
    }
}

export abstract class ASTNode extends TokenTypeable {
    protected type: Token;

    protected value: string;

    constructor(type: Token) {
        super(type);
        this.type = type;
        this.value = type;
    }

    abstract evaluate(scope: Scope): RuntimeValue;
    abstract compile(scope: CompileScope): Instruction | Instruction[];

    public getType(): Token {
        return this.type;
    }

    public getValue(): string {
        return this.value;
    }
}

export class DecoratorASTNode extends ASTNode {
    evaluate(): RuntimeValue {
        throw new TannerError('Unexpected call to DecoratorASTNode.evaluate');
    }

    addChild(): void {
        throw new TannerError('Unexpected call to DecoratorASTNode.addChild');
    }

    getChildren(): ASTNode[] {
        return [];
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        return [];
    }
}

export class EOFASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.EOF);
    }

    evaluate(): RuntimeValue {
        throw new TannerError('Unexpected call to EOF.evaluate');
    }
}

export class RParenASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.RPAREN);
    }

    evaluate(): RuntimeValue {
        throw new TannerError('Unexpected call to RParen.evaluate');
    }
}

export class SemiASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.SEMI);
    }

    evaluate(): RuntimeValue {
        throw new TannerError('Unexpected call to Semi.evaluate');
    }
}

export class LParenASTNode extends ASTNode {
    public child: IChildrenEnumerable;

    constructor(child: IChildrenEnumerable) {
        super(Token.LPAREN);
        this.child = child;
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.child.evaluate(scope);
    }

    getChildren(): IChildrenEnumerable[] {
        return [this.child];
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        return this.child.compile(scope);
    }
}

export class IdentifierASTNode extends ASTNode {
    constructor(value: string) {
        super(Token.IDENTIFIER);
        this.value = value;
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getVariable(this.value);
    }

    getChildren(): ASTNode[] {
        return [];
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        const address = scope.getVariableAddress(this.value);
        return new LoadInstruction(address);
    }
}

export class AssignASTNode extends ASTNode {
    public identifier: IdentifierASTNode;

    public valueAST: ASTNode;

    constructor(left: IdentifierASTNode, right: ASTNode) {
        super(Token.ASSIGN);
        this.identifier = left;
        this.valueAST = right;
    }

    evaluate(scope: Scope, isSignal = false): RuntimeValue {
        const evaluatedValue = this.valueAST.evaluate(scope);
        if (!isSignal) {
            scope.setVariable(this.identifier.getValue(), evaluatedValue);
        }
        return evaluatedValue;
    }

    getChildren(): ASTNode[] {
        return [this.identifier, this.valueAST];
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        const address = scope.getVariableAddress(this.identifier.getValue());
        const valueInstructions = [this.valueAST.compile(scope)].flat();

        return valueInstructions.concat(new StoreInstruction(address));
    }
}

export class DeclarationASTNode extends ASTNode {
    public child: AssignASTNode | IdentifierASTNode;

    constructor(child: AssignASTNode | IdentifierASTNode) {
        super(Token.DECLERATION);
        this.child = child;
    }

    evaluate(scope: Scope): RuntimeValue {
        if (this.child.isType(Token.IDENTIFIER)) {
            scope.addVariable(this.child.getValue(), undefined);
            return null;
        }

        if (this.child.isType(Token.ASSIGN)) {
            const { identifier, valueAST } = this.child as AssignASTNode;

            // Special case for lambdas
            // Yes this should be a token.lambda but I'm lazy
            if (valueAST.isType(Token.FUNCTION)) {
                valueAST.evaluate(scope);
                return undefined;
            }

            const evaluatedValue = valueAST.evaluate(scope);
            scope.addVariable(identifier.getValue(), evaluatedValue);
            return evaluatedValue;
        }

        if (this.child.isOneOf(Token.SIGNAL_ASSIGN, Token.COMPUTE_ASSIGN)) {
            return this.child.evaluate(scope);
        }

        throw new TannerError(
            `Unexpectd AST Type as child for decl. Got: ${this.child.getType()}`,
        );
    }

    getChildren(): ASTNode[] {
        return [this.child];
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        // The allocation is already handled by hoisting in the block scope

        if (this.child instanceof IdentifierASTNode) {
            scope.addVariable(this.child.getValue());
        } else {
            scope.addVariable(this.child.identifier.getValue());
        }

        if (this.child instanceof AssignASTNode) {
            return this.child.compile(scope);
        }

        return [];
    }
}

export class StringASTNode extends ASTNode {
    constructor(value: string) {
        super(Token.STRING);
        this.value = value;
    }

    evaluate(): string {
        return this.value;
    }

    getChildren(): ASTNode[] {
        return [];
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        return this.value
            .split('')
            .map((char) => {
                const asciiValue = char.charCodeAt(0);
                return new PushInstruction(asciiValue);
            })
            .reverse();
    }
}

export class BlockASTNode extends ASTNode {
    public children: ASTNode[];

    constructor(children: ASTNode[]) {
        super(Token.LCURLY);
        this.children = children;
    }

    evaluate(scope: Scope): RuntimeValue {
        const newScope = new Scope(scope.globalScope, scope);
        let retValue: RuntimeValue = undefined;

        for (const statement of this.children) {
            if (scope.isReturning()) {
                return scope.getReturnValue();
            }
            retValue = statement.evaluate(newScope);
        }

        return retValue;
    }

    addChild(node: ASTNode): void {
        this.children.push(node);
    }

    getChildren(): ASTNode[] {
        return this.children;
    }

    setChildren(children: ASTNode[]): void {
        this.children = children;
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        const blockScope = new CompileScope(scope);
        let instructions = this.children
            .map((child) => child.compile(blockScope))
            .flat()
            .filter(Boolean);

        const totalFunctionLengths = blockScope.getTotalFunctionSize();
        if (totalFunctionLengths > 0) {
            instructions = [
                new JumpInstruction(totalFunctionLengths),
                ...instructions,
            ];
        }

        const numVars = blockScope.getNumVariables(true);
        if (numVars !== 0) {
            const allocs = new AllocInstruction(numVars);
            const unallocs = new AllocInstruction(-numVars);
            instructions = [allocs, ...instructions, unallocs];
        }

        return instructions;
    }
}
