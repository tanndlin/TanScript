import {
    AST,
    ASTNode,
    AddASTNode,
    DivideASTNode,
    EOFASTNode,
    MultiplyASTNode,
    NumberASTNode,
    SubtractASTNode,
} from './src/AST';
import { LexerToken, Token } from './src/types';

export default class Parser {
    constructor(private tokens: LexerToken[]) {}

    parse(): AST {
        // parse the tokens
        const root = this.parseNext(0);

        return new AST(root);
    }

    private parseNext(pos: number): ASTNode {
        const curToken = this.tokens[pos];
        if (curToken.getType() === Token.EOF) {
            return new EOFASTNode();
        }

        // If the current token is a number, make sure next token is either an operator or EOF
        if (curToken.getType() === Token.NUMBER) {
            const numberAST = new NumberASTNode(curToken.getValue());
            if (pos + 1 >= this.tokens.length) {
                return numberAST;
            }

            const nextToken = this.tokens[pos + 1];
            switch (nextToken.getType()) {
                case Token.PLUS:
                    return new AddASTNode(numberAST, this.parseNext(pos + 2));
                case Token.MINUS:
                    return new SubtractASTNode(
                        numberAST,
                        this.parseNext(pos + 2)
                    );
                case Token.MULTIPLY:
                    return new MultiplyASTNode(
                        numberAST,
                        this.parseNext(pos + 2)
                    );
                case Token.DIVIDE:
                    return new DivideASTNode(
                        numberAST,
                        this.parseNext(pos + 2)
                    );
                case Token.EOF:
                    return numberAST;
                default:
                    throw new Error(
                        'Unexpected token: ' + nextToken.getValue()
                    );
            }
        }

        return new EOFASTNode();
    }
}
