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

    public getType(): Token {
        return this.type;
    }

    public getValue(): string {
        return this.value;
    }
}

export class DecoratorASTNode extends ASTNode {
    constructor(token: Token) {
        super(token);
    }

    evaluate(scope: Scope): RuntimeValue {
        throw new TannerError('Unexpected call to DecoratorASTNode.evaluate');
    }

    addChild(node: ASTNode): void {
        throw new TannerError('Unexpected call to DecoratorASTNode.addChild');
    }

    getChildren(): ASTNode[] {
        return [];
    }
}

export class EOFASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.EOF);
    }

    evaluate(scope: Scope): RuntimeValue {
        throw new TannerError('Unexpected call to EOF.evaluate');
    }
}

export class RParenASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.RPAREN);
    }

    evaluate(scope: Scope): RuntimeValue {
        throw new TannerError('Unexpected call to RParen.evaluate');
    }
}

export class SemiASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.SEMI);
    }

    evaluate(scope: Scope): RuntimeValue {
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
            `Unexpectd AST Type as child for decl. Got: ${this.child.getType()}`
        );
    }

    getChildren(): ASTNode[] {
        return [this.child];
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
        if (!isSignal)
            scope.setVariable(this.identifier.getValue(), evaluatedValue);
        return evaluatedValue;
    }

    getChildren(): ASTNode[] {
        return [this.identifier, this.valueAST];
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
}
