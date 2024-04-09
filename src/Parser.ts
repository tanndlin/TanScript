import {
    AST,
    ASTNode,
    AssignASTNode,
    BlockASTNode,
    DeclarationASTNode,
    EOFASTNode,
    IdentifierASTNode,
    LParenASTNode,
    RParenASTNode,
} from './AST/AST';
import {
    AndASTNode,
    BooleanOpASTNode,
    EqualASTNode,
    GreaterEqASTNode,
    GreaterThanASTNode,
    LessEqASTNode,
    LessThanASTNode,
    NotASTNode,
    NotEqualASTNode,
    OrASTNode,
} from './AST/BoolAST';
import {
    ForASTNode,
    FunctionCallASTNode,
    FunctionDefASTNode,
    IfASTNode,
    WhileASTNode,
} from './AST/ControlAST';
import {
    AddASTNode,
    DivideASTNode,
    INumberableAST,
    MultiplyASTNode,
    NumberASTNode,
    SubtractASTNode,
} from './AST/NumberAST';
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
            case Token.SEMI:
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

            case Token.FUNCTION:
                return this.parseFunctionDef();

            case Token.NOT:
                return this.parseNot();
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

    parseFunctionDef(): FunctionDefASTNode {
        this.consumeToken(Token.FUNCTION);
        const identToken = this.consumeToken(Token.IDENTIFIER);

        this.consumeToken(Token.LPAREN);
        const args: IdentifierASTNode[] = [];
        while (this.tokens[this.pos].getType() !== Token.RPAREN) {
            const argToken = this.consumeToken(Token.IDENTIFIER);
            args.push(new IdentifierASTNode(argToken.getValue()));

            if (this.tokens[this.pos].getType() === Token.COMMA) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RPAREN);
        const block = this.parseBlock();
        return new FunctionDefASTNode(identToken.getValue(), args, block);
    }

    parseFunctionCall(identToken: LexerToken): FunctionCallASTNode {
        this.consumeToken(Token.LPAREN);

        const args: ASTNode[] = [];
        while (this.tokens[this.pos].getType() !== Token.RPAREN) {
            args.push(this.parseNext());

            if (this.tokens[this.pos].getType() === Token.COMMA) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RPAREN);
        return new FunctionCallASTNode(identToken.getValue(), args);
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

        let leftAST: INumberableAST | BooleanOpASTNode;
        if (curToken.getType() === Token.LPAREN) {
            leftAST = this.parseLParen() as INumberableAST;
        } else if (
            curToken.getType() === Token.IDENTIFIER &&
            this.tokens[this.pos].getType() === Token.LPAREN
        ) {
            // This is a function call
            leftAST = this.parseFunctionCall(curToken) as
                | INumberableAST
                | BooleanOpASTNode;
        } else if (curToken.getType() === Token.IDENTIFIER) {
            leftAST = new IdentifierASTNode(
                curToken.getValue()
            ) as INumberableAST;
        } else {
            leftAST = new NumberASTNode(curToken.getValue());
        }

        // There needs to be at least 2 more tokens to form an expression
        if (this.pos + 1 >= this.tokens.length) {
            this.pos++;
            return leftAST;
        }

        const nextToken = this.tokens[this.pos];
        switch (nextToken.getType()) {
            case Token.PLUS:
                this.pos++;
                return new AddASTNode(
                    leftAST as INumberableAST,
                    this.parseExpressionOrNumber() as INumberableAST
                );
            case Token.MINUS:
                this.pos++;
                return new SubtractASTNode(
                    leftAST as INumberableAST,
                    this.parseExpressionOrNumber() as INumberableAST
                );
            case Token.MULTIPLY:
                this.pos++;
                return new MultiplyASTNode(
                    leftAST as INumberableAST,
                    this.parseExpressionOrNumber() as INumberableAST
                );
            case Token.DIVIDE:
                this.pos++;
                return new DivideASTNode(
                    leftAST as INumberableAST,
                    this.parseExpressionOrNumber() as INumberableAST
                );
            case Token.LESS:
                this.pos++;
                return new LessThanASTNode(
                    leftAST as INumberableAST,
                    this.parseExpressionOrNumber() as INumberableAST
                );
            case Token.GREATER:
                this.pos++;
                return new GreaterThanASTNode(
                    leftAST as INumberableAST,
                    this.parseExpressionOrNumber() as INumberableAST
                );
            case Token.LEQ:
                this.pos++;
                return new LessEqASTNode(
                    leftAST as INumberableAST,
                    this.parseExpressionOrNumber() as INumberableAST
                );
            case Token.GEQ:
                this.pos++;
                return new GreaterEqASTNode(
                    leftAST as INumberableAST,
                    this.parseExpressionOrNumber() as INumberableAST
                );

            case Token.EQUAL:
                this.pos++;
                return new EqualASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                ) as INumberableAST;
            case Token.NEQ:
                this.pos++;
                return new NotEqualASTNode(
                    leftAST,
                    this.parseExpressionOrNumber()
                ) as INumberableAST;
            case Token.AND:
                this.pos++;
                return new AndASTNode(leftAST, this.parseExpressionOrNumber());
            case Token.OR:
                this.pos++;
                return new OrASTNode(leftAST, this.parseExpressionOrNumber());
            case Token.EOF:
            case Token.SEMI:
            case Token.COMMA:
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

    parseLParen(): LParenASTNode {
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
            const resultExpression = new AddASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST
            );
            return new AssignASTNode(identAST, resultExpression);
        }

        // -=
        if (assignToken.getType() === Token.MINUS) {
            const resultExpression = new SubtractASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST
            );
            return new AssignASTNode(identAST, resultExpression);
        }

        // *=
        if (assignToken.getType() === Token.MULTIPLY) {
            const resultExpression = new MultiplyASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST
            );
            return new AssignASTNode(identAST, resultExpression);
        }

        // /=
        const resultExpression = new DivideASTNode(
            identAST as INumberableAST,
            expressionAST as INumberableAST
        );
        return new AssignASTNode(identAST, resultExpression);
    }

    parseNot(): NotASTNode {
        const notToken = this.consumeToken(Token.NOT);
        const expression = this.parseExpressionOrNumber();
        return new NotASTNode(expression);
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
