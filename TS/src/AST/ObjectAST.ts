import Scope from '../Scope';
import { Object, RuntimeValue, Token } from '../types';
import { ASTNode, IdentifierASTNode } from './AST';

export class ObjectASTNode extends ASTNode {
    constructor(children: AttributeASTNode[]) {
        super(Token.LCURLY, children);
    }

    evaluate(scope: Scope): RuntimeValue {
        const obj: Object = { attributes: {}, methods: {} };
        this.children.forEach((attribute: AttributeASTNode) => {
            obj.attributes[attribute.getValue()] = attribute.evaluate(scope);
        });

        return obj;
    }
}

export class AttributeASTNode extends ASTNode {
    constructor(name: string, value: ASTNode) {
        super(Token.IDENTIFIER, [value]);
        this.value = name;
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.children[0].evaluate(scope);
    }
}

export class ObjectAccessAST extends ASTNode {
    constructor(
        objIdentifier: IdentifierASTNode,
        attribute: IdentifierASTNode
    ) {
        super(Token.IDENTIFIER, [objIdentifier, attribute]);
    }

    evaluate(scope: Scope): RuntimeValue {
        const [objIdentifier, attribute] = this.children;
        const obj = scope.getVariable<Object>(objIdentifier.getValue());

        return obj.attributes[attribute.getValue()];
    }
}
