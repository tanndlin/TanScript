import * as AST from './AST';
import { ParserError } from './errors';
import { PRECEDENCE } from './precedence';
import {
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
            const next = this.parseNext();
            if (next) {
                children.push(next as AST.Stmt);
            }
        }

        return new AST.Program(new AST.ASTBlock(children));
    }

    private parseNext(): AST.Stmt | AST.Expr | null {
        const curToken = this.tokens[this.pos];
        switch (curToken.getType()) {
            case Token.EOF:
                this.pos++;
                return null;

            case Token.SEMI:
                this.pos++;
                return null;

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
                if (this.tokens[this.pos]?.isType(Token.SEMI)) {
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
                return new AST.ASTReturn(this.parseNext() as AST.Expr);
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

        return new AST.ASTWhile(condition, block);
    }

    parseFor(): AST.ASTFor {
        this.consumeToken(Token.FOR);
        this.consumeToken(Token.LPAREN);

        const init = this.parseNext() as AST.Stmt;
        const condition = this.parseNext() as AST.Expr;
        const update = this.parseNext() as AST.Stmt;
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new AST.ASTFor(init, condition, update, block);
    }

    parseForEach(): AST.ASTForEach {
        this.consumeToken(Token.FOREACH);
        this.consumeToken(Token.LPAREN);

        const ident = this.consumeToken(Token.IDENTIFIER);
        const identAST = new AST.ASTIdentifier(ident.getValue());
        this.consumeToken(Token.IN);

        const iterable = this.parseNext() as IterableResolvable;
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        return new AST.ASTForEach(identAST, iterable, block);
    }

    parseIf(): AST.ASTIf {
        this.consumeToken(Token.IF);
        this.consumeToken(Token.LPAREN);

        const condition = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const block = this.parseBlock();

        // Check if there is an else block
        if (this.tokens[this.pos].isType(Token.ELSE)) {
            this.consumeToken(Token.ELSE);
            return new AST.ASTIf(condition, block, this.parseBlock());
        }

        return new AST.ASTIf(condition, block);
    }

    parseFunctionDef(): AST.ASTFunctionDef {
        this.consumeToken(Token.FUNCTION);
        const identToken = this.consumeToken(Token.IDENTIFIER);

        const args: AST.ASTIdentifier[] = this.parseParameters();
        const block = this.parseBlock();
        return new AST.ASTFunctionDef(identToken.getValue(), args, block);
    }

    private parseParameters() {
        this.consumeToken(Token.LPAREN);
        const args: AST.ASTIdentifier[] = [];
        while (this.tokens[this.pos].getType() !== Token.RPAREN) {
            const argToken = this.consumeToken(Token.IDENTIFIER);
            args.push(new AST.ASTIdentifier(argToken.getValue()));

            if (this.tokens[this.pos].isType(Token.COMMA)) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RPAREN);
        return args;
    }

    parseFunctionCall(identToken: LexerToken): AST.ASTFunctionCall {
        this.consumeToken(Token.LPAREN);

        const args: AST.Expr[] = [];
        while (this.tokens[this.pos].getType() !== Token.RPAREN) {
            args.push(this.parseNext() as AST.Expr);

            if (this.tokens[this.pos].isType(Token.COMMA)) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RPAREN);
        return new AST.ASTFunctionCall(identToken.getValue(), args);
    }

    parseAssignmentOrExpression(): AST.Stmt | AST.Expr {
        const identToken = this.consumeToken(Token.IDENTIFIER);

        if (this.tokens[this.pos].isType(Token.INCREMENT)) {
            return this.parseIncrementDecrement(AST.ASTAdd, identToken);
        }

        if (this.tokens[this.pos].isType(Token.DECREMENT)) {
            return this.parseIncrementDecrement(AST.ASTSubtract, identToken);
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
        T extends new (left: AST.Expr, right: AST.Expr) => AST.ASTMathType,
    >(Ctor: T, identToken: LexerToken): AST.ASTAssign {
        this.consumeToken(Token.INCREMENT, Token.DECREMENT);

        return new AST.ASTAssign(
            new AST.ASTIdentifier(identToken.getValue()),
            new Ctor(
                new AST.ASTIdentifier(identToken.getValue()),
                new AST.ASTNumber(1),
            ),
        );
    }

    parseBlock(): AST.ASTBlock {
        this.consumeToken(Token.LCURLY);

        const children: AST.Stmt[] = [];
        while (this.tokens[this.pos].getType() !== Token.RCURLY) {
            children.push(this.parseNext() as AST.Stmt);
        }

        this.consumeToken(Token.RCURLY);
        return new AST.ASTBlock(children);
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
            return this.parseLParen();
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

            return new AST.ASTIdentifier(consumedToken.getValue());
        }

        const tokenType = consumedToken.getType();
        if (tokenType === Token.TRUE || tokenType === Token.FALSE) {
            return new AST.ASTBoolean(tokenType);
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
            return new AST.ASTString(consumedToken.getValue());
        }

        return new AST.ASTNumber(+consumedToken.getValue());
    }

    parseObjectAccess(identToken: LexerToken) {
        const identAST = new AST.ASTIdentifier(identToken.getValue());
        this.consumeToken(Token.PERIOD);

        const key = this.consumeToken(Token.IDENTIFIER);
        const attributeIdent = new AST.ASTIdentifier(key.getValue());
        return new AST.ObjectAccessAST(identAST, attributeIdent);
    }

    parseArray(consumedToken?: LexerToken): AST.ASTList {
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
        return new AST.ASTList(elements);
    }

    parseObject(consumedToken?: LexerToken) {
        if (!consumedToken) {
            consumedToken = this.consumeToken(Token.LCURLY);
        }

        const attributes: AST.ASTAttribute[] = [];
        while (this.tokens[this.pos].getType() !== Token.RCURLY) {
            const key = this.consumeToken(Token.IDENTIFIER);
            this.consumeToken(Token.COLON);

            const value = this.parseNext() as AST.Expr;
            attributes.push(new AST.ASTAttribute(key.getValue(), value));

            if (this.tokens[this.pos].isType(Token.COMMA)) {
                this.consumeToken(Token.COMMA);
            }
        }

        this.consumeToken(Token.RCURLY);
        return new AST.ASTObject(attributes);
    }

    parseLParen(): AST.ASTLParen {
        const expression = this.parseExpressionOrNumber();
        this.consumeToken(Token.RPAREN);

        const lParenNode = new AST.ASTLParen(expression);
        return lParenNode;
    }

    parseDecleration(): AST.ASTDeclaration {
        this.consumeToken(Token.DECLARATION);

        const ASTassign = this.parseAssignment(undefined, false);
        const declAST = new AST.ASTDeclaration(ASTassign);

        if (this.tokens[this.pos] && this.tokens[this.pos].isType(Token.SEMI)) {
            this.consumeToken(Token.SEMI);
        }
        return declAST;
    }

    parseAssignment(
        identToken: LexerToken = this.consumeToken(Token.IDENTIFIER),
        allowShortHand = true,
    ): AST.ASTAssign {
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

        const identAST = new AST.ASTIdentifier(identToken.getValue());

        // If looks like a lambda
        if (this.tokens[this.pos].isType(Token.LPAREN)) {
            const ret = this.tryParseLambda(identToken.getValue());
            if (ret) {
                return new AST.ASTAssign(identAST, ret);
            }
        }

        // Already comsumed the assign token
        const expressionAST = this.parseExpressionOrNumber();

        return new AST.ASTAssign(identAST, expressionAST);
    }

    tryParseLambda(name: string) {
        const parseLambda = (name: string) => {
            const args = this.parseParameters();
            this.consumeToken(Token.LAMBDA);

            const block = this.parseBlock();
            return new AST.ASTFunctionDef(name, args, block);
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
    ): AST.ASTAssign {
        this.consumeToken(Token.ASSIGN);
        const expressionAST = this.parseExpressionOrNumber();
        const identAST = new AST.ASTIdentifier(identToken.getValue());

        // +=
        if (assignToken.isType(Token.PLUS)) {
            const resultExpression = new AST.ASTAdd(identAST, expressionAST);
            return new AST.ASTAssign(identAST, resultExpression);
        }

        // -=
        if (assignToken.isType(Token.MINUS)) {
            const resultExpression = new AST.ASTSubtract(
                identAST,
                expressionAST,
            );
            return new AST.ASTAssign(identAST, resultExpression);
        }

        // *=
        if (assignToken.isType(Token.MULTIPLY)) {
            const resultExpression = new AST.ASTMultiply(
                identAST,
                expressionAST,
            );
            return new AST.ASTAssign(identAST, resultExpression);
        }

        // /=
        if (assignToken.isType(Token.DIVIDE)) {
            const resultExpression = new AST.ASTDivide(identAST, expressionAST);
            return new AST.ASTAssign(identAST, resultExpression);
        }

        if (assignToken.isType(Token.INT_DIVIDE)) {
            const resultExpression = new AST.ASTIntegerDivide(
                identAST,
                expressionAST,
            );
            return new AST.ASTAssign(identAST, resultExpression);
        }

        throw new ParserError(
            `Unexpected token ${assignToken.getValue()}. Expected an assignment operator`,
        );
    }

    parseNot(notToken?: LexerToken): AST.ASTNot {
        if (!notToken) {
            this.consumeToken(Token.NOT);
        }

        // Check special case where the next token is a LPAREN
        if (this.tokens[this.pos].isType(Token.LPAREN)) {
            this.consumeToken(Token.LPAREN);
            return new AST.ASTNot(this.parseLParen());
        }

        const expression = this.parseExpressionOrNumber();
        return new AST.ASTNot(expression);
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
