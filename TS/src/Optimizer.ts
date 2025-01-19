import * as AST from './AST';
import { INumberableAST, Token } from './types';

export default class Optimizer {
    public static optimize(ast: AST.Program): AST.Program {
        const root = ast.getRoot();
        const children = root.children;
        const newChildren = children
            .map((child) => {
                return Optimizer.optimizeStatement(child);
            })
            .filter(Boolean) as AST.Stmt[];

        root.setChildren(newChildren);
        return ast;
    }

    private static optimizeStatement(child: AST.Stmt): AST.Stmt {
        if (child instanceof AST.IfASTNode) {
            child = Optimizer.optimizeIf(child)!;
        }

        if (child instanceof AST.ForASTNode) {
            child = Optimizer.optimizeFor(child);
        }

        if (child instanceof AST.Expr) {
            child = Optimizer.optimizeExpression(child);
        }

        return child;
    }

    private static optimizeExpression(child: AST.Expr): AST.Expr {
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

        if (child instanceof AST.FunctionCallASTNode) {
            child = Optimizer.optimizeFunctionCallArgs(child);
        }

        return child;
    }

    private static optimizeIf(
        node: AST.IfASTNode,
    ): AST.BlockASTNode | AST.IfASTNode | undefined {
        let { condition, block, elseBlock } = node;

        condition = Optimizer.optimizeExpression(condition);

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

    private static optimizeFor(node: AST.ForASTNode): AST.ForASTNode {
        let { init, condition, update, block } = node;

        init = Optimizer.optimizeStatement(init);
        condition = Optimizer.optimizeExpression(condition);
        update = Optimizer.optimizeStatement(update);
        block = Optimizer.optimizeStatement(block) as AST.BlockASTNode;

        return new AST.ForASTNode(init, condition, update, block);
    }

    private static optimizeFunctionCallArgs(
        node: AST.FunctionCallASTNode,
    ): AST.FunctionCallASTNode {
        const { args } = node;
        return new AST.FunctionCallASTNode(
            node.getName(),
            args.map(Optimizer.optimizeExpression),
        );
    }

    private static simplifyLogicalExpression(
        node:
            | AST.BooleanASTNode
            | AST.NumberASTNode
            | AST.ComparisonASTNode
            | AST.MathASTNode
            | AST.NotASTNode,
    ) {
        const type: Token = node.type;
        if (node instanceof AST.BooleanASTNode) {
            return node;
        }

        if (node instanceof AST.NumberASTNode) {
            return node;
        }

        if (node instanceof AST.NotASTNode) {
            let { child } = node;
            child = Optimizer.optimizeExpression(child);
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
        const leftValue = Optimizer.optimizeExpression(left);
        const rightValue = Optimizer.optimizeExpression(right);

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

    private static simplifyMathExpression(
        node: AST.MathASTNode,
    ): INumberableAST {
        if (node instanceof AST.NumberASTNode) {
            return node;
        }

        const { left, right } = node;
        const leftValue = Optimizer.optimizeExpression(left) as INumberableAST;
        const rightValue = Optimizer.optimizeExpression(
            right,
        ) as INumberableAST;

        if (
            leftValue.type !== Token.NUMBER &&
            rightValue.type !== Token.NUMBER
        ) {
            node.left = leftValue;
            node.right = rightValue;
            return node;
        }

        if (
            leftValue.type === Token.NUMBER &&
            rightValue.type === Token.NUMBER
        ) {
            const leftNum = (leftValue as AST.NumberASTNode).getValue();
            const rightNum = (rightValue as AST.NumberASTNode).getValue();

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

    private static simplifyParenthesis(node: AST.LParenASTNode) {
        let { child } = node;
        child = this.optimizeExpression(child);

        if (child.isOneOf(Token.TRUE, Token.FALSE, Token.NUMBER)) {
            return child;
        }

        node.child = child;
        return node;
    }
}
