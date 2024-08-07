import Engine from '../Engine';

describe('Return', () => {
    it('should return a value explicitly', () => {
        const script = 'def foo() {return 10;}foo();';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(10);
    });

    it('should return a value implicitly', () => {
        const script = 'def foo() {10;}foo();';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(10);
    });

    it('should return a variable implicitly', () => {
        const script = 'def foo() {let a = 10; a;}foo();';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(10);
    });

    it('return should break early', () => {
        const script = 'def foo() {return 10; 20;}foo();';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(10);
    });

    it('return should break early in a loop', () => {
        const script = `def foo() {
            let list = [1,2,3,4,5,6,7,8,9,10];
            foreach (e in list){
                if (e == 5){
                    return e;
                }
            }
        }
        foo();`;
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(5);
    });
});
