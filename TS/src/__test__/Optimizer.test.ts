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
        const [block, eof] = root.getChildren();
        expect(block.getChildren().length).toBe(1);

        const [retStatement] = block.getChildren();
        expect(retStatement.getChildren().length).toBe(1);
        expect(+retStatement.getChildren()[0].getValue()).toBe(1);
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
        const [block, eof] = root.getChildren();
        expect(block.getChildren().length).toBe(1);

        const [retStatement] = block.getChildren();
        expect(retStatement.getChildren().length).toBe(1);
        expect(+retStatement.getChildren()[0].getValue()).toBe(2);
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
        const [block, eof] = root.getChildren();
        expect(block.getChildren().length).toBe(1);

        const [retStatement] = block.getChildren();
        expect(retStatement.getChildren().length).toBe(1);
        expect(+retStatement.getChildren()[0].getValue()).toBe(1);
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
        const [block, eof] = root.getChildren();
        console.log(block.getChildren());
        expect(block.getChildren().length).toBe(0);
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
        const [block, eof] = root.getChildren();
        expect(block.getChildren().length).toBe(1);

        const [retStatement] = block.getChildren();
        expect(retStatement.getChildren().length).toBe(1);
        expect(+retStatement.getChildren()[0].getValue()).toBe(1);
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
        const [block, eof] = root.getChildren();
        expect(block.getChildren().length).toBe(1);

        const [retStatement] = block.getChildren();
        expect(retStatement.getChildren().length).toBe(1);
        expect(+retStatement.getChildren()[0].getValue()).toBe(1);
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
        const [bool, eof] = root.getChildren();
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
        const [bool, eof] = root.getChildren();
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
        const [bool, eof] = root.getChildren();
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
        const [bool, eof] = root.getChildren();
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
        const [bool, eof] = root.getChildren();
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
        const [bool, eof] = root.getChildren();
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
        const [num, eof] = root.getChildren();
        expect(+num.getValue()).toBe(expected);
    });
});
