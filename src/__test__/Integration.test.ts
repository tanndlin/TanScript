import Engine from '../Engine';
import { UndeclaredVariableError } from '../errors';

describe('Integration Tests', () => {
    it('should run basic script', () => {
        const script = 'let x = 10;';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(10);
    });

    it.each([
        ['let x = 10; x = x + 5;', 15],
        ['let x = 10; x = x - 5;', 5],
        ['let x = 10; x = x * 5;', 50],
        ['let x = 10; x = x / 5;', 2],
        ['let x = 10; x = x % 5;', 0],
    ])('should run script with arithmetic %s', (script, expected) => {
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(expected);
    });

    it('should check equals with a math expression', () => {
        const script = '10 % 5 == 0;';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(true);
    });

    it('should respect PEMDAS', () => {
        const script = '2 * 5 + 7;';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(19);
    });

    it('should respect heirarchy', () => {
        const script = 'true || false == false && true';
        const engine = new Engine(script);
        expect(engine.run()).toBe(true);
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

    it.each([
        ['!true || false', false],
        ['!(true) || true', true],
        ['!(2+(5*2) > 5 - (2/2)) || true', true],
    ])('should run a complex script %s', (script, expected) => {
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(expected);
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

    it('Variables in global scope should be in scope for function', () => {
        const script = 'let x = 10;\ndef add(a) { x += a; }\nadd(5);';
        const engine = new Engine(script);
        expect(() => engine.run()).not.toThrow();
    });

    it('modifing a variable in a function should not affect the callers scope', () => {
        const script = 'def foo(a) { a += 10; }\n let a = 0;\n foo(a);';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(10);

        const scope = engine.getEnvironment().getGlobalScope();
        expect(scope.getVariable<number>('a')).toBe(0);
    });

    it('allows recursion', () => {
        const script =
            'def fib(n) { if (n <= 1) { n; } else { fib(n - 1) + fib(n - 2); } }\nfib(10);';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(55);
    });

    it('allows nested functions', () => {
        const script = `def add3(a,b,c) {\
            def add2(d,e) {\
                d + e;\
            }\

            add2(a, add2(b,c));\
        }\

        add3(69,420,69420);`;

        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(69909);
    });
});

describe.each([
    ['0 && 1', false],
    ['1 && 1', true],
    ['0 || 1', true],
    ['0 || 0', false],
    ['1 || 0', true],
    ['1 || 1', true],
    ['1 && 0', false],
    ['0 && 0', false],
    ['1 && 1 && 1', true],
    ['1 && 1 && 0', false],
    ['1 || 1 || 1', true],
    ['1 || 1 || 0', true],
    ['0 || 0 || 0', false],
    ['0 || 0 || 1', true],
    ['0 && 0 && 0', false],
    ['0 && 0 && 1', false],
    ['1 || 0 || 0', true],
    ['1 || 0 || 1', true],
    ['2 + 3', 5],
    ['6 / 2', 3],
    ['10-3', 7],
    ['6 * 2', 12],
    ['1 < 2', true],
    ['1 > 2', false],
    ['1 <= 2', true],
    ['1 >= 2', false],
    ['1 == 1', true],
    ['1 != 1', false],
    ['1 != 2', true],
    ['1 == 2', false],
    ['!0', true],
    ['!1', false],
])('Expression Tests', (script, expected) => {
    it(`should evaluate ${script} to ${expected}`, () => {
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(expected);
    });
});

describe.each([
    ['true && true', true],
    ['true && false', false],
    ['false && true', false],
    ['false && false', false],
    ['true || true', true],
    ['true || false', true],
    ['false || true', true],
    ['false || false', false],
    ['true && true && false', false],
])('Boolean Expression Tests', (script, expected) => {
    it(`should evaluate ${script} to ${expected}`, () => {
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(expected);
    });
});
