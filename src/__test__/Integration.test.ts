import Engine from '../Engine';
import { UndeclaredVariableError } from '../errors';

describe('Integration Tests', () => {
    it('should run basic script', () => {
        const script = 'let x = 10;';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(10);
    });

    it('should run script with addition', () => {
        const script = 'let x = 10; x = x + 5;';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(15);
    });

    it('should run script with multiple variables', () => {
        const script = 'let x = 2;\n let y = x*13;\n let z = x + y;\n z += 1;';
        const engine = new Engine(script);
        const result = engine.run();
        expect(result).toBe(29);

        const scope = engine.getEnvironment().getGlobalScope();
        const x = scope.getVariable<number>('x');
        const y = scope.getVariable<number>('y');
        const z = scope.getVariable<number>('z');

        expect(x).toBe(2);
        expect(y).toBe(26);
        expect(z).toBe(29);
    });

    it('should run a while loop', () => {
        const script = 'let x = 0;\nwhile (x < 10) {\n  x += 1;\n}';
        const engine = new Engine(script);
        const result = engine.run();

        const scope = engine.getEnvironment().getGlobalScope();
        const x = scope.getVariable<number>('x');

        expect(result).toBe(10);
        expect(x).toBe(10);
    });

    it('should run a for loop', () => {
        const script =
            'let x = 0;\nfor (let i = 0; i < 10; i += 1) {\n  x += 1;\n}';
        const engine = new Engine(script);
        const result = engine.run();

        const scope = engine.getEnvironment().getGlobalScope();
        const x = scope.getVariable<number>('x');

        expect(result).toBe(10);
        expect(x).toBe(10);

        // I should not be in the global scope
        expect(() => scope.getVariable<number>('i')).toThrow(
            UndeclaredVariableError
        );
    });
});

describe('Function Tests', () => {
    it('should run a function', () => {
        const script = 'def add(a, b) { a + b; }\nadd(1, 2);';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(3);
    });

    it('should create variables in its own scope', () => {
        const script = 'def add(a, b) { let x = a + b; }\nadd(1, 2);';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(3);

        const scope = engine.getEnvironment().getGlobalScope();
        expect(() => scope.getVariable<number>('x')).toThrow(
            UndeclaredVariableError
        );
    });

    it('Variables in global scope should not be in scope for function', () => {
        const script = 'let x = 10;\ndef add(a) { x += a; }\nadd(5);';
        const engine = new Engine(script);
        expect(() => engine.run()).toThrow(UndeclaredVariableError);
    });

    it('modifing a variable in a function should not affect the callers scope', () => {
        const script = 'def foo(a) { a += 10; }\n let a = 0;\n foo(a);';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(10);

        const scope = engine.getEnvironment().getGlobalScope();
        expect(scope.getVariable<number>('a')).toBe(0);
    });
});
