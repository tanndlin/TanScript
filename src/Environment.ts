import { AST, ASTNode } from './AST';
import Scope from './Scope';
import { RuntimeValue, Token } from './types';

export default class Environment {
    private globalScope: Scope;

    constructor(private ast: AST) {
        this.globalScope = new Scope(null);
    }

    public evaluate(): RuntimeValue {
        let retValue: RuntimeValue = null;

        const root = this.ast.getRoot();
        const statements = root.getChildren();
        statements.forEach((statement, i) => {
            if (statement.getType() === Token.EOF) return;

            console.log(`Statement ${i + 1}:`);
            retValue = this.evaluateNode(statement, this.globalScope);
            console.log(retValue);
        });

        return retValue;
    }

    private evalExpression<T>(
        node: ASTNode,
        scope: Scope,
        cb: (a: T, b: T) => RuntimeValue
    ): RuntimeValue {
        const [left, right] = node.getChildren();
        return cb(
            this.evaluateNode(left, scope) as T,
            this.evaluateNode(right, scope) as T
        );
    }

    private evaluateNode(node: ASTNode, scope: Scope): RuntimeValue {
        if (!node) return null;

        switch (node.getType()) {
            case Token.NUMBER:
                return parseInt(node.getValue());
            case Token.PLUS:
                return this.evalExpression<number>(
                    node,
                    scope,
                    (a, b) => a + b
                );
            case Token.MINUS:
                return this.evalExpression<number>(
                    node,
                    scope,
                    (a, b) => a - b
                );
            case Token.MULTIPLY:
                return this.evalExpression<number>(
                    node,
                    scope,
                    (a, b) => a * b
                );
            case Token.DIVIDE:
                return this.evalExpression<number>(
                    node,
                    scope,
                    (a, b) => a / b
                );
            case Token.LESS:
                return this.evalExpression<number>(node, scope, (a, b) =>
                    a < b ? 1 : 0
                );
            case Token.GREATER:
                return this.evalExpression<number>(node, scope, (a, b) =>
                    a > b ? 1 : 0
                );

            case Token.LPAREN:
                return this.evaluateNode(node.getChildren()[0], scope);
            case Token.DECLERATION: {
                const [assignment] = node.getChildren();
                const [identifier, value] = assignment.getChildren();

                const evaluatedValue = this.evaluateNode(value, scope);
                scope.addVariable(identifier.getValue(), evaluatedValue);
                return evaluatedValue;
            }
            case Token.ASSIGN: {
                const [identifier, value] = node.getChildren();
                const evaluatedValue = this.evaluateNode(value, scope);
                scope.setVariable(identifier.getValue(), evaluatedValue);
                return evaluatedValue;
            }
            case Token.IDENTIFIER: {
                return scope.getVariable(node.getValue());
            }
            case Token.LCURLY: {
                const newScope = new Scope(scope);
                const statements = node.getChildren();

                let retValue: RuntimeValue = null;
                statements.forEach(
                    (statement) =>
                        (retValue = this.evaluateNode(statement, newScope))
                );

                return retValue;
            }
            case Token.WHILE: {
                const [condition, body] = node.getChildren();
                let retValue: RuntimeValue = null;

                while (this.evaluateNode(condition, scope)) {
                    retValue = this.evaluateNode(body, scope);
                }

                return retValue;
            }
            case Token.SEMI:
            case Token.EOF:
                return;

            default:
                throw new Error(`Unexpected token: ${node.getValue()}`);
        }
    }

    public getGlobalScope(): Scope {
        return this.globalScope;
    }
}
