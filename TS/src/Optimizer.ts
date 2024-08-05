import { AST, ASTNode } from './AST/AST';
import { BooleanASTNode } from './AST/BoolAST';
import { ForASTNode, IfASTNode } from './AST/ControlAST';
import { NumberASTNode } from './AST/NumberAST';
import { Maybe, Token } from './types';

export default class Optimizer {
    public static optimize(ast: AST): AST {
        const root = ast.getRoot();
        const children = root.getChildren();
        const newChildren = children
            .map((child) => {
                return Optimizer.optimizeNode(child);
            })
            .filter(Boolean) as ASTNode[];

        root.setChildren(newChildren);
        return ast;
    }

    private static optimizeNode(child: ASTNode): ASTNode {
        if (child.isType(Token.LPAREN)) {
            child = Optimizer.simplifyParenthesis(child);
        }

        if (
            child.isOneOf(
                Token.PLUS,
                Token.MINUS,
                Token.MULTIPLY,
                Token.DIVIDE,
                Token.MOD
            )
        ) {
            child = Optimizer.simplifyMathExpression(child);
        }

        if (
            child.isOneOf(
                Token.LESS,
                Token.LEQ,
                Token.GREATER,
                Token.GEQ,
                Token.EQUAL,
                Token.NEQ,
                Token.AND,
                Token.OR,
                Token.NOT
            )
        ) {
            child = Optimizer.simplifyLogicalExpression(child);
        }

        if (child.isType(Token.IF)) {
            return Optimizer.optimizeIf(child)!;
        }

        if (child instanceof ForASTNode) {
            return Optimizer.optimizeFor(child);
        }

        return child;
    }

    private static optimizeIf(node: IfASTNode): Maybe<ASTNode> {
        let [condition, ifBlock, elseBlock] = node.getChildren();

        condition = Optimizer.optimizeNode(condition);

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

    private static optimizeFor(node: ForASTNode): ASTNode {
        let { init, condition, update, block } = node;

        init = Optimizer.optimizeNode(init);
        condition = Optimizer.optimizeNode(condition);
        update = Optimizer.optimizeNode(update);
        block = Optimizer.optimizeNode(block);

        return new ForASTNode(init, condition, update, block);
    }

    private static simplifyLogicalExpression(node: ASTNode): ASTNode {
        const type: Token = node.getType();
        if (type == Token.TRUE || type == Token.FALSE) {
            return node;
        }

        if (node.isType(Token.NUMBER)) {
            return node;
        }

        if (type == Token.NOT) {
            let [child] = node.getChildren();
            child = Optimizer.optimizeNode(child);
            if (child.isType(Token.TRUE))
                return new BooleanASTNode(Token.FALSE);
            if (child.isType(Token.FALSE))
                return new BooleanASTNode(Token.TRUE);

            node.setChildren([child]);
            return node;
        }

        // Simplify left and right nodes
        const [left, right] = node.getChildren();
        const leftValue = Optimizer.optimizeNode(left);
        const rightValue = Optimizer.optimizeNode(right);

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

        const leftisBool =
            leftValue.isType(Token.TRUE) || leftValue.isType(Token.FALSE);
        const rightisBool =
            rightValue.isType(Token.TRUE) || rightValue.isType(Token.FALSE);
        if (leftisBool && rightisBool) {
            const leftBool = leftValue.isType(Token.TRUE);
            const rightBool = rightValue.isType(Token.TRUE);

            switch (type) {
                case Token.AND:
                    return new BooleanASTNode(
                        leftBool && rightBool ? Token.TRUE : Token.FALSE
                    );
                case Token.OR:
                    return new BooleanASTNode(
                        leftBool || rightBool ? Token.TRUE : Token.FALSE
                    );
            }
        }

        node.setChildren([leftValue, rightValue]);
        return node;
    }

    private static simplifyMathExpression(node: ASTNode): ASTNode {
        if (node.isType(Token.NUMBER)) {
            return node;
        }

        if (
            !node.isOneOf(
                Token.PLUS,
                Token.MINUS,
                Token.MULTIPLY,
                Token.DIVIDE,
                Token.MOD
            )
        ) {
            return node;
        }

        const [left, right] = node.getChildren();
        const leftValue = Optimizer.optimizeNode(left);
        const rightValue = Optimizer.optimizeNode(right);

        const leftIsNumber = leftValue.isType(Token.NUMBER);
        const rightIsNumber = rightValue.isType(Token.NUMBER);
        if (!leftIsNumber && !rightIsNumber) {
            node.setChildren([leftValue, rightValue]);
            return node;
        }

        if (leftValue.isType(Token.NUMBER) && rightValue.isType(Token.NUMBER)) {
            const leftNum = +leftValue.getValue();
            const rightNum = +rightValue.getValue();

            switch (node.getType()) {
                case Token.PLUS:
                    return new NumberASTNode(leftNum + rightNum + '');
                case Token.MINUS:
                    return new NumberASTNode(leftNum - rightNum + '');
                case Token.MULTIPLY:
                    return new NumberASTNode(leftNum * rightNum + '');
                case Token.DIVIDE:
                    return new NumberASTNode(leftNum / rightNum + '');
                case Token.MOD:
                    return new NumberASTNode((leftNum % rightNum) + '');
            }
        }

        node.setChildren([leftValue, rightValue]);
        return node;
    }

    private static simplifyParenthesis(node: ASTNode): ASTNode {
        let [child] = node.getChildren();
        child = this.optimizeNode(child);
        child = this.optimizeNode(child);

        if (child.isOneOf(Token.TRUE, Token.FALSE, Token.NUMBER)) {
            return child;
        }

        node.setChildren([child]);
        return node;
    }
}
