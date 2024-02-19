import { AST, ASTNode } from './AST';
import { Token } from './types';

export default class Environment {
    private globalScope: Scope;

    constructor(private ast: AST) {
        this.globalScope = new Scope(null);
    }

    public evaluate() {
        const root = this.ast.getRoot();
        const statements = root.getChildren();
        statements.forEach((statement, i) => {
            console.log(`Statement ${i + 1}:`);
            console.log(this.evaluateNode(statement, this.globalScope));
        });
    }

    private evaluateNode(node: ASTNode, scope: Scope): any {
        if (!node) return null;

        switch (node.getType()) {
            case Token.NUMBER:
                return parseInt(node.getValue());
            case Token.PLUS: {
                const [left, right] = node.getChildren();
                return (
                    this.evaluateNode(left, scope) +
                    this.evaluateNode(right, scope)
                );
            }

            case Token.MINUS: {
                const [left, right] = node.getChildren();
                return (
                    this.evaluateNode(left, scope) -
                    this.evaluateNode(right, scope)
                );
            }
            case Token.MULTIPLY: {
                const [left, right] = node.getChildren();
                return (
                    this.evaluateNode(left, scope) *
                    this.evaluateNode(right, scope)
                );
            }
            case Token.DIVIDE: {
                const [left, right] = node.getChildren();
                return (
                    this.evaluateNode(left, scope) /
                    this.evaluateNode(right, scope)
                );
            }
            case Token.LPAREN:
                return this.evaluateNode(node.getChildren()[0], scope);
            case Token.DECLERATION: {
                const [assignment] = node.getChildren();
                const [identifier, value] = assignment.getChildren();

                const evaluatedValue = this.evaluateNode(value, scope);
                scope.addVariable(identifier.getValue(), evaluatedValue);
                return evaluatedValue;
            }
            case Token.IDENTIFIER: {
                return scope.getVariable(node.getValue());
            }
            case Token.SEMI:
            case Token.EOF:
                return;

            default:
                throw new Error(`Unexpected token: ${node.getValue()}`);
        }
    }
}

class Scope {
    private parent: Scope | null;
    private variables: Map<string, any>;
    private scopes: Map<string, Scope>;

    constructor(parent: Scope | null) {
        this.parent = parent;
        this.variables = new Map();
        this.scopes = new Map();
    }

    getVariable<T>(name: string): T {
        if (this.variables.has(name)) return this.variables.get(name) as T;
        let scope: Scope | null = this.parent;
        while (scope) {
            if (scope.variables.has(name))
                return scope.variables.get(name) as T;
            scope = scope.parent;
        }

        throw new Error(`Variable ${name} not found`);
    }

    addScope(name: string, scope: Scope) {
        this.scopes.set(name, scope);
        scope.parent = this;
    }

    addVariable(name: string, value: any) {
        this.variables.set(name, value);
    }
}
