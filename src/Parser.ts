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
    SemiASTNode,
    StringASTNode,
} from './AST/AST';
import { BooleanASTNode, NotASTNode } from './AST/BoolAST';
import {
    ForASTNode,
    FunctionCallASTNode,
    FunctionDefASTNode,
    IfASTNode,
    WhileASTNode,
} from './AST/ControlAST';
import { ForEachASTNode, ListASTNode } from './AST/IterableAST';
import {
    AddASTNode,
    DivideASTNode,
    IntegerDivideASTNode,
    MathASTNode,
    MultiplyASTNode,
    NumberASTNode,
    SubtractASTNode,
} from './AST/NumberAST';
import {
    SignalAST,
    SignalAssignmentAST,
    SignalComputeAST,
    SignalComputeAssignmentAST,
} from './AST/SignalAST';
import { ParserError } from './errors';
import { PRECEDENCE } from './precedence';
import {
    BooleanToken,
    ExpressionableAST,
    INumberableAST,
    LexerToken,
    OPERATORS,
    PrimitiveValues,
    SIGNAL_OPERATORS,
    Token,
} from './types';

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
                return new SemiASTNode();

            case Token.NUMBER:
            case Token.LPAREN:
            case Token.TRUE:
            case Token.FALSE:
            case Token.STRING:
            case Token.NOT:
                return this.parseExpressionOrNumber();

            case Token.DECLERATION:
                return this.parseDecleration();

            case Token.WHILE:
                return this.parseWhile();

            case Token.FOR:
                return this.parseFor();

            case Token.FOREACH:
                return this.parseForEach();

            case Token.IF:
                return this.parseIf();

            case Token.IDENTIFIER:
            case Token.SIGNAL:
            case Token.COMPUTE: {
                let ret = this.parseAssignmentOrExpression();
                if (
                    this.tokens[this.pos] &&
                    this.tokens[this.pos].getType() === Token.SEMI
                ) {
                    this.consumeToken(Token.SEMI);
                }

                return ret;
            }

            case Token.LCURLY:
                return this.parseBlock();

            case Token.LBRACKET:
                return this.parseArray();

            case Token.FUNCTION:
                return this.parseFunctionDef();
        }

        if (OPERATORS.has(curToken.getType())) {
            // The last token was the left side of an expression
            return this.parseExpressionOrNumber();
        }

        throw new ParserError(
            `Unexpected token at line ${curToken.getLineNumber()}: ${curToken.getValue()}`
        );
    }

    parseWhile(): ASTNode {
        this.consumeToken(Token.WHILE);
        this.consumeToken(Token.LPAREN);

        const condition = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new WhileASTNode(condition, block);
    }

    parseFor(): ForASTNode {
        this.consumeToken(Token.FOR);
        this.consumeToken(Token.LPAREN);

        const init = this.parseNext();
        const condition = this.parseNext();
        const update = this.parseNext();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new ForASTNode(init, condition, update, block);
    }

    parseForEach(): ForEachASTNode {
        this.consumeToken(Token.FOREACH);
        this.consumeToken(Token.LPAREN);

        const ident = this.consumeToken(Token.IDENTIFIER);
        const identAST = new IdentifierASTNode(ident.getValue());
        this.consumeToken(Token.IN);

        const iterable = this.parseNext();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new ForEachASTNode(identAST, iterable, block);
    }

    parseIf(): IfASTNode {
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

        const args: IdentifierASTNode[] = this.parseParameters();
        const block = this.parseBlock();
        return new FunctionDefASTNode(identToken.getValue(), args, block);
    }

    private parseParameters() {
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
        return args;
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
        const identToken = this.consumeOneOf([
            Token.IDENTIFIER,
            Token.SIGNAL,
            Token.COMPUTE,
        ]);

        if (this.tokens[this.pos].getType() === Token.INCREMENT) {
            return this.parseIncrementDecrement(AddASTNode, identToken);
        }

        if (this.tokens[this.pos].getType() === Token.DECREMENT) {
            return this.parseIncrementDecrement(SubtractASTNode, identToken);
        }

        // Check if the next token is a shorhand assign
        if (OPERATORS.has(this.tokens[this.pos].getType())) {
            if (this.tokens[this.pos + 1].getType() === Token.ASSIGN) {
                return this.parseAssignment(identToken);
            }
        }

        if (
            this.tokens[this.pos].getType() === Token.SIGNAL_ASSIGN ||
            this.tokens[this.pos].getType() === Token.COMPUTE_ASSIGN
        ) {
            return this.parseSignalAssign(identToken);
        }

        // Check if the next token is an assignment
        if (this.tokens[this.pos].getType() === Token.ASSIGN) {
            return this.parseAssignment(identToken);
        }

        // The token is not an assignment, so it must be an expression
        this.pos--;
        return this.parseExpressionOrNumber();
    }

    parseIncrementDecrement<
        T extends new (
            left: INumberableAST,
            right: INumberableAST
        ) => MathASTNode
    >(Ctor: T, identToken: LexerToken) {
        this.consumeOneOf([Token.INCREMENT, Token.DECREMENT]);

        if (identToken.getType() === Token.SIGNAL) {
            return new SignalAssignmentAST(
                new AssignASTNode(
                    new IdentifierASTNode(identToken.getValue()),
                    new Ctor(
                        new SignalAST(identToken.getValue()) as INumberableAST,
                        new NumberASTNode('1')
                    )
                )
            );
        }

        if (identToken.getType() === Token.COMPUTE) {
            return new SignalComputeAssignmentAST(
                new AssignASTNode(
                    new IdentifierASTNode(identToken.getValue()),
                    new Ctor(
                        new SignalComputeAST(
                            identToken.getValue()
                        ) as INumberableAST,
                        new NumberASTNode('1')
                    )
                )
            );
        }

        return new AssignASTNode(
            new IdentifierASTNode(identToken.getValue()),
            new Ctor(
                new IdentifierASTNode(identToken.getValue()) as INumberableAST,
                new NumberASTNode('1')
            )
        );
    }

    parseSignalAssign(
        identToken: LexerToken,
        assignToken?: LexerToken
    ): SignalAssignmentAST | SignalComputeAssignmentAST {
        if (!assignToken)
            assignToken = this.consumeOneOf([
                Token.SIGNAL_ASSIGN,
                Token.COMPUTE_ASSIGN,
            ]);

        if (assignToken.getType() === Token.SIGNAL_ASSIGN) {
            return new SignalAssignmentAST(
                new AssignASTNode(
                    new IdentifierASTNode(identToken.getValue()),
                    this.parseExpressionOrNumber()
                )
            );
        }

        return new SignalComputeAssignmentAST(
            new AssignASTNode(
                new IdentifierASTNode(identToken.getValue()),
                this.parseExpressionOrNumber()
            )
        );
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

    private parseExpressionOrNumber(): ExpressionableAST {
        // logical expression is the least priority
        const expr = this.parseNextPrecedence(PRECEDENCE.length - 1);

        if (
            this.tokens[this.pos] &&
            this.tokens[this.pos].getType() === Token.SEMI
        ) {
            this.consumeToken(Token.SEMI);
        }

        return expr;
    }

    // Goes bottom up, starting with the lowest precedence
    // This is because each level calls the next higher precedence
    private parseNextPrecedence(depth: number): ExpressionableAST {
        if (depth === -1) {
            return this.parseFactor() as ExpressionableAST;
        }

        let left = this.parseNextPrecedence(depth - 1);

        const pairs = PRECEDENCE[depth];
        while (this.pos < this.tokens.length) {
            let matchFound = false;
            const curTokenType = this.tokens[this.pos].getType();

            for (const { token, ast } of pairs) {
                if (curTokenType === token) {
                    this.pos++;

                    // TODO: Remove any casts
                    left = new ast(
                        left as any,
                        this.parseNextPrecedence(depth - 1) as any
                    );
                    matchFound = true;
                    break;
                }
            }

            // If the next token matches none, return
            if (!matchFound) break;
        }

        return left;
    }

    private parseFactor(): ASTNode {
        const token = this.consumeOneOf([
            Token.IDENTIFIER,
            Token.LPAREN,
            Token.LBRACKET,
            Token.NOT,
            ...PrimitiveValues,
            ...SIGNAL_OPERATORS,
        ]);

        return this.getLeftASTFromToken(token);
    }

    private getLeftASTFromToken(consumedToken: LexerToken): ASTNode {
        if (consumedToken.getType() === Token.LPAREN) {
            return this.parseLParen() as INumberableAST;
        }

        // If it looks something like x()
        if (
            consumedToken.getType() === Token.IDENTIFIER &&
            this.tokens[this.pos].getType() === Token.LPAREN
        ) {
            // This is a function call
            return this.parseFunctionCall(consumedToken) as
                | INumberableAST
                | BooleanASTNode;
        }

        if (consumedToken.getType() === Token.IDENTIFIER) {
            return new IdentifierASTNode(consumedToken.getValue()) as
                | INumberableAST
                | BooleanASTNode;
        }

        if (
            consumedToken.getType() === Token.TRUE ||
            consumedToken.getType() === Token.FALSE
        ) {
            return new BooleanASTNode(consumedToken.getType() as BooleanToken);
        }

        if (consumedToken.getType() === Token.NOT) {
            return this.parseNot(consumedToken);
        }

        if (consumedToken.getType() === Token.STRING) {
            return new StringASTNode(consumedToken.getValue());
        }

        if (consumedToken.getType() === Token.SIGNAL) {
            return new SignalAST(consumedToken.getValue());
        }

        if (consumedToken.getType() === Token.COMPUTE) {
            return new SignalComputeAST(consumedToken.getValue());
        }

        if (consumedToken.getType() === Token.LBRACKET) {
            return this.parseArray(consumedToken);
        }

        return new NumberASTNode(consumedToken.getValue());
    }

    parseArray(consumedToken?: LexerToken): ListASTNode {
        if (!consumedToken) consumedToken = this.consumeToken(Token.LBRACKET);

        const elements: ASTNode[] = [];
        while (this.tokens[this.pos].getType() !== Token.RBRACKET) {
            elements.push(this.parseNext());

            if (this.tokens[this.pos].getType() === Token.COMMA) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RBRACKET);
        return new ListASTNode(elements);
    }

    parseLParen(): LParenASTNode {
        const expression = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const lParenNode = new LParenASTNode([expression, new RParenASTNode()]);
        return lParenNode;
    }

    parseDecleration(): DeclarationASTNode {
        this.consumeToken(Token.DECLERATION);

        const assignASTNode = this.parseAssignment(undefined, false);
        const declAST = new DeclarationASTNode(assignASTNode);

        if (
            this.tokens[this.pos] &&
            this.tokens[this.pos].getType() === Token.SEMI
        ) {
            this.consumeToken(Token.SEMI);
        }
        return declAST;
    }

    parseAssignment(
        identToken: LexerToken = this.consumeToken(Token.IDENTIFIER),
        allowShortHand = true
    ): AssignASTNode {
        const assignToken = this.consumeOneOf([
            Token.ASSIGN,
            Token.SIGNAL_ASSIGN,
            Token.COMPUTE_ASSIGN,
            ...OPERATORS.values(),
        ]);

        // +=, -=, *=, /=
        if (OPERATORS.has(assignToken.getType())) {
            // The token we consumed is an operator
            if (!allowShortHand)
                throw new ParserError(
                    `Unexpected token ${assignToken.getValue()}. Expected an assignment operator. Cannot use shorthand assignment in this context.`
                );

            return this.parseShortHandAssign(identToken, assignToken);
        }

        if (
            assignToken.getType() === Token.SIGNAL_ASSIGN ||
            assignToken.getType() === Token.COMPUTE_ASSIGN
        ) {
            return this.parseSignalAssign(identToken, assignToken);
        }

        const identAST = new IdentifierASTNode(identToken.getValue());

        // If looks like a lambda
        if (this.tokens[this.pos].getType() === Token.LPAREN) {
            const ret = this.tryParseLambda(identToken.getValue());
            if (ret) return new AssignASTNode(identAST, ret);
        }

        // Already comsumed the assign token
        const expressionAST = this.parseExpressionOrNumber();

        return new AssignASTNode(identAST, expressionAST);
    }

    tryParseLambda(name: string) {
        const parseLambda = (name: string) => {
            const args = this.parseParameters();
            this.consumeToken(Token.LAMBDA);

            const block = this.parseBlock();
            return new FunctionDefASTNode(name, args, block);
        };

        let counter = this.pos;
        while (this.tokens[counter].getType() !== Token.RPAREN) {
            counter++;
        }

        if (this.tokens[counter + 1].getType() === Token.LAMBDA) {
            return parseLambda(name);
        }
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
        if (assignToken.getType() === Token.DIVIDE) {
            const resultExpression = new DivideASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST
            );
            return new AssignASTNode(identAST, resultExpression);
        }

        if (assignToken.getType() === Token.INT_DIVIDE) {
            const resultExpression = new IntegerDivideASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST
            );
            return new AssignASTNode(identAST, resultExpression);
        }

        throw new ParserError(
            `Unexpected token ${assignToken.getValue()}. Expected an assignment operator`
        );
    }

    parseNot(notToken?: LexerToken): NotASTNode {
        if (!notToken) this.consumeToken(Token.NOT);

        // Check special case where the next token is a LPAREN
        if (this.tokens[this.pos].getType() === Token.LPAREN) {
            this.consumeToken(Token.LPAREN);
            return new NotASTNode(this.parseLParen());
        }

        const expression = this.parseExpressionOrNumber();
        return new NotASTNode(expression);
    }

    consumeToken(token: Token): LexerToken {
        if (this.pos >= this.tokens.length)
            throw new ParserError('Unexpected EOF');

        if (this.tokens[this.pos].getType() !== token)
            throw new ParserError(
                `Unexpected token at line ${this.tokens[
                    this.pos
                ].getLineNumber()}: ${this.tokens[
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
                `Unexpected token at line ${this.tokens[
                    this.pos
                ].getLineNumber()}: ${this.tokens[
                    this.pos
                ].getValue()}. Expected one of ${tokens.join(', ')}`
            );

        return this.tokens[this.pos++];
    }
}
