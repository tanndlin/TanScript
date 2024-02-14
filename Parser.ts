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
    private pos = 0;

    constructor(private tokens: LexerToken[]) {}

    parse(): AST {
        // parse the tokens
        const root = this.parseNext();

        return new AST(root);
    }

    private parseNext(): ASTNode {
        const curToken = this.tokens[this.pos];
        if (curToken.getType() === Token.EOF) {
            return new EOFASTNode();
        }

        // If the current token is a number, make sure next token is either an operator or EOF
        if (curToken.getType() === Token.NUMBER) {
            const numberAST = new NumberASTNode(curToken.getValue());
            if (this.pos + 1 >= this.tokens.length) {
                return numberAST;
            }

            const nextToken = this.tokens[this.pos + 1];
            switch (nextToken.getType()) {
                case Token.PLUS:
                    this.pos += 2;
                    return new AddASTNode(numberAST, this.parseNext());
                case Token.MINUS:
                    this.pos += 2;
                    return new SubtractASTNode(numberAST, this.parseNext());
                case Token.MULTIPLY:
                    this.pos += 2;
                    return new MultiplyASTNode(numberAST, this.parseNext());
                case Token.DIVIDE:
                    this.pos += 2;
                    return new DivideASTNode(numberAST, this.parseNext());
                case Token.EOF:
                    this.pos++;
                    return numberAST;
                default:
                    throw new Error(
                        'Unexpected token: ' + nextToken.getValue()
                    );
            }
        }

        this.pos++;
        return new EOFASTNode();
    }
}
