import { AST, ASTNode, RootASTNode } from './AST/AST';
import { BooleanASTNode } from './AST/BoolAST';
import { IfASTNode } from './AST/ControlAST';
import { Maybe, Token } from './types';

export default class Optimizer {
    public static optimize(ast: AST): AST {
        const root = ast.getRoot();
        Optimizer.optimizeNode(root);

        return ast;
    }

    private static optimizeNode(node: RootASTNode): RootASTNode {
        const children = node.getChildren();
        const newChildren = children.map((child) => {
            if (
                child.isOneOf(
                    Token.LESS,
                    Token.LEQ,
                    Token.GREATER,
                    Token.GEQ,
                    Token.EQUAL,
                    Token.NEQ
                )
            ) {
                child = Optimizer.simplifyLogicalExpression(child);
            }

            if (child.isType(Token.IF)) {
                return Optimizer.optimizeIf(child);
            }

            return child;
        });

        node.setChildren(newChildren.filter(Boolean) as ASTNode[]);
        return node;
    }

    private static optimizeIf(node: IfASTNode): Maybe<ASTNode> {
        let [condition, ifBlock, elseBlock] = node.getChildren();

        condition = Optimizer.simplifyLogicalExpression(condition);

        if (
            condition.isType(Token.TRUE) ||
            (condition.isType(Token.NUMBER) && +condition.getValue() !== 0)
        ) {
            return ifBlock;
        } else if (condition.isType(Token.FALSE)) {
            return elseBlock;
        }

        return new IfASTNode(condition, ifBlock, elseBlock);
    }

    private static simplifyLogicalExpression(node: ASTNode): ASTNode {
        const type: Token = node.getType();
        if (type == Token.NUMBER || type == Token.TRUE || type == Token.FALSE) {
            return node;
        }

        // Simplify left and right nodes
        const [left, right] = node.getChildren();
        const leftValue = Optimizer.simplifyLogicalExpression(left);
        const rightValue = Optimizer.simplifyLogicalExpression(right);

        // If both are numbers, evaluate the expression
        if (leftValue.isType(Token.NUMBER) && rightValue.isType(Token.NUMBER)) {
            const leftNum = +leftValue.getValue();
            const rightNum = +rightValue.getValue();

            switch (type) {
                case Token.LESS:
                    return new BooleanASTNode(
                        leftNum < rightNum ? Token.TRUE : Token.FALSE
                    );
                case Token.LEQ:
                    return new BooleanASTNode(
                        leftNum <= rightNum ? Token.TRUE : Token.FALSE
                    );
                case Token.GREATER:
                    return new BooleanASTNode(
                        leftNum > rightNum ? Token.TRUE : Token.FALSE
                    );
                case Token.GEQ:
                    return new BooleanASTNode(
                        leftNum >= rightNum ? Token.TRUE : Token.FALSE
                    );
                case Token.EQUAL:
                    return new BooleanASTNode(
                        leftNum === rightNum ? Token.TRUE : Token.FALSE
                    );
                case Token.NEQ:
                    return new BooleanASTNode(
                        leftNum !== rightNum ? Token.TRUE : Token.FALSE
                    );
            }
        }

        node.setChildren([leftValue, rightValue]);
        return node;
    }
}
