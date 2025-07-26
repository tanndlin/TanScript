import * as AST from '../AST';
import { ParserError } from '../errors';
import Parser from '../Parser';
import { LexerToken, Token } from '../types';

const expectNumber = (node: AST.AnyAST): node is AST.ASTNumber => {
    expect(node.type).toBe(Token.NUMBER);
    expect(node).toBeInstanceOf(AST.ASTNumber);
    return node.type === Token.NUMBER;
};

describe('Parser Math Operators', () => {
    it('should parse a simple expression', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST.Program);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);
        if (!(children[0].type === Token.PLUS)) {
            throw new Error('Expected ASTAdd');
        }

        // Left then right
        const { left, right } = children[0];
        expect(left.type).toBe(Token.NUMBER);
        if (!expectNumber(left)) {
            return;
        }
        expect(left.getValue()).toBe(1);
        expect(right.type).toBe(Token.NUMBER);
        if (!expectNumber(right)) {
            return;
        }
        expect(right.getValue()).toBe(2);
    });

    it('should respect PEMDAS', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.MULTIPLY, '*'),
            new LexerToken(Token.NUMBER, '3'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST.Program);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [addAST] = children;
        expect(addAST).toBeInstanceOf(AST.ASTAdd);
        if (!(addAST.type === Token.PLUS)) {
            throw new Error('Expected ASTAdd');
        }

        // Left then right
        const { left, right } = addAST;
        expect(left.type).toBe(Token.NUMBER);
        if (!expectNumber(left)) {
            return;
        }
        expect(left.getValue()).toBe(1);
        expect(right.type).toBe(Token.MULTIPLY);
        expect(right).toBeInstanceOf(AST.ASTMultiply);
        if (!(right instanceof AST.ASTMultiply)) {
            throw new Error('Expected ASTMultiply');
        }

        // Left then right
        const { left: leftRight, right: rightRight } = right;
        expect(leftRight.type).toBe(Token.NUMBER);
        if (!expectNumber(leftRight)) {
            return;
        }
        expect(leftRight.getValue()).toBe(2);
        expect(rightRight.type).toBe(Token.NUMBER);
        if (!expectNumber(rightRight)) {
            return;
        }
        expect(rightRight.getValue()).toBe(3);
    });

    it('Parentheses should be respected', () => {
        const tokens = [
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.MULTIPLY, '*'),
            new LexerToken(Token.NUMBER, '3'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST.Program);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [multAST] = children;
        expect(multAST).toBeInstanceOf(AST.ASTMultiply);
        if (!(multAST instanceof AST.ASTMultiply)) {
            throw new Error('Expected ASTMultiply');
        }

        // Left should be an the parentheses
        const { left, right } = multAST;
        expect(left).toBeInstanceOf(AST.ASTLParen);
        if (!(left instanceof AST.ASTLParen)) {
            throw new Error('Expected ASTLParen');
        }

        const { child } = left;
        expect(child).toBeInstanceOf(AST.ASTAdd);

        // Right should be a number (3)
        expect(right.type).toBe(Token.NUMBER);
        if (!expectNumber(right)) {
            return;
        }
        expect(right.getValue()).toBe(3);
    });

    it.each([
        new LexerToken(Token.INCREMENT, '++'),
        new LexerToken(Token.DECREMENT, '--'),
    ])('should parse an increment/decrement', (token) => {
        const tokens = [new LexerToken(Token.IDENTIFIER, 'x'), token];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST.Program);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AST.ASTAssign);
        if (!(assignAST.type === Token.ASSIGN)) {
            throw new Error('Expected ASTAssign');
        }

        // Left should be an identifier
        const { identifier } = assignAST;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');
    });

    it('should parse a basic conditional', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST.Program);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [lessAST] = children;
        expect(lessAST).toBeInstanceOf(AST.ASTLessThan);
        if (!(lessAST instanceof AST.ASTLessThan)) {
            throw new Error('Expected ASTLessThan');
        }

        // Left then right
        const { left, right } = lessAST;
        expect(left.type).toBe(Token.NUMBER);
        if (!expectNumber(left)) {
            return;
        }
        expect(left.getValue()).toBe(1);
        expect(right.type).toBe(Token.NUMBER);
        if (!expectNumber(right)) {
            return;
        }
        expect(right.getValue()).toBe(2);
    });
});

describe.each([
    [Token.LEQ, AST.ASTLessEq],
    [Token.GEQ, AST.ASTGreaterEq],
    [Token.LESS, AST.ASTLessThan],
    [Token.GREATER, AST.ASTGreaterThan],
])('should parse boolean operator %s', (tokenType, expectedNode) => {
    it('should parse correctly', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(tokenType, tokenType),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST.Program);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [expectedAST] = children;
        expect(expectedAST).toBeInstanceOf(expectedNode);
        if (!(expectedAST instanceof expectedNode)) {
            throw new Error(`Expected ${expectedNode}`);
        }

        // Left then right
        const { left, right } = expectedAST;
        expect(left.type).toBe(Token.NUMBER);
        if (!expectNumber(left)) {
            return;
        }
        expect(left.getValue()).toBe(1);
        expect(right.type).toBe(Token.NUMBER);
        if (!expectNumber(right)) {
            return;
        }
        expect(right.getValue()).toBe(2);
    });
});

describe('Parser Assignment', () => {
    it('should parse a simple assignment', () => {
        const tokens = [
            new LexerToken(Token.DECLARATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.children;
        expect(children).toHaveLength(1);

        const [declAST] = children;
        expect(declAST).toBeInstanceOf(AST.ASTDeclaration);
        if (!(declAST.type === Token.DECLARATION)) {
            throw new Error('Expected ASTDeclaration');
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AST.ASTAssign);
        if (!(child.type === Token.ASSIGN)) {
            throw new Error('Expected ASTAssign');
        }

        const { identifier, valueAST } = child;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');

        expect(valueAST).toBeInstanceOf(AST.ASTNumber);
        if (!expectNumber(valueAST)) {
            return;
        }
        expect(valueAST.getValue()).toBe(1);
    });

    it('should parse a simple assignment with an expression', () => {
        const tokens = [
            new LexerToken(Token.DECLARATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.children;
        expect(children).toHaveLength(1);

        const [declAST] = children;
        expect(declAST).toBeInstanceOf(AST.ASTDeclaration);
        if (!(declAST.type === Token.DECLARATION)) {
            throw new Error('Expected ASTDeclaration');
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AST.ASTAssign);
        if (!(child.type === Token.ASSIGN)) {
            throw new Error('Expected ASTAssign');
        }

        const { identifier, valueAST } = child;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');

        expect(valueAST).toBeInstanceOf(AST.ASTAdd);
    });

    it('should allow shorthand add assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.children;
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AST.ASTAssign);
        if (!(assignAST.type === Token.ASSIGN)) {
            throw new Error('Expected ASTAssign');
        }

        const { identifier, valueAST } = assignAST;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');

        expect(valueAST).toBeInstanceOf(AST.ASTAdd);
    });

    it('should allow shorthand minus assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.MINUS, '-'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.children;
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AST.ASTAssign);
        if (!(assignAST.type === Token.ASSIGN)) {
            throw new Error('Expected ASTAssign');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(AST.ASTSubtract);
    });

    it('should allow shorthand multiply assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.MULTIPLY, '*'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.children;
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AST.ASTAssign);
        if (!(assignAST.type === Token.ASSIGN)) {
            throw new Error('Expected ASTAssign');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(AST.ASTMultiply);
    });

    it('should allow shorthand divide assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.DIVIDE, '/'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.children;
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AST.ASTAssign);
        if (!(assignAST.type === Token.ASSIGN)) {
            throw new Error('Expected ASTAssign');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(AST.ASTDivide);
    });

    it('should allow using var without assignment', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '1'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const children = root.children;
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AST.ASTAdd);
    });

    it('should not allow shorthand assignment during declaration', () => {
        const tokens = [
            new LexerToken(Token.DECLARATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.EOF, ''),
        ];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });
});

describe('Parser Error Cases', () => {
    it('should throw error for unexpected token', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.PLUS, '+'),
        ];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });

    it('should throw error for unexpected token in parseExpressionOrNumber', () => {
        const tokens = [new LexerToken(Token.PLUS, '+')];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });

    it('should throw error for missing right paren', () => {
        const tokens = [
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.EOF, ''),
        ];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });

    it('should throw error for unexpected EOF in parseAssignment', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.EOF, ''),
        ];

        const parser = new Parser(tokens);
        expect(() => parser.parse()).toThrow(ParserError);
    });
});

describe('Parser Curly Braces', () => {
    it('should parse a simple block', () => {
        const tokens = [
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [blockAST] = root.children;
        expect(blockAST).toBeInstanceOf(AST.ASTBlock);
        if (!(blockAST instanceof AST.ASTBlock)) {
            return;
        }

        const blockChildren = blockAST.children;
        expect(blockChildren).toHaveLength(1);

        const [numberAST] = blockChildren;
        expect(numberAST).toBeInstanceOf(AST.ASTNumber);
    });

    it('should parse a block with multiple children', () => {
        const tokens = [
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [blockAST] = root.children;
        expect(blockAST).toBeInstanceOf(AST.ASTBlock);
        if (!(blockAST instanceof AST.ASTBlock)) {
            return;
        }

        const blockChildren = blockAST.children;
        expect(blockChildren).toHaveLength(2);

        const [numberAST1, numberAST2] = blockChildren;
        expect(numberAST1).toBeInstanceOf(AST.ASTNumber);
        expect(numberAST2).toBeInstanceOf(AST.ASTNumber);
    });
});

describe('Control Structures', () => {
    it('should parse a while loop', () => {
        const tokens = [
            new LexerToken(Token.WHILE, 'while'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [whileAST] = root.children;
        expect(whileAST).toBeInstanceOf(AST.ASTWhile);
        if (!(whileAST instanceof AST.ASTWhile)) {
            throw new Error('Expected ASTBlock');
        }

        const { condition, block } = whileAST;
        expect(condition).toBeInstanceOf(AST.ASTLessThan);
        expect(block).toBeInstanceOf(AST.ASTBlock);
    });

    it('should parse a for loop', () => {
        const tokens = [
            new LexerToken(Token.FOR, 'for'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '0'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [forAST] = root.children;
        if (!(forAST instanceof AST.ASTFor)) {
            throw new Error('Expected ASTFor');
        }

        const { init, condition, update, block } = forAST;
        expect(init).toBeInstanceOf(AST.ASTAssign);
        expect(condition).toBeInstanceOf(AST.ASTLessThan);
        expect(update).toBeInstanceOf(AST.ASTAssign);
        expect(block).toBeInstanceOf(AST.ASTBlock);
    });

    it('should parser a for loop with a declaration', () => {
        const tokens = [
            new LexerToken(Token.FOR, 'for'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.DECLARATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '0'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [forAST] = root.children;
        if (!(forAST instanceof AST.ASTFor)) {
            throw new Error('Expected ASTFor');
        }

        const { init, condition, update, block } = forAST;
        expect(init).toBeInstanceOf(AST.ASTDeclaration);
        expect(condition).toBeInstanceOf(AST.ASTLessThan);
        expect(update).toBeInstanceOf(AST.ASTAssign);
        expect(block).toBeInstanceOf(AST.ASTBlock);
    });

    it('should parse a simple if statement', () => {
        const tokens = [
            new LexerToken(Token.IF, 'if'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [ifAST] = root.children;
        if (!(ifAST instanceof AST.ASTIf)) {
            throw new Error('Expected ASTIf');
        }

        const { condition, block, elseBlock } = ifAST;
        expect(condition).toBeInstanceOf(AST.ASTLessThan);
        expect(block).toBeInstanceOf(AST.ASTBlock);
        expect(elseBlock).toBeUndefined();
    });

    it('should parse an if statement with an else block', () => {
        const tokens = [
            new LexerToken(Token.IF, 'if'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '10'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.ELSE, 'else'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [ifAST] = root.children;
        expect(ifAST).toBeInstanceOf(AST.ASTIf);
        if (!(ifAST instanceof AST.ASTIf)) {
            throw new Error('Expected ASTIf');
        }

        const { condition, block, elseBlock } = ifAST;
        expect(condition).toBeInstanceOf(AST.ASTLessThan);
        expect(block).toBeInstanceOf(AST.ASTBlock);
        expect(elseBlock).toBeInstanceOf(AST.ASTBlock);
    });

    it('should parse a function with no params', () => {
        const tokens = [
            new LexerToken(Token.FUNCTION, 'def'),
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionAST] = root.children;
        expect(functionAST).toBeInstanceOf(AST.ASTFunctionDef);
        if (!(functionAST instanceof AST.ASTFunctionDef)) {
            throw new Error('Expected ASTFunctionDef');
        }

        const { block } = functionAST;
        expect(block).toBeInstanceOf(AST.ASTBlock);
    });

    it('should parse a function with params', () => {
        const tokens = [
            new LexerToken(Token.FUNCTION, 'def'),
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.COMMA, ','),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.LCURLY, '{'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.IDENTIFIER, 'y'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.RCURLY, '}'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionDef] = root.children;
        expect(functionDef).toBeInstanceOf(AST.ASTFunctionDef);
        if (!(functionDef instanceof AST.ASTFunctionDef)) {
            throw new Error('Expected ASTFunctionDef');
        }

        const { block } = functionDef;
        expect(block).toBeInstanceOf(AST.ASTBlock);

        const params = (functionDef as AST.ASTFunctionDef).getParamList();
        expect(params).toHaveLength(2);

        const [param] = params;
        expect(param).toBeInstanceOf(AST.ASTIdentifier);
    });

    it('should parse function calls', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.COMMA, ','),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionCall] = root.children;
        expect(functionCall).toBeInstanceOf(AST.ASTFunctionCall);
        if (!(functionCall instanceof AST.ASTFunctionCall)) {
            throw new Error('Expected ASTFunctionCall');
        }

        expect(functionCall.args).toHaveLength(2);

        const [arg1, arg2] = functionCall.args;
        expect(arg1).toBeInstanceOf(AST.ASTNumber);
        expect(arg2).toBeInstanceOf(AST.ASTNumber);
    });

    it('should parse function calls with expressions', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionCall] = root.children;
        expect(functionCall).toBeInstanceOf(AST.ASTFunctionCall);
        if (!(functionCall instanceof AST.ASTFunctionCall)) {
            throw new Error('Expected ASTFunctionCall');
        }

        expect(functionCall.args).toHaveLength(1);
        const [expression] = functionCall.args;
        expect(expression).toBeInstanceOf(AST.ASTAdd);
    });

    it('should parse function calls with identifiers', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionCall] = root.children;
        expect(functionCall).toBeInstanceOf(AST.ASTFunctionCall);
        if (!(functionCall instanceof AST.ASTFunctionCall)) {
            throw new Error('Expected ASTFunctionCall');
        }
        expect(functionCall.args).toHaveLength(1);

        const [identifier] = functionCall.args;
        expect(identifier).toBeInstanceOf(AST.ASTIdentifier);
    });

    it('should parse function calls with no args', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [functionCall] = root.children;
        expect(functionCall).toBeInstanceOf(AST.ASTFunctionCall);
        if (!(functionCall instanceof AST.ASTFunctionCall)) {
            throw new Error('Expected ASTFunctionCall');
        }

        expect(functionCall.args).toHaveLength(0);
    });

    it('should be able to add function calls', () => {
        const tokens = [
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.COMMA, ','),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.IDENTIFIER, 'foo'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.COMMA, ','),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.RPAREN, ')'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [addAST] = root.children;
        expect(addAST).toBeInstanceOf(AST.ASTAdd);
        if (!(addAST.type === Token.PLUS)) {
            throw new Error('Expected ASTAdd');
        }

        const { left, right } = addAST;
        expect(left).toBeInstanceOf(AST.ASTFunctionCall);
        expect(right).toBeInstanceOf(AST.ASTFunctionCall);
    });
});

describe('String Parsing', () => {
    it('should parse a simgple string', () => {
        const tokens = [
            new LexerToken(Token.DECLARATION, 'let'),
            new LexerToken(Token.IDENTIFIER, 'x'),
            new LexerToken(Token.ASSIGN, '='),
            new LexerToken(Token.STRING, 'Hello world!'),
            new LexerToken(Token.SEMI, ';'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [declAST] = root.children;
        expect(declAST).toBeInstanceOf(AST.ASTDeclaration);
        if (!(declAST.type === Token.DECLARATION)) {
            return;
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AST.ASTAssign);
        if (!(child.type === Token.ASSIGN)) {
            return;
        }

        const { identifier, valueAST } = child;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');

        expect(valueAST).toBeInstanceOf(AST.ASTString);
        if (!(valueAST instanceof AST.ASTString)) {
            return;
        }

        expect(valueAST.getValue()).toBe('Hello world!');
    });

    it('should parse string concatenation', () => {
        const tokens = [
            new LexerToken(Token.STRING, 'hello'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.STRING, 'world'),
            new LexerToken(Token.EOF, 'EOF'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [addAST] = root.children;
        expect(addAST).toBeInstanceOf(AST.ASTAdd);
        if (!((addAST as unknown as AST.AnyAST).type === Token.PLUS)) {
            throw new Error('Expected ASTAdd');
        }

        const { left, right } = addAST as unknown as AST.ASTAdd;
        expect(left).toBeInstanceOf(AST.ASTString);
        expect(right).toBeInstanceOf(AST.ASTString);
    });
});
