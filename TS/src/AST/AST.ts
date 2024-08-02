import Scope from '../Scope';
import { TannerError } from '../errors';
import { RuntimeValue, Token, TokenTypeable } from '../types';

export class AST {
    constructor(private root: RootASTNode) {}

    getRoot() {
        return this.root;
    }
}

export class RootASTNode extends TokenTypeable {
    protected children: ASTNode[];

    constructor(children?: ASTNode[]) {
        super(Token.ROOT);
        this.children = children ?? [];
    }

    addChild(node: ASTNode) {
        this.children.push(node);
    }

    public getChildren(): ASTNode[] {
        return this.children;
    }
}

export abstract class ASTNode extends RootASTNode {
    protected type: Token;
    protected value: string;

    constructor(type: Token, children: ASTNode[]) {
        super(children);
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
        super(token, []);
    }

    evaluate(scope: Scope): RuntimeValue {
        throw new TannerError('Unexpected call to DecoratorASTNode.evaluate');
    }

    addChild(node: ASTNode): void {
        throw new TannerError('Unexpected call to DecoratorASTNode.addChild');
    }
}

export class EOFASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.EOF);
    }
}

export class RParenASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.RPAREN);
    }
}

export class SemiASTNode extends DecoratorASTNode {
    constructor() {
        super(Token.SEMI);
    }
}

export class LParenASTNode extends ASTNode {
    constructor(children: ASTNode[]) {
        super(Token.LPAREN, children);
    }

    evaluate(scope: Scope): RuntimeValue {
        const [child] = this.getChildren();
        return child.evaluate(scope);
    }
}

export class DeclarationASTNode extends ASTNode {
    constructor(assign: AssignASTNode | IdentifierASTNode) {
        super(Token.DECLERATION, [assign]);
    }

    evaluate(scope: Scope): RuntimeValue {
        const [child] = this.getChildren();
        if (child.isType(Token.IDENTIFIER)) {
            scope.addVariable(child.getValue(), undefined);
            return null;
        }

        if (child instanceof AssignASTNode) {
            const [identifier, value] = child.getChildren();

            // Special case for lambdas
            // Yes this should be a token.lambda but I'm lazy
            if (value.isType(Token.FUNCTION)) {
                value.evaluate(scope);
                return undefined;
            }

            const evaluatedValue = value.evaluate(scope);
            scope.addVariable(identifier.getValue(), evaluatedValue);
            return evaluatedValue;
        }

        if (child.isOneOf(Token.SIGNAL_ASSIGN, Token.COMPUTE_ASSIGN)) {
            return child.evaluate(scope);
        }

        throw new TannerError(
            `Unexpectd AST Type as child for decl. Got: ${child.getType()}`
        );
    }
}

export class IdentifierASTNode extends ASTNode {
    constructor(value: string) {
        super(Token.IDENTIFIER, []);
        this.value = value;
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getVariable(this.value);
    }
}

export class StringASTNode extends ASTNode {
    constructor(value: string) {
        super(Token.STRING, []);
        this.value = value;
    }

    evaluate(): string {
        return this.value;
    }
}

export class AssignASTNode extends ASTNode {
    constructor(left: IdentifierASTNode, right: ASTNode) {
        super(Token.ASSIGN, [left, right]);
    }

    evaluate(scope: Scope, isSignal = false): RuntimeValue {
        const [identifier, value] = this.getChildren();
        const evaluatedValue = value.evaluate(scope);
        if (!isSignal) scope.setVariable(identifier.getValue(), evaluatedValue);
        return evaluatedValue;
    }
}

export class BlockASTNode extends ASTNode {
    constructor(children: ASTNode[]) {
        super(Token.LCURLY, children);
    }

    evaluate(scope: Scope): void {
        const newScope = new Scope(scope);
        let retValue;

        this.getChildren().forEach((statement) => {
            retValue = statement.evaluate(newScope);
        });

        return retValue;
    }
}
