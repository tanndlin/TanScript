import Environment from '../Environment';
import Lexer from '../Lexer';
import Parser from '../Parser';

describe('Integration Tests', () => {
    it('should run basic script', () => {
        const script = 'let x = 10;';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        const ast = parser.parse();

        const env = new Environment(ast);
        const result = env.evaluate();

        expect(result).toBe(10);
    });

    it('should run script with addition', () => {
        const script = 'let x = 10; x = x + 5;';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        const ast = parser.parse();

        const env = new Environment(ast);
        const result = env.evaluate();

        expect(result).toBe(15);
    });

    it('should run script with multiple variables', () => {
        const script = 'let x = 2;\n let y = x*13;\n let z = x + y;\n z += 1;';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const parser = new Parser(tokens);
        const ast = parser.parse();

        const env = new Environment(ast);
        const result = env.evaluate();
        expect(result).toBe(29);

        const scope = env.getGlobalScope();
        const x = scope.getVariable<number>('x');
        const y = scope.getVariable<number>('y');
        const z = scope.getVariable<number>('z');

        expect(x).toBe(2);
        expect(y).toBe(26);
        expect(z).toBe(29);
    });
});
