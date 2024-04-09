import Scope from '../Scope';
import { TannerError } from '../errors';
import { RuntimeValue, Token } from '../types';

export class AST {
    constructor(private root: RootASTNode) {}

    getRoot() {
        return this.root;
    }
}

export class RootASTNode {
    protected children: ASTNode[];

    constructor(children?: ASTNode[]) {
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

    constructor(type: Token, value: string, children: ASTNode[]) {
        super(children);
        this.type = type;
        this.value = value;
    }

    abstract evaluate(scope: Scope): RuntimeValue;

    public getType(): Token {
        return this.type;
    }

    public getValue(): string {
        return this.value;
    }
}

export class EOFASTNode extends ASTNode {
    constructor() {
        super(Token.EOF, '', []);
    }

    evaluate(scope: Scope): RuntimeValue {}
}

export class LParenASTNode extends ASTNode {
    constructor() {
        super(Token.LPAREN, '(', []);
    }

    evaluate(scope: Scope): RuntimeValue {
        const [child] = this.getChildren();
        return child.evaluate(scope);
    }
}

export class RParenASTNode extends ASTNode {
    constructor() {
        super(Token.RPAREN, ')', []);
    }

    evaluate(scope: Scope): RuntimeValue {
        throw new TannerError('Unexpected RParen.eval');
    }
}

export class SemiASTNode extends ASTNode {
    constructor() {
        super(Token.SEMI, ';', []);
    }

    evaluate(scope: Scope): RuntimeValue {
        throw new TannerError('Unexpected Semi.eval');
    }
}

export class DeclarationASTNode extends ASTNode {
    constructor() {
        super(Token.DECLERATION, 'let', []);
    }

    evaluate(scope: Scope): RuntimeValue {
        const [assignment] = this.getChildren();
        const [identifier, value] = assignment.getChildren();

        const evaluatedValue = value.evaluate(scope);
        scope.addVariable(identifier.getValue(), evaluatedValue);
        return evaluatedValue;
    }
}

export class IdentifierASTNode extends ASTNode {
    constructor(value: string) {
        super(Token.IDENTIFIER, value, []);
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getVariable(this.value);
    }
}

export class AssignASTNode extends ASTNode {
    constructor(left: ASTNode, right: ASTNode) {
        super(Token.ASSIGN, '=', [left, right]);
    }

    evaluate(scope: Scope): RuntimeValue {
        const [identifier, value] = this.getChildren();
        const evaluatedValue = value.evaluate(scope);
        scope.setVariable(identifier.getValue(), evaluatedValue);
        return evaluatedValue;
    }
}

export class BlockASTNode extends ASTNode {
    constructor(children: ASTNode[]) {
        super(Token.LCURLY, '{', children);
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
