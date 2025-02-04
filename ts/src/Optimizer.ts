import * as AST from './AST';
import { INumberableAST, Token } from './types';

export default class Optimizer {
    public static optimize(ast: AST.Program): AST.Program {
        const root = ast.getRoot();
        const children = root.children;
        const newChildren = children
            .map((child) => {
                return Optimizer.optimizeAny(child);
            })
            .filter(Boolean) as AST.Stmt[];

        root.setChildren(newChildren);
        return ast;
    }

    private static optimizeAny<T extends AST.AnyAST>(child: T): T {
        if (child.type === Token.IF) {
            return Optimizer.optimizeIf(child)! as T;
        }

        if (child.type === Token.FOR) {
            return Optimizer.optimizeFor(child) as T;
        }

        return Optimizer.optimizeExpression(child) as T;
    }

    private static optimizeExpression(child: AST.Expr): AST.Expr {
        if (child.type === Token.LPAREN) {
            child = Optimizer.simplifyParenthesis(child);
        }

        if (child instanceof AST.MathASTNode) {
            child = Optimizer.simplifyMathExpression(child);
        }

        if (
            child.type === Token.TRUE ||
            child.type === Token.FALSE ||
            child.type === Token.NUMBER ||
            child.type === Token.NOT ||
            child instanceof AST.ComparisonASTNode
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

        if (condition.type === Token.TRUE) {
            console.log(condition);
        }

        if (
            condition.type === Token.TRUE ||
            (condition.type === Token.NUMBER &&
                +(condition as AST.NumberASTNode).getValue() !== 0)
        ) {
            return block;
        } else if (condition.type === Token.FALSE) {
            return elseBlock;
        }

        return new AST.IfASTNode(condition, block, elseBlock);
    }

    private static optimizeFor(node: AST.ForASTNode): AST.ForASTNode {
        let { init, condition, update, block } = node;

        init = Optimizer.optimizeAny(init);
        condition = Optimizer.optimizeExpression(condition);
        update = Optimizer.optimizeAny(update);
        block = Optimizer.optimizeAny(block) as AST.BlockASTNode;

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

        if (node.type === Token.NUMBER) {
            return node;
        }

        if (node instanceof AST.NotASTNode) {
            let { child } = node;
            child = Optimizer.optimizeExpression(child);
            if (child.type === Token.TRUE) {
                return new AST.BooleanASTNode(Token.FALSE);
            }
            if (child.type === Token.FALSE) {
                return new AST.BooleanASTNode(Token.TRUE);
            }

            node.child = child;
            return node;
        }

        // Simplify left and right nodes
        const { left, right } = node;
        const leftValue = Optimizer.optimizeExpression(left as AST.Expr);
        const rightValue = Optimizer.optimizeExpression(right as AST.Expr);

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
            leftValue.type === Token.TRUE || leftValue.type === Token.FALSE;
        const rightisBool =
            rightValue.type === Token.TRUE || rightValue.type === Token.FALSE;
        if (leftisBool && rightisBool) {
            const leftBool = leftValue.type === Token.TRUE;
            const rightBool = rightValue.type === Token.TRUE;

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
        node: AST.MathASTNodeType | AST.NumberASTNode,
    ): AST.MathASTNodeType | AST.NumberASTNode {
        if (node.type === Token.NUMBER) {
            return node;
        }

        const { left, right } = node;
        const leftValue = Optimizer.optimizeExpression(
            left as AST.Expr,
        ) as INumberableAST;
        const rightValue = Optimizer.optimizeExpression(
            right as AST.Expr,
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

        if (
            child.type === Token.TRUE ||
            child.type === Token.FALSE ||
            child.type === Token.NUMBER
        ) {
            return child;
        }

        node.child = child;
        return node;
    }
}
