import { AST, ASTNode, RootASTNode } from './AST/AST';
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
            if (child.isType(Token.IF)) {
                return Optimizer.optimizeIf(child);
            }

            return child;
        });

        node.setChildren(newChildren.filter(Boolean) as ASTNode[]);
        return node;
    }

    private static optimizeIf(node: IfASTNode): Maybe<ASTNode> {
        const [condition, ifBlock, elseBlock] = node.getChildren();

        if (condition.isType(Token.TRUE)) {
            return ifBlock;
        } else if (condition.isType(Token.FALSE)) {
            return elseBlock;
        }

        return node;
    }
}
