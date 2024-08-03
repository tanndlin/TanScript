import Engine from '../Engine';
import { UndeclaredVariableError } from '../errors';

describe('Control Structure Tests', () => {
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

    it('should run a while loop', () => {
        const script = 'let x = 0;\nwhile (x < 10) {\n  x += 1;\n}';
        const engine = new Engine(script);
        const result = engine.run();

        const scope = engine.getEnvironment().getGlobalScope();
        const x = scope.getVariable<number>('x');

        expect(result).toBe(10);
        expect(x).toBe(10);
    });
});
