import {
    AST,
    ASTNode,
    AddASTNode,
    AssignASTNode,
    DeclarationASTNode,
    DivideASTNode,
    EOFASTNode,
    IdentifierASTNode,
    LParenASTNode,
    MultiplyASTNode,
    NumberASTNode,
    RParenASTNode,
    RootASTNode,
    SubtractASTNode,
} from './AST';
import { LexerToken, OPERATORS, Token } from './types';

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

        if (
            curToken.getType() === Token.NUMBER ||
            curToken.getType() === Token.LPAREN
        ) {
            return this.parseExpressionOrNumber();
        }

        if (curToken.getType() === Token.DECLERATION) {
            return this.parseDecleration();
        }

        if (curToken.getType() === Token.IDENTIFIER) {
            console.log('parsing assignment');

            return this.parseAssignment();
        }

        if (OPERATORS.has(curToken.getType())) {
            // The last token was the left side of an expression
            return this.parseExpressionOrNumber();
        }

        throw new Error(`Unexpected token: ${curToken.getValue()}`);
    }

    // If the current token is a number, make sure next token is either an operator or EOF
    parseExpressionOrNumber(): ASTNode {
        const curToken = this.tokens[this.pos];
        if (
            curToken.getType() !== Token.NUMBER &&
            curToken.getType() !== Token.LPAREN &&
            curToken.getType() !== Token.IDENTIFIER
        ) {
            throw new Error(
                `Unexpected token: ${curToken.getValue()}. Expected an expression or number`
            );
        }

        let leftAST: ASTNode;
        if (curToken.getType() === Token.LPAREN) {
            leftAST = this.parseLParen();
        } else if (curToken.getType() === Token.IDENTIFIER) {
            leftAST = new IdentifierASTNode(curToken.getValue());
            this.pos++;
        } else {
            leftAST = new NumberASTNode(curToken.getValue());
            this.pos++;
        }

        if (!leftAST) throw new Error('leftAST is undefined');

        if (this.pos + 1 >= this.tokens.length) {
            this.pos++;
            return leftAST;
        }

        const nextToken = this.tokens[this.pos];
        switch (nextToken.getType()) {
            case Token.PLUS:
                this.pos++;
                return new AddASTNode(leftAST, this.parseExpressionOrNumber());
            case Token.MINUS:
                this.pos++;
                return new SubtractASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                );
            case Token.MULTIPLY:
                this.pos++;
                return new MultiplyASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                );
            case Token.DIVIDE:
                this.pos++;
                return new DivideASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                );
            case Token.EOF:
            case Token.SEMI:
                this.pos++;
                return leftAST;
            case Token.RPAREN:
                return leftAST;
            default:
                throw new Error(
                    `Unexpected token: ${nextToken.getValue()}. Expected an operator or EOF`
                );
        }
    }

    parseLParen(): ASTNode {
        this.pos++;
        const lParenNode = new LParenASTNode();
        const expression = this.parseExpressionOrNumber();
        const nextToken = this.tokens[this.pos];
        if (nextToken.getType() !== Token.RPAREN) {
            throw new Error(
                `Unexpected token: ${nextToken.getValue()}. Expected RPAREN`
            );
        }

        lParenNode.addChild(expression);
        lParenNode.addChild(new RParenASTNode());

        // Consume the RParen
        this.pos++;
        return lParenNode;
    }

    parseDecleration(): DeclarationASTNode {
        const curToken = this.tokens[this.pos];
        if (curToken.getType() !== Token.DECLERATION) {
            throw new Error(
                `Unexpected token: ${curToken.getValue()}. Expected DECLERATION`
            );
        }

        this.pos++;
        const assignASTNode = this.parseAssignment();
        const declAST = new DeclarationASTNode();
        declAST.addChild(assignASTNode);
        return declAST;
    }

    parseAssignment(): AssignASTNode {
        const identToken = this.tokens[this.pos];
        if (identToken.getType() !== Token.IDENTIFIER) {
            throw new Error(
                `Unexpected token: ${identToken.getValue()}. Expected IDENTIFIER`
            );
        }

        this.pos++;
        const assignToken = this.tokens[this.pos];
        if (
            assignToken.getType() !== Token.ASSIGN &&
            !OPERATORS.has(assignToken.getType())
        ) {
            throw new ParserError(
                `Unexpected token: ${assignToken.getValue()}. Expected ASSIGN or OPERATOR`
            );
        }

        // +=, -=, *=, /=
        if (OPERATORS.has(assignToken.getType())) {
            this.pos++;
            return this.parseShortHandAssign(identToken, assignToken);
        }

        // Last token was an =
        this.pos++;
        const expressionAST = this.parseExpressionOrNumber();

        const identAST = new IdentifierASTNode(identToken.getValue());
        return new AssignASTNode(identAST, expressionAST);
    }

    parseShortHandAssign(
        identToken: LexerToken,
        assignToken: LexerToken
    ): AssignASTNode {
        const equalsToken = this.tokens[this.pos];
        if (equalsToken.getType() !== Token.ASSIGN) {
            throw new Error(
                `Unexpected token: ${equalsToken.getValue()}. Expected EQUALS`
            );
        }

        this.pos++;
        const expressionAST = this.parseExpressionOrNumber();
        const identAST = new IdentifierASTNode(identToken.getValue());

        // +=
        if (assignToken.getType() === Token.PLUS) {
            const resultExpression = new AddASTNode(identAST, expressionAST);
            return new AssignASTNode(identAST, resultExpression);
        }

        // -=
        if (assignToken.getType() === Token.MINUS) {
            const resultExpression = new SubtractASTNode(
                identAST,
                expressionAST
            );
            return new AssignASTNode(identAST, resultExpression);
        }

        // *=
        if (assignToken.getType() === Token.MULTIPLY) {
            const resultExpression = new MultiplyASTNode(
                identAST,
                expressionAST
            );
            return new AssignASTNode(identAST, resultExpression);
        }

        // /=
        const resultExpression = new DivideASTNode(identAST, expressionAST);
        return new AssignASTNode(identAST, resultExpression);
    }
}
