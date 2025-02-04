import * as AST from './AST';
import { ParserError } from './errors';
import { PRECEDENCE } from './precedence';
import {
    INumberableAST,
    IterableResolvable,
    LexerToken,
    OPERATORS,
    PrimitiveValues,
    Token,
} from './types';

export default class Parser {
    private pos = 0;

    constructor(private tokens: LexerToken[]) {}

    parse(): AST.Program {
        // parse the tokens
        const children = [];
        while (this.pos < this.tokens.length) {
            children.push(this.parseNext() as AST.Stmt);
        }

        return new AST.Program(new AST.BlockASTNode(children));
    }

    private parseNext(): AST.Stmt | AST.Expr {
        const curToken = this.tokens[this.pos];
        switch (curToken.getType()) {
            case Token.EOF:
                this.pos++;
                return new AST.EOFASTNode();
            case Token.SEMI:
                this.pos++;
                return new AST.SemiASTNode();

            case Token.NUMBER:
            case Token.LPAREN:
            case Token.TRUE:
            case Token.FALSE:
            case Token.STRING:
            case Token.NOT:
                return this.parseExpressionOrNumber();

            case Token.DECLARATION:
                return this.parseDecleration();

            case Token.WHILE:
                return this.parseWhile();

            case Token.FOR:
                return this.parseFor();

            case Token.FOREACH:
                return this.parseForEach();

            case Token.IF:
                return this.parseIf();

            case Token.IDENTIFIER: {
                let ret = this.parseAssignmentOrExpression();
                if (
                    this.tokens[this.pos] &&
                    this.tokens[this.pos].isType(Token.SEMI)
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
            case Token.RETURN:
                this.consumeToken(Token.RETURN);
                return new AST.ReturnASTNode(this.parseNext() as AST.Expr);
        }

        if (OPERATORS.has(curToken.getType())) {
            // The last token was the left side of an expression
            return this.parseExpressionOrNumber();
        }

        throw new ParserError(
            `Unexpected token at line ${curToken.getLineNumber()}: ${curToken.getValue()}`,
        );
    }

    parseWhile() {
        this.consumeToken(Token.WHILE);
        this.consumeToken(Token.LPAREN);

        const condition = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new AST.WhileASTNode(condition, block);
    }

    parseFor(): AST.ForASTNode {
        this.consumeToken(Token.FOR);
        this.consumeToken(Token.LPAREN);

        const init = this.parseNext() as AST.Stmt;
        const condition = this.parseNext() as AST.Expr;
        const update = this.parseNext() as AST.Stmt;
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new AST.ForASTNode(init, condition, update, block);
    }

    parseForEach(): AST.ForEachASTNode {
        this.consumeToken(Token.FOREACH);
        this.consumeToken(Token.LPAREN);

        const ident = this.consumeToken(Token.IDENTIFIER);
        const identAST = new AST.IdentifierASTNode(ident.getValue());
        this.consumeToken(Token.IN);

        const iterable = this.parseNext() as IterableResolvable;
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new AST.ForEachASTNode(identAST, iterable, block);
    }

    parseIf(): AST.IfASTNode {
        this.consumeToken(Token.IF);
        this.consumeToken(Token.LPAREN);

        const condition = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        // Check if there is an else block
        if (this.tokens[this.pos].isType(Token.ELSE)) {
            this.consumeToken(Token.ELSE);
            return new AST.IfASTNode(condition, block, this.parseBlock());
        }

        return new AST.IfASTNode(condition, block);
    }

    parseFunctionDef(): AST.FunctionDefASTNode {
        this.consumeToken(Token.FUNCTION);
        const identToken = this.consumeToken(Token.IDENTIFIER);

        const args: AST.IdentifierASTNode[] = this.parseParameters();
        const block = this.parseBlock();
        return new AST.FunctionDefASTNode(identToken.getValue(), args, block);
    }

    private parseParameters() {
        this.consumeToken(Token.LPAREN);
        const args: AST.IdentifierASTNode[] = [];
        while (this.tokens[this.pos].getType() !== Token.RPAREN) {
            const argToken = this.consumeToken(Token.IDENTIFIER);
            args.push(new AST.IdentifierASTNode(argToken.getValue()));

            if (this.tokens[this.pos].isType(Token.COMMA)) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RPAREN);
        return args;
    }

    parseFunctionCall(identToken: LexerToken): AST.FunctionCallASTNode {
        this.consumeToken(Token.LPAREN);

        const args: AST.Expr[] = [];
        while (this.tokens[this.pos].getType() !== Token.RPAREN) {
            args.push(this.parseNext() as AST.Expr);

            if (this.tokens[this.pos].isType(Token.COMMA)) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RPAREN);
        return new AST.FunctionCallASTNode(identToken.getValue(), args);
    }

    parseAssignmentOrExpression(): AST.Stmt | AST.Expr {
        const identToken = this.consumeToken(Token.IDENTIFIER);

        if (this.tokens[this.pos].isType(Token.INCREMENT)) {
            return this.parseIncrementDecrement(AST.AddASTNode, identToken);
        }

        if (this.tokens[this.pos].isType(Token.DECREMENT)) {
            return this.parseIncrementDecrement(
                AST.SubtractASTNode,
                identToken,
            );
        }

        // Check if the next token is a shorhand assign
        if (OPERATORS.has(this.tokens[this.pos].getType())) {
            if (this.tokens[this.pos + 1].isType(Token.ASSIGN)) {
                return this.parseAssignment(identToken);
            }
        }

        // Check if the next token is an assignment
        if (this.tokens[this.pos].isType(Token.ASSIGN)) {
            return this.parseAssignment(identToken);
        }

        // The token is not an assignment, so it must be an expression
        this.pos--;
        return this.parseExpressionOrNumber();
    }

    parseIncrementDecrement<
        T extends new (
            left: INumberableAST,
            right: INumberableAST,
        ) => AST.MathASTNode,
    >(Ctor: T, identToken: LexerToken): AST.AssignASTNode {
        this.consumeToken(Token.INCREMENT, Token.DECREMENT);

        return new AST.AssignASTNode(
            new AST.IdentifierASTNode(identToken.getValue()),
            new Ctor(
                new AST.IdentifierASTNode(
                    identToken.getValue(),
                ) as INumberableAST,
                new AST.NumberASTNode(1),
            ),
        );
    }

    parseBlock(): AST.BlockASTNode {
        this.consumeToken(Token.LCURLY);

        const children: AST.Stmt[] = [];
        while (this.tokens[this.pos].getType() !== Token.RCURLY) {
            children.push(this.parseNext() as AST.Stmt);
        }

        this.consumeToken(Token.RCURLY);
        return new AST.BlockASTNode(children);
    }

    private parseExpressionOrNumber(): AST.Expr {
        // logical expression is the least priority
        const expr = this.parseNextPrecedence(PRECEDENCE.length - 1);

        if (this.tokens[this.pos] && this.tokens[this.pos].isType(Token.SEMI)) {
            this.consumeToken(Token.SEMI);
        }

        return expr;
    }

    // Goes bottom up, starting with the lowest precedence
    // This is because each level calls the next higher precedence
    private parseNextPrecedence(depth: number): AST.Expr {
        if (depth === -1) {
            return this.parseFactor() as AST.Expr;
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
                        this.parseNextPrecedence(depth - 1) as any,
                    );
                    matchFound = true;
                    break;
                }
            }

            // If the next token matches none, return
            if (!matchFound) {
                break;
            }
        }

        return left;
    }

    private parseFactor() {
        const token = this.consumeToken(
            Token.IDENTIFIER,
            Token.LPAREN,
            Token.LBRACKET,
            Token.LCURLY,
            Token.NOT,
            ...PrimitiveValues,
        );

        return this.getLeftASTFromToken(token);
    }

    private getLeftASTFromToken(consumedToken: LexerToken) {
        if (consumedToken.isType(Token.LPAREN)) {
            return this.parseLParen() as INumberableAST;
        }

        // If it looks something like x()
        if (consumedToken.isType(Token.IDENTIFIER)) {
            if (this.tokens[this.pos].isType(Token.LPAREN)) {
                // This is a function call
                return this.parseFunctionCall(consumedToken);
            }

            if (this.tokens[this.pos].isType(Token.PERIOD)) {
                return this.parseObjectAccess(consumedToken);
            }

            return new AST.IdentifierASTNode(consumedToken.getValue());
        }

        const tokenType = consumedToken.getType();
        if (tokenType === Token.TRUE || tokenType === Token.FALSE) {
            return new AST.BooleanASTNode(tokenType);
        }

        if (consumedToken.isType(Token.NOT)) {
            return this.parseNot(consumedToken);
        }

        if (consumedToken.isType(Token.LBRACKET)) {
            return this.parseArray(consumedToken);
        }

        if (consumedToken.isType(Token.LCURLY)) {
            return this.parseObject(consumedToken);
        }

        if (consumedToken.isType(Token.STRING)) {
            return new AST.StringASTNode(consumedToken.getValue());
        }

        return new AST.NumberASTNode(+consumedToken.getValue());
    }

    parseObjectAccess(identToken: LexerToken) {
        const identAST = new AST.IdentifierASTNode(identToken.getValue());
        this.consumeToken(Token.PERIOD);

        const key = this.consumeToken(Token.IDENTIFIER);
        const attributeIdent = new AST.IdentifierASTNode(key.getValue());
        return new AST.ObjectAccessAST(identAST, attributeIdent);
    }

    parseArray(consumedToken?: LexerToken): AST.ListASTNode {
        if (!consumedToken) {
            consumedToken = this.consumeToken(Token.LBRACKET);
        }

        const elements = [];
        while (this.tokens[this.pos].getType() !== Token.RBRACKET) {
            elements.push(this.parseNext() as AST.Expr);

            if (this.tokens[this.pos].isType(Token.COMMA)) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RBRACKET);
        return new AST.ListASTNode(elements);
    }

    parseObject(consumedToken?: LexerToken) {
        if (!consumedToken) {
            consumedToken = this.consumeToken(Token.LCURLY);
        }

        const attributes: AST.AttributeASTNode[] = [];
        while (this.tokens[this.pos].getType() !== Token.RCURLY) {
            const key = this.consumeToken(Token.IDENTIFIER);
            this.consumeToken(Token.COLON);

            const value = this.parseNext() as AST.Expr;
            attributes.push(new AST.AttributeASTNode(key.getValue(), value));

            if (this.tokens[this.pos].isType(Token.COMMA)) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RCURLY);
        return new AST.ObjectASTNode(attributes);
    }

    parseLParen(): AST.LParenASTNode {
        const expression = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const lParenNode = new AST.LParenASTNode(expression);
        return lParenNode;
    }

    parseDecleration(): AST.DeclarationASTNode {
        this.consumeToken(Token.DECLARATION);

        const assignASTNode = this.parseAssignment(undefined, false);
        const declAST = new AST.DeclarationASTNode(assignASTNode);

        if (this.tokens[this.pos] && this.tokens[this.pos].isType(Token.SEMI)) {
            this.consumeToken(Token.SEMI);
        }
        return declAST;
    }

    parseAssignment(
        identToken: LexerToken = this.consumeToken(Token.IDENTIFIER),
        allowShortHand = true,
    ): AST.AssignASTNode {
        const assignToken = this.consumeToken(
            Token.ASSIGN,
            ...OPERATORS.values(),
        );

        // +=, -=, *=, /=
        if (OPERATORS.has(assignToken.getType())) {
            // The token we consumed is an operator
            if (!allowShortHand) {
                throw new ParserError(
                    `Unexpected token ${assignToken.getValue()}. Expected an assignment operator. Cannot use shorthand assignment in this context.`,
                );
            }

            return this.parseShortHandAssign(identToken, assignToken);
        }

        const identAST = new AST.IdentifierASTNode(identToken.getValue());

        // If looks like a lambda
        if (this.tokens[this.pos].isType(Token.LPAREN)) {
            const ret = this.tryParseLambda(identToken.getValue());
            if (ret) {
                return new AST.AssignASTNode(identAST, ret);
            }
        }

        // Already comsumed the assign token
        const expressionAST = this.parseExpressionOrNumber();

        return new AST.AssignASTNode(identAST, expressionAST);
    }

    tryParseLambda(name: string) {
        const parseLambda = (name: string) => {
            const args = this.parseParameters();
            this.consumeToken(Token.LAMBDA);

            const block = this.parseBlock();
            return new AST.FunctionDefASTNode(name, args, block);
        };

        let counter = this.pos;
        while (this.tokens[counter].getType() !== Token.RPAREN) {
            counter++;
        }

        if (this.tokens[counter + 1].isType(Token.LAMBDA)) {
            return parseLambda(name);
        }
    }

    parseShortHandAssign(
        identToken: LexerToken,
        assignToken: LexerToken,
    ): AST.AssignASTNode {
        this.consumeToken(Token.ASSIGN);
        const expressionAST = this.parseExpressionOrNumber();
        const identAST = new AST.IdentifierASTNode(identToken.getValue());

        // +=
        if (assignToken.isType(Token.PLUS)) {
            const resultExpression = new AST.AddASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST,
            );
            return new AST.AssignASTNode(identAST, resultExpression);
        }

        // -=
        if (assignToken.isType(Token.MINUS)) {
            const resultExpression = new AST.SubtractASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST,
            );
            return new AST.AssignASTNode(identAST, resultExpression);
        }

        // *=
        if (assignToken.isType(Token.MULTIPLY)) {
            const resultExpression = new AST.MultiplyASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST,
            );
            return new AST.AssignASTNode(identAST, resultExpression);
        }

        // /=
        if (assignToken.isType(Token.DIVIDE)) {
            const resultExpression = new AST.DivideASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST,
            );
            return new AST.AssignASTNode(identAST, resultExpression);
        }

        if (assignToken.isType(Token.INT_DIVIDE)) {
            const resultExpression = new AST.IntegerDivideASTNode(
                identAST as INumberableAST,
                expressionAST as INumberableAST,
            );
            return new AST.AssignASTNode(identAST, resultExpression);
        }

        throw new ParserError(
            `Unexpected token ${assignToken.getValue()}. Expected an assignment operator`,
        );
    }

    parseNot(notToken?: LexerToken): AST.NotASTNode {
        if (!notToken) {
            this.consumeToken(Token.NOT);
        }

        // Check special case where the next token is a LPAREN
        if (this.tokens[this.pos].isType(Token.LPAREN)) {
            this.consumeToken(Token.LPAREN);
            return new AST.NotASTNode(this.parseLParen());
        }

        const expression = this.parseExpressionOrNumber();
        return new AST.NotASTNode(expression);
    }

    consumeToken(...tokens: Token[]): LexerToken {
        if (this.pos >= this.tokens.length) {
            throw new ParserError('Unexpected EOF');
        }

        if (!tokens.includes(this.tokens[this.pos].getType())) {
            throw new ParserError(
                `Unexpected token at line ${this.tokens[
                    this.pos
                ].getLineNumber()}: ${this.tokens[
                    this.pos
                ].getValue()}. Expected one of ${tokens.join(', ')}`,
            );
        }

        return this.tokens[this.pos++];
    }
}
