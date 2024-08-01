import Scope from '../Scope';
import { Object, RuntimeValue, Token } from '../types';
import { ASTNode } from './AST';

export class ObjectAST extends ASTNode {
    constructor(children: AttributeAST[]) {
        super(Token.LCURLY, children);
    }

    evaluate(scope: Scope): RuntimeValue {
        const obj: Object = { attributes: {}, methods: {} };
        this.children.forEach((attribute: AttributeAST) => {
            obj.attributes[attribute.getValue()] = attribute.evaluate(scope);
        });

        return obj;
    }
}

export class AttributeAST extends ASTNode {
    constructor(name: string, value: ASTNode) {
        super(Token.IDENTIFIER, [value]);
        this.value = name;
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.children[0].evaluate(scope);
    }
}
