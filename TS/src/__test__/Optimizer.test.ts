import { BlockASTNode, EOFASTNode } from '../AST/AST';
import { LessThanASTNode } from '../AST/BoolAST';
import { ForASTNode, ReturnASTNode } from '../AST/ControlAST';
import Lexer from '../Lexer';
import Optimizer from '../Optimizer';
import Parser from '../Parser';
import { Token } from '../types';

describe('Optimizer: Simplify always true/false', () => {
    it('should simplify always true', () => {
        const script = `
            if (true) {
                return 1;
            } else {
                return 2;
            }
        `;

        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block, eof] = root.children;
        expect(block).toBeInstanceOf(BlockASTNode);
        if (!(block instanceof BlockASTNode)) {
            throw new Error('Block is not an instance of BlockASTNode');
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(ReturnASTNode);
        if (!(retStatement instanceof ReturnASTNode)) {
            throw new Error(
                'Return statement is not an instance of ReturnASTNode'
            );
        }

        expect(+retStatement.valueAST.getValue()).toBe(1);
    });

    it('should simplify always false', () => {
        const script = `
            if (false) {
                return 1;
            } else {
                return 2;
            }
        `;

        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block, eof] = root.children;
        expect(block).toBeInstanceOf(BlockASTNode);
        if (!(block instanceof BlockASTNode)) {
            throw new Error('Block is not an instance of BlockASTNode');
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(ReturnASTNode);
        if (!(retStatement instanceof ReturnASTNode)) {
            throw new Error(
                'Return statement is not an instance of ReturnASTNode'
            );
        }

        expect(+retStatement.valueAST.getValue()).toBe(2);
    });

    it('should simplify always true without else block', () => {
        const script = `
            if (true) {
                return 1;
            }
        `;

        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block, eof] = root.children;
        expect(block).toBeInstanceOf(BlockASTNode);
        if (!(block instanceof BlockASTNode)) {
            throw new Error('Block is not an instance of BlockASTNode');
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(ReturnASTNode);
        if (!(retStatement instanceof ReturnASTNode)) {
            throw new Error(
                'Return statement is not an instance of ReturnASTNode'
            );
        }

        expect(+retStatement.valueAST.getValue()).toBe(1);
    });

    it('should simplify always false without else block', () => {
        const script = `
            if (false) {
                return 1;
            }
        `;

        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();

        expect(root.children).toHaveLength(1);
        const [eof] = root.children;
        expect(eof).toBeInstanceOf(EOFASTNode);
    });

    it('should simplify if condition is not a boolean', () => {
        const script = `
            if (1) {
                return 1;
            } else {
                return 2;
            }
        `;

        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block, eof] = root.children;
        expect(block).toBeInstanceOf(BlockASTNode);
        if (!(block instanceof BlockASTNode)) {
            throw new Error('Block is not an instance of BlockASTNode');
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(ReturnASTNode);
        if (!(retStatement instanceof ReturnASTNode)) {
            throw new Error(
                'Return statement is not an instance of ReturnASTNode'
            );
        }

        expect(+retStatement.valueAST.getValue()).toBe(1);
    });

    it('should simplify ifs with compound expressions', () => {
        const script = `
            if (1 < 2) {
                return 1;
            } else {
                return 2;
            }
        `;

        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block, eof] = root.children;
        expect(block).toBeInstanceOf(BlockASTNode);
        if (!(block instanceof BlockASTNode)) {
            throw new Error('Block is not an instance of BlockASTNode');
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(ReturnASTNode);
        if (!(retStatement instanceof ReturnASTNode)) {
            throw new Error(
                'Return statement is not an instance of ReturnASTNode'
            );
        }

        expect(+retStatement.valueAST.getValue()).toBe(1);
    });
});

describe('Optimizer: Simplify compound expressions', () => {
    it('should simplifiy constant number comparisons', () => {
        const script = '0 < 1';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool, eof] = root.children;
        expect(bool.getType()).toBe(Token.TRUE);
    });

    it('should simplify boolean logic and', () => {
        const script = 'true && false';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool, eof] = root.children;
        expect(bool.getType()).toBe(Token.FALSE);
    });

    it('should simplify boolean logic or', () => {
        const script = 'true || false';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool, eof] = root.children;
        expect(bool.getType()).toBe(Token.TRUE);
    });

    it('should simplify mulitple boolean logic', () => {
        const script = 'true && false || true && true';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool, eof] = root.children;
        expect(bool.getType()).toBe(Token.TRUE);
    });

    it('should simplify boolean logic not', () => {
        const script = '!true';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool, eof] = root.children;
        expect(bool.getType()).toBe(Token.FALSE);
    });

    it('should simplify nested boolean logic nots', () => {
        const script = '!!true';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool, eof] = root.children;
        expect(bool.getType()).toBe(Token.TRUE);
    });
});

describe('Optimizer: Simplify math expressions', () => {
    it.each([
        ['1 + 2', 3],
        ['1 - 2', -1],
        ['1 * 2', 2],
        ['1 / 2', 0.5],
        ['1 % 2', 1],
    ])(`should simplify %s to %i`, (script, expected) => {
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [num, eof] = root.children;
        expect(+num.getValue()).toBe(expected);
    });
});

describe('Optimizer: Simplify for loop conditions', () => {
    it('should simplify for loop with math in condition', () => {
        const script = `
            for (let i = 0; i < 10+10; i = i + 1) {
                return i;
            }
        `;

        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [forLoop, eof] = root.children;
        const { init, condition, update, block } = forLoop as ForASTNode;

        expect(condition).toBeInstanceOf(LessThanASTNode);
        if (!(condition instanceof LessThanASTNode)) {
            throw new Error('Condition is not an instance of LessThanASTNode');
        }

        const { left, right } = condition;
        expect(right.getType()).toBe(Token.NUMBER);
        expect(+right.getValue()).toBe(20);
    });
});
