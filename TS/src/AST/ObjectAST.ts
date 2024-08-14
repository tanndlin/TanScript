import { CompileScope } from '../Compilation/CompileScope';
import Scope from '../Scope';
import { Object, RuntimeValue, Token } from '../types';
import { ASTNode, IdentifierASTNode } from './AST';

export class ObjectASTNode extends ASTNode {
    public attributes: AttributeASTNode[];

    constructor(attributes: AttributeASTNode[]) {
        super(Token.LCURLY);
        this.attributes = attributes;
    }

    evaluate(scope: Scope): RuntimeValue {
        const obj: Object = { attributes: {}, methods: {} };
        this.attributes.forEach((attribute: AttributeASTNode) => {
            obj.attributes[attribute.getValue()] = attribute.evaluate(scope);
        });

        return obj;
    }

    compile(scope: CompileScope): never {
        throw new Error('Method not implemented.');
    }
}

export class AttributeASTNode extends ASTNode {
    public valueAST: ASTNode;

    constructor(name: string, valueAST: ASTNode) {
        super(Token.IDENTIFIER);
        this.value = name;
        this.valueAST = valueAST;
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.valueAST.evaluate(scope);
    }

    compile(scope: CompileScope): never {
        throw new Error('Method not implemented.');
    }
}

export class ObjectAccessAST extends ASTNode {
    public objIdentifier: IdentifierASTNode;

    public attribute: IdentifierASTNode;

    constructor(
        objIdentifier: IdentifierASTNode,
        attribute: IdentifierASTNode,
    ) {
        super(Token.IDENTIFIER);
        this.objIdentifier = objIdentifier;
        this.attribute = attribute;
    }

    evaluate(scope: Scope): RuntimeValue {
        const obj = scope.getVariable<Object>(this.objIdentifier.getValue());

        return obj.attributes[this.attribute.getValue()];
    }

    compile(scope: CompileScope): never {
        throw new Error('Method not implemented.');
    }
}
