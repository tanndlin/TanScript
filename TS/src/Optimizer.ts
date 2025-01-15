import * as AST from './AST';
import { IHasChildren, INumberableAST, Maybe, Token } from './types';

export default class Optimizer {
    public static optimize(ast: AST.AST): AST.AST {
        const root = ast.getRoot();
        const children = root.getChildren();
        const newChildren = children
            .map((child) => {
                return Optimizer.optimizeNode(child);
            })
            .filter(Boolean) as AST.ASTNode[];

        root.setChildren(newChildren);
        return ast;
    }

    private static optimizeNode(child: AST.ASTNode): AST.ASTNode {
        if (child instanceof AST.LParenASTNode) {
            child = Optimizer.simplifyParenthesis(child);
        }

        if (child instanceof AST.MathASTNode) {
            child = Optimizer.simplifyMathExpression(child);
        }

        if (
            child instanceof AST.BooleanASTNode ||
            child instanceof AST.NumberASTNode ||
            child instanceof AST.ComparisonASTNode ||
            child instanceof AST.NotASTNode
        ) {
            child = Optimizer.simplifyLogicalExpression(child);
        }

        if (child instanceof AST.IfASTNode) {
            return Optimizer.optimizeIf(child)!;
        }

        if (child instanceof AST.ForASTNode) {
            return Optimizer.optimizeFor(child);
        }

        if (child instanceof AST.FunctionCallASTNode) {
            return Optimizer.optimizeFunctionCallArgs(child);
        }

        return child;
    }

    private static optimizeIf(node: AST.IfASTNode): Maybe<AST.ASTNode> {
        let { condition, block, elseBlock } = node;

        condition = Optimizer.optimizeNode(condition);

        if (
            condition.type === Token.TRUE ||
            (condition.type === Token.NUMBER &&
                +(condition as AST.NumberASTNode).getValue() !== 0)
        ) {
            return block;
        } else if (condition.isType(Token.FALSE)) {
            return elseBlock;
        }

        return new AST.IfASTNode(condition, block, elseBlock);
    }

    private static optimizeFor(node: AST.ForASTNode): AST.ASTNode {
        let { init, condition, update, block } = node;

        init = Optimizer.optimizeNode(init);
        condition = Optimizer.optimizeNode(condition);
        update = Optimizer.optimizeNode(update);
        block = Optimizer.optimizeNode(block);

        return new AST.ForASTNode(init, condition, update, block);
    }

    private static optimizeFunctionCallArgs(
        node: AST.FunctionCallASTNode,
    ): AST.ASTNode {
        const { args } = node;
        return new AST.FunctionCallASTNode(
            node.getName(),
            args.map(Optimizer.optimizeNode),
        );
    }

    private static simplifyLogicalExpression(
        node:
            | AST.BooleanASTNode
            | AST.NumberASTNode
            | AST.ComparisonASTNode
            | AST.MathASTNode
            | AST.NotASTNode,
    ): AST.ASTNode {
        const type: Token = node.type;
        if (node instanceof AST.BooleanASTNode) {
            return node;
        }

        if (node instanceof AST.NumberASTNode) {
            return node;
        }

        if (node instanceof AST.NotASTNode) {
            let { child } = node;
            child = Optimizer.optimizeNode(child);
            if (child.isType(Token.TRUE)) {
                return new AST.BooleanASTNode(Token.FALSE);
            }
            if (child.isType(Token.FALSE)) {
                return new AST.BooleanASTNode(Token.TRUE);
            }

            node.child = child;
            return node;
        }

        // Simplify left and right nodes
        const { left, right } = node;
        const leftValue = Optimizer.optimizeNode(left);
        const rightValue = Optimizer.optimizeNode(right);

        // If both are numbers, evaluate the expression
        if (
            leftValue.type === Token.NUMBER &&
            rightValue.type === Token.NUMBER
        ) {
            const leftNum = (leftValue as AST.NumberASTNode).getValue();
            const rightNum = (rightValue as AST.NumberASTNode).getValue();

            switch (type) {
                case Token.LESS:
                    return new AST.BooleanASTNode(
                        leftNum < rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.LEQ:
                    return new AST.BooleanASTNode(
                        leftNum <= rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.GREATER:
                    return new AST.BooleanASTNode(
                        leftNum > rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.GEQ:
                    return new AST.BooleanASTNode(
                        leftNum >= rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.EQUAL:
                    return new AST.BooleanASTNode(
                        leftNum === rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.NEQ:
                    return new AST.BooleanASTNode(
                        leftNum !== rightNum ? Token.TRUE : Token.FALSE,
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
                    return new AST.BooleanASTNode(
                        leftBool && rightBool ? Token.TRUE : Token.FALSE,
                    );
                case Token.OR:
                    return new AST.BooleanASTNode(
                        leftBool || rightBool ? Token.TRUE : Token.FALSE,
                    );
            }
        }

        node.left = leftValue as INumberableAST;
        node.right = rightValue as INumberableAST;
        return node;
    }

    private static simplifyMathExpression(node: AST.MathASTNode): AST.ASTNode {
        if (node instanceof AST.NumberASTNode) {
            return node;
        }

        const { left, right } = node;
        const leftValue = Optimizer.optimizeNode(left) as INumberableAST;
        const rightValue = Optimizer.optimizeNode(right) as INumberableAST;

        if (
            !AST.NumberASTNode.isNumberAST(leftValue) &&
            !AST.NumberASTNode.isNumberAST(rightValue)
        ) {
            node.left = leftValue;
            node.right = rightValue;
            return node;
        }

        if (
            AST.NumberASTNode.isNumberAST(leftValue) &&
            AST.NumberASTNode.isNumberAST(rightValue)
        ) {
            const leftNum = leftValue.getValue();
            const rightNum = rightValue.getValue();

            switch (node.type) {
                case Token.PLUS:
                    return new AST.NumberASTNode(leftNum + rightNum);
                case Token.MINUS:
                    return new AST.NumberASTNode(leftNum - rightNum);
                case Token.MULTIPLY:
                    return new AST.NumberASTNode(leftNum * rightNum);
                case Token.DIVIDE:
                    return new AST.NumberASTNode(leftNum / rightNum);
                case Token.MOD:
                    return new AST.NumberASTNode(leftNum % rightNum);
            }
        }

        node.left = leftValue;
        node.right = rightValue;
        return node;
    }

    private static simplifyParenthesis(node: AST.LParenASTNode): AST.ASTNode {
        let { child } = node;
        child = this.optimizeNode(child) as IHasChildren;
        child = this.optimizeNode(child) as IHasChildren;

        if (child.isOneOf(Token.TRUE, Token.FALSE, Token.NUMBER)) {
            return child;
        }

        node.child = child;
        return node;
    }
}
