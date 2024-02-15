import {
    AST,
    ASTNode,
    AddASTNode,
    DivideASTNode,
    EOFASTNode,
    MultiplyASTNode,
    NumberASTNode,
    RootASTNode,
    SubtractASTNode,
} from './AST';
import { LexerToken, Token } from './types';

export default class Parser {
    private pos = 0;

    constructor(private tokens: LexerToken[]) {}

    parse(): AST {
        // parse the tokens
        const root = new RootASTNode();
        while (this.pos < this.tokens.length) {
            root.addChild(this.parseNext());
        }

        return new AST(root);
    }

    private parseNext(): ASTNode {
        const curToken = this.tokens[this.pos];
        if (curToken.getType() === Token.EOF) {
            this.pos++;
            return new EOFASTNode();
        }

        if (curToken.getType() === Token.NUMBER) {
            return this.parseExpressionOrNumber();
        }

        this.pos++;
        return new EOFASTNode();
    }

    // If the current token is a number, make sure next token is either an operator or EOF
    parseExpressionOrNumber(): ASTNode {
        const curToken = this.tokens[this.pos];
        if (curToken.getType() !== Token.NUMBER) {
            throw new Error(`Unexpected token: ${curToken.getValue()}`);
        }

        const numberAST = new NumberASTNode(curToken.getValue());
        if (this.pos + 1 >= this.tokens.length) {
            this.pos++;
            return numberAST;
        }

        const nextToken = this.tokens[this.pos + 1];
        switch (nextToken.getType()) {
            case Token.PLUS:
                this.pos += 2;
                return new AddASTNode(
                    numberAST,
                    this.parseExpressionOrNumber()
                );
            case Token.MINUS:
                this.pos += 2;
                return new SubtractASTNode(
                    numberAST,
                    this.parseExpressionOrNumber()
                );
            case Token.MULTIPLY:
                this.pos += 2;
                return new MultiplyASTNode(
                    numberAST,
                    this.parseExpressionOrNumber()
                );
            case Token.DIVIDE:
                this.pos += 2;
                return new DivideASTNode(
                    numberAST,
                    this.parseExpressionOrNumber()
                );
            case Token.EOF:
                this.pos++;
                return numberAST;
            default:
                throw new Error(`Unexpected token: ${nextToken.getValue()}`);
        }
    }
}
