import * as AST from '../AST';
import Lexer from '../Lexer';
import Optimizer from '../Optimizer';
import Parser from '../Parser';
import { Token } from '../types';
import { getTokens } from './Lexer.test';

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
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block] = root.children;
        expect(block).toBeInstanceOf(AST.BlockASTNode);
        if (!(block instanceof AST.BlockASTNode)) {
            return;
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(AST.ReturnASTNode);
        if (!(retStatement instanceof AST.ReturnASTNode)) {
            return;
        }

        if (!(retStatement.valueAST instanceof AST.NumberASTNode)) {
            return;
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
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block] = root.children;
        expect(block).toBeInstanceOf(AST.BlockASTNode);
        if (!(block instanceof AST.BlockASTNode)) {
            return;
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(AST.ReturnASTNode);
        if (!(retStatement instanceof AST.ReturnASTNode)) {
            return;
        }

        if (!(retStatement.valueAST instanceof AST.NumberASTNode)) {
            return;
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
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block] = root.children;
        expect(block).toBeInstanceOf(AST.BlockASTNode);
        if (!(block instanceof AST.BlockASTNode)) {
            return;
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(AST.ReturnASTNode);
        if (!(retStatement instanceof AST.ReturnASTNode)) {
            return;
        }

        if (!(retStatement.valueAST instanceof AST.NumberASTNode)) {
            return;
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
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();

        expect(root.children).toHaveLength(1);
        const [eof] = root.children;
        expect(eof).toBeInstanceOf(AST.EOFASTNode);
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
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block] = root.children;
        expect(block).toBeInstanceOf(AST.BlockASTNode);
        if (!(block instanceof AST.BlockASTNode)) {
            return;
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(AST.ReturnASTNode);
        if (!(retStatement instanceof AST.ReturnASTNode)) {
            return;
        }

        if (!(retStatement.valueAST instanceof AST.NumberASTNode)) {
            return;
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
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [block] = root.children;
        expect(block).toBeInstanceOf(AST.BlockASTNode);
        if (!(block instanceof AST.BlockASTNode)) {
            return;
        }

        expect(block.children.length).toBe(1);

        const [retStatement] = block.children;
        expect(retStatement).toBeInstanceOf(AST.ReturnASTNode);
        if (!(retStatement instanceof AST.ReturnASTNode)) {
            return;
        }

        if (!(retStatement.valueAST instanceof AST.NumberASTNode)) {
            return;
        }

        expect(+retStatement.valueAST.getValue()).toBe(1);
    });
});

describe('Optimizer: Simplify compound expressions', () => {
    it('should simplifiy constant number comparisons', () => {
        const script = '0 < 1';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool] = root.children;
        expect(bool.type).toBe(Token.TRUE);
    });

    it('should simplify boolean logic and', () => {
        const script = 'true && false';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool] = root.children;
        expect(bool.type).toBe(Token.FALSE);
    });

    it('should simplify boolean logic or', () => {
        const script = 'true || false';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool] = root.children;
        expect(bool.type).toBe(Token.TRUE);
    });

    it('should simplify mulitple boolean logic', () => {
        const script = 'true && false || true && true';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool] = root.children;
        expect(bool.type).toBe(Token.TRUE);
    });

    it('should simplify boolean logic not', () => {
        const script = '!true';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool] = root.children;
        expect(bool.type).toBe(Token.FALSE);
    });

    it('should simplify nested boolean logic nots', () => {
        const script = '!!true';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [bool] = root.children;
        expect(bool.type).toBe(Token.TRUE);
    });
});

describe('Optimizer: Simplify math expressions', () => {
    it.each([
        ['1 + 2', 3],
        ['1 - 2', -1],
        ['1 * 2', 2],
        ['1 / 2', 0.5],
        ['1 % 2', 1],
    ])('should simplify %s to %i', (script, expected) => {
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [num] = root.children;
        expect(num.type).toBe(Token.NUMBER);
        if (!(num instanceof AST.NumberASTNode)) {
            return;
        }
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
        const tokens = getTokens(lexer);
        const parser = new Parser(tokens);
        let ast = parser.parse();
        ast = Optimizer.optimize(ast);

        const root = ast.getRoot();
        const [forLoop] = root.children;
        const { condition } = forLoop as AST.ForASTNode;

        expect(condition).toBeInstanceOf(AST.LessThanASTNode);
        if (!(condition instanceof AST.LessThanASTNode)) {
            return;
        }

        const { right } = condition;
        expect(right.type).toBe(Token.NUMBER);
        if (!(right instanceof AST.NumberASTNode)) {
            return;
        }
        expect(+right.getValue()).toBe(20);
    });
});
