import Lexer from '../Lexer';
import Optimizer from '../Optimizer';
import Parser from '../Parser';

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
});
