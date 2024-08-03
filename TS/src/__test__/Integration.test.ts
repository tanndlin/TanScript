import Engine from '../Engine';

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

    it.each([
        ['2 + 3 * 4', 14], // Multiplication before addition
        ['2 * 3 + 4', 10], // Multiplication before addition
        ['2 * (3 + 4)', 14], // Parentheses before multiplication
        ['(2 + 3) * 4', 20], // Parentheses before multiplication
        ['2 + 3 * 4 / 2', 8], // Multiplication and division before addition
        ['2 * 3 / 4 + 2', 3.5], // Multiplication and division before addition
        ['(2 + 3) * 4 / 2', 10], // Parentheses before multiplication and division
        ['2 + 3 * (4 / 2)', 8], // Parentheses before multiplication and addition
    ])('should respect PEMDAS in script %s', (script, expected) => {
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(expected);
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

    it.each([
        ['!true || false', false],
        ['!(true) || true', true],
        ['!(2+(5*2) > 5 - (2/2)) || true', true],
    ])('should run a complex script %s', (script, expected) => {
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(expected);
    });

    it('should allow incrementing a signal', () => {
        const script = 'let x #= 0;\n #x++;';
        const engine = new Engine(script);
        const result = engine.run();

        const scope = engine.getEnvironment().getGlobalScope();
        const x = scope.getSignal('x');

        expect(result).toBe(1);
        expect(x.getValue()).toBe(1);
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
