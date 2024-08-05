import {
    AST,
    AssignASTNode,
    BlockASTNode,
    DeclarationASTNode,
    IdentifierASTNode,
    LParenASTNode,
    StringASTNode,
} from '../AST/AST';
import {
    GreaterEqASTNode,
    GreaterThanASTNode,
    LessEqASTNode,
    LessThanASTNode,
} from '../AST/BoolAST';
import {
    ForASTNode,
    FunctionCallASTNode,
    FunctionDefASTNode,
    IfASTNode,
    WhileASTNode,
} from '../AST/ControlAST';
import {
    AddASTNode,
    DivideASTNode,
    MultiplyASTNode,
    NumberASTNode,
    SubtractASTNode,
} from '../AST/NumberAST';
import Parser from '../Parser';
import { ParserError } from '../errors';
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

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);
        if (!(children[0] instanceof AddASTNode)) {
            throw new Error('Expected AddASTNode');
        }

        // Left then right
        const { left, right } = children[0];
        expect(left.getType()).toBe(Token.NUMBER);
        expect(left.getValue()).toBe('1');
        expect(right.getType()).toBe(Token.NUMBER);
        expect(right.getValue()).toBe('2');
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

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [addAST] = children;
        expect(addAST).toBeInstanceOf(AddASTNode);
        if (!(addAST instanceof AddASTNode)) {
            throw new Error('Expected AddASTNode');
        }

        // Left then right
        const { left, right } = addAST;
        expect(left.getType()).toBe(Token.NUMBER);
        expect(left.getValue()).toBe('1');
        expect(right.getType()).toBe(Token.MULTIPLY);
        expect(right.getValue()).toBe('*');
        expect(right).toBeInstanceOf(MultiplyASTNode);
        if (!(right instanceof MultiplyASTNode)) {
            throw new Error('Expected MultiplyASTNode');
        }

        // Left then right
        const { left: leftRight, right: rightRight } = right;
        expect(leftRight.getType()).toBe(Token.NUMBER);
        expect(leftRight.getValue()).toBe('2');
        expect(rightRight.getType()).toBe(Token.NUMBER);
        expect(rightRight.getValue()).toBe('3');
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

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [multAST] = children;
        expect(multAST).toBeInstanceOf(MultiplyASTNode);
        if (!(multAST instanceof MultiplyASTNode)) {
            throw new Error('Expected MultiplyASTNode');
        }

        // Left should be an the parentheses
        const { left, right } = multAST;
        expect(left).toBeInstanceOf(LParenASTNode);
        if (!(left instanceof LParenASTNode)) {
            throw new Error('Expected LParenASTNode');
        }

        const { child } = left;
        expect(child).toBeInstanceOf(AddASTNode);

        // Right should be a number (3)
        expect(right.getType()).toBe(Token.NUMBER);
        expect(right.getValue()).toBe('3');
    });

    it.each([
        new LexerToken(Token.INCREMENT, '++'),
        new LexerToken(Token.DECREMENT, '--'),
    ])('should parse an increment/decrement', (token) => {
        const tokens = [new LexerToken(Token.IDENTIFIER, 'x'), token];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [assignAST] = children;
        expect(assignAST).toBeInstanceOf(AssignASTNode);
        if (!(assignAST instanceof AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        // Left should be an identifier
        const { identifier } = assignAST;
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');
    });

    it('should parse a basic conditional', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(Token.LESS, '<'),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST);

        const root = ast.getRoot();
        const children = root.children;
        expect(children).toHaveLength(1);

        const [lessAST] = children;
        expect(lessAST).toBeInstanceOf(LessThanASTNode);
        if (!(lessAST instanceof LessThanASTNode)) {
            throw new Error('Expected LessThanASTNode');
        }

        // Left then right
        const { left, right } = lessAST;
        expect(left.getType()).toBe(Token.NUMBER);
        expect(left.getValue()).toBe('1');
        expect(right.getType()).toBe(Token.NUMBER);
        expect(right.getValue()).toBe('2');
    });
});

describe.each([
    [Token.LEQ, LessEqASTNode],
    [Token.GEQ, GreaterEqASTNode],
    [Token.LESS, LessThanASTNode],
    [Token.GREATER, GreaterThanASTNode],
])('should parse boolean operator %s', (tokenType, expectedNode) => {
    it('should parse correctly', () => {
        const tokens = [
            new LexerToken(Token.NUMBER, '1'),
            new LexerToken(tokenType, tokenType),
            new LexerToken(Token.NUMBER, '2'),
        ];

        const parser = new Parser(tokens);
        const ast = parser.parse();

        expect(ast).toBeInstanceOf(AST);

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
        expect(left.getType()).toBe(Token.NUMBER);
        expect(left.getValue()).toBe('1');
        expect(right.getType()).toBe(Token.NUMBER);
        expect(right.getValue()).toBe('2');
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
        expect(declAST).toBeInstanceOf(DeclarationASTNode);
        if (!(declAST instanceof DeclarationASTNode)) {
            throw new Error('Expected DeclarationASTNode');
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AssignASTNode);
        if (!(child instanceof AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { identifier, valueAST } = child;
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');

        expect(valueAST).toBeInstanceOf(NumberASTNode);
        expect(valueAST.getValue()).toBe('1');
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
        expect(declAST).toBeInstanceOf(DeclarationASTNode);
        if (!(declAST instanceof DeclarationASTNode)) {
            throw new Error('Expected DeclarationASTNode');
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AssignASTNode);
        if (!(child instanceof AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { identifier, valueAST } = child;
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');

        expect(valueAST).toBeInstanceOf(AddASTNode);
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
        expect(assignAST).toBeInstanceOf(AssignASTNode);
        if (!(assignAST instanceof AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { identifier, valueAST } = assignAST;
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');

        expect(valueAST).toBeInstanceOf(AddASTNode);
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
        expect(assignAST).toBeInstanceOf(AssignASTNode);
        if (!(assignAST instanceof AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(SubtractASTNode);
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
        expect(assignAST).toBeInstanceOf(AssignASTNode);
        if (!(assignAST instanceof AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(MultiplyASTNode);
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
        expect(assignAST).toBeInstanceOf(AssignASTNode);
        if (!(assignAST instanceof AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { valueAST } = assignAST;
        expect(valueAST).toBeInstanceOf(DivideASTNode);
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
        expect(assignAST).toBeInstanceOf(AddASTNode);
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
        expect(blockAST).toBeInstanceOf(BlockASTNode);
        if (!(blockAST instanceof BlockASTNode)) {
            throw new Error('Expected BlockASTNode');
        }

        const blockChildren = blockAST.children;
        expect(blockChildren).toHaveLength(1);

        const [numberAST] = blockChildren;
        expect(numberAST).toBeInstanceOf(NumberASTNode);
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
        expect(blockAST).toBeInstanceOf(BlockASTNode);
        if (!(blockAST instanceof BlockASTNode)) {
            throw new Error('Expected BlockASTNode');
        }

        const blockChildren = blockAST.children;
        expect(blockChildren).toHaveLength(2);

        const [numberAST1, numberAST2] = blockChildren;
        expect(numberAST1).toBeInstanceOf(NumberASTNode);
        expect(numberAST2).toBeInstanceOf(NumberASTNode);
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
        expect(whileAST).toBeInstanceOf(WhileASTNode);
        if (!(whileAST instanceof WhileASTNode)) {
            throw new Error('Expected BlockASTNode');
        }

        const { condition, block } = whileAST;
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(block).toBeInstanceOf(BlockASTNode);
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
        if (!(forAST instanceof ForASTNode)) {
            throw new Error('Expected ForASTNode');
        }

        const { init, condition, update, block } = forAST;
        expect(init).toBeInstanceOf(AssignASTNode);
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(update).toBeInstanceOf(AssignASTNode);
        expect(block).toBeInstanceOf(BlockASTNode);
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
        if (!(forAST instanceof ForASTNode)) {
            throw new Error('Expected ForASTNode');
        }

        const { init, condition, update, block } = forAST;
        expect(init).toBeInstanceOf(DeclarationASTNode);
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(update).toBeInstanceOf(AssignASTNode);
        expect(block).toBeInstanceOf(BlockASTNode);
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
        if (!(ifAST instanceof IfASTNode)) {
            throw new Error('Expected IfASTNode');
        }

        const { condition, block, elseBlock } = ifAST;
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(block).toBeInstanceOf(BlockASTNode);
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
        expect(ifAST).toBeInstanceOf(IfASTNode);
        if (!(ifAST instanceof IfASTNode)) {
            throw new Error('Expected IfASTNode');
        }

        const { condition, block, elseBlock } = ifAST;
        expect(condition).toBeInstanceOf(LessThanASTNode);
        expect(block).toBeInstanceOf(BlockASTNode);
        expect(elseBlock).toBeInstanceOf(BlockASTNode);
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
        expect(functionAST).toBeInstanceOf(FunctionDefASTNode);
        if (!(functionAST instanceof FunctionDefASTNode)) {
            throw new Error('Expected FunctionDefASTNode');
        }

        const { block } = functionAST;
        expect(block).toBeInstanceOf(BlockASTNode);
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
        expect(functionDef).toBeInstanceOf(FunctionDefASTNode);
        if (!(functionDef instanceof FunctionDefASTNode)) {
            throw new Error('Expected FunctionDefASTNode');
        }

        const { block } = functionDef;
        expect(block).toBeInstanceOf(BlockASTNode);

        const params = (functionDef as FunctionDefASTNode).getParamList();
        expect(params).toHaveLength(2);

        const [param] = params;
        expect(param).toBeInstanceOf(IdentifierASTNode);
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
        expect(functionCall).toBeInstanceOf(FunctionCallASTNode);
        if (!(functionCall instanceof FunctionCallASTNode)) {
            throw new Error('Expected FunctionCallASTNode');
        }

        expect(functionCall.args).toHaveLength(2);

        const [arg1, arg2] = functionCall.args;
        expect(arg1).toBeInstanceOf(NumberASTNode);
        expect(arg2).toBeInstanceOf(NumberASTNode);
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
        expect(functionCall).toBeInstanceOf(FunctionCallASTNode);
        if (!(functionCall instanceof FunctionCallASTNode)) {
            throw new Error('Expected FunctionCallASTNode');
        }

        expect(functionCall.args).toHaveLength(1);
        const [expression] = functionCall.args;
        expect(expression).toBeInstanceOf(AddASTNode);
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
        expect(functionCall).toBeInstanceOf(FunctionCallASTNode);
        if (!(functionCall instanceof FunctionCallASTNode)) {
            throw new Error('Expected FunctionCallASTNode');
        }
        expect(functionCall.args).toHaveLength(1);

        const [identifier] = functionCall.args;
        expect(identifier).toBeInstanceOf(IdentifierASTNode);
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
        expect(functionCall).toBeInstanceOf(FunctionCallASTNode);
        if (!(functionCall instanceof FunctionCallASTNode)) {
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
        expect(addAST).toBeInstanceOf(AddASTNode);
        if (!(addAST instanceof AddASTNode)) {
            throw new Error('Expected AddASTNode');
        }

        const { left, right } = addAST;
        expect(left).toBeInstanceOf(FunctionCallASTNode);
        expect(right).toBeInstanceOf(FunctionCallASTNode);
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
        expect(declAST).toBeInstanceOf(DeclarationASTNode);
        if (!(declAST instanceof DeclarationASTNode)) {
            throw new Error('Expected DeclarationASTNode');
        }

        const { child } = declAST;
        expect(child).toBeInstanceOf(AssignASTNode);
        if (!(child instanceof AssignASTNode)) {
            throw new Error('Expected AssignASTNode');
        }

        const { identifier, valueAST } = child;
        expect(identifier.getType()).toBe(Token.IDENTIFIER);
        expect(identifier.getValue()).toBe('x');

        expect(valueAST).toBeInstanceOf(StringASTNode);
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
        expect(addAST).toBeInstanceOf(AddASTNode);
        if (!(addAST instanceof AddASTNode)) {
            throw new Error('Expected AddASTNode');
        }

        const { left, right } = addAST;
        expect(left).toBeInstanceOf(StringASTNode);
        expect(right).toBeInstanceOf(StringASTNode);
    });
});
