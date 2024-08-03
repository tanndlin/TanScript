import Engine from '../Engine';
import { UndeclaredVariableError } from '../errors';

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

    it('Variables in local scope should not be in function scope', () => {
        const script =
            'def foo() { print(x); }\ndef bar() {let x = 10; foo();} bar();';
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
