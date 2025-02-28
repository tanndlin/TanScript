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

        if (child instanceof AST.ASTMath) {
            child = Optimizer.simplifyMathExpression(child);
        }

        if (
            child.type === Token.TRUE ||
            child.type === Token.FALSE ||
            child.type === Token.NUMBER ||
            child.type === Token.NOT ||
            child instanceof AST.ASTComparison
        ) {
            child = Optimizer.simplifyLogicalExpression(child);
        }

        if (child instanceof AST.ASTFunctionCall) {
            child = Optimizer.optimizeFunctionCallArgs(child);
        }

        return child;
    }

    private static optimizeIf(
        node: AST.ASTIf,
    ): AST.ASTBlock | AST.ASTIf | undefined {
        let { condition, block, elseBlock } = node;

        condition = Optimizer.optimizeExpression(condition);

        if (
            condition.type === Token.TRUE ||
            (condition.type === Token.NUMBER &&
                +(condition as AST.ASTNumber).getValue() !== 0)
        ) {
            return block;
        } else if (condition.type === Token.FALSE) {
            return elseBlock;
        }

        return new AST.ASTIf(condition, block, elseBlock);
    }

    private static optimizeFor(node: AST.ASTFor): AST.ASTFor {
        let { init, condition, update, block } = node;

        init = Optimizer.optimizeAny(init);
        condition = Optimizer.optimizeExpression(condition);
        update = Optimizer.optimizeAny(update);
        block = Optimizer.optimizeAny(block) as AST.ASTBlock;

        return new AST.ASTFor(init, condition, update, block);
    }

    private static optimizeFunctionCallArgs(
        node: AST.ASTFunctionCall,
    ): AST.ASTFunctionCall {
        const { args } = node;
        return new AST.ASTFunctionCall(
            node.getName(),
            args.map(Optimizer.optimizeExpression),
        );
    }

    private static simplifyLogicalExpression(
        node:
            | AST.ASTBoolean
            | AST.ASTNumber
            | AST.ASTComparison
            | AST.ASTMath
            | AST.ASTNot,
    ) {
        const type: Token = node.type;
        if (node instanceof AST.ASTBoolean) {
            return node;
        }

        if (node.type === Token.NUMBER) {
            return node;
        }

        if (node instanceof AST.ASTNot) {
            let { child } = node;
            child = Optimizer.optimizeExpression(child);
            if (child.type === Token.TRUE) {
                return new AST.ASTBoolean(Token.FALSE);
            }
            if (child.type === Token.FALSE) {
                return new AST.ASTBoolean(Token.TRUE);
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
            const leftNum = (leftValue as AST.ASTNumber).getValue();
            const rightNum = (rightValue as AST.ASTNumber).getValue();

            switch (type) {
                case Token.LESS:
                    return new AST.ASTBoolean(
                        leftNum < rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.LEQ:
                    return new AST.ASTBoolean(
                        leftNum <= rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.GREATER:
                    return new AST.ASTBoolean(
                        leftNum > rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.GEQ:
                    return new AST.ASTBoolean(
                        leftNum >= rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.EQUAL:
                    return new AST.ASTBoolean(
                        leftNum === rightNum ? Token.TRUE : Token.FALSE,
                    );
                case Token.NEQ:
                    return new AST.ASTBoolean(
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
                    return new AST.ASTBoolean(
                        leftBool && rightBool ? Token.TRUE : Token.FALSE,
                    );
                case Token.OR:
                    return new AST.ASTBoolean(
                        leftBool || rightBool ? Token.TRUE : Token.FALSE,
                    );
            }
        }

        node.left = leftValue as INumberableAST;
        node.right = rightValue as INumberableAST;
        return node;
    }

    private static simplifyMathExpression(
        node: AST.ASTMathType | AST.ASTNumber,
    ): AST.ASTMathType | AST.ASTNumber {
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
            const leftNum = (leftValue as AST.ASTNumber).getValue();
            const rightNum = (rightValue as AST.ASTNumber).getValue();

            switch (node.type) {
                case Token.PLUS:
                    return new AST.ASTNumber(leftNum + rightNum);
                case Token.MINUS:
                    return new AST.ASTNumber(leftNum - rightNum);
                case Token.MULTIPLY:
                    return new AST.ASTNumber(leftNum * rightNum);
                case Token.DIVIDE:
                    return new AST.ASTNumber(leftNum / rightNum);
                case Token.MOD:
                    return new AST.ASTNumber(leftNum % rightNum);
            }
        }

        node.left = leftValue;
        node.right = rightValue;
        return node;
    }

    private static simplifyParenthesis(node: AST.ASTLParen) {
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
