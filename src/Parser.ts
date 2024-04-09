import {
    AST,
    ASTNode,
    AddASTNode,
    AssignASTNode,
    BlockASTNode,
    DeclarationASTNode,
    DivideASTNode,
    EOFASTNode,
    ForASTNode,
    GreaterEqASTNode,
    GreaterThanASTNode,
    IdentifierASTNode,
    IfASTNode,
    LParenASTNode,
    LessEqASTNode,
    LessThanASTNode,
    MultiplyASTNode,
    NumberASTNode,
    RParenASTNode,
    SubtractASTNode,
    WhileASTNode,
} from './AST';
import { ParserError } from './errors';
import { LexerToken, OPERATORS, Token } from './types';

export default class Parser {
    private pos = 0;

    constructor(private tokens: LexerToken[]) {}

    parse(): AST {
        // parse the tokens
        const root = new BlockASTNode([]);
        while (this.pos < this.tokens.length) {
            root.addChild(this.parseNext());
        }

        return new AST(root);
    }

    private parseNext(): ASTNode {
        const curToken = this.tokens[this.pos];
        switch (curToken.getType()) {
            case Token.EOF:
                this.pos++;
                return new EOFASTNode();

            case Token.NUMBER:
            case Token.LPAREN:
                return this.parseExpressionOrNumber();

            case Token.DECLERATION:
                return this.parseDecleration();

            case Token.WHILE:
                return this.parseWhile();

            case Token.FOR:
                return this.parseFor();

            case Token.IF:
                return this.parseIf();

            case Token.IDENTIFIER:
                return this.parseAssignmentOrExpression();

            case Token.LCURLY:
                return this.parseBlock();
        }

        if (OPERATORS.has(curToken.getType())) {
            // The last token was the left side of an expression
            return this.parseExpressionOrNumber();
        }

        throw new ParserError(`Unexpected token: ${curToken.getValue()}`);
    }

    parseWhile(): ASTNode {
        this.consumeToken(Token.WHILE);
        this.consumeToken(Token.LPAREN);

        const condition = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new WhileASTNode(condition, block);
    }

    parseFor(): ASTNode {
        this.consumeToken(Token.FOR);
        this.consumeToken(Token.LPAREN);

        const init = this.parseNext();
        const condition = this.parseNext();
        const update = this.parseNext();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new ForASTNode(init, condition, update, block);
    }

    parseIf(): ASTNode {
        this.consumeToken(Token.IF);
        this.consumeToken(Token.LPAREN);

        const condition = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        // Check if there is an else block
        if (this.tokens[this.pos].getType() === Token.ELSE) {
            this.consumeToken(Token.ELSE);
            return new IfASTNode(condition, block, this.parseBlock());
        }

        return new IfASTNode(condition, block);
    }

    parseAssignmentOrExpression(): ASTNode {
        const identToken = this.consumeToken(Token.IDENTIFIER);

        // Check if the next token is a shorhand assign
        if (OPERATORS.has(this.tokens[this.pos].getType())) {
            if (this.tokens[this.pos + 1].getType() === Token.ASSIGN) {
                return this.parseAssignment(identToken);
            }
        }

        // Check if the next token is an assignment
        if (this.tokens[this.pos].getType() === Token.ASSIGN) {
            return this.parseAssignment(identToken);
        }

        // The token is not an assignment, so it must be an expression
        return this.parseExpressionOrNumber(identToken);
    }

    parseBlock(): ASTNode {
        this.consumeToken(Token.LCURLY);

        const children: ASTNode[] = [];
        while (this.tokens[this.pos].getType() !== Token.RCURLY) {
            children.push(this.parseNext());
        }

        this.consumeToken(Token.RCURLY);
        return new BlockASTNode(children);
    }

    // If the current token is a number, make sure next token is either an operator or EOF
    parseExpressionOrNumber(curToken?: LexerToken): ASTNode {
        if (!curToken) {
            curToken = this.consumeOneOf([
                Token.NUMBER,
                Token.LPAREN,
                Token.IDENTIFIER,
            ]);
        }

        let leftAST: ASTNode;
        if (curToken.getType() === Token.LPAREN) {
            leftAST = this.parseLParen();
        } else if (curToken.getType() === Token.IDENTIFIER) {
            leftAST = new IdentifierASTNode(curToken.getValue());
        } else {
            leftAST = new NumberASTNode(curToken.getValue());
        }

        if (!leftAST) throw new ParserError('leftAST is undefined');

        // TODO CHECK THIS
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
            case Token.LESS:
                this.pos++;
                return new LessThanASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                );
            case Token.GREATER:
                this.pos++;
                return new GreaterThanASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                );
            case Token.LEQ:
                this.pos++;
                return new LessEqASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                );
            case Token.GEQ:
                this.pos++;
                return new GreaterEqASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                );
            case Token.EOF:
            case Token.SEMI:
                this.pos++;
                return leftAST;
            case Token.RPAREN:
            case Token.RCURLY:
                return leftAST;
            default:
                throw new ParserError(
                    `Unexpected token: ${nextToken.getValue()}. Expected an operator or EOF`
                );
        }
    }

    parseLParen(): ASTNode {
        const lParenNode = new LParenASTNode();
        const expression = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        lParenNode.addChild(expression);
        lParenNode.addChild(new RParenASTNode());

        return lParenNode;
    }

    parseDecleration(): DeclarationASTNode {
        this.consumeToken(Token.DECLERATION);

        const assignASTNode = this.parseAssignment();
        const declAST = new DeclarationASTNode();
        declAST.addChild(assignASTNode);
        return declAST;
    }

    parseAssignment(
        identToken: LexerToken = this.consumeToken(Token.IDENTIFIER)
    ): AssignASTNode {
        const assignToken = this.consumeOneOf([
            Token.ASSIGN,
            ...OPERATORS.values(),
        ]);

        // +=, -=, *=, /=
        if (OPERATORS.has(assignToken.getType())) {
            // The token we consumed is an operator
            return this.parseShortHandAssign(identToken, assignToken);
        }

        // Already comsumed the assign token
        const expressionAST = this.parseExpressionOrNumber();

        const identAST = new IdentifierASTNode(identToken.getValue());
        return new AssignASTNode(identAST, expressionAST);
    }

    parseShortHandAssign(
        identToken: LexerToken,
        assignToken: LexerToken
    ): AssignASTNode {
        this.consumeToken(Token.ASSIGN);
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

    consumeToken(token: Token): LexerToken {
        if (this.pos >= this.tokens.length)
            throw new ParserError('Unexpected EOF');

        if (this.tokens[this.pos].getType() !== token)
            throw new ParserError(
                `Unexpected token: ${this.tokens[
                    this.pos
                ].getValue()}. Expected ${token}`
            );

        return this.tokens[this.pos++];
    }

    consumeOneOf(tokens: Token[]): LexerToken {
        if (this.pos >= this.tokens.length)
            throw new ParserError('Unexpected EOF');

        if (!tokens.includes(this.tokens[this.pos].getType()))
            throw new ParserError(
                `Unexpected token: ${this.tokens[
                    this.pos
                ].getValue()}. Expected one of ${tokens.join(', ')}`
            );

        return this.tokens[this.pos++];
    }
}
