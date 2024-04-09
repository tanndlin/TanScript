import {
    AST,
    ASTNode,
    AssignASTNode,
    BlockASTNode,
    DeclarationASTNode,
    ForASTNode,
    IfASTNode,
    WhileASTNode,
} from './AST';
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

        const evalWrapper = <T>(cb: (a: T, b: T) => RuntimeValue) => {
            return this.evalExpression<T>(node, scope, cb);
        };

        switch (node.getType()) {
            case Token.NUMBER:
                return parseInt(node.getValue());
            case Token.PLUS:
                return evalWrapper<number>((a, b) => a + b);
            case Token.MINUS:
                return evalWrapper<number>((a, b) => a - b);
            case Token.MULTIPLY:
                return evalWrapper<number>((a, b) => a * b);
            case Token.DIVIDE:
                return evalWrapper<number>((a, b) => a / b);
            case Token.LESS:
                return evalWrapper<number>((a, b) => a < b);
            case Token.GREATER:
                return evalWrapper<number>((a, b) => a > b);
            case Token.LEQ:
                return evalWrapper<number>((a, b) => a <= b);
            case Token.GEQ:
                return evalWrapper<number>((a, b) => a >= b);

            case Token.LPAREN:
                return this.evaluateNode(node.getChildren()[0], scope);
            case Token.DECLERATION:
                return this.runDecleration(node, scope);
            case Token.ASSIGN:
                return this.runAssign(node, scope);
            case Token.IDENTIFIER:
                return scope.getVariable(node.getValue());
            case Token.LCURLY:
                return this.runBlock(node, scope);
            case Token.WHILE:
                return this.runWhile(node, scope);
            case Token.FOR:
                return this.runFor(node, scope);
            case Token.IF:
                return this.runIf(node, scope);

            case Token.SEMI:
            case Token.EOF:
                return;

            default:
                throw new Error(`Unexpected token: ${node.getValue()}`);
        }
    }

    private runDecleration(node: DeclarationASTNode, scope: Scope) {
        const [assignment] = node.getChildren();
        const [identifier, value] = assignment.getChildren();

        const evaluatedValue = this.evaluateNode(value, scope);
        scope.addVariable(identifier.getValue(), evaluatedValue);
        return evaluatedValue;
    }

    private runAssign(node: AssignASTNode, scope: Scope) {
        const [identifier, value] = node.getChildren();
        const evaluatedValue = this.evaluateNode(value, scope);
        scope.setVariable(identifier.getValue(), evaluatedValue);
        return evaluatedValue;
    }

    private runBlock(node: BlockASTNode, scope: Scope) {
        const newScope = new Scope(scope);
        const statements = node.getChildren();

        let retValue: RuntimeValue = null;
        statements.forEach(
            (statement) => (retValue = this.evaluateNode(statement, newScope))
        );

        return retValue;
    }

    private runWhile(node: WhileASTNode, scope: Scope) {
        const [condition, body] = node.getChildren();
        let retValue: RuntimeValue = null;

        while (this.evaluateNode(condition, scope)) {
            retValue = this.evaluateNode(body, scope);
        }

        return retValue;
    }

    private runFor(node: ForASTNode, scope: Scope) {
        const [init, condition, update, body] = node.getChildren();
        let retValue: RuntimeValue = null;

        // The declared iter var should be in the for loop scope
        const newScope = new Scope(scope);

        this.evaluateNode(init, newScope);
        while (this.evaluateNode(condition, newScope)) {
            retValue = this.evaluateNode(body, newScope);
            this.evaluateNode(update, newScope);
        }

        return retValue;
    }

    private runIf(node: IfASTNode, scope: Scope) {
        const [condition, body, elseBody] = node.getChildren();
        const retValue = this.evaluateNode(condition, scope)
            ? this.evaluateNode(body, scope)
            : this.evaluateNode(elseBody, scope);

        return retValue;
    }

    public getGlobalScope(): Scope {
        return this.globalScope;
    }
}
