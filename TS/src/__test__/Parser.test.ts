import * as AST from '../AST';
import { ParserError } from '../errors';
import Parser from '../Parser';
import { LexerToken, Token } from '../types';

describe('Parser Math Operators', () => {
    it('should parse a simple expression', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.PLUS, '+'),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST.AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);
        if (!(children[0] instanceof AST.AddASTNode)) {
            throw new Error('Expected AddASTNode');
        }

        // Left then right
        const { left, right } = children[0];
        expect(left.type).toBe(Token.NUMBER);
        if (!(left instanceof AST.NumberASTNode)) {
            throw new Error(
                'Left AST node is not an instance of NumberASTNode',
            );
        }
        expect(left.getValue()).toBe(1);
        expect(right.type).toBe(Token.NUMBER);
        if (!(right instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right AST node is not an instance of NumberASTNode',
            );
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

        expect(ast).toBeInstanceOf(AST.AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [addAST] = children;
        expect(addAST).toBeInstanceOf(AST.AddASTNode);
        if (!(addAST instanceof AST.AddASTNode)) {
            throw new Error('Expected AddASTNode');
        }

        // Left then right
        const { left, right } = addAST;
        expect(left.type).toBe(Token.NUMBER);
        if (!(left instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right AST node is not an instance of NumberASTNode',
            );
        }
        expect(left.getValue()).toBe(1);
        expect(right.type).toBe(Token.MULTIPLY);
        expect(right).toBeInstanceOf(AST.MultiplyASTNode);
        if (!(right instanceof AST.MultiplyASTNode)) {
            throw new Error('Expected MultiplyASTNode');
        }

        // Left then right
        const { left: leftRight, right: rightRight } = right;
        expect(leftRight.type).toBe(Token.NUMBER);
        if (!(leftRight instanceof AST.NumberASTNode)) {
            throw new Error(
                'Left right AST node is not an instance of NumberASTNode',
            );
        }
        expect(leftRight.getValue()).toBe(2);
        expect(rightRight.type).toBe(Token.NUMBER);
        if (!(rightRight instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right right AST node is not an instance of NumberASTNode',
            );
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

        expect(ast).toBeInstanceOf(AST.AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [multAST] = children;
        expect(multAST).toBeInstanceOf(AST.MultiplyASTNode);
        if (!(multAST instanceof AST.MultiplyASTNode)) {
            throw new Error('Expected MultiplyASTNode');
        }

        // Left should be an the parentheses
        const { left, right } = multAST;
        expect(left).toBeInstanceOf(AST.LParenASTNode);
        if (!(left instanceof AST.LParenASTNode)) {
            throw new Error('Expected LParenASTNode');
        }

        const { child } = left;
        expect(child).toBeInstanceOf(AST.AddASTNode);

        // Right should be a number (3)
        expect(right.type).toBe(Token.NUMBER);
        if (!(right instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right AST node is not an instance of NumberASTNode',
            );
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

        expect(ast).toBeInstanceOf(AST.AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AST.AssignASTNode);
        if (!(assignAST instanceof AST.AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
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

        expect(ast).toBeInstanceOf(AST.AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [lessAST] = children;
        expect(lessAST).toBeInstanceOf(AST.LessThanASTNode);
        if (!(lessAST instanceof AST.LessThanASTNode)) {
            throw new Error('Expected LessThanASTNode');
        }

        // Left then right
        const { left, right } = lessAST;
        expect(left.type).toBe(Token.NUMBER);
        if (!(left instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right AST node is not an instance of NumberASTNode',
            );
        }
        expect(left.getValue()).toBe(1);
        expect(right.type).toBe(Token.NUMBER);
        if (!(right instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right AST node is not an instance of NumberASTNode',
            );
        }
        expect(right.getValue()).toBe(2);
    });
});

describe.each([
    [Token.LEQ, AST.LessEqASTNode],
    [Token.GEQ, AST.GreaterEqASTNode],
    [Token.LESS, AST.LessThanASTNode],
    [Token.GREATER, AST.GreaterThanASTNode],
])('should parse boolean operator %s', (tokenType, expectedNode) => {
    it('should parse correctly', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(tokenType, tokenType),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST.AST);

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
        if (!(left instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right AST node is not an instance of NumberASTNode',
            );
        }
        expect(left.getValue()).toBe(1);
        expect(right.type).toBe(Token.NUMBER);
        if (!(right instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right AST node is not an instance of NumberASTNode',
            );
        }
        expect(right.getValue()).toBe(2);
    });
});

describe('Parser Assignment', () => {
    it('should parse a simple assignment', () => {
        const tokens = [
            new LexerToken(Token.DECLERATION, 'let'),
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
        expect(declAST).toBeInstanceOf(AST.DeclarationASTNode);
        if (!(declAST instanceof AST.DeclarationASTNode)) {
            throw new Error('Expected DeclarationASTNode');
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AST.AssignASTNode);
        if (!(child instanceof AST.AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { identifier, valueAST } = child;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');

        expect(valueAST).toBeInstanceOf(AST.NumberASTNode);
        if (!(valueAST instanceof AST.NumberASTNode)) {
            throw new Error(
                'Right AST node is not an instance of NumberASTNode',
            );
        }
        expect(valueAST.getValue()).toBe(1);
    });

    it('should parse a simple assignment with an expression', () => {
        const tokens = [
            new LexerToken(Token.DECLERATION, 'let'),
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
        expect(declAST).toBeInstanceOf(AST.DeclarationASTNode);
        if (!(declAST instanceof AST.DeclarationASTNode)) {
            throw new Error('Expected DeclarationASTNode');
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AST.AssignASTNode);
        if (!(child instanceof AST.AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { identifier, valueAST } = child;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');

        expect(valueAST).toBeInstanceOf(AST.AddASTNode);
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
        expect(assignAST).toBeInstanceOf(AST.AssignASTNode);
        if (!(assignAST instanceof AST.AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { identifier, valueAST } = assignAST;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');

        expect(valueAST).toBeInstanceOf(AST.AddASTNode);
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
        expect(assignAST).toBeInstanceOf(AST.AssignASTNode);
        if (!(assignAST instanceof AST.AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(AST.SubtractASTNode);
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
        expect(assignAST).toBeInstanceOf(AST.AssignASTNode);
        if (!(assignAST instanceof AST.AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(AST.MultiplyASTNode);
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
        expect(assignAST).toBeInstanceOf(AST.AssignASTNode);
        if (!(assignAST instanceof AST.AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(AST.DivideASTNode);
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
        expect(assignAST).toBeInstanceOf(AST.AddASTNode);
    });

    it('should not allow shorthand assignment during declaration', () => {
        const tokens = [
            new LexerToken(Token.DECLERATION, 'let'),
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
        expect(blockAST).toBeInstanceOf(AST.BlockASTNode);
        if (!(blockAST instanceof AST.BlockASTNode)) {
            throw new Error('Expected BlockASTNode');
        }

        const blockChildren = blockAST.children;
        expect(blockChildren).toHaveLength(1);

        const [numberAST] = blockChildren;
        expect(numberAST).toBeInstanceOf(AST.NumberASTNode);
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
        expect(blockAST).toBeInstanceOf(AST.BlockASTNode);
        if (!(blockAST instanceof AST.BlockASTNode)) {
            throw new Error('Expected BlockASTNode');
        }

        const blockChildren = blockAST.children;
        expect(blockChildren).toHaveLength(2);

        const [numberAST1, numberAST2] = blockChildren;
        expect(numberAST1).toBeInstanceOf(AST.NumberASTNode);
        expect(numberAST2).toBeInstanceOf(AST.NumberASTNode);
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
        expect(whileAST).toBeInstanceOf(AST.WhileASTNode);
        if (!(whileAST instanceof AST.WhileASTNode)) {
            throw new Error('Expected BlockASTNode');
        }

        const { condition, block } = whileAST;
        expect(condition).toBeInstanceOf(AST.LessThanASTNode);
        expect(block).toBeInstanceOf(AST.BlockASTNode);
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
        if (!(forAST instanceof AST.ForASTNode)) {
            throw new Error('Expected ForASTNode');
        }

        const { init, condition, update, block } = forAST;
        expect(init).toBeInstanceOf(AST.AssignASTNode);
        expect(condition).toBeInstanceOf(AST.LessThanASTNode);
        expect(update).toBeInstanceOf(AST.AssignASTNode);
        expect(block).toBeInstanceOf(AST.BlockASTNode);
    });

    it('should parser a for loop with a declaration', () => {
        const tokens = [
            new LexerToken(Token.FOR, 'for'),
            new LexerToken(Token.LPAREN, '('),
            new LexerToken(Token.DECLERATION, 'let'),
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
        if (!(forAST instanceof AST.ForASTNode)) {
            throw new Error('Expected ForASTNode');
        }

        const { init, condition, update, block } = forAST;
        expect(init).toBeInstanceOf(AST.DeclarationASTNode);
        expect(condition).toBeInstanceOf(AST.LessThanASTNode);
        expect(update).toBeInstanceOf(AST.AssignASTNode);
        expect(block).toBeInstanceOf(AST.BlockASTNode);
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
        if (!(ifAST instanceof AST.IfASTNode)) {
            throw new Error('Expected IfASTNode');
        }

        const { condition, block, elseBlock } = ifAST;
        expect(condition).toBeInstanceOf(AST.LessThanASTNode);
        expect(block).toBeInstanceOf(AST.BlockASTNode);
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
        expect(ifAST).toBeInstanceOf(AST.IfASTNode);
        if (!(ifAST instanceof AST.IfASTNode)) {
            throw new Error('Expected IfASTNode');
        }

        const { condition, block, elseBlock } = ifAST;
        expect(condition).toBeInstanceOf(AST.LessThanASTNode);
        expect(block).toBeInstanceOf(AST.BlockASTNode);
        expect(elseBlock).toBeInstanceOf(AST.BlockASTNode);
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
        expect(functionAST).toBeInstanceOf(AST.FunctionDefASTNode);
        if (!(functionAST instanceof AST.FunctionDefASTNode)) {
            throw new Error('Expected FunctionDefASTNode');
        }

        const { block } = functionAST;
        expect(block).toBeInstanceOf(AST.BlockASTNode);
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
        expect(functionDef).toBeInstanceOf(AST.FunctionDefASTNode);
        if (!(functionDef instanceof AST.FunctionDefASTNode)) {
            throw new Error('Expected FunctionDefASTNode');
        }

        const { block } = functionDef;
        expect(block).toBeInstanceOf(AST.BlockASTNode);

        const params = (functionDef as AST.FunctionDefASTNode).getParamList();
        expect(params).toHaveLength(2);

        const [param] = params;
        expect(param).toBeInstanceOf(AST.IdentifierASTNode);
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
        expect(functionCall).toBeInstanceOf(AST.FunctionCallASTNode);
        if (!(functionCall instanceof AST.FunctionCallASTNode)) {
            throw new Error('Expected FunctionCallASTNode');
        }

        expect(functionCall.args).toHaveLength(2);

        const [arg1, arg2] = functionCall.args;
        expect(arg1).toBeInstanceOf(AST.NumberASTNode);
        expect(arg2).toBeInstanceOf(AST.NumberASTNode);
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
        expect(functionCall).toBeInstanceOf(AST.FunctionCallASTNode);
        if (!(functionCall instanceof AST.FunctionCallASTNode)) {
            throw new Error('Expected FunctionCallASTNode');
        }

        expect(functionCall.args).toHaveLength(1);
        const [expression] = functionCall.args;
        expect(expression).toBeInstanceOf(AST.AddASTNode);
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
        expect(functionCall).toBeInstanceOf(AST.FunctionCallASTNode);
        if (!(functionCall instanceof AST.FunctionCallASTNode)) {
            throw new Error('Expected FunctionCallASTNode');
        }
        expect(functionCall.args).toHaveLength(1);

        const [identifier] = functionCall.args;
        expect(identifier).toBeInstanceOf(AST.IdentifierASTNode);
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
        expect(functionCall).toBeInstanceOf(AST.FunctionCallASTNode);
        if (!(functionCall instanceof AST.FunctionCallASTNode)) {
            throw new Error('Expected FunctionCallASTNode');
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
        expect(addAST).toBeInstanceOf(AST.AddASTNode);
        if (!(addAST instanceof AST.AddASTNode)) {
            throw new Error('Expected AddASTNode');
        }

        const { left, right } = addAST;
        expect(left).toBeInstanceOf(AST.FunctionCallASTNode);
        expect(right).toBeInstanceOf(AST.FunctionCallASTNode);
    });
});

describe('String Parsing', () => {
    it('should parse a simgple string', () => {
        const tokens = [
            new LexerToken(Token.DECLERATION, 'let'),
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
        expect(declAST).toBeInstanceOf(AST.DeclarationASTNode);
        if (!(declAST instanceof AST.DeclarationASTNode)) {
            return;
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AST.AssignASTNode);
        if (!(child instanceof AST.AssignASTNode)) {
            return;
        }

        const { identifier, valueAST } = child;
        expect(identifier.type).toBe(Token.IDENTIFIER);
        expect(identifier.getName()).toBe('x');

        expect(valueAST).toBeInstanceOf(AST.StringASTNode);
        if (!(valueAST instanceof AST.StringASTNode)) {
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
        expect(addAST).toBeInstanceOf(AST.AddASTNode);
        if (!(addAST instanceof AST.AddASTNode)) {
            throw new Error('Expected AddASTNode');
        }

        const { left, right } = addAST;
        expect(left).toBeInstanceOf(AST.StringASTNode);
        expect(right).toBeInstanceOf(AST.StringASTNode);
    });
});
